//! Inline value/member lowering. Records travel as field snapshots, not owning pointers.
use super::{Error, checked_arithmetic, flow, return_if_detailed};
#[path = "value_profile.rs"]
mod profile;
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::settings::Configurable;
use cranelift_codegen::{
    ir::{self, AbiParam, InstBuilder, MemFlags, StackSlotData, StackSlotKind, types},
    isa, settings,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{DataDescription, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use neoclr::metadata::Instruction as Op;
use profile::{Profile, Ty};

fn lanes(p: &Profile<'_>, t: &Ty) -> Vec<ir::Type> {
    p.pointer_lanes(t).into_iter().map(|pointer| if pointer { types::I64 } else { types::I32 }).collect()
}
fn read(
    b: &mut FunctionBuilder<'_>,
    p: &Profile<'_>,
    t: &Ty,
    pointer: ir::Value,
) -> Vec<ir::Value> {
    lanes(p, t).into_iter().zip(p.byte_lanes(t)).enumerate()
        .map(|(i, (ty, byte))| {
            let value = b.ins().load(if byte { types::I8 } else { ty }, MemFlags::new(), pointer, i as i32 * 8);
            if byte { b.ins().uextend(types::I32, value) } else { value }
        })
        .collect()
}
fn write(b: &mut FunctionBuilder<'_>, pointer: ir::Value, values: &[ir::Value]) {
    for (i, value) in values.iter().enumerate() {
        b.ins()
            .store(MemFlags::new(), *value, pointer, i as i32 * 8);
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
    let values: Vec<_> = normalize(b, p, t, values).into_iter().zip(p.byte_lanes(t))
        .map(|(value, byte)| if byte { b.ins().ireduce(types::I8, value) } else { value }).collect();
    write(b, pointer, &values);
}
fn slot(b: &mut FunctionBuilder<'_>, bytes: u32) -> ir::StackSlot {
    b.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, bytes, 3))
}
fn args(values: &[ir::Value]) -> Vec<ir::BlockArg> {
    values.iter().copied().map(Into::into).collect()
}

fn null_reference(b: &mut FunctionBuilder<'_>, pointer: ir::Value, site: Option<&crate::fault_details::Site>) {
    let null = b.ins().icmp_imm(IntCC::Equal, pointer, 0);
    let status = b.ins().iconst(types::I32, 6);
    return_if_detailed(b, null, status, site);
}

pub(super) fn compile(input: &neoclr::Module, root: &str, details: Option<&crate::fault_details::Options>) -> Result<Vec<u8>, Error> {
    let references = details.is_some_and(|d| d.reference_arena);
    let p = Profile::new(input, references)?;
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
    // Immutable length-prefixed UTF-8. Strings originate from these image literals
    // or explicit, bounded invocation-arena producers. No arbitrary host String,
    // null/default, erasure, object storage or escaping export is admitted.
    // A default neoCLR Char is the single NUL grapheme, never a null pointer.
    let default_character = module.declare_data("neoclr_default_character", Linkage::Local, false, false)?;
    let mut character_data = DataDescription::new();
    let mut character_bytes = 1u64.to_le_bytes().to_vec();
    character_bytes.push(0);
    character_data.define(character_bytes.into_boxed_slice());
    character_data.set_align(8);
    module.define_data(default_character, &character_data)?;
    let mut literals = std::collections::HashMap::new();
    for (i, f) in input.functions.iter().enumerate() {
        for (pc, op) in f.body.iter().enumerate() {
            if let Op::String(text) = op {
                let id = module.declare_data(&format!("neoclr_literal_{i}_{pc}"), Linkage::Local, false, false)?;
                let mut data = DataDescription::new();
                let mut bytes = (text.len() as u64).to_le_bytes().to_vec();
                bytes.extend_from_slice(text.as_bytes());
                data.define(bytes.into_boxed_slice());
                data.set_align(8);
                module.define_data(id, &data)?;
                literals.insert((i, pc), id);
            }
        }
    }
    let diagnostic_data = details.map(|options| crate::fault_details::Data::new(&mut module, input, options)).transpose()?;
    let input_service = if details.is_some_and(|d| !d.console_read_byte.is_empty()) {
        let mut sig = module.make_signature();
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_console_read_byte_v1", Linkage::Import, &sig)?)
    } else { None };
    let output_service = if details.is_some_and(|d| !d.console_write_line.is_empty()) {
        let mut sig = module.make_signature();
        sig.params.extend([AbiParam::new(types::I64), AbiParam::new(types::I64)]);
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_console_write_line_utf8_v1", Linkage::Import, &sig)?)
    } else { None };
    let text_arena = references || details.is_some_and(|d| !d.int32_to_string.is_empty());
    let format_service = if details.is_some_and(|d| !d.int32_to_string.is_empty()) {
        let mut sig = module.make_signature();
        sig.params.extend([AbiParam::new(types::I32), AbiParam::new(types::I64), AbiParam::new(types::I64)]);
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_int32_to_string_v1", Linkage::Import, &sig)?)
    } else { None };
    let character_service = if details.is_some_and(|d| !d.char_from_string.is_empty()) {
        let mut sig = module.make_signature();
        sig.params.extend([types::I64, types::I64].map(AbiParam::new));
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_is_single_grapheme_v1", Linkage::Import, &sig)?)
    } else { None };
    let object_service = if references && input.types.iter().any(|t| t.is_reference_type && !crate::selection::static_owner(t)) {
        let mut sig = module.make_signature();
        sig.params.extend([AbiParam::new(types::I64), AbiParam::new(types::I32), AbiParam::new(types::I32), AbiParam::new(types::I64)]);
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_allocate_object_v1", Linkage::Import, &sig)?)
    } else { None };
    let array_service = if references && input.functions.iter().any(|f| f.body.iter().any(|op| matches!(op, Op::NewArray(_)))) {
        let mut sig = module.make_signature();
        sig.params.extend([AbiParam::new(types::I64), AbiParam::new(types::I32), AbiParam::new(types::I64)]);
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_allocate_bytes_v1", Linkage::Import, &sig)?)
    } else { None };
    let mut ids = vec![];
    for (i, _) in input.functions.iter().enumerate() {
        let mut sig = module.make_signature();
        for t in &p.args[i] {
            sig.params
                .extend(lanes(&p, t).into_iter().map(AbiParam::new));
        }
        sig.params.push(AbiParam::new(types::I64)); // caller-owned result storage
        if details.is_some() { sig.params.push(AbiParam::new(types::I64)); }
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
            let output = parameters[parameters.len() - 1 - usize::from(details.is_some())];
            let fault_context = details.map(|_| *parameters.last().unwrap());
            if details.is_some_and(|d| d.console_read_byte.contains(&i)) {
                let service = module.declare_func_in_func(input_service.unwrap(), b.func);
                let call = b.ins().call(service, &[]);
                let value = b.inst_results(call)[0];
                let byte = b.ins().icmp_imm(IntCC::UnsignedLessThan, value, 256);
                let eof = b.ins().icmp_imm(IntCC::Equal, value, -1);
                let unavailable = b.ins().icmp_imm(IntCC::Equal, value, -2);
                let failed = b.ins().icmp_imm(IntCC::Equal, value, -3);
                let zero = b.ins().iconst(types::I32, 0);
                let one = b.ins().iconst(types::I32, 1);
                let two = b.ins().iconst(types::I32, 2);
                let tag = b.ins().select(eof, zero, one);
                let tag = b.ins().select(byte, two, tag);
                let error = b.ins().select(failed, two, zero);
                let error = b.ins().select(unavailable, one, error);
                let payload = b.ins().select(byte, value, error);
                write(&mut b, output, &[tag, payload]);
                b.ins().return_(&[zero]);
                b.seal_all_blocks();
                b.finalize();
                module.define_function(ids[i], &mut context)?;
                continue;
            }
            if details.is_some_and(|d| d.console_write_line.contains(&i)) {
                let service = module.declare_func_in_func(output_service.unwrap(), b.func);
                // Preserve the interpreter's invalid-native-argument RuntimeError
                // for default/null String instead of dereferencing address zero.
                let null = b.ins().icmp_imm(IntCC::Equal, parameters[0], 0);
                let null_status = b.ins().iconst(types::I32, 3);
                let mut null_site = diagnostic_data.as_ref().unwrap().site(&mut module, &mut b, fault_context.unwrap(), i, 0);
                null_site.capture_frame = false;
                return_if_detailed(&mut b, null, null_status, Some(&null_site));
                let length = b.ins().load(types::I64, MemFlags::new(), parameters[0], 0);
                let bytes = b.ins().iadd_imm(parameters[0], 8);
                let call = b.ins().call(service, &[bytes, length]);
                let result = b.inst_results(call)[0];
                let failed = b.ins().icmp_imm(IntCC::NotEqual, result, 0);
                let status = b.ins().iconst(types::I32, 3); // RuntimeError, same as interpreter
                let mut site = diagnostic_data.as_ref().unwrap().site(&mut module, &mut b, fault_context.unwrap(), i, 0);
                site.capture_frame = false; // InternalCall has no managed frame
                return_if_detailed(&mut b, failed, status, Some(&site));
                let zero = b.ins().iconst(types::I32, 0);
                if !f.no_result { write(&mut b, output, &[zero]); }
                b.ins().return_(&[zero]);
                b.seal_all_blocks();
                b.finalize();
                module.define_function(ids[i], &mut context)?;
                continue;
            }
            if details.is_some_and(|d| d.char_from_string.contains(&i) || d.char_text.contains(&i)) {
                let pointer = parameters[0];
                let mut site = diagnostic_data.as_ref().unwrap().site(&mut module, &mut b, fault_context.unwrap(), i, 0);
                site.capture_frame = false;
                let status = b.ins().iconst(types::I32, 3);
                let null = b.ins().icmp_imm(IntCC::Equal, pointer, 0);
                return_if_detailed(&mut b, null, status, Some(&site));
                if details.unwrap().char_from_string.contains(&i) {
                    let length = b.ins().load(types::I64, MemFlags::new(), pointer, 0);
                    let bytes = b.ins().iadd_imm(pointer, 8);
                    let service = module.declare_func_in_func(character_service.unwrap(), b.func);
                    let call = b.ins().call(service, &[bytes, length]);
                    let valid = b.inst_results(call)[0];
                    let invalid = b.ins().icmp_imm(IntCC::NotEqual, valid, 1);
                    return_if_detailed(&mut b, invalid, status, Some(&site));
                }
                write(&mut b, output, &[pointer]);
                let zero = b.ins().iconst(types::I32, 0);
                b.ins().return_(&[zero]);
                b.seal_all_blocks();
                b.finalize();
                module.define_function(ids[i], &mut context)?;
                continue;
            }
            if details.is_some_and(|d| d.int32_to_string.contains(&i)) {
                let service = module.declare_func_in_func(format_service.unwrap(), b.func);
                let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                let call = b.ins().call(service, &[parameters[0], arena, output]);
                let raw = b.inst_results(call)[0];
                // Only success and resource exhaustion are service outcomes. Other
                // failures become RuntimeError; adapters cannot manufacture UserFault.
                let exhausted = b.ins().icmp_imm(IntCC::Equal, raw, 5);
                let memory = b.ins().iconst(types::I32, 5);
                let runtime = b.ins().iconst(types::I32, 3);
                let status = b.ins().select(exhausted, memory, runtime);
                let failed = b.ins().icmp_imm(IntCC::NotEqual, raw, 0);
                let mut site = diagnostic_data.as_ref().unwrap().site(&mut module, &mut b, fault_context.unwrap(), i, 0);
                site.capture_frame = false;
                return_if_detailed(&mut b, failed, status, Some(&site));
                let zero = b.ins().iconst(types::I32, 0);
                b.ins().return_(&[zero]);
                b.seal_all_blocks();
                b.finalize();
                module.define_function(ids[i], &mut context)?;
                continue;
            }
            let mut frame_bytes = 64usize;
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
            // All profile return values fit in eight padded scalar lanes; snapshots are read
            // immediately after each successful call, before this storage can be reused.
            let call_result = slot(&mut b, 64);
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
                        for lane in lanes(&p, t) {
                            b.append_block_param(blocks[pc], lane);
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
                let mut site = diagnostic_data.as_ref().map(|d| d.site(&mut module, &mut b, fault_context.unwrap(), i, pc));
                match op {
                    Op::Int(v) => stack.push(b.ins().iconst(types::I32, i64::from(*v))),
                    Op::String(_) => {
                        let data = module.declare_data_in_func(literals[&(i, pc)], b.func);
                        stack.push(b.ins().global_value(types::I64, data));
                    }
                    Op::IsInstance(neoclr::metadata::Type::String) | Op::CastClass(neoclr::metadata::Type::String) => (),
                    Op::ReferenceIsNull => {
                        let value = pop(&mut stack);
                        let is_null = b.ins().icmp_imm(IntCC::Equal, value, 0);
                        stack.push(b.ins().uextend(types::I32, is_null));
                    }
                    Op::ReferenceEqual => {
                        let right = pop(&mut stack);
                        let left = pop(&mut stack);
                        let equal = b.ins().icmp(IntCC::Equal, left, right);
                        stack.push(b.ins().uextend(types::I32, equal));
                    }
                    Op::ConvertInt32 => {
                        if matches!(top(), Ty::Size) {
                            let value = pop(&mut stack);
                            stack.push(b.ins().ireduce(types::I32, value));
                        }
                    }
                    Op::ConvertUInt8 => {
                        let value = pop(&mut stack);
                        let value = if matches!(top(), Ty::Size) { b.ins().ireduce(types::I32, value) } else { value };
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
                            return_if_detailed(&mut b, wrong, status, site.as_ref());
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
                        let values = if ty == Ty::Character {
                            let data = module.declare_data_in_func(default_character, b.func);
                            vec![b.ins().global_value(types::I64, data)]
                        } else {
                            lanes(&p, &ty).into_iter().map(|t| b.ins().iconst(t, 0)).collect()
                        };
                        write(&mut b, address, &values);
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
                    Op::NewArray(neoclr::metadata::Type::Byte) => {
                        let count = pop(&mut stack);
                        let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                        let output = b.ins().stack_addr(types::I64, call_result, 0);
                        let service = module.declare_func_in_func(array_service.unwrap(), b.func);
                        let call = b.ins().call(service, &[arena, count, output]);
                        let status = b.inst_results(call)[0];
                        let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                        return_if_detailed(&mut b, failed, status, site.as_ref());
                        stack.push(b.ins().load(types::I64, MemFlags::new(), output, 0));
                    }
                    Op::ArrayLength => {
                        let array = pop(&mut stack);
                        null_reference(&mut b, array, site.as_ref());
                        stack.push(b.ins().load(types::I64, MemFlags::new(), array, 8));
                    }
                    Op::ArrayElement(neoclr::metadata::Type::Byte) | Op::ArrayAddress(neoclr::metadata::Type::Byte) | Op::StoreArrayElement(neoclr::metadata::Type::Byte) => {
                        let value = if matches!(op, Op::StoreArrayElement(_)) { Some(pop(&mut stack)) } else { None };
                        let index = pop(&mut stack);
                        let array = pop(&mut stack);
                        null_reference(&mut b, array, site.as_ref());
                        let length = b.ins().load(types::I32, MemFlags::new(), array, 8);
                        let outside = b.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, index, length);
                        let status = b.ins().iconst(types::I32, 8);
                        return_if_detailed(&mut b, outside, status, site.as_ref());
                        let offset = b.ins().uextend(types::I64, index);
                        let data = b.ins().iadd_imm(array, 16);
                        let address = b.ins().iadd(data, offset);
                        if let Some(value) = value {
                            let byte = b.ins().ireduce(types::I8, value);
                            b.ins().store(MemFlags::new(), byte, address, 0);
                        } else if matches!(op, Op::ArrayAddress(_)) {
                            stack.push(address);
                        } else {
                            let byte = b.ins().load(types::I8, MemFlags::new(), address, 0);
                            stack.push(b.ins().uextend(types::I32, byte));
                        }
                    }
                    Op::Field(n) | Op::FieldAddress(n) => {
                        let field = p.field(top(), *n)?;
                        let offset = p.field_offset(top(), *n);
                        if matches!(top(), Ty::Address(_) | Ty::Reference(_)) {
                            let owner = pop(&mut stack);
                            let header = if matches!(top(), Ty::Reference(_)) {
                                null_reference(&mut b, owner, site.as_ref());
                                8
                            } else { 0 };
                            let address = b.ins().iadd_imm(owner, (header + offset * 8) as i64);
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
                        if matches!(owner, Ty::Address(_) | Ty::Reference(_)) {
                            let is_reference = matches!(owner, Ty::Reference(_));
                            let owner = pop(&mut stack);
                            let header = if is_reference {
                                null_reference(&mut b, owner, site.as_ref());
                                8
                            } else { 0 };
                            let address = b.ins().iadd_imm(owner, (header + offset * 8) as i64);
                            write_typed(&mut b, &p, &p.field(&shape[shape.len() - 2], *n)?, address, &value);
                            if !is_reference { stack.push(b.ins().iconst(types::I32, 0)); } // borrowed value store returns Void
                        } else {
                            let start = stack.len() - p.lanes(owner) + offset;
                            stack[start..start + value.len()].copy_from_slice(&value);
                        }
                    }
                    Op::Call(target) | Op::CallVirtual(target) | Op::Construct(target) => {
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
                            let receiver = if let Ty::Reference(index) = t {
                                let service = module.declare_func_in_func(object_service.unwrap(), b.func);
                                let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                                let type_id = b.ins().iconst(types::I32, index as i64);
                                let bytes = b.ins().iconst(types::I32, p.object_bytes(index) as i64);
                                let call = b.ins().call(service, &[arena, type_id, bytes, address]);
                                let raw = b.inst_results(call)[0];
                                let exhausted = b.ins().icmp_imm(IntCC::Equal, raw, 5);
                                let memory = b.ins().iconst(types::I32, 5);
                                let runtime = b.ins().iconst(types::I32, 3);
                                let status = b.ins().select(exhausted, memory, runtime);
                                let failed = b.ins().icmp_imm(IntCC::NotEqual, raw, 0);
                                return_if_detailed(&mut b, failed, status, site.as_ref());
                                b.ins().load(types::I64, MemFlags::new(), address, 0)
                            } else {
                                let zeros: Vec<_> = lanes(&p, &t).into_iter().map(|t| b.ins().iconst(t, 0)).collect();
                                write(&mut b, address, &zeros);
                                address
                            };
                            call_args.insert(0, receiver);
                            Some((t, address))
                        } else {
                            None
                        };
                        if matches!(op, Op::CallVirtual(_)) {
                            // Unlike direct call, callvirt checks null before entering
                            // even a nonvirtual target; capture the caller's site.
                            null_reference(&mut b, call_args[0], site.as_ref());
                        }
                        let result = b.ins().stack_addr(types::I64, call_result, 0);
                        call_args.push(result);
                        if let Some(context) = fault_context { call_args.push(context); }
                        let target = module.declare_func_in_func(ids[c], b.func);
                        let call = b.ins().call(target, &call_args);
                        let status = b.inst_results(call)[0];
                        let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                        return_if_detailed(&mut b, failed, status, site.as_ref());
                        if let Some((t, address)) = constructed {
                            stack.extend(read(&mut b, &p, &t, address));
                        } else if let Some(t) = &p.results[c] {
                            stack.extend(read(&mut b, &p, t, result));
                        }
                    }
                    Op::Fault(_) => {
                        if details.is_some_and(|d| d.user_faults.contains(&i)) {
                            let pointer = b.ins().stack_load(types::I64, arguments[0], 0);
                            let diagnostic = site.as_mut().unwrap();
                            diagnostic.message = None;
                            diagnostic.capture_frame = false;
                            let null = b.ins().icmp_imm(IntCC::Equal, pointer, 0);
                            let invalid = b.ins().iconst(types::I32, 3);
                            return_if_detailed(&mut b, null, invalid, Some(diagnostic));
                            diagnostic.message = Some(pointer);
                            // The interpreter reports the managed call site, not an
                            // artificial frame for an InternalCall implementation.
                            site.as_mut().unwrap().capture_frame = false;
                        }
                        let status = b.ins().iconst(types::I32, 4);
                        if let Some(site) = &site { site.record(&mut b, status); }
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
                        stack.push(checked_arithmetic(&mut b, op, l, r, site.as_ref()));
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
    if details.is_some() { context.func.signature.params.push(AbiParam::new(types::I64)); }
    context
        .func
        .signature
        .returns
        .push(AbiParam::new(types::I32));
    let export =
        module.declare_function(if text_arena { "neoclr_entry_v4" } else if details.is_some() { "neoclr_entry_v3" } else { "neoclr_entry_v2" }, Linkage::Export, &context.func.signature)?;
    let target = module.declare_func_in_func(ids[root], &mut context.func);
    let mut fb = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut context.func, &mut fb);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let params = b.block_params(entry).to_vec();
        if details.is_some() { crate::fault_details::reset(&mut b, params[2]); }
        if text_arena {
            // Reset only the cursor; capacity/data belong to the host. Previous
            // dynamic text, including fault messages, expires on this next call.
            let zero = b.ins().iconst(types::I64, 0);
            b.ins().store(MemFlags::new(), zero, params[2], 1064);
        }
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
