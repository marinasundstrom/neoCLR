//! Experimental scalar lowering, deliberately separate from the public runtime API.
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::settings::Configurable;
use cranelift_codegen::{
    ir::{self, AbiParam, InstBuilder, MemFlags, StackSlotData, StackSlotKind, types},
    isa, settings,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};

#[path = "flow.rs"]
mod flow;
#[path = "values.rs"]
pub(super) mod values;
use cranelift_module::{DataDescription, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use neoclr::metadata::{Function, FunctionRef, Instruction as Op, Type};
use std::collections::{HashMap, HashSet};

type Error = Box<dyn std::error::Error>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Target {
    MacosArm64,
    WindowsX64,
}

impl Target {
    pub(super) fn parse(value: &str) -> Result<Self, Error> {
        match value {
            "aarch64-apple-darwin" => Ok(Self::MacosArm64),
            "x86_64-pc-windows-msvc" => Ok(Self::WindowsX64),
            _ => Err(format!("unsupported AOT target: {value}").into()),
        }
    }

    pub(super) fn configure_stack(self, flags: &mut settings::Builder) -> Result<(), Error> {
        if self == Self::WindowsX64 {
            // AOT and JIT must both touch each Windows stack guard page.
            flags.set("enable_probestack", "true")?;
            flags.set("probestack_strategy", "inline")?;
            flags.set("probestack_size_log2", "12")?;
        }
        Ok(())
    }

    pub(super) fn triple(self) -> &'static str {
        match self {
            Self::MacosArm64 => "aarch64-apple-darwin",
            Self::WindowsX64 => "x86_64-pc-windows-msvc",
        }
    }
}

pub(super) fn trace_layout(
    input: &neoclr::Module,
    details: Option<&crate::fault_details::Options>,
) -> Result<serde_json::Value, Error> {
    values::trace_layout(input, details)
}

pub(super) fn compile(
    input: &neoclr::Module,
    root: &str,
    console: bool,
    details: Option<&crate::fault_details::Options>,
) -> Result<Vec<u8>, Error> {
    compile_for_target(input, root, console, details, Target::MacosArm64)
}

pub(super) fn compile_for_target(
    input: &neoclr::Module,
    root: &str,
    console: bool,
    details: Option<&crate::fault_details::Options>,
    target: Target,
) -> Result<Vec<u8>, Error> {
    compile_for_target_with_imports(input, root, console, details, target, false, None)
}

pub(super) fn compile_for_target_with_imports(
    input: &neoclr::Module,
    root: &str,
    console: bool,
    details: Option<&crate::fault_details::Options>,
    target: Target,
    native_imports: bool,
    verification: Option<&neoclr::LoadedProgram>,
) -> Result<Vec<u8>, Error> {
    if native_imports && (target != Target::MacosArm64 || details.is_some()) {
        return Err("native imports currently require the macOS ARM64 scalar AOT profile".into());
    }
    if details.is_some() || (!native_imports && !input.types.is_empty())
        || input.functions.iter().any(|f| {
            f.parameters
                .iter()
                .chain(&f.locals)
                .chain([&f.returns])
                .any(|t| {
                    matches!(t, Type::Value | Type::String | Type::RuntimeTypeHandle | Type::Tuple(_) | Type::Union(_))
                        || (!native_imports && matches!(t, Type::Single | Type::Double))
                        || matches!(t, Type::ByRef(inner) if matches!(**inner, Type::Value | Type::String | Type::Tuple(_) | Type::Union(_)))
                })
                || f.body.iter().any(|op| {
                    matches!(op, Op::New(Type::Tuple(_)) | Op::CastClass(Type::Union(_)) | Op::ShiftLeft | Op::ShiftRight | Op::ShiftRightUnsigned | Op::LoadTypeToken(_) | Op::LoadMetadataToken(_) | Op::PackValue(_) | Op::IsValue(_) | Op::UnpackValue(_))
                        || (!native_imports && matches!(op, Op::Float32 { .. } | Op::Float64 { .. }))
                        || crate::checked_conversion::spec(op).is_some()
                        || matches!(op, Op::Call(target) if target.definition.is_some())
                })
        })
    {
        if target == Target::WindowsX64 && !details.is_some_and(|d| d.windows_stack_experiment || d.windows_console_experiment || d.windows_http_experiment) {
            return Err("Windows x64 currently supports only scalar and literal-console lowering; value/managed profiles remain macOS ARM64-only".into());
        }
        if target == Target::WindowsX64 && details.is_some_and(|d| d.windows_console_experiment || d.windows_http_experiment)
            && !details.is_some_and(|d| d.native_gc && d.native_stack_budget && d.reference_arena && d.probe_stack_roots) {
            return Err("Windows console/HTTP requires published GC frames, reference arena and stack budget".into());
        }
        if target == Target::WindowsX64 && !details.is_some_and(|d| d.windows_console_experiment || d.windows_http_experiment) {
            // Heap opt-in adds only Int32 arrays, not arbitrary managed services.
            let heap = details.is_some_and(|d| d.windows_heap_experiment);
            let admitted = |t: &Type| *t == Type::Int32 || heap && matches!(t, Type::ArrayRef(inner) if **inner == Type::Int32);
            if !details.is_some_and(|d| d.native_gc && d.native_stack_budget && d.reference_arena && d.probe_stack_roots)
                || !input.types.is_empty() || input.name == "System"
                || input.functions.iter().any(|f| {
                    f.is_internal_call() || f.pinvoke.is_some() || f.owner.is_some() || f.instance
                        || f.parameters.iter().chain(&f.locals).chain([&f.returns]).any(|t| !admitted(t))
                        || f.body.iter().any(|op| !(matches!(op,
                            Op::Int(_) | Op::Arg(_) | Op::Load(_) | Op::Store(_) |
                            Op::Add | Op::Sub | Op::Mul | Op::Equal | Op::Less | Op::Greater |
                            Op::Branch(_) | Op::BranchTrue(_) | Op::BranchFalse(_) |
                            Op::Dup | Op::Pop | Op::Return | Op::Call(_)) || heap && matches!(op,
                            Op::NewArray(Type::Int32) | Op::ArrayElement(Type::Int32) |
                            Op::StoreArrayElement(Type::Int32) | Op::ArrayLength | Op::ConvertUInt32
            | Op::ConvertInt32)))
                }) {
                return Err("Windows stack experiment admits only Int32 functions and bounded integer control flow with GC frames and stack budget".into());
            }
        }
        if native_imports {
            return Err("native imports currently require the scalar AOT profile".into());
        }
        return values::compile(input, root, details, target);
    }
    let target = target.triple().parse().expect("fixed valid target triple");
    let mut settings = crate::backend_settings();
    settings.set("is_pic", "true")?;
    let isa = isa::lookup(target)?.finish(settings::Flags::new(settings))?;
    let mut module = ObjectModule::new(ObjectBuilder::new(
        isa,
        "neoclr_scalar",
        cranelift_module::default_libcall_names(),
    )?);
    compile_scalar_with_imports(
        input,
        root,
        console,
        &mut module,
        false,
        native_imports,
        verification,
    )?;
    finish(module)
}

