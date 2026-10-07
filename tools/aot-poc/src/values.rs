//! Inline value/member lowering. Records travel as field snapshots, not owning pointers.
use super::{Error, checked_arithmetic, flow, return_if};
#[path = "value_profile.rs"]
mod profile;
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::settings::Configurable;
use cranelift_codegen::{
    ir::{self, AbiParam, InstBuilder, MemFlags, StackSlotData, StackSlotKind, types},
    isa, settings,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use neoclr::metadata::Instruction as Op;
use profile::{Profile, Ty};

fn lane(t: &Ty) -> ir::Type {
    if matches!(t, Ty::Address(_)) {
        types::I64
    } else {
        types::I32
    }
}
fn read(
    b: &mut FunctionBuilder<'_>,
    p: &Profile<'_>,
    t: &Ty,
    pointer: ir::Value,
) -> Vec<ir::Value> {
    (0..p.lanes(t))
        .map(|i| {
            b.ins()
                .load(lane(t), MemFlags::new(), pointer, i as i32 * 4)
        })
        .collect()
}
fn write(b: &mut FunctionBuilder<'_>, pointer: ir::Value, values: &[ir::Value]) {
    for (i, value) in values.iter().enumerate() {
        b.ins()
            .store(MemFlags::new(), *value, pointer, i as i32 * 4);
    }
}
fn normalize(
    b: &mut FunctionBuilder<'_>,
    p: &Profile<'_>,
    t: &Ty,
    values: &[ir::Value],
) -> Vec<ir::Value> {
    values
        .iter()
        .zip(p.byte_lanes(t))
        .map(
            |(v, byte)| {
                if byte { b.ins().band_imm(*v, 255) } else { *v }
            },
        )
        .collect()
}
fn write_typed(
    b: &mut FunctionBuilder<'_>,
    p: &Profile<'_>,
    t: &Ty,
    pointer: ir::Value,
    values: &[ir::Value],
) {
    let values = normalize(b, p, t, values);
    write(b, pointer, &values);
}
fn slot(b: &mut FunctionBuilder<'_>, bytes: u32) -> ir::StackSlot {
    b.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, bytes, 3))
}
fn args(values: &[ir::Value]) -> Vec<ir::BlockArg> {
    values.iter().copied().map(Into::into).collect()
}

