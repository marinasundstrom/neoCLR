//! Experimental scalar lowering, deliberately separate from the public runtime API.
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::{
    ir::{self, AbiParam, InstBuilder, MemFlags, StackSlotData, StackSlotKind, types},
    isa, settings,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};

#[path = "flow.rs"]
mod flow;
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
    let mut flows = Vec::new();
    for (index, function) in input.functions.iter().enumerate() {
        if names.insert(function.name.as_str(), index).is_some() {
            return Err("scalar profile does not support overloaded names".into());
        }
        flows.push(check_function(function)?);
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
        sig.params.push(AbiParam::new(types::I64)); // writable Int32 result pointer
        sig.returns.push(AbiParam::new(types::I32)); // status: 0 success, 1 zero, 2 overflow
        let (name, linkage) = if index == root_index {
            ("neoclr_entry_v2".to_owned(), Linkage::Export)
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
            let output = args[function.parameters.len()];
            let call_result = builder.create_sized_stack_slot(StackSlotData::new(
                StackSlotKind::ExplicitSlot,
                4,
                2,
            ));
            let blocks: Vec<_> = function
                .body
                .iter()
                .map(|_| builder.create_block())
                .collect();
            for (pc, depth) in flows[index].iter().enumerate() {
                if let Some(depth) = depth {
                    for _ in 0..*depth {
                        builder.append_block_param(blocks[pc], types::I32);
                    }
                }
            }
            for local in 0..function.locals.len() {
                builder.declare_var(Variable::from_u32(local as u32), types::I32);
            }
            builder.ins().jump(blocks[0], &[]);
            for (pc, op) in function.body.iter().enumerate() {
                if flows[index][pc].is_none() {
                    continue;
                }
                builder.switch_to_block(blocks[pc]);
                // SSA local parameters may be appended while translating predecessors;
                // only the explicit first parameters represent the IL operand stack.
                let mut stack: Vec<ir::Value> =
                    builder.block_params(blocks[pc])[..flows[index][pc].unwrap()].to_vec();
                match op {
                    Op::Int(value) => {
                        stack.push(builder.ins().iconst(types::I32, i64::from(*value)))
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
                        stack.push(checked_arithmetic(&mut builder, op, left, right));
                    }
                    Op::Call(target) => {
                        let callee = names[target.name.as_str()];
                        let reference = module.declare_func_in_func(ids[callee], builder.func);
                        let mut arguments = stack.split_off(stack.len() - target.parameters.len());
                        arguments.push(builder.ins().stack_addr(types::I64, call_result, 0));
                        let call = builder.ins().call(reference, &arguments);
                        let status = builder.inst_results(call)[0];
                        let failed = builder.ins().icmp_imm(IntCC::NotEqual, status, 0);
                        return_if(&mut builder, failed, status);
                        stack.push(builder.ins().stack_load(types::I32, call_result, 0));
                    }
                    Op::Return => {
                        let result = stack.pop().expect("validated stack");
                        builder.ins().store(MemFlags::new(), result, output, 0);
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

fn check_function(function: &Function) -> Result<Vec<Option<usize>>, Error> {
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
        || function.locals.iter().any(|ty| *ty != Type::Int32)
        || function.locals.len() > 1024
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
        return Err(format!("{}: scalar profile requires nongeneric free Int32 functions with Int32 locals and no native imports", function.name).into());
    }
    flow::analyze(function)
}

fn block_args(stack: &[ir::Value]) -> Vec<ir::BlockArg> {
    stack.iter().copied().map(Into::into).collect()
}

// Faults use status returns, never a guest Result or unwinding through C/Rust.
fn return_if(builder: &mut FunctionBuilder<'_>, failed: ir::Value, status: ir::Value) {
    let fault = builder.create_block();
    let next = builder.create_block();
    builder.ins().brif(failed, fault, &[], next, &[]);
    builder.switch_to_block(fault);
    builder.ins().return_(&[status]);
    builder.switch_to_block(next);
}

fn checked_arithmetic(
    builder: &mut FunctionBuilder<'_>,
    op: &Op,
    left: ir::Value,
    right: ir::Value,
) -> ir::Value {
    let overflow_status = builder.ins().iconst(types::I32, 2);
    if matches!(
        op,
        Op::Divide | Op::DivideUnsigned | Op::Remainder | Op::RemainderUnsigned
    ) {
        let zero = builder.ins().icmp_imm(IntCC::Equal, right, 0);
        let zero_status = builder.ins().iconst(types::I32, 1);
        return_if(builder, zero, zero_status);
        if matches!(op, Op::Divide | Op::Remainder) {
            let min = builder
                .ins()
                .icmp_imm(IntCC::Equal, left, i64::from(i32::MIN));
            let negative_one = builder.ins().icmp_imm(IntCC::Equal, right, -1);
            let overflow = builder.ins().band(min, negative_one);
            return_if(builder, overflow, overflow_status);
        }
        return match op {
            Op::Divide => builder.ins().sdiv(left, right),
            Op::DivideUnsigned => builder.ins().udiv(left, right),
            Op::Remainder => builder.ins().srem(left, right),
            Op::RemainderUnsigned => builder.ins().urem(left, right),
            _ => unreachable!(),
        };
    }
    let unsigned = matches!(
        op,
        Op::AddCheckedUnsigned | Op::SubCheckedUnsigned | Op::MulCheckedUnsigned
    );
    let (left, right) = if unsigned {
        (
            builder.ins().uextend(types::I64, left),
            builder.ins().uextend(types::I64, right),
        )
    } else {
        (
            builder.ins().sextend(types::I64, left),
            builder.ins().sextend(types::I64, right),
        )
    };
    let wide = match op {
        Op::AddChecked | Op::AddCheckedUnsigned => builder.ins().iadd(left, right),
        Op::SubChecked | Op::SubCheckedUnsigned => builder.ins().isub(left, right),
        Op::MulChecked | Op::MulCheckedUnsigned => builder.ins().imul(left, right),
        _ => unreachable!(),
    };
    let result = builder.ins().ireduce(types::I32, wide);
    let restored = if unsigned {
        builder.ins().uextend(types::I64, result)
    } else {
        builder.ins().sextend(types::I64, result)
    };
    let overflow = builder.ins().icmp(IntCC::NotEqual, wide, restored);
    return_if(builder, overflow, overflow_status);
    result
}