/// Shared verified scalar lowering for object and in-memory module implementations.
pub(super) fn compile_scalar<M: Module>(
    input: &neoclr::Module,
    root: &str,
    console: bool,
    module: &mut M,
    long_range_calls: bool,
) -> Result<(), Error> {
    compile_scalar_with_imports(input, root, console, module, long_range_calls, false, None)
}

pub(super) fn compile_scalar_with_imports<M: Module>(
    input: &neoclr::Module,
    root: &str,
    console: bool,
    module: &mut M,
    long_range_calls: bool,
    native_imports: bool,
    verification: Option<&neoclr::LoadedProgram>,
) -> Result<(), Error> {
    compile_scalar_impl(
        input,
        Some(root),
        console,
        module,
        long_range_calls,
        native_imports,
        verification,
        &[],
    )
}

/// Select native executable bodies only after verifying the entire source program.
/// Attribute normalization affects the private emission copy, never verification.
pub(super) fn prepare_native_entry<'a>(
    input: &neoclr::Module,
    root: &str,
    modules: impl Iterator<Item = &'a neoclr::Module>,
    verification: &neoclr::LoadedProgram,
) -> Result<neoclr::Module, Error> {
    verification.verify().map_err(|error| error.to_string())?;
    let input = crate::native_layout::normalize_annotations(input, modules)?;
    // Unused bodies may be outside the scalar subset, but declarations must not
    // hide an unsupported native ABI contract behind reachability.
    for function in input.functions.iter().filter(|f| f.pinvoke.is_some()) {
        crate::native_imports::validate(function)?;
    }
    let roots: Vec<_> = input
        .functions
        .iter()
        .enumerate()
        .filter(|(_, function)| function.name == root)
        .map(|(index, _)| (index, String::new()))
        .collect();
    if roots.len() != 1 {
        return Err("native executable root must identify exactly one function".into());
    }
    Ok(select_native_exports(&input, &roots)?.0)
}

pub(super) fn compile_native_library(
    input: &neoclr::Module,
    exports: &[(usize, String)],
    native_imports: bool,
    verification: Option<&neoclr::LoadedProgram>,
) -> Result<Vec<u8>, Error> {
    if !native_imports && input.functions.iter().any(|f| f.pinvoke.is_some()) {
        return Err("native library imports require --native-imports".into());
    }
    // Verify the complete source before selecting executable bodies. An unused
    // constructor need not fit the scalar backend, but must still be valid IL.
    let owned_verification;
    let verification = match verification {
        Some(program) => program,
        None => {
            owned_verification =
                neoclr::LoadedProgram::new(input).map_err(|error| error.to_string())?;
            &owned_verification
        }
    };
    verification.verify().map_err(|error| error.to_string())?;
    let (selected, exports) = select_native_exports(input, exports)?;
    let input = &selected;
    let mut flags = crate::backend_settings();
    flags.set("is_pic", "true")?;
    let isa = isa::lookup(Target::MacosArm64.triple().parse().unwrap())?
        .finish(settings::Flags::new(flags))?;
    let mut module = ObjectModule::new(ObjectBuilder::new(
        isa,
        "neoclr_native_library",
        cranelift_module::default_libcall_names(),
    )?);
    // Native exports admit the same scalar/pointer representation as imports.
    compile_scalar_impl(
        input,
        None,
        false,
        &mut module,
        false,
        true,
        Some(verification),
        &exports,
    )?;
    finish(module)
}