pub(super) fn compile(input: &neoclr::Module, root: &str) -> Result<Vec<u8>, Error> {
    let p = Profile::new(input)?;
    let root = p.root(root)?;
    let flows: Vec<_> = (0..input.functions.len())
        .map(|i| p.analyze(i))
        .collect::<Result<_, _>>()?;
    // The backend's narrow shape analysis is additional admission, not a replacement
    // for type/member identity, accessibility, initialization or byref lifetime checks.
    neoclr::LoadedProgram::new(input)
        .and_then(|v| v.verify())
        .map_err(|e| e.to_string())?;
    let mut flags = settings::builder();
    flags.set("is_pic", "true")?;
    let isa = isa::lookup("aarch64-apple-darwin".parse().unwrap())?
        .finish(settings::Flags::new(flags))?;
    let mut module = ObjectModule::new(ObjectBuilder::new(
        isa,
        "neoclr_values",
        cranelift_module::default_libcall_names(),
    )?);
    let mut ids = vec![];
    for (i, _) in input.functions.iter().enumerate() {
        let mut sig = module.make_signature();
        for t in &p.args[i] {
            sig.params
                .extend((0..p.lanes(t)).map(|_| AbiParam::new(lane(t))));
        }
        sig.params.push(AbiParam::new(types::I64)); // caller-owned result storage
        sig.returns.push(AbiParam::new(types::I32)); // Fault status, same as scalar profile
        ids.push(module.declare_function(&format!("neoclr_value_{i}"), Linkage::Local, &sig)?);
    }
    for (i, f) in input.functions.iter().enumerate() {
        let mut context = module.make_context();
        context.func.signature = module
            .declarations()
            .get_function_decl(ids[i])
            .signature
            .clone();
        let mut fb = FunctionBuilderContext::new();
        {
            let mut b = FunctionBuilder::new(&mut context.func, &mut fb);
            let entry = b.create_block();
            b.append_block_params_for_function_params(entry);
            b.switch_to_block(entry);
            let parameters = b.block_params(entry).to_vec();
            let output = *parameters.last().unwrap();
            let mut frame_bytes = 32usize;
            let mut arguments = vec![];
            let mut at = 0;
            for t in &p.args[i] {
                let s = slot(&mut b, p.bytes(t));
                frame_bytes += p.bytes(t) as usize + 7;
                let address = b.ins().stack_addr(types::I64, s, 0);
                write_typed(&mut b, &p, t, address, &parameters[at..at + p.lanes(t)]);
                arguments.push(s);
                at += p.lanes(t);
            }
            let locals: Vec<_> = p.locals[i]
                .iter()
                .map(|t| {
                    frame_bytes += p.bytes(t) as usize + 7;
                    slot(&mut b, p.bytes(t))
                })
                .collect();
            // All profile return values fit in eight 32-bit lanes; snapshots are read
            // immediately after each successful call, before this storage can be reused.
            let call_result = slot(&mut b, 32);
            let mut constructors = std::collections::HashMap::new();
            for (pc, op) in f.body.iter().enumerate() {
                if let Op::Construct(target) = op {
                    let bytes = p.bytes(&p.ty(target.owner.as_ref().unwrap())?);
                    frame_bytes += bytes as usize + 7;
                    constructors.insert(pc, slot(&mut b, bytes));
                }
            }
            if frame_bytes > 65536 {
                return Err("value profile frame storage exceeds 64 KiB".into());
            }
            let blocks: Vec<_> = f.body.iter().map(|_| b.create_block()).collect();
            for (pc, stack) in flows[i].iter().enumerate() {
                if let Some(stack) = stack {
                    for t in stack {
                        for _ in 0..p.lanes(t) {
                            b.append_block_param(blocks[pc], lane(t));
                        }
                    }
                }
            }
            b.ins().jump(blocks[0], &[]);
            for (pc, op) in f.body.iter().enumerate() {
                let Some(shape) = &flows[i][pc] else {
                    continue;
                };
                b.switch_to_block(blocks[pc]);
                let mut stack = b.block_params(blocks[pc]).to_vec();
                let top = || shape.last().unwrap();
                let pop = |s: &mut Vec<ir::Value>| s.pop().expect("checked stack");
                match op {
                    Op::Int(v) => stack.push(b.ins().iconst(types::I32, i64::from(*v))),
                    Op::ConvertInt32 => (),
                    Op::ConvertUInt8 => {
                        let value = pop(&mut stack);
                        stack.push(b.ins().band_imm(value, 255));
                    }
                    Op::Bool(v) => stack.push(b.ins().iconst(types::I32, i64::from(*v))),
                    Op::Void => stack.push(b.ins().iconst(types::I32, 0)),
                    Op::PackValue(t) => {
                        let payload = pop(&mut stack);
                        let payload = normalize(&mut b, &p, &p.ty(t)?, &[payload])[0];
                        stack.push(b.ins().iconst(types::I32, profile::erased_tag(t)?));
                        stack.push(payload);
                    }
                    Op::IsValue(t) | Op::UnpackValue(t) => {
                        let payload = pop(&mut stack);
                        let tag = pop(&mut stack);
                        let matches = b.ins().icmp_imm(IntCC::Equal, tag, profile::erased_tag(t)?);
                        if matches!(op, Op::IsValue(_)) {
                            stack.push(b.ins().uextend(types::I32, matches));
                        } else {
                            let wrong = b.ins().icmp_imm(IntCC::Equal, matches, 0);
                            let status = b.ins().iconst(types::I32, 3); // RuntimeError
                            return_if(&mut b, wrong, status);
                            stack.push(payload);
                        }
                    }
                    Op::Arg(n) | Op::Load(n) => {
                        let (s, t) = if matches!(op, Op::Arg(_)) {
                            (arguments[*n], &p.args[i][*n])
                        } else {
                            (locals[*n], &p.locals[i][*n])
                        };
                        let address = b.ins().stack_addr(types::I64, s, 0);
                        stack.extend(read(&mut b, &p, t, address));
                    }
                    Op::ArgumentAddress(n) | Op::LocalAddress(n) => {
                        let s = if matches!(op, Op::ArgumentAddress(_)) {
                            arguments[*n]
                        } else {
                            locals[*n]
                        };
                        stack.push(b.ins().stack_addr(types::I64, s, 0));
                    }
                    Op::StoreArg(n) | Op::Store(n) => {
                        let s = if matches!(op, Op::StoreArg(_)) {
                            arguments[*n]
                        } else {
                            locals[*n]
                        };
                        let value = stack.split_off(stack.len() - p.lanes(top()));
                        let address = b.ins().stack_addr(types::I64, s, 0);
                        let ty = if matches!(op, Op::StoreArg(_)) {
                            &p.args[i][*n]
                        } else {
                            &p.locals[i][*n]
                        };
                        write_typed(&mut b, &p, ty, address, &value);
                    }
                    Op::Dup => {
                        let value = stack[stack.len() - p.lanes(top())..].to_vec();
                        stack.extend(value);
                    }
                    Op::Pop => {
                        stack.truncate(stack.len() - p.lanes(top()));
                    }
                    Op::InitializeObject(t) => {
                        let address = pop(&mut stack);
                        let ty = p.ty(t)?;
                        let zero = b.ins().iconst(types::I32, 0);
                        write(&mut b, address, &vec![zero; p.lanes(&ty)]);
                    }
                    Op::LoadObject(t) => {
                        let address = pop(&mut stack);
                        stack.extend(read(&mut b, &p, &p.ty(t)?, address));
                    }
                    Op::StoreObject(t) => {
                        let value = stack.split_off(stack.len() - p.lanes(&p.ty(t)?));
                        let address = pop(&mut stack);
                        write_typed(&mut b, &p, &p.ty(t)?, address, &value);
                    }
                    Op::New(t) => {
                        let Ty::Record(owner) = p.ty(t)? else {
                            unreachable!()
                        };
                        // The fields are already snapshots in declaration order.
                        if input.types[owner].fields.is_empty() {
                            stack.push(b.ins().iconst(types::I32, 0));
                        }
                        let ty = Ty::Record(owner);
                        let value = stack.split_off(stack.len() - p.lanes(&ty));
                        stack.extend(normalize(&mut b, &p, &ty, &value));
                    }
                    Op::Field(n) | Op::FieldAddress(n) => {
                        let field = p.field(top(), *n)?;
                        let offset = p.field_offset(top(), *n);
                        if matches!(top(), Ty::Address(_)) {
                            let owner = pop(&mut stack);
                            let address = b.ins().iadd_imm(owner, (offset * 4) as i64);
                            if matches!(op, Op::FieldAddress(_)) {
                                stack.push(address);
                            } else {
                                stack.extend(read(&mut b, &p, &field, address));
                            }
                        } else {
                            let value = stack.split_off(stack.len() - p.lanes(top()));
                            stack.extend_from_slice(&value[offset..offset + p.lanes(&field)]);
                        }
                    }
                    Op::SetField(n) => {
                        let value = stack.split_off(stack.len() - p.lanes(top()));
                        let owner = &shape[shape.len() - 2];
                        let offset = p.field_offset(owner, *n);
                        let value = normalize(&mut b, &p, &p.field(owner, *n)?, &value);
                        if matches!(owner, Ty::Address(_)) {
                            let owner = pop(&mut stack);
                            let address = b.ins().iadd_imm(owner, (offset * 4) as i64);
                            write(&mut b, address, &value);
                            stack.push(b.ins().iconst(types::I32, 0)); // inhabited Void
                        } else {
                            let start = stack.len() - p.lanes(owner) + offset;
                            stack[start..start + value.len()].copy_from_slice(&value);
                        }
                    }
                    Op::Call(target) | Op::Construct(target) => {
                        let c = p.callee(target)?;
                        let construct = matches!(op, Op::Construct(_));
                        let count: usize = p.args[c]
                            .iter()
                            .skip(usize::from(construct))
                            .map(|t| p.lanes(t))
                            .sum();
                        let mut call_args = stack.split_off(stack.len() - count);
                        let constructed = if construct {
                            let t = p.ty(target.owner.as_ref().unwrap())?;
                            let address = b.ins().stack_addr(types::I64, constructors[&pc], 0);
                            let zero = b.ins().iconst(types::I32, 0);
                            write(&mut b, address, &vec![zero; p.lanes(&t)]);
                            call_args.insert(0, address);
                            Some((t, address))
                        } else {
                            None
                        };
                        let result = b.ins().stack_addr(types::I64, call_result, 0);
                        call_args.push(result);
                        let target = module.declare_func_in_func(ids[c], b.func);
                        let call = b.ins().call(target, &call_args);
                        let status = b.inst_results(call)[0];
                        let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                        return_if(&mut b, failed, status);
                        if let Some((t, address)) = constructed {
                            stack.extend(read(&mut b, &p, &t, address));
                        } else if let Some(t) = &p.results[c] {
                            stack.extend(read(&mut b, &p, t, result));
                        }
                    }
                    Op::Fault(_) => {
                        // Terminal UserFault; the experimental ABI carries category only.
                        let status = b.ins().iconst(types::I32, 4);
                        b.ins().return_(&[status]);
                        continue;
                    }
                    Op::Return => {
                        if let Some(t) = &p.results[i] {
                            write_typed(&mut b, &p, t, output, &stack);
                        }
                        let success = b.ins().iconst(types::I32, 0);
                        b.ins().return_(&[success]);
                        continue;
                    }
                    Op::Add | Op::Sub | Op::Mul => {
                        let r = pop(&mut stack);
                        let l = pop(&mut stack);
                        stack.push(match op {
                            Op::Add => b.ins().iadd(l, r),
                            Op::Sub => b.ins().isub(l, r),
                            _ => b.ins().imul(l, r),
                        });
                    }
                    Op::Equal | Op::Greater | Op::GreaterUnsigned | Op::Less | Op::LessUnsigned => {
                        let r = pop(&mut stack);
                        let l = pop(&mut stack);
                        let cc = match op {
                            Op::Equal => IntCC::Equal,
                            Op::Greater => IntCC::SignedGreaterThan,
                            Op::GreaterUnsigned => IntCC::UnsignedGreaterThan,
                            Op::Less => IntCC::SignedLessThan,
                            _ => IntCC::UnsignedLessThan,
                        };
                        let bool8 = b.ins().icmp(cc, l, r);
                        stack.push(b.ins().uextend(types::I32, bool8));
                    }
                    Op::Branch(n) => {
                        b.ins().jump(blocks[*n], &args(&stack));
                        continue;
                    }
                    Op::BranchTrue(n) | Op::BranchFalse(n) => {
                        let v = pop(&mut stack);
                        let cc = if matches!(op, Op::BranchTrue(_)) {
                            IntCC::NotEqual
                        } else {
                            IntCC::Equal
                        };
                        let condition = b.ins().icmp_imm(cc, v, 0);
                        b.ins().brif(
                            condition,
                            blocks[*n],
                            &args(&stack),
                            blocks[pc + 1],
                            &args(&stack),
                        );
                        continue;
                    }
                    _ if flow::comparison(op).is_some() => {
                        let (n, cc) = flow::comparison(op).unwrap();
                        let r = pop(&mut stack);
                        let l = pop(&mut stack);
                        let condition = b.ins().icmp(cc, l, r);
                        b.ins().brif(
                            condition,
                            blocks[n],
                            &args(&stack),
                            blocks[pc + 1],
                            &args(&stack),
                        );
                        continue;
                    }
                    _ => {
                        let r = pop(&mut stack);
                        let l = pop(&mut stack);
                        stack.push(checked_arithmetic(&mut b, op, l, r));
                    }
                }
                b.ins().jump(blocks[pc + 1], &args(&stack));
            }
            b.seal_all_blocks();
            b.finalize();
        }
        module.define_function(ids[i], &mut context)?;
    }
    // Only the stable experiment C entry is exported. Private aggregate signatures
    // are intentionally not a public ARM64 struct ABI.
    let mut context = module.make_context();
    context.func.signature.params = vec![AbiParam::new(types::I32), AbiParam::new(types::I64)];
    context
        .func
        .signature
        .returns
        .push(AbiParam::new(types::I32));
    let export =
        module.declare_function("neoclr_entry_v2", Linkage::Export, &context.func.signature)?;
    let target = module.declare_func_in_func(ids[root], &mut context.func);
    let mut fb = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut context.func, &mut fb);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let params = b.block_params(entry).to_vec();
        let call_args = if p.args[root].is_empty() {
            &params[1..]
        } else {
            &params[..]
        };
        let call = b.ins().call(target, call_args);
        let status = b.inst_results(call)[0];
        b.ins().return_(&[status]);
        b.seal_all_blocks();
        b.finalize();
    }
    module.define_function(export, &mut context)?;
    super::finish(module)
}
