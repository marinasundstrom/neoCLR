//! Experimental scalar lowering, deliberately separate from the public runtime API.
use cranelift_codegen::{
    ir::{self, AbiParam, InstBuilder, types},
    isa, settings,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use neoclr::metadata::{Function, Instruction as Op, Type};
use std::collections::{HashMap, HashSet};

type Error = Box<dyn std::error::Error>;

pub(super) fn compile(source: &str, root: &str) -> Result<Vec<u8>, Error> {
    let input = neoclr::assemble(source).map_err(|error| error.to_string())?;
    if input.name == "System" || !input.types.is_empty() || input.functions.len() > 128 {
        return Err(
            "scalar profile requires an application with no types and at most 128 functions".into(),
        );
    }
    let mut names = HashMap::new();
    for (index, function) in input.functions.iter().enumerate() {
        if names.insert(function.name.as_str(), index).is_some() {
            return Err("scalar profile does not support overloaded names".into());
        }
        check_function(function)?;
    }
    let root_index = *names.get(root).ok_or("root function not found")?;
    if input.functions[root_index].parameters != [Type::Int32] {
        return Err("root must have signature (Int32) -> Int32".into());
    }
    // Validate every declared body, including unreachable functions/instructions.
    // Resolve only unqualified local calls; no ad-hoc cross-module name lookup.
    let mut edges = vec![vec![]; input.functions.len()];
    for (index, function) in input.functions.iter().enumerate() {
        for (pc, op) in function.body.iter().enumerate() {
            if let Op::Call(target) = op {
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
    // Native stack budgets/Fault propagation are not implemented: reject cycles.
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
    neoclr::LoadedProgram::new(&input)
        .and_then(|program| program.verify())
        .map_err(|error| error.to_string())?;

    let target = "aarch64-apple-darwin"
        .parse()
        .expect("fixed valid target triple");
    let isa = isa::lookup(target)?.finish(settings::Flags::new(settings::builder()))?;
    let mut module = ObjectModule::new(ObjectBuilder::new(
        isa,
        "neoclr_scalar",
        cranelift_module::default_libcall_names(),
    )?);
    let mut ids = Vec::new();
    for (index, function) in input.functions.iter().enumerate() {
        let mut sig = module.make_signature();
        sig.params = vec![AbiParam::new(types::I32); function.parameters.len()];
        sig.returns.push(AbiParam::new(types::I32));
        let (name, linkage) = if index == root_index {
            ("neoclr_entry".to_owned(), Linkage::Export)
        } else {
            (format!("neoclr_scalar_{index}"), Linkage::Local)
        };
        ids.push(module.declare_function(&name, linkage, &sig)?);
    }
    for (index, function) in input.functions.iter().enumerate() {
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
            let mut stack: Vec<ir::Value> = Vec::new();
            for op in &function.body {
                match op {
                    Op::Int(value) => {
                        stack.push(builder.ins().iconst(types::I32, i64::from(*value)))
                    }
                    Op::Arg(index) => stack.push(args[*index]),
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
                    Op::Call(target) => {
                        let callee = names[target.name.as_str()];
                        let reference = module.declare_func_in_func(ids[callee], builder.func);
                        let arguments = stack.split_off(stack.len() - target.parameters.len());
                        let call = builder.ins().call(reference, &arguments);
                        stack.push(builder.inst_results(call)[0]);
                    }
                    Op::Return => {
                        let result = stack.pop().expect("validated stack");
                        builder.ins().return_(&[result]);
                    }
                    _ => unreachable!("profile validation rejects unsupported instructions"),
                }
            }
            builder.seal_all_blocks();
            builder.finalize();
        }
        module.define_function(ids[index], &mut context)?;
    }
    let mut product = module.finish();
    // Baseline macOS ARM64 object, not a claim of testing every macOS version.
    // No SDK was used to compile these scalar functions (sdk = 0).
    let mut version = object::write::MachOBuildVersion::default();
    version.platform = object::macho::PLATFORM_MACOS;
    version.minos = 11 << 16;
    product.object.set_macho_build_version(version);
    Ok(product.emit()?)
}

fn check_function(function: &Function) -> Result<(), Error> {
    if function.owner.is_some()
        || function.instance
        || function.is_abstract
        || function.is_virtual
        || function.is_override
        || function.is_internal_call()
        || function.pinvoke.is_some()
        || function.no_result
        || function.returns != Type::Int32
        || function.parameters.iter().any(|ty| *ty != Type::Int32)
        || !function.locals.is_empty()
        || !function.generic_parameters.is_empty()
        || !function.generic_arguments.is_empty()
        || !function.generic_constraints.is_empty()
        || !function.out_parameters.is_empty()
        || !function.out_when_true.is_empty()
        || !function.readonly_parameters.is_empty()
        || function.receiver_byref
        || function.receiver_readonly
        || function.impl_flags != 0
        || function.body.len() > 8192
    {
        return Err(format!("{}: scalar profile requires nongeneric free Int32 functions without locals or native imports", function.name).into());
    }
    let mut depth = 0usize;
    for (pc, op) in function.body.iter().enumerate() {
        let (pops, pushes) = match op {
            Op::Int(_) => (0, 1),
            Op::Arg(index) if *index < function.parameters.len() => (0, 1),
            Op::Dup => (1, 2),
            Op::Pop => (1, 0),
            Op::Add | Op::Sub | Op::Mul => (2, 1),
            Op::Call(target) => (target.parameters.len(), 1),
            Op::Return if pc + 1 == function.body.len() && depth == 1 => (1, 0),
            _ => {
                return Err(format!(
                    "{} instruction {pc}: unsupported instruction or invalid return: {op:?}",
                    function.name
                )
                .into());
            }
        };
        if depth < pops {
            return Err(format!("{} instruction {pc}: stack underflow", function.name).into());
        }
        depth = depth - pops + pushes;
    }
    if !matches!(function.body.last(), Some(Op::Return)) {
        return Err(format!("{}: missing final return", function.name).into());
    }
    Ok(())
}