// Scalar exports have direct calls only. Reuse nominal/signature resolution;
// unsupported dispatch instructions remain errors in selected body admission.
fn select_native_exports(
    input: &neoclr::Module,
    exports: &[(usize, String)],
) -> Result<(neoclr::Module, Vec<(usize, String)>), Error> {
    let mut retained = std::collections::BTreeSet::new();
    let mut pending: Vec<_> = exports.iter().map(|(index, _)| *index).collect();
    while let Some(index) = pending.pop() {
        let function = input
            .functions
            .get(index)
            .ok_or("native export index out of range")?;
        if !retained.insert(index) {
            continue;
        }
        for op in &function.body {
            if let Op::Call(target) = op {
                pending.push(crate::selection::resolve(input, target)?);
            }
        }
    }
    let rows: Vec<_> = retained.into_iter().collect();
    let mut selected = input.clone();
    selected.functions = rows
        .iter()
        .map(|index| input.functions[*index].clone())
        .collect();
    let exports = exports
        .iter()
        .map(|(index, name)| {
            (
                rows.binary_search(index).expect("retained export"),
                name.clone(),
            )
        })
        .collect();
    // Keep nominal layouts and source identity intact; verification uses the
    // original loaded program, not this private emission inventory.
    Ok((selected, exports))
}

#[allow(clippy::too_many_arguments)]
fn compile_scalar_impl<M: Module>(
    input: &neoclr::Module,
    root: Option<&str>,
    console: bool,
    module: &mut M,
    long_range_calls: bool,
    native_imports: bool,
    verification: Option<&neoclr::LoadedProgram>,
    exports: &[(usize, String)],
) -> Result<(), Error> {
    if native_imports {
        let layouts = crate::native_layout::Layouts::build(input)?;
        for function in &input.functions {
            for ty in function
                .parameters
                .iter()
                .chain(&function.locals)
                .chain([&function.returns])
            {
                layouts.check_pointer(ty)?;
            }
            for op in &function.body {
                if let Op::PointerCast(ty) | Op::NullPointer(ty) = op {
                    layouts.check_pointer(&Type::Ptr(Box::new(ty.clone())))?;
                }
            }
        }
    } else if !input.types.is_empty() {
        return Err("scalar profile does not support type definitions".into());
    }
    if input.name == "System" || input.functions.len() > 128 {
        return Err(
            "scalar profile requires an application with no types and at most 128 functions".into(),
        );
    }
    if input.functions.iter().any(|f| {
        matches!(
            f.name.as_str(),
            "neoCLR.Runtime.WriteLine" | "System.Console.WriteLine"
        )
    }) {
        return Err("application cannot replace the reserved console service".into());
    }
    let mut uses_console = false;
    let mut names = HashMap::new();
    let mut flows = Vec::new();
    for (index, function) in input.functions.iter().enumerate() {
        if names.insert(function.name.as_str(), index).is_some() {
            return Err("scalar profile does not support overloaded names".into());
        }
        if native_imports && function.pinvoke.is_some() {
            crate::native_imports::validate(function)?;
            flows.push(Vec::new());
        } else {
            flows.push(check_function(function, &input.functions, native_imports)?);
        }
    }
    let root_index = root
        .map(|name| names.get(name).copied().ok_or("root function not found"))
        .transpose()?;
    let parameterless_root = root_index.is_some_and(|i| input.functions[i].parameters.is_empty());
    if let Some(i) = root_index {
        if input.functions[i].pinvoke.is_some() {
            return Err("native import cannot be the managed AOT root".into());
        }
        if input.functions[i].returns != Type::Int32
            || (!parameterless_root && input.functions[i].parameters != [Type::Int32])
        {
            return Err("root must have signature () -> Int32 or (Int32) -> Int32".into());
        }
    }
    // Validate every declared body, including unreachable functions/instructions.
    // Resolve only unqualified local calls; no ad-hoc cross-module name lookup.
    let mut edges = vec![vec![]; input.functions.len()];
    for (index, function) in input.functions.iter().enumerate() {
        for (pc, op) in function.body.iter().enumerate() {
            if let Op::Call(target) = op {
                if console_call(target) {
                    if !console {
                        return Err("console service requires --console capability".into());
                    }
                    uses_console = true;
                    continue;
                }
                if target.owner.is_some()
                    || target.instance
                    || target.definition.is_some()
                    || !target.generic_arguments.is_empty()
                {
                    return Err(format!(
                        "{} instruction {pc}: only unqualified local calls are supported",
                        function.name
                    )
                    .into());
                }
                let callee = *names.get(target.name.as_str()).ok_or_else(|| {
                    format!(
                        "{} instruction {pc}: external call is unsupported",
                        function.name
                    )
                })?;
                if input.functions[callee].parameters != target.parameters {
                    return Err(format!(
                        "{} instruction {pc}: call signature mismatch",
                        function.name
                    )
                    .into());
                }
                edges[index].push(callee);
            }
        }
    }
    // Native recursion budgets are not implemented: reject call-graph cycles.
    fn visit(
        node: usize,
        edges: &[Vec<usize>],
        active: &mut HashSet<usize>,
        done: &mut HashSet<usize>,
    ) -> Result<(), Error> {
        if done.contains(&node) {
            return Ok(());
        }
        if !active.insert(node) {
            return Err("recursive calls require a native stack-budget contract".into());
        }
        for &next in &edges[node] {
            visit(next, edges, active, done)?;
        }
        active.remove(&node);
        done.insert(node);
        Ok(())
    }
    let mut done = HashSet::new();
    for node in 0..edges.len() {
        visit(node, &edges, &mut HashSet::new(), &mut done)?;
    }
    if input
        .functions
        .iter()
        .flat_map(|f| &f.body)
        .any(|op| matches!(op, Op::Call(target) if public_console_call(target)))
    {
        check_console_wrapper()?;
    }
    // Verify the source artifact in its declared load context. Dependencies are
    // metadata inputs only: scalar admission above still forbids external calls,
    // managed values and type definitions in the emitted application.
    if let Some(program) = verification {
        program.verify().map_err(|error| error.to_string())?;
    } else {
        neoclr::LoadedProgram::new(input)
            .and_then(|program| program.verify())
            .map_err(|error| error.to_string())?;
    }

    let console_id = if uses_console {
        let mut signature = module.make_signature();
        signature.params = vec![AbiParam::new(types::I64), AbiParam::new(types::I64)];
        signature.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function(
            "neoclr_console_write_line_utf8_v1",
            Linkage::Import,
            &signature,
        )?)
    } else {
        None
    };
    // Literal-only temporary representation: [u64 byte count][UTF-8 bytes].
    // This is private read-only object data, not a managed String ABI.
    let mut literals = HashMap::new();
    for (index, function) in input.functions.iter().enumerate() {
        for (pc, op) in function.body.iter().enumerate() {
            if let Op::String(text) = op {
                let id = module.declare_data(
                    &format!("neoclr_literal_{index}_{pc}"),
                    Linkage::Local,
                    false,
                    false,
                )?;
                let mut data = DataDescription::new();
                let mut bytes = (text.len() as u64).to_le_bytes().to_vec();
                bytes.extend_from_slice(text.as_bytes());
                data.define(bytes.into_boxed_slice());
                data.set_align(8);
                module.define_data(id, &data)?;
                literals.insert((index, pc), id);
            }
        }
    }
    let mut ids = Vec::new();
    for (index, function) in input.functions.iter().enumerate() {
        let mut sig = module.make_signature();
        sig.params = function
            .parameters
            .iter()
            .map(|t| AbiParam::new(crate::scalar_abi::managed_scalar_type(t).unwrap()))
            .collect();
        sig.params.push(AbiParam::new(types::I64)); // writable typed result pointer
        sig.returns.push(AbiParam::new(types::I32)); // status: 0 success, 1 zero, 2 overflow
        let (name, linkage) = if Some(index) == root_index && !parameterless_root {
            ("neoclr_entry_v2".to_owned(), Linkage::Export)
        } else {
            (format!("neoclr_scalar_{index}"), Linkage::Local)
        };
        ids.push(module.declare_function(&name, linkage, &sig)?);
    }
    for (index, function) in input.functions.iter().enumerate() {
        if function.pinvoke.is_some() {
            crate::native_imports::emit(module, function, ids[index], index)?;
            continue;
        }
        let mut context = module.make_context();
        context.func.signature = module
            .declarations()
            .get_function_decl(ids[index])
            .signature
            .clone();
        let mut builder_context = FunctionBuilderContext::new();
        {
            let mut builder = FunctionBuilder::new(&mut context.func, &mut builder_context);
            let entry = builder.create_block();
            builder.append_block_params_for_function_params(entry);
            builder.switch_to_block(entry);
            let args = builder.block_params(entry).to_vec();
            let output = args[function.parameters.len()];
            let call_result = builder.create_sized_stack_slot(StackSlotData::new(
                StackSlotKind::ExplicitSlot,
                8,
                3,
            ));
            let blocks: Vec<_> = function
                .body
                .iter()
                .map(|_| builder.create_block())
                .collect();
            for (pc, stack) in flows[index].iter().enumerate() {
                if let Some(stack) = stack {
                    for kind in stack {
                        builder.append_block_param(
                            blocks[pc],
                            match kind {
                                flow::Kind::Single => types::F32,
                                flow::Kind::Double => types::F64,
                                flow::Kind::Literal
                                | flow::Kind::Pointer
                                | flow::Kind::Int64
                                | flow::Kind::NativeInt
                                | flow::Kind::NativeUInt => types::I64,
                                flow::Kind::Int32
                                | flow::Kind::Boolean
                                | flow::Kind::Unit
                                | flow::Kind::LocalAddress(_) => types::I32,
                            },
                        );
                    }
                }
            }
            for local in 0..function.locals.len() {
                builder.declare_var(
                    Variable::from_u32(local as u32),
                    crate::scalar_abi::managed_scalar_type(&function.locals[local]).unwrap(),
                );
            }
            builder.ins().jump(blocks[0], &[]);
            for (pc, op) in function.body.iter().enumerate() {
                if flows[index][pc].is_none() {
                    continue;
                }
                builder.switch_to_block(blocks[pc]);
                // SSA local parameters may be appended while translating predecessors;
                // only the explicit first parameters represent the IL operand stack.
                let mut stack: Vec<ir::Value> = builder.block_params(blocks[pc])
                    [..flows[index][pc].as_ref().unwrap().len()]
                    .to_vec();
                match op {
                    Op::Int(value) => {
                        stack.push(builder.ins().iconst(types::I32, i64::from(*value)))
                    }
                    // Only exact local-address initobj is admitted. The address is
                    // a flow marker, never a native pointer or an escaping byref.
                    Op::LocalAddress(_) => stack.push(builder.ins().iconst(types::I32, 0)),
                    Op::InitializeObject(ty) => {
                        let Some(flow::Kind::LocalAddress(local)) =
                            flows[index][pc].as_ref().unwrap().last()
                        else {
                            unreachable!("validated local initialization");
                        };
                        stack.pop().expect("validated stack");
                        let zero = match ty {
                            Type::Single => builder.ins().f32const(0.0),
                            Type::Double => builder.ins().f64const(0.0),
                            _ => builder
                                .ins()
                                .iconst(crate::scalar_abi::managed_scalar_type(ty).unwrap(), 0),
                        };
                        builder.def_var(Variable::from_u32(*local as u32), zero);
                    }
                    Op::Int64(value) => stack.push(builder.ins().iconst(types::I64, *value)),
                    Op::Float32 { bits } => stack.push(
                        builder
                            .ins()
                            .f32const(ir::immediates::Ieee32::with_bits(*bits)),
                    ),
                    Op::Float64 { bits } => stack.push(
                        builder
                            .ins()
                            .f64const(ir::immediates::Ieee64::with_bits(*bits)),
                    ),
                    Op::ConvertInt64
                    | Op::ConvertUInt64
                    | Op::ConvertNativeInt
                    | Op::ConvertNativeUInt
                    | Op::ConvertUInt32
                    | Op::ConvertInt32 => {
                        let value = stack.pop().expect("validated integer conversion");
                        let source = builder.func.dfg.value_type(value);
                        let target = if matches!(op, Op::ConvertInt32 | Op::ConvertUInt32) {
                            types::I32
                        } else {
                            types::I64
                        };
                        let converted = if source == target {
                            value
                        } else if target == types::I32 {
                            builder.ins().ireduce(target, value)
                        } else if matches!(op, Op::ConvertUInt64 | Op::ConvertNativeUInt) {
                            builder.ins().uextend(target, value)
                        } else {
                            builder.ins().sextend(target, value)
                        };
                        stack.push(converted);
                    }
                    Op::LoadObject(_)
                    | Op::LoadIndirectInt32
                    | Op::LoadIndirectUInt8
                    | Op::StoreObject(_)
                    | Op::StoreIndirectInt32
                    | Op::StoreIndirectInt8 => {
                        let store = matches!(
                            op,
                            Op::StoreObject(_) | Op::StoreIndirectInt32 | Op::StoreIndirectInt8
                        );
                        let value = if store {
                            Some(stack.pop().expect("validated value"))
                        } else {
                            None
                        };
                        let pointer = stack.pop().expect("validated pointer");
                        let byte = matches!(
                            op,
                            Op::LoadObject(Type::Byte)
                                | Op::StoreObject(Type::Byte)
                                | Op::LoadIndirectUInt8
                                | Op::StoreIndirectInt8
                        );
                        // Preserve deterministic faults for null/misaligned accesses. Other
                        // address validity remains the trusted native caller's responsibility.
                        let null = builder.ins().icmp_imm(IntCC::Equal, pointer, 0);
                        let status = builder.ins().iconst(types::I32, 3);
                        return_if(&mut builder, null, status);
                        if !byte {
                            let low = builder.ins().band_imm(pointer, 3);
                            let misaligned = builder.ins().icmp_imm(IntCC::NotEqual, low, 0);
                            return_if(&mut builder, misaligned, status);
                        }
                        if let Some(value) = value {
                            let value = if byte {
                                builder.ins().ireduce(types::I8, value)
                            } else {
                                value
                            };
                            builder.ins().store(MemFlags::new(), value, pointer, 0);
                        } else {
                            let value = builder.ins().load(
                                if byte { types::I8 } else { types::I32 },
                                MemFlags::new(),
                                pointer,
                                0,
                            );
                            let value = if byte {
                                builder.ins().uextend(types::I32, value)
                            } else {
                                value
                            };
                            stack.push(value);
                        }
                    }
                    Op::NullPointer(_) => stack.push(builder.ins().iconst(types::I64, 0)),
                    // A typed-pointer cast changes metadata only, never address bits.
                    Op::PointerCast(_) => (),
                    Op::Bool(value) => {
                        stack.push(builder.ins().iconst(types::I32, i64::from(*value)))
                    }
                    Op::Equal | Op::Less | Op::Greater => {
                        let right = stack.pop().expect("validated stack");
                        let left = stack.pop().expect("validated stack");
                        let condition = match op {
                            Op::Equal => IntCC::Equal,
                            Op::Less => IntCC::SignedLessThan,
                            Op::Greater => IntCC::SignedGreaterThan,
                            _ => unreachable!(),
                        };
                        let result = builder.ins().icmp(condition, left, right);
                        stack.push(builder.ins().uextend(types::I32, result));
                    }
                    Op::String(_) => {
                        let data =
                            module.declare_data_in_func(literals[&(index, pc)], builder.func);
                        stack.push(builder.ins().global_value(types::I64, data));
                    }
                    Op::Arg(index) => stack.push(args[*index]),
                    Op::Load(index) => {
                        stack.push(builder.use_var(Variable::from_u32(*index as u32)))
                    }
                    Op::Store(index) => {
                        let value = stack.pop().expect("validated stack");
                        builder.def_var(Variable::from_u32(*index as u32), value);
                    }
                    Op::Dup => stack.push(*stack.last().expect("validated stack")),
                    Op::Pop => {
                        stack.pop().expect("validated stack");
                    }
                    Op::Add | Op::Sub | Op::Mul => {
                        let right = stack.pop().expect("validated stack");
                        let left = stack.pop().expect("validated stack");
                        let value = match op {
                            Op::Add => builder.ins().iadd(left, right),
                            Op::Sub => builder.ins().isub(left, right),
                            Op::Mul => builder.ins().imul(left, right),
                            _ => unreachable!(),
                        };
                        stack.push(value);
                    }
                    Op::AddChecked
                    | Op::SubChecked
                    | Op::MulChecked
                    | Op::AddCheckedUnsigned
                    | Op::SubCheckedUnsigned
                    | Op::MulCheckedUnsigned
                    | Op::Divide
                    | Op::DivideUnsigned
                    | Op::Remainder
                    | Op::RemainderUnsigned => {
                        let right = stack.pop().expect("validated stack");
                        let left = stack.pop().expect("validated stack");
                        stack.push(checked_arithmetic(&mut builder, op, left, right, None));
                    }
                    Op::Call(target) if console_call(target) => {
                        let literal = stack.pop().expect("validated literal");
                        let length = builder.ins().load(types::I64, MemFlags::new(), literal, 0);
                        let bytes = builder.ins().iadd_imm(literal, 8);
                        let reference = module.declare_func_in_func(
                            console_id.expect("declared service"),
                            builder.func,
                        );
                        let call = builder.ins().call(reference, &[bytes, length]);
                        let service_status = builder.inst_results(call)[0];
                        let failed = builder.ins().icmp_imm(IntCC::NotEqual, service_status, 0);
                        let fault = builder.ins().iconst(types::I32, 3);
                        return_if(&mut builder, failed, fault);
                        // Preserve inhabited Void on the IL stack without a public layout.
                        stack.push(builder.ins().iconst(types::I32, 0));
                    }
                    Op::Call(target) => {
                        let callee = names[target.name.as_str()];
                        if crate::inlining::eligible(&input.functions[callee]) {
                            let args = stack.split_off(stack.len() - target.parameters.len());
                            stack.push(crate::inlining::emit(
                                &mut builder,
                                &input.functions[callee],
                                &args,
                            ));
                            builder.ins().jump(blocks[pc + 1], &block_args(&stack));
                            continue;
                        }
                        let reference = module.declare_func_in_func(ids[callee], builder.func);
                        let mut arguments = stack.split_off(stack.len() - target.parameters.len());
                        arguments.push(builder.ins().stack_addr(types::I64, call_result, 0));
                        let call = builder.ins().call(reference, &arguments);
                        let status = builder.inst_results(call)[0];
                        let failed = builder.ins().icmp_imm(IntCC::NotEqual, status, 0);
                        return_if(&mut builder, failed, status);
                        if !input.functions[callee].no_result {
                            stack.push(
                                builder.ins().stack_load(
                                    crate::scalar_abi::managed_scalar_type(
                                        &input.functions[callee].returns,
                                    )
                                    .unwrap(),
                                    call_result,
                                    0,
                                ),
                            );
                        }
                    }
                    Op::Fault(_) => {
                        // Terminal UserFault; the experimental ABI carries category only.
                        let status = builder.ins().iconst(types::I32, 4);
                        builder.ins().return_(&[status]);
                        continue;
                    }
                    Op::Return => {
                        if !function.no_result {
                            let result = stack.pop().expect("validated stack");
                            builder.ins().store(MemFlags::new(), result, output, 0);
                        }
                        let success = builder.ins().iconst(types::I32, 0);
                        builder.ins().return_(&[success]);
                        continue;
                    }
                    Op::Branch(target) => {
                        builder.ins().jump(blocks[*target], &block_args(&stack));
                        continue;
                    }
                    Op::BranchTrue(target) | Op::BranchFalse(target) => {
                        let value = stack.pop().expect("validated stack");
                        let condition = builder.ins().icmp_imm(
                            if matches!(op, Op::BranchTrue(_)) {
                                IntCC::NotEqual
                            } else {
                                IntCC::Equal
                            },
                            value,
                            0,
                        );
                        builder.ins().brif(
                            condition,
                            blocks[*target],
                            &block_args(&stack),
                            blocks[pc + 1],
                            &block_args(&stack),
                        );
                        continue;
                    }
                    _ if flow::comparison(op).is_some() => {
                        let (target, comparison) = flow::comparison(op).unwrap();
                        let right = stack.pop().expect("validated stack");
                        let left = stack.pop().expect("validated stack");
                        let condition = builder.ins().icmp(comparison, left, right);
                        builder.ins().brif(
                            condition,
                            blocks[target],
                            &block_args(&stack),
                            blocks[pc + 1],
                            &block_args(&stack),
                        );
                        continue;
                    }
                    _ => unreachable!("profile validation rejects unsupported instructions"),
                }
                builder.ins().jump(blocks[pc + 1], &block_args(&stack));
            }
            builder.seal_all_blocks();
            builder.finalize();
        }
        prepare_calls(&mut context, long_range_calls);
        module.define_function(ids[index], &mut context)?;
    }
    for (index, name) in exports {
        crate::native_exports::emit(module, &input.functions[*index], ids[*index], name)?;
    }
    if parameterless_root {
        // Preserve the v2 host ABI while calling the actual parameterless Raven entry.
        // Internal calls keep the function's declared signature and target its local symbol.
        let mut context = module.make_context();
        context.func.signature.params = vec![AbiParam::new(types::I32), AbiParam::new(types::I64)];
        context
            .func
            .signature
            .returns
            .push(AbiParam::new(types::I32));
        let export =
            module.declare_function("neoclr_entry_v2", Linkage::Export, &context.func.signature)?;
        let mut builder_context = FunctionBuilderContext::new();
        {
            let callee = module.declare_func_in_func(ids[root_index.unwrap()], &mut context.func);
            let mut builder = FunctionBuilder::new(&mut context.func, &mut builder_context);
            let entry = builder.create_block();
            builder.append_block_params_for_function_params(entry);
            builder.switch_to_block(entry);
            let output = builder.block_params(entry)[1];
            let call = builder.ins().call(callee, &[output]);
            let status = builder.inst_results(call)[0];
            builder.ins().return_(&[status]);
            builder.seal_all_blocks();
            builder.finalize();
        }
        prepare_calls(&mut context, long_range_calls);
        module.define_function(export, &mut context)?;
    }
    Ok(())
}

fn finish(module: ObjectModule) -> Result<Vec<u8>, Error> {
    let mut product = module.finish();
    // Baseline macOS ARM64 object, not a claim of testing every macOS version.
    // No SDK was used to compile these scalar functions (sdk = 0).
    let mut version = object::write::MachOBuildVersion::default();
    version.platform = object::macho::PLATFORM_MACOS;
    version.minos = 11 << 16;
    if product.object.format() == object::BinaryFormat::MachO {
        product.object.set_macho_build_version(version);
    }
    Ok(product.emit()?)
}

fn check_function(
    function: &Function,
    functions: &[Function],
    native_imports: bool,
) -> Result<flow::Stacks, Error> {
    let admitted = |t: &Type| {
        *t == Type::Int32 || native_imports && crate::scalar_abi::managed_scalar_type(t).is_some()
    };
    if function.owner.is_some()
        || function.instance
        || function.is_abstract
        || function.is_virtual
        || function.is_override
        || function.is_internal_call()
        || function.pinvoke.is_some()
        || if function.no_result {
            !native_imports || function.returns != Type::Void
        } else {
            !admitted(&function.returns)
        }
        || function.parameters.iter().any(|ty| !admitted(ty))
        || function.locals.iter().any(|ty| !admitted(ty))
        || function.locals.len() > 1024
        || !function.generic_parameters.is_empty()
        || !function.generic_arguments.is_empty()
        || !function.generic_constraints.is_empty()
        || !function.out_parameters.is_empty()
        || !function.out_when_true.is_empty()
        || !function.readonly_parameters.is_empty()
        || function.receiver_byref
        || function.receiver_readonly
        || function.impl_flags & !neoclr::metadata::INLINING_FLAGS != 0
        || function.body.len() > 8192
    {
        return Err(format!("{}: scalar profile requires nongeneric free functions with admitted scalar types (pointers require native-import opt-in)", function.name).into());
    }
    flow::analyze(function, functions, native_imports)
}

fn block_args(stack: &[ir::Value]) -> Vec<ir::BlockArg> {
    stack.iter().copied().map(Into::into).collect()
}

// Faults use status returns, never a guest Result or unwinding through C/Rust.
fn return_if(builder: &mut FunctionBuilder<'_>, failed: ir::Value, status: ir::Value) {
    return_if_detailed(builder, failed, status, None);
}

pub(super) fn return_if_detailed(
    builder: &mut FunctionBuilder<'_>,
    failed: ir::Value,
    status: ir::Value,
    site: Option<&crate::fault_details::Site>,
) {
    let fault = builder.create_block();
    let next = builder.create_block();
    builder.ins().brif(failed, fault, &[], next, &[]);
    builder.switch_to_block(fault);
    if let Some(site) = site {
        site.record(builder, status);
    }
    builder.ins().return_(&[status]);
    builder.switch_to_block(next);
}

fn checked_arithmetic(
    builder: &mut FunctionBuilder<'_>,
    op: &Op,
    left: ir::Value,
    right: ir::Value,
    site: Option<&crate::fault_details::Site>,
) -> ir::Value {
    let overflow_status = builder.ins().iconst(types::I32, 2);
    if matches!(
        op,
        Op::Divide | Op::DivideUnsigned | Op::Remainder | Op::RemainderUnsigned
    ) {
        let zero = builder.ins().icmp_imm(IntCC::Equal, right, 0);
        let zero_status = builder.ins().iconst(types::I32, 1);
        return_if_detailed(builder, zero, zero_status, site);
        if matches!(op, Op::Divide | Op::Remainder) {
            let minimum = if builder.func.dfg.value_type(left) == types::I64 {
                i64::MIN
            } else {
                i64::from(i32::MIN)
            };
            let min = builder.ins().icmp_imm(IntCC::Equal, left, minimum);
            let negative_one = builder.ins().icmp_imm(IntCC::Equal, right, -1);
            let overflow = builder.ins().band(min, negative_one);
            return_if_detailed(builder, overflow, overflow_status, site);
        }
        return match op {
            Op::Divide => builder.ins().sdiv(left, right),
            Op::DivideUnsigned => builder.ins().udiv(left, right),
            Op::Remainder => builder.ins().srem(left, right),
            Op::RemainderUnsigned => builder.ins().urem(left, right),
            _ => unreachable!(),
        };
    }
    // Keep the operation at its evaluation width. Widening through I64 only
    // detects I32 overflow and cannot represent an I64 product or carry.
    let (result, overflow) = match op {
        Op::AddChecked => builder.ins().sadd_overflow(left, right),
        Op::SubChecked => builder.ins().ssub_overflow(left, right),
        Op::MulChecked => builder.ins().smul_overflow(left, right),
        Op::AddCheckedUnsigned => builder.ins().uadd_overflow(left, right),
        Op::SubCheckedUnsigned => builder.ins().usub_overflow(left, right),
        Op::MulCheckedUnsigned => builder.ins().umul_overflow(left, right),
        _ => unreachable!(),
    };
    return_if_detailed(builder, overflow, overflow_status, site);
    result
}

fn runtime_console_call(target: &FunctionRef) -> bool {
    target.name == "neoCLR.Runtime.WriteLine"
        && target.parameters == [Type::String]
        && target.owner.is_none()
        && target.definition.is_none()
        && !target.instance
        && target.generic_arguments.is_empty()
}

// This is a backend intrinsic for the exact bundled wrapper, not a name-only
// replacement for arbitrary library code or a .NET System.Console implementation.
fn public_console_call(target: &FunctionRef) -> bool {
    target.name == "System.Console.WriteLine"
        && target.owner == Some(Type::Named("System.Console".into()))
        && target.parameters == [Type::String]
        && target.definition.is_none()
        && !target.instance
        && target.generic_arguments.is_empty()
}

fn console_call(target: &FunctionRef) -> bool {
    runtime_console_call(target) || public_console_call(target)
}

fn check_console_wrapper() -> Result<(), Error> {
    let library = neoclr::library::system().map_err(|error| error.to_string())?;
    let candidates: Vec<_> = library
        .functions
        .iter()
        .filter(|f| f.name == "System.Console.WriteLine" && f.parameters == [Type::String])
        .collect();
    let [wrapper] = candidates.as_slice() else {
        return Err("bundled console wrapper is missing or ambiguous".into());
    };
    if wrapper.owner != Some(Type::Named("System.Console".into()))
        || wrapper.instance
        || wrapper.returns != Type::Void
        || wrapper.no_result
        || wrapper.impl_flags & !neoclr::metadata::INLINING_FLAGS != 0
        || wrapper.pinvoke.is_some()
        || !wrapper.locals.is_empty()
        || !wrapper.generic_parameters.is_empty()
        || !matches!(wrapper.body.as_slice(), [Op::Arg(0), Op::Call(call), Op::Return] if runtime_console_call(call))
    {
        return Err("bundled console wrapper changed; review native console lowering".into());
    }
    Ok(())
}

// Independently allocated JIT functions and host helpers need not fit ARM64's
// direct branch range. AOT retains its existing linker-managed relocation policy.
pub(super) fn prepare_calls(context: &mut cranelift_codegen::Context, long_range: bool) {
    if long_range {
        for function in context.func.dfg.ext_funcs.values_mut() {
            function.colocated = false;
        }
    }
}
