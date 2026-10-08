//! Inline value/member lowering. Records travel as field snapshots, not owning pointers.
use super::{Error, checked_arithmetic, flow, return_if_detailed};
#[path = "value_profile.rs"]
mod profile;
#[path = "gc_layout.rs"]
mod gc_layout;
#[path = "gc_points.rs"]
mod gc_points;
#[path = "gc_probe.rs"]
mod gc_probe;

pub(super) fn trace_layout(input: &neoclr::Module, details: Option<&crate::fault_details::Options>) -> Result<serde_json::Value, Error> {
    let p = Profile::new(input, details.is_some_and(|d| d.reference_arena), details.and_then(|d| d.object_base), details.and_then(|d| d.array_backing), details.map_or(&[], |d| d.reference_array_backings.as_slice()), details.map(|d| &d.object_display), details.map(|d| &d.string_dispatch), details.map(|d| d.primitive_receivers.as_slice()), details.is_some_and(|d| d.native_stack_budget))?;
    let mut report = gc_layout::report(&p);
    report["preOperationPlans"] = gc_points::report(&p, details)?;
    Ok(report)
}

use cranelift_codegen::ir::condcodes::{FloatCC, IntCC};
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
    match t {
        Ty::Double => vec![types::F64],
        Ty::Record(i) if !p.input.types[*i].fields.is_empty() => p.input.types[*i].fields.iter()
            .flat_map(|f| lanes(p, &p.ty(&f.ty).expect("admitted field"))).collect(),
        _ => p.pointer_lanes(t).into_iter().map(|pointer| if pointer { types::I64 } else { types::I32 }).collect(),
    }
}
fn zero(b: &mut FunctionBuilder<'_>, ty: ir::Type) -> ir::Value {
    if ty == types::F64 { b.ins().f64const(0.0) } else { b.ins().iconst(ty, 0) }
}

// Cranelift 0.121's ARM64 lowering lacks unordered relational FloatCCs.
// Invert the complementary ordered test so NaN still follows CLI .un semantics.
fn float_compare(b: &mut FunctionBuilder<'_>, cc: IntCC, left: ir::Value, right: ir::Value) -> ir::Value {
    let (cc, invert) = match cc {
        IntCC::Equal => (FloatCC::Equal, false),
        IntCC::NotEqual => (FloatCC::NotEqual, false),
        IntCC::SignedGreaterThan => (FloatCC::GreaterThan, false),
        IntCC::SignedLessThan => (FloatCC::LessThan, false),
        IntCC::SignedGreaterThanOrEqual => (FloatCC::GreaterThanOrEqual, false),
        IntCC::SignedLessThanOrEqual => (FloatCC::LessThanOrEqual, false),
        IntCC::UnsignedGreaterThan => (FloatCC::LessThanOrEqual, true),
        IntCC::UnsignedLessThan => (FloatCC::GreaterThanOrEqual, true),
        IntCC::UnsignedGreaterThanOrEqual => (FloatCC::LessThan, true),
        IntCC::UnsignedLessThanOrEqual => (FloatCC::GreaterThan, true),
    };
    let result = b.ins().fcmp(cc, left, right);
    if invert { b.ins().bxor_imm(result, 1) } else { result }
}

fn read(
    b: &mut FunctionBuilder<'_>,
    p: &Profile<'_>,
    t: &Ty,
    pointer: ir::Value,
) -> Vec<ir::Value> {
    lanes(p, t).into_iter().zip(p.narrow_lanes(t)).enumerate()
        .map(|(i, (ty, narrow))| {
            let storage = narrow.map_or(ty, |(bits, _)| if bits == 8 { types::I8 } else { types::I16 });
            let value = b.ins().load(storage, MemFlags::new(), pointer, i as i32 * 8);
            match narrow {
                Some((_, true)) => b.ins().sextend(types::I32, value),
                Some((_, false)) => b.ins().uextend(types::I32, value),
                None => value,
            }
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
        .zip(p.narrow_lanes(t))
        .map(
            |(v, narrow)| {
                if let Some((bits, signed)) = narrow {
                    let narrow = b.ins().ireduce(if bits == 8 { types::I8 } else { types::I16 }, *v);
                    if signed { b.ins().sextend(types::I32, narrow) } else { b.ins().uextend(types::I32, narrow) }
                } else { *v }
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
    let values: Vec<_> = normalize(b, p, t, values).into_iter().zip(p.narrow_lanes(t))
        .map(|(value, narrow)| if let Some((bits, _)) = narrow { b.ins().ireduce(if bits == 8 { types::I8 } else { types::I16 }, value) } else { value }).collect();
    write(b, pointer, &values);
}
fn slot(b: &mut FunctionBuilder<'_>, bytes: u32) -> ir::StackSlot {
    b.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, bytes, 3))
}
fn args(values: &[ir::Value]) -> Vec<ir::BlockArg> {
    values.iter().copied().map(Into::into).collect()
}

fn matches_type(b: &mut FunctionBuilder<'_>, p: &Profile<'_>, tag: ir::Value, index: usize) -> ir::Value {
    if p.array_backing == Some(index) {
        let ordinary = b.ins().icmp_imm(IntCC::Equal, tag, 0x80000001);
        let reserved = b.ins().icmp_imm(IntCC::Equal, tag, 0x80000002);
        b.ins().bor(ordinary, reserved)
    } else if p.reference_array_backings.contains(&index) {
        b.ins().icmp_imm(IntCC::Equal, tag, (((index as u64 + 1) << 32) | 0x80000005) as i64)
    } else { b.ins().icmp_imm(IntCC::Equal, tag, index as i64) }
}

// Private Object views use the low bit of aligned text pointers. Null stays zero.
// Every dereference of a dynamically typed receiver must distinguish this view first.
fn text_object_view(b: &mut FunctionBuilder<'_>, value: ir::Value) -> ir::Value {
    let present = b.ins().icmp_imm(IntCC::NotEqual, value, 0);
    let tagged = b.ins().bor_imm(value, 1);
    b.ins().select(present, tagged, value)
}

// Copy immutable bytes into a fresh String owner when identity is observable.
fn copy_text(module: &mut ObjectModule, b: &mut FunctionBuilder<'_>,
    binding: (cranelift_module::FuncId, cranelift_module::DataId), pointer: ir::Value,
    context: ir::Value, site: Option<&crate::fault_details::Site>) -> ir::Value {
    let service = module.declare_func_in_func(binding.0, b.func);
    let empty = module.declare_data_in_func(binding.1, b.func);
    let empty = b.ins().global_value(types::I64, empty);
    let storage = slot(b, 8);
    let output = b.ins().stack_addr(types::I64, storage, 0);
    let arena = b.ins().iadd_imm(context, 1048);
    let call = b.ins().call(service, &[pointer, empty, arena, output]);
    let raw = b.inst_results(call)[0];
    let failed = b.ins().icmp_imm(IntCC::NotEqual, raw, 0);
    let exhausted = b.ins().icmp_imm(IntCC::Equal, raw, 5);
    let memory = b.ins().iconst(types::I32, 5);
    let runtime = b.ins().iconst(types::I32, 3);
    let status = b.ins().select(exhausted, memory, runtime);
    return_if_detailed(b, failed, status, site);
    b.ins().load(types::I64, MemFlags::new(), output, 0)
}

// String ceq compares exact UTF-8 contents, independently of owner identity.
// Both pointers are verified image/arena text or null, never arbitrary host memory.
fn string_equal(b: &mut FunctionBuilder<'_>, left: ir::Value, right: ir::Value) -> ir::Value {
    let different = b.create_block();
    let lengths = b.create_block();
    let scan = b.create_block();
    let byte = b.create_block();
    let advance = b.create_block();
    let done = b.create_block();
    b.append_block_param(scan, types::I64);
    b.append_block_param(done, types::I32);
    let yes = b.ins().iconst(types::I32, 1);
    let no = b.ins().iconst(types::I32, 0);
    let same = b.ins().icmp(IntCC::Equal, left, right);
    b.ins().brif(same, done, &[yes.into()], different, &[]);
    b.switch_to_block(different);
    let left_null = b.ins().icmp_imm(IntCC::Equal, left, 0);
    let right_null = b.ins().icmp_imm(IntCC::Equal, right, 0);
    let null = b.ins().bor(left_null, right_null);
    b.ins().brif(null, done, &[no.into()], lengths, &[]);
    b.switch_to_block(lengths);
    let length = b.ins().load(types::I64, MemFlags::new(), left, 0);
    let other = b.ins().load(types::I64, MemFlags::new(), right, 0);
    let equal = b.ins().icmp(IntCC::Equal, length, other);
    let zero = b.ins().iconst(types::I64, 0);
    b.ins().brif(equal, scan, &[zero.into()], done, &[no.into()]);
    b.switch_to_block(scan);
    let index = b.block_params(scan)[0];
    let complete = b.ins().icmp(IntCC::Equal, index, length);
    b.ins().brif(complete, done, &[yes.into()], byte, &[]);
    b.switch_to_block(byte);
    let l = b.ins().iadd(left, index);
    let r = b.ins().iadd(right, index);
    let l = b.ins().load(types::I8, MemFlags::new(), l, 8);
    let r = b.ins().load(types::I8, MemFlags::new(), r, 8);
    let equal = b.ins().icmp(IntCC::Equal, l, r);
    b.ins().brif(equal, advance, &[], done, &[no.into()]);
    b.switch_to_block(advance);
    let next = b.ins().iadd_imm(index, 1);
    b.ins().jump(scan, &[next.into()]);
    b.switch_to_block(done);
    b.block_params(done)[0]
}

fn null_reference(b: &mut FunctionBuilder<'_>, pointer: ir::Value, site: Option<&crate::fault_details::Site>) {
    let null = b.ins().icmp_imm(IntCC::Equal, pointer, 0);
    let status = b.ins().iconst(types::I32, 6);
    return_if_detailed(b, null, status, site);
}

// Value-array slots have fixed extent after their first assignment, matching the
// interpreter even though immutable snapshots can share their backing bytes.
fn check_byte_value_replacement(b: &mut FunctionBuilder<'_>, address: ir::Value, value: ir::Value, site: Option<&crate::fault_details::Site>) {
    let old = b.ins().load(types::I64, MemFlags::new(), address, 0);
    let assigned = b.ins().icmp_imm(IntCC::NotEqual, old, 0);
    let check = b.create_block();
    let ready = b.create_block();
    b.ins().brif(assigned, check, &[], ready, &[]);
    b.switch_to_block(check);
    let old_length = b.ins().load(types::I64, MemFlags::new(), old, 8);
    let new_length = b.ins().load(types::I64, MemFlags::new(), value, 8);
    let mismatch = b.ins().icmp(IntCC::NotEqual, old_length, new_length);
    let status = b.ins().iconst(types::I32, 3);
    return_if_detailed(b, mismatch, status, site);
    b.ins().jump(ready, &[]);
    b.switch_to_block(ready);
}

pub(super) fn compile(input: &neoclr::Module, root: &str, details: Option<&crate::fault_details::Options>) -> Result<Vec<u8>, Error> {
    let reservations = input.functions.iter().any(|f| f.body.iter().any(|op| matches!(op, Op::ReserveArray(_))));
    let references = details.is_some_and(|d| d.reference_arena);
    if details.is_some_and(|d| d.native_gc && (!d.reference_arena || !d.probe_stack_roots)) {
        return Err("native GC requires an admitted reference-arena profile and published roots".into());
    }
    let stack_budget = details.is_some_and(|d| d.native_stack_budget);
    if stack_budget && !details.is_some_and(|d| d.native_gc) {
        return Err("native stack budget requires GC frame publication".into());
    }
    let p = Profile::new(input, references, details.and_then(|d| d.object_base), details.and_then(|d| d.array_backing), details.map_or(&[], |d| d.reference_array_backings.as_slice()), details.map(|d| &d.object_display), details.map(|d| &d.string_dispatch), details.map(|d| d.primitive_receivers.as_slice()), details.is_some_and(|d| d.native_stack_budget))?;
    let root = p.root(root)?;
    let flows: Vec<_> = (0..input.functions.len())
        .map(|i| p.analyze(i))
        .collect::<Result<_, _>>()?;
    let text_identity = input.functions.iter().enumerate().any(|(i, f)| f.body.iter().enumerate().any(|(pc, op)| {
        let Some(shape) = flows[i].get(pc).and_then(Option::as_ref) else { return false; };
        match op {
            Op::CastClass(t) | Op::IsInstance(t) => shape.last() == Some(&Ty::Literal) && *t != neoclr::metadata::Type::String
                || *t == neoclr::metadata::Type::String && shape.last() != Some(&Ty::Literal),
            Op::ReferenceEqual => shape.iter().rev().take(2).any(|t| *t == Ty::Literal),
            _ => false,
        }
    }));
    let string_interfaces = details.and_then(|d| d.string_interfaces.as_ref());
    if text_identity {
        let interface_casts = input.functions.iter().any(|f| f.body.iter().any(|op| {
            matches!(op, Op::CastClass(t) | Op::IsInstance(t) if matches!(p.ty(t), Ok(Ty::Interface(_))))
        }));
        if interface_casts && string_interfaces.is_none() {
            return Err("String interface views require verified load-set conformance".into());
        }
    }
    let string_view = |ty: &Ty| p.is_object_base(ty)
        || matches!(ty, Ty::Interface(i) if string_interfaces.is_some_and(|indices| indices.contains(i)));
    // The backend's narrow shape analysis is additional admission, not a replacement
    // for type/member identity, accessibility, initialization or byref lifetime checks.
    neoclr::LoadedProgram::new(input).and_then(|v| v.verify()).map_err(|e| e.to_string())?;
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
    // or explicit, bounded invocation-arena producers. When identity is observable,
    // literal evaluations receive fresh arena storage; image data remains a template.
    // String Object views retain that pointer; no arbitrary host/escaping String is admitted.
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
    let text_copy = if text_identity {
        let id = module.declare_data("neoclr_identity_empty", Linkage::Local, false, false)?;
        let mut data = DataDescription::new();
        data.define(vec![0; 8].into_boxed_slice()); data.set_align(8);
        module.define_data(id, &data)?;
        let mut sig = module.make_signature();
        sig.params.extend([types::I64; 4].map(AbiParam::new));
        sig.returns.push(AbiParam::new(types::I32));
        Some((module.declare_function("neoclr_string_concat_v1", Linkage::Import, &sig)?, id))
    } else { None };
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
    let mut stream_services = std::collections::HashMap::new();
    if let Some(d) = details {
        for (indices, symbol, parameters) in [
            (&d.console_write_bytes, "neoclr_console_write_bytes_v1", vec![types::I32, types::I64, types::I64, types::I32, types::I32]),
            (&d.console_flush, "neoclr_console_flush_v1", vec![types::I32]),
        ] {
            if indices.is_empty() { continue; }
            let mut sig = module.make_signature();
            sig.params.extend(parameters.into_iter().map(AbiParam::new));
            sig.returns.push(AbiParam::new(types::I32));
            let service = module.declare_function(symbol, Linkage::Import, &sig)?;
            for index in indices { stream_services.insert(*index, service); }
        }
    }
    let entry_drain_services = if details.is_some_and(|d| !d.entry_task_drain.is_empty()) {
        let mut sig = module.make_signature();
        sig.params.extend([types::I64, types::I32, types::I32, types::I32, types::I64].map(AbiParam::new));
        sig.returns.push(AbiParam::new(types::I32));
        let begin = module.declare_function("neoclr_entry_tasks_begin_v1", Linkage::Import, &sig)?;
        sig.params.truncate(1);
        sig.returns.clear();
        let end = module.declare_function("neoclr_entry_tasks_end_v1", Linkage::Import, &sig)?;
        Some((begin, end))
    } else { None };
    let mut socket_services = std::collections::HashMap::new();
    if let Some(d) = details {
        for (indices, symbol, parameters) in [
            (&d.task_queue_register, "neoclr_task_queue_register_v1", vec![types::I64, types::I64, types::I64]),
            (&d.task_queue_default, "neoclr_task_queue_default_v1", vec![types::I64, types::I64]),
            (&d.task_queue_current, "neoclr_task_queue_current_v1", vec![types::I64, types::I32, types::I32, types::I64]),
            (&d.socket_deadline_after, "neoclr_socket_deadline_after_v1", vec![types::I32, types::I64, types::I64]),
            (&d.socket_deadline_expired, "neoclr_socket_deadline_expired_v1", vec![types::I64, types::I64, types::I64]),
            (&d.socket_receive_until, "neoclr_socket_receive_until_v1", vec![types::I64, types::I64, types::I32, types::I32, types::I64, types::I64, types::I64, types::I64]),
            (&d.socket_send_until, "neoclr_socket_send_until_v1", vec![types::I64, types::I64, types::I32, types::I32, types::I64, types::I64, types::I64, types::I64]),
            (&d.socket_receive, "neoclr_socket_receive_v1", vec![types::I64, types::I64, types::I32, types::I32, types::I64, types::I64, types::I64]),
            (&d.socket_send, "neoclr_socket_send_v1", vec![types::I64, types::I64, types::I32, types::I32, types::I64, types::I64, types::I64]),
            (&d.socket_transfer_result, "neoclr_socket_transfer_result_v1", vec![types::I64, types::I64, types::I64]),
            (&d.socket_accept, "neoclr_socket_accept_v1", vec![types::I64, types::I64, types::I64, types::I64]),
            (&d.socket_connect_result, "neoclr_socket_connect_result_v1", vec![types::I64, types::I64, types::I64]),
            (&d.socket_cancel, "neoclr_socket_cancel_v1", vec![types::I64, types::I64, types::I64]),
            (&d.socket_listen, "neoclr_socket_listen_v1", vec![types::I64, types::I32, types::I32, types::I64, types::I64]),
            (&d.socket_local_port, "neoclr_socket_local_port_v1", vec![types::I64, types::I64, types::I64]),
            (&d.socket_close, "neoclr_socket_close_v1", vec![types::I64, types::I64, types::I64]),
        ] {
            if indices.is_empty() { continue; }
            let mut sig = module.make_signature();
            sig.params.extend(parameters.into_iter().map(AbiParam::new));
            sig.returns.push(AbiParam::new(types::I32));
            let service = module.declare_function(symbol, Linkage::Import, &sig)?;
            for index in indices { socket_services.insert(*index, service); }
        }
    }
    let mut utf8_services = std::collections::HashMap::new();
    if let Some(d) = details {
        for (indices, symbol, parameters) in [
            (&d.file_output, "neoclr_file_write_utf8_v1", vec![types::I64, types::I64, types::I32, types::I64]),
            (&d.path_combine, "neoclr_path_combine_unix_v1", vec![types::I64, types::I64, types::I64, types::I64]),
            (&d.path_file_name, "neoclr_path_file_name_unix_v1", vec![types::I64, types::I64, types::I64]),
            (&d.string_compare_ordinal, "neoclr_string_compare_ordinal_v1", vec![types::I64, types::I64, types::I64]),
            (&d.string_contains_ordinal, "neoclr_string_contains_ordinal_v1", vec![types::I64, types::I64, types::I64]),
            (&d.string_starts_with_ordinal, "neoclr_string_starts_with_ordinal_v1", vec![types::I64, types::I64, types::I64]),
            (&d.string_ends_with_ordinal, "neoclr_string_ends_with_ordinal_v1", vec![types::I64, types::I64, types::I64]),
            (&d.parse_int32, "neoclr_parse_int32_v1", vec![types::I64, types::I64]),
            (&d.utf8_encode, "neoclr_utf8_encode_v1", vec![types::I64, types::I64, types::I64]),
            (&d.utf8_decode, "neoclr_utf8_decode_v1", vec![types::I64, types::I64, types::I64]),
            (&d.string_join_parts, "neoclr_string_join_parts_v1", vec![types::I64, types::I32, types::I64, types::I32, types::I64, types::I64]),
            (&d.string_concat, "neoclr_string_concat_v1", vec![types::I64, types::I64, types::I64, types::I64]),
            (&d.string_byte_count, "neoclr_string_byte_count_v1", vec![types::I64, types::I64]),
            (&d.string_slice_utf8, "neoclr_string_slice_utf8_v1", vec![types::I64, types::I32, types::I32, types::I64, types::I64]),
        ] {
            if indices.is_empty() { continue; }
            let mut sig = module.make_signature();
            sig.params.extend(parameters.into_iter().map(AbiParam::new));
            sig.returns.push(AbiParam::new(types::I32));
            let service = module.declare_function(symbol, Linkage::Import, &sig)?;
            for index in indices { utf8_services.insert(*index, service); }
        }
    }
    let text_arena = references || details.is_some_and(|d| !d.int32_to_string.is_empty() || !d.int64_to_string.is_empty() || !d.uint64_to_string.is_empty());
    let format_service = if details.is_some_and(|d| !d.int32_to_string.is_empty() || d.boxed_int32_display) {
        let mut sig = module.make_signature();
        sig.params.extend([AbiParam::new(types::I32), AbiParam::new(types::I64), AbiParam::new(types::I64)]);
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_int32_to_string_v1", Linkage::Import, &sig)?)
    } else { None };
    let mut wide_format_services = std::collections::HashMap::new();
    if let Some(d) = details {
        for (indices, symbol) in [(&d.int64_to_string, "neoclr_int64_to_string_v1"), (&d.uint64_to_string, "neoclr_uint64_to_string_v1")] {
            if indices.is_empty() { continue; }
            let mut sig = module.make_signature();
            sig.params.extend([types::I64, types::I64, types::I64].map(AbiParam::new));
            sig.returns.push(AbiParam::new(types::I32));
            let service = module.declare_function(symbol, Linkage::Import, &sig)?;
            for index in indices { wide_format_services.insert(*index, service); }
        }
    }
    let character_service = if details.is_some_and(|d| !d.char_from_string.is_empty()) {
        let mut sig = module.make_signature();
        sig.params.extend([types::I64, types::I64].map(AbiParam::new));
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_is_single_grapheme_v1", Linkage::Import, &sig)?)
    } else { None };
    let object_service = if references && (input.functions.iter().any(|f| f.body.iter().any(|op| matches!(op, Op::BindFunction { .. }))) || input.types.iter().any(|t| t.is_reference_type && !crate::selection::static_owner(t)) || details.is_some_and(|d| !d.empty_record_boxes.is_empty())) {
        let mut sig = module.make_signature();
        sig.params.extend([AbiParam::new(types::I64), AbiParam::new(types::I32), AbiParam::new(types::I32), AbiParam::new(types::I64)]);
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_allocate_object_v1", Linkage::Import, &sig)?)
    } else { None };
    let array_service = if references && input.functions.iter().any(|f| f.body.iter().any(|op| matches!(op, Op::NewArray(neoclr::metadata::Type::Byte)))) {
        let mut sig = module.make_signature();
        sig.params.extend([AbiParam::new(types::I64), AbiParam::new(types::I32), AbiParam::new(types::I64)]);
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_allocate_bytes_v1", Linkage::Import, &sig)?)
    } else { None };
    let reserve_service = if references && input.functions.iter().any(|f| f.body.iter().any(|op| matches!(op, Op::ReserveArray(neoclr::metadata::Type::Byte)))) {
        let mut sig = module.make_signature();
        sig.params.extend([types::I64, types::I32, types::I64].map(AbiParam::new));
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_reserve_bytes_v1", Linkage::Import, &sig)?)
    } else { None };
    let scalar_array_service = if references && input.functions.iter().any(|f| f.body.iter().any(|op|
        matches!(op, Op::NewArray(t) | Op::ReserveArray(t) if crate::selection::scalar_array_element(t)))) {
        let mut sig = module.make_signature();
        sig.params.extend([types::I64, types::I32, types::I32, types::I64].map(AbiParam::new));
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_allocate_scalars_v1", Linkage::Import, &sig)?)
    } else { None };
    let default_reference_array_service = if references && input.functions.iter().any(|f| f.body.iter().any(|op|
        matches!(op, Op::NewArray(neoclr::metadata::Type::Named(_))))) {
        let mut sig = module.make_signature();
        sig.params.extend([types::I64, types::I32, types::I64].map(AbiParam::new));
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_allocate_references_v1", Linkage::Import, &sig)?)
    } else { None };
    let record_array_service = if references && input.functions.iter().any(|f| f.body.iter().any(|op|
        matches!(op, Op::ReserveArray(neoclr::metadata::Type::Named(_))))) {
        let mut sig = module.make_signature();
        sig.params.extend([types::I64, types::I32, types::I32, types::I64].map(AbiParam::new));
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_reserve_records_v1", Linkage::Import, &sig)?)
    } else { None };
    let reference_array_service = if references && input.functions.iter().any(|f| f.body.iter().any(|op|
        matches!(op, Op::NewArray(t) | Op::ReserveArray(t) if matches!(t, neoclr::metadata::Type::String | neoclr::metadata::Type::Function(_))))) {
        let mut sig = module.make_signature();
        sig.params.extend([types::I64, types::I32, types::I32, types::I64].map(AbiParam::new));
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_allocate_strings_v1", Linkage::Import, &sig)?)
    } else { None };
    let initialized_service = if reservations && details.is_some_and(|d| !d.console_write_bytes.is_empty()) {
        let mut sig = module.make_signature();
        sig.params.extend([types::I64, types::I32, types::I32].map(AbiParam::new));
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_check_bytes_initialized_v1", Linkage::Import, &sig)?)
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
    let stack_check = if stack_budget {
        let mut sig = module.make_signature();
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_native_stack_check_v1", Linkage::Import, &sig)?)
    } else { None };
    let root_probes = if details.is_some_and(|d| d.probe_stack_roots) {
        Some(gc_probe::Probes::prepare(&mut module, &p, &flows, details)?)
    } else { None };
    for (i, f) in input.functions.iter().enumerate() {
        let mut context = module.make_context();
        context.func.signature = module
            .declarations()
            .get_function_decl(ids[i])
            .signature
            .clone();
        let mut fb = FunctionBuilderContext::new();
        let mut probe_frame = None;
        {
            let mut b = FunctionBuilder::new(&mut context.func, &mut fb);
            let entry = b.create_block();
            b.append_block_params_for_function_params(entry);
            b.switch_to_block(entry);
            let parameters = b.block_params(entry).to_vec();
            let output = parameters[parameters.len() - 1 - usize::from(details.is_some())];
            let fault_context = details.map(|_| *parameters.last().unwrap());
            if gc_points::native_body(&p, i, details) {
                if let Some(probes) = &root_probes {
                    // Native wrappers keep typed argument copies visible across service
                    // calls/dispatch. These frames do not enter the guest fault trace.
                    let table_bytes = probes.table_bytes(i);
                    let mut frame_bytes = 104 + 7 + table_bytes as usize + 7;
                    let mut arguments = vec![];
                    let mut at = 0;
                    for t in &p.args[i] {
                        let storage = slot(&mut b, p.bytes(t));
                        frame_bytes += p.bytes(t) as usize + 7;
                        let address = b.ins().stack_addr(types::I64, storage, 0);
                        write_typed(&mut b, &p, t, address, &parameters[at..at + p.lanes(t)]);
                        arguments.push(storage);
                        at += p.lanes(t);
                    }
                    if frame_bytes > 65536 {
                        return Err("native probe frame storage exceeds 64 KiB".into());
                    }
                    let frame = slot(&mut b, 104);
                    let table = slot(&mut b, table_bytes);
                    probes.enter(&mut module, &mut b, frame, fault_context.unwrap(), i, table, &arguments, &[]);
                    probe_frame = Some(frame);
                    emit_stack_check(&mut module, &mut b, stack_check, diagnostic_data.as_ref(), fault_context.unwrap(), i, false);
                }
            }
            if let Some(&record) = details.and_then(|d| d.empty_record_boxes.get(&i).or_else(|| d.int32_boxes.get(&i))) {
                let service = module.declare_func_in_func(object_service.unwrap(), b.func);
                let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                let tag = b.ins().iconst(types::I32, record as i64);
                let primitive = details.unwrap().int32_boxes.contains_key(&i);
                let bytes = b.ins().iconst(types::I32, if primitive { 16 } else { 8 });
                let call = b.ins().call(service, &[arena, tag, bytes, output]);
                let raw = b.inst_results(call)[0];
                let exhausted = b.ins().icmp_imm(IntCC::Equal, raw, 5);
                let memory = b.ins().iconst(types::I32, 5);
                let runtime = b.ins().iconst(types::I32, 3);
                let status = b.ins().select(exhausted, memory, runtime);
                let failed = b.ins().icmp_imm(IntCC::NotEqual, raw, 0);
                let mut site = diagnostic_data.as_ref().unwrap().site(&mut module, &mut b, fault_context.unwrap(), i, 0);
                site.capture_frame = false;
                return_if_detailed(&mut b, failed, status, Some(&site));
                if primitive {
                    let pointer = b.ins().load(types::I64, MemFlags::new(), output, 0);
                    b.ins().store(MemFlags::new(), parameters[0], pointer, 8);
                }
                let zero = b.ins().iconst(types::I32, 0);
                b.ins().return_(&[zero]);
                b.seal_all_blocks(); b.finalize();
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(&mut module, &mut context.func, frame);
                }
                define_checked(&mut module, ids[i], &mut context, stack_budget)?;
                continue;
            }
            if let Some(targets) = p.dispatch.get(&i) {
                // Caller checks null. Forward the original receiver, result slot and
                // context; no synthetic interface frame enters the managed trace.
                let display = details.is_some_and(|d| d.object_display.contains_key(&i));
                let string_target = details.and_then(|d| d.string_dispatch.get(&i));
                if text_identity {
                    let text = b.create_block();
                    let object = b.create_block();
                    let tagged = b.ins().band_imm(parameters[0], 1);
                    b.ins().brif(tagged, text, &[], object, &[]);
                    b.switch_to_block(text);
                    let pointer = b.ins().band_imm(parameters[0], -2);
                    if let Some(&target) = string_target {
                        let mut args = parameters.clone();
                        args[0] = pointer;
                        let target = module.declare_func_in_func(ids[target], b.func);
                        let call = b.ins().call(target, &args);
                        let status = b.inst_results(call)[0];
                        b.ins().return_(&[status]);
                    } else if display {
                        write(&mut b, output, &[pointer]);
                        let zero = b.ins().iconst(types::I32, 0);
                        b.ins().return_(&[zero]);
                    } else {
                        let status = b.ins().iconst(types::I32, 3);
                        if let Some(d) = &diagnostic_data {
                            let mut site = d.site(&mut module, &mut b, fault_context.unwrap(), i, 0);
                            site.capture_frame = false;
                            site.record(&mut b, status);
                        }
                        b.ins().return_(&[status]);
                    }
                    b.switch_to_block(object);
                }
                let tag = b.ins().load(types::I64, MemFlags::new(), parameters[0], 0);
                if display && details.is_some_and(|d| d.boxed_int32_display) {
                    for &type_index in details.unwrap().int32_boxes.values() {
                        let matched = b.create_block();
                        let next = b.create_block();
                        let equal = matches_type(&mut b, &p, tag, type_index);
                        b.ins().brif(equal, matched, &[], next, &[]);
                        b.switch_to_block(matched);
                        let value = b.ins().load(types::I32, MemFlags::new(), parameters[0], 8);
                        let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                        let service = module.declare_func_in_func(format_service.unwrap(), b.func);
                        let call = b.ins().call(service, &[value, arena, output]);
                        let raw = b.inst_results(call)[0];
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
                        b.switch_to_block(next);
                    }
                }
                for &(type_index, target) in targets {
                    let matched = b.create_block();
                    let next = b.create_block();
                    let equal = matches_type(&mut b, &p, tag, type_index);
                    b.ins().brif(equal, matched, &[], next, &[]);
                    b.switch_to_block(matched);
                    let target = module.declare_func_in_func(ids[target], b.func);
                    let call = b.ins().call(target, &parameters);
                    let status = b.inst_results(call)[0];
                    b.ins().return_(&[status]);
                    b.switch_to_block(next);
                }
                let status = b.ins().iconst(types::I32, 3);
                if let Some(d) = &diagnostic_data {
                    let mut site = d.site(&mut module, &mut b, fault_context.unwrap(), i, 0);
                    site.capture_frame = false;
                    site.record(&mut b, status);
                }
                b.ins().return_(&[status]);
                b.seal_all_blocks();
                b.finalize();
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(&mut module, &mut context.func, frame);
                }
                define_checked(&mut module, ids[i], &mut context, stack_budget)?;
                continue;
            }
            if details.is_some_and(|d| d.entry_task_drain.contains(&i)) {
                let d = details.unwrap();
                let ctx = fault_context.unwrap();
                let (begin, end) = entry_drain_services.unwrap();
                let begin = module.declare_func_in_func(begin, b.func);
                let end = module.declare_func_in_func(end, b.func);
                let entry = b.ins().iconst(types::I32, root as i64);
                let run = b.ins().iconst(types::I32, d.task_queue_run.map_or(-1, |i| i as i64));
                let drain = b.ins().iconst(types::I32, d.task_queue_drain.map_or(-1, |i| i as i64));
                let storage = slot(&mut b, 16);
                let queue_output = b.ins().stack_addr(types::I64, storage, 0);
                let call = b.ins().call(begin, &[ctx, entry, run, drain, queue_output]);
                let status = b.inst_results(call)[0];
                let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                let mut site = diagnostic_data.as_ref().unwrap().site(&mut module, &mut b, ctx, i, 0);
                site.capture_frame = false;
                return_if_detailed(&mut b, failed, status, Some(&site));
                let done = b.create_block();
                b.append_block_param(done, types::I32);
                let zero = b.ins().iconst(types::I32, 0);
                if let Some(drain) = d.task_queue_drain {
                    let queue = b.ins().load(types::I64, MemFlags::new(), queue_output, 0);
                    let absent = b.ins().icmp_imm(IntCC::Equal, queue, 0);
                    let work = b.create_block();
                    b.ins().brif(absent, done, &[zero.into()], work, &[]);
                    b.switch_to_block(work);
                    // Queue is strongly rooted by the task scope. Startup roots stay
                    // published; the ordinary Drain frame checks its own stack budget.
                    let result = b.ins().stack_addr(types::I64, storage, 8);
                    let callee = module.declare_func_in_func(ids[drain], b.func);
                    let call = b.ins().call(callee, &[queue, result, ctx]);
                    let status = b.inst_results(call)[0];
                    b.ins().jump(done, &[status.into()]);
                } else {
                    b.ins().jump(done, &[zero.into()]);
                }
                b.switch_to_block(done);
                let status = b.block_params(done)[0];
                b.ins().call(end, &[ctx]);
                let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                return_if_detailed(&mut b, failed, status, None);
                b.ins().store(MemFlags::new(), zero, output, 0);
                b.ins().return_(&[zero]);
                b.seal_all_blocks(); b.finalize();
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(&mut module, &mut context.func, frame);
                }
                define_checked(&mut module, ids[i], &mut context, stack_budget)?;
                continue;
            }
            if let Some(service) = socket_services.get(&i) {
                let service = module.declare_func_in_func(*service, b.func);
                let mut args = parameters[..p.args[i].len()].to_vec();
                args.push(fault_context.unwrap());
                if details.unwrap().task_queue_current.contains(&i) {
                    let run = b.ins().iconst(types::I32, details.unwrap().task_queue_run.map_or(-1, |i| i as i64));
                    let drain = b.ins().iconst(types::I32, details.unwrap().task_queue_drain.map_or(-1, |i| i as i64));
                    args.extend([run, drain]);
                }
                args.push(output);
                let call = b.ins().call(service, &args);
                let raw = b.inst_results(call)[0];
                let failed = b.ins().icmp_imm(IntCC::NotEqual, raw, 0);
                let exhausted = b.ins().icmp_imm(IntCC::Equal, raw, 5);
                let memory = b.ins().iconst(types::I32, 5);
                let runtime = b.ins().iconst(types::I32, 3);
                let status = b.ins().select(exhausted, memory, runtime);
                let mut site = diagnostic_data.as_ref().unwrap().site(&mut module, &mut b, fault_context.unwrap(), i, 0);
                site.capture_frame = false;
                return_if_detailed(&mut b, failed, status, Some(&site));
                let zero = b.ins().iconst(types::I32, 0);
                b.ins().return_(&[zero]);
                b.seal_all_blocks(); b.finalize();
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(&mut module, &mut context.func, frame);
                }
                define_checked(&mut module, ids[i], &mut context, stack_budget)?;
                continue;
            }
            if let Some(service) = stream_services.get(&i) {
                let service = module.declare_func_in_func(*service, b.func);
                let mut call_args = vec![parameters[0]];
                if details.unwrap().console_write_bytes.contains(&i) {
                    let array = parameters[1];
                    let null = b.ins().icmp_imm(IntCC::Equal, array, 0);
                    let status = b.ins().iconst(types::I32, 3);
                    let mut site = diagnostic_data.as_ref().unwrap().site(&mut module, &mut b, fault_context.unwrap(), i, 0);
                    site.capture_frame = false;
                    return_if_detailed(&mut b, null, status, Some(&site));
                    if let Some(check) = initialized_service {
                        let check = module.declare_func_in_func(check, b.func);
                        let call = b.ins().call(check, &[array, parameters[2], parameters[3]]);
                        let status = b.inst_results(call)[0];
                        let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                        return_if_detailed(&mut b, failed, status, Some(&site));
                    }
                    let bytes = b.ins().iadd_imm(array, 16);
                    let length = b.ins().load(types::I64, MemFlags::new(), array, 8);
                    call_args.extend([bytes, length, parameters[2], parameters[3]]);
                }
                let call = b.ins().call(service, &call_args);
                let raw = b.inst_results(call)[0];
                let negative = b.ins().icmp_imm(IntCC::SignedLessThan, raw, 0);
                let oversized = if details.unwrap().console_write_bytes.contains(&i) {
                    b.ins().icmp(IntCC::SignedGreaterThan, raw, parameters[3])
                } else { b.ins().icmp_imm(IntCC::NotEqual, raw, 0) };
                let failed = b.ins().bor(negative, oversized);
                let byte_tag = b.ins().iconst(types::I32, 2);
                let int_tag = b.ins().iconst(types::I32, 1);
                let tag = b.ins().select(failed, byte_tag, int_tag);
                let mut code = b.ins().iconst(types::I32, 10);
                if details.unwrap().console_write_bytes.contains(&i) {
                    for status in [7, 8] {
                        let matches = b.ins().icmp_imm(IntCC::Equal, raw, -status);
                        let value = b.ins().iconst(types::I32, status);
                        code = b.ins().select(matches, value, code);
                    }
                }
                let payload = b.ins().select(failed, code, raw);
                let payload = b.ins().uextend(types::I64, payload);
                write(&mut b, output, &[tag, payload]);
                let zero = b.ins().iconst(types::I32, 0);
                b.ins().return_(&[zero]);
                b.seal_all_blocks();
                b.finalize();
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(&mut module, &mut context.func, frame);
                }
                define_checked(&mut module, ids[i], &mut context, stack_budget)?;
                continue;
            }
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
                let payload = b.ins().uextend(types::I64, payload);
                write(&mut b, output, &[tag, payload]);
                b.ins().return_(&[zero]);
                b.seal_all_blocks();
                b.finalize();
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(&mut module, &mut context.func, frame);
                }
                define_checked(&mut module, ids[i], &mut context, stack_budget)?;
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
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(&mut module, &mut context.func, frame);
                }
                define_checked(&mut module, ids[i], &mut context, stack_budget)?;
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
                let pointer = if details.unwrap().char_text.contains(&i) {
                    if let Some(binding) = text_copy {
                        copy_text(&mut module, &mut b, binding, pointer, fault_context.unwrap(), Some(&site))
                    } else { pointer }
                } else { pointer };
                write(&mut b, output, &[pointer]);
                let zero = b.ins().iconst(types::I32, 0);
                b.ins().return_(&[zero]);
                b.seal_all_blocks();
                b.finalize();
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(&mut module, &mut context.func, frame);
                }
                define_checked(&mut module, ids[i], &mut context, stack_budget)?;
                continue;
            }
            if details.is_some_and(|d| d.native_integer_to64.contains(&i)) {
                write(&mut b, output, &[parameters[0]]);
                let zero = b.ins().iconst(types::I32, 0);
                b.ins().return_(&[zero]);
                b.seal_all_blocks();
                b.finalize();
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(&mut module, &mut context.func, frame);
                }
                define_checked(&mut module, ids[i], &mut context, stack_budget)?;
                continue;
            }
            if let Some(service) = utf8_services.get(&i) {
                let service = module.declare_func_in_func(*service, b.func);
                let args = if details.unwrap().file_output.contains(&i) {
                    vec![parameters[0], parameters[1], parameters[2], output]
                } else if details.unwrap().string_join_parts.contains(&i) {
                    let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                    vec![parameters[0], parameters[1], parameters[2], parameters[3], arena, output]
                } else if details.unwrap().string_slice_utf8.contains(&i) {
                    let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                    vec![parameters[0], parameters[1], parameters[2], arena, output]
                } else if details.unwrap().string_concat.contains(&i) || details.unwrap().path_combine.contains(&i) {
                    let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                    vec![parameters[0], parameters[1], arena, output]
                } else if details.unwrap().utf8_encode.contains(&i) || details.unwrap().utf8_decode.contains(&i) || details.unwrap().path_file_name.contains(&i) {
                    let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                    vec![parameters[0], arena, output]
                } else if details.unwrap().string_compare_ordinal.contains(&i)
                    || details.unwrap().string_contains_ordinal.contains(&i)
                    || details.unwrap().string_starts_with_ordinal.contains(&i)
                    || details.unwrap().string_ends_with_ordinal.contains(&i) {
                    vec![parameters[0], parameters[1], output]
                } else { vec![parameters[0], output] };
                let call = b.ins().call(service, &args);
                let raw = b.inst_results(call)[0];
                let exhausted = b.ins().icmp_imm(IntCC::Equal, raw, 5);
                let memory = b.ins().iconst(types::I32, 5);
                let runtime = b.ins().iconst(types::I32, 3);
                let status = b.ins().select(exhausted, memory, runtime);
                let status = if details.unwrap().utf8_encode.contains(&i) {
                    let limited = b.ins().icmp_imm(IntCC::Equal, raw, 7);
                    let array_limit = b.ins().iconst(types::I32, 7);
                    b.ins().select(limited, array_limit, status)
                } else { status };
                let failed = b.ins().icmp_imm(IntCC::NotEqual, raw, 0);
                let mut site = diagnostic_data.as_ref().unwrap().site(&mut module, &mut b, fault_context.unwrap(), i, 0);
                site.capture_frame = false;
                return_if_detailed(&mut b, failed, status, Some(&site));
                let zero = b.ins().iconst(types::I32, 0);
                b.ins().return_(&[zero]);
                b.seal_all_blocks(); b.finalize();
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(&mut module, &mut context.func, frame);
                }
                define_checked(&mut module, ids[i], &mut context, stack_budget)?;
                continue;
            }
            if details.is_some_and(|d| d.int32_to_string.contains(&i)) || wide_format_services.contains_key(&i) {
                let service = module.declare_func_in_func(wide_format_services.get(&i).copied().or(format_service).unwrap(), b.func);
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
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(&mut module, &mut context.func, frame);
                }
                define_checked(&mut module, ids[i], &mut context, stack_budget)?;
                continue;
            }
            let call_result_bytes = p.call_result_bytes();
            let mut frame_bytes = call_result_bytes as usize;
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
            for (ty, local) in p.locals[i].iter().zip(&locals) {
                // Keep not-yet-assigned reference storage safe for future root
                // registration. The verifier still rejects guest reads before assignment.
                // This includes ByteValues' existing internal unassigned marker.
                for lane in gc_layout::seed_lanes(&p, ty) {
                    let zero = b.ins().iconst(types::I64, 0);
                    b.ins().stack_store(zero, *local, (lane * 8) as i32);
                }
            }
            // Size scratch for the selected layouts (up to sixty-four lanes); snapshots are read
            // immediately after each successful call, before this storage can be reused.
            let call_result = slot(&mut b, call_result_bytes);
            let mut constructors = std::collections::HashMap::new();
            for (pc, op) in f.body.iter().enumerate() {
                if let Op::Construct(target) = op {
                    let bytes = p.bytes(&p.ty(target.owner.as_ref().unwrap())?);
                    frame_bytes += bytes as usize + 7;
                    constructors.insert(pc, slot(&mut b, bytes));
                }
            }
            let root_spill = root_probes.as_ref().and_then(|probes| probes.storage_bytes(i)).map(|bytes| {
                frame_bytes += bytes as usize + 7;
                slot(&mut b, bytes)
            });
            let storage_table = root_probes.as_ref().map(|probes| {
                let bytes = probes.table_bytes(i);
                frame_bytes += 104 + 7 + bytes as usize + 7;
                probe_frame = Some(slot(&mut b, 104));
                slot(&mut b, bytes)
            });
            let transient_table = root_probes.as_ref().and_then(|probes| probes.transient_bytes(i)).map(|bytes| {
                frame_bytes += bytes as usize + 7;
                slot(&mut b, bytes)
            });
            if frame_bytes > 65536 {
                return Err("value profile frame storage exceeds 64 KiB".into());
            }
            if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                probes.enter(&mut module, &mut b, frame, fault_context.unwrap(), i, storage_table.unwrap(), &arguments, &locals);
                emit_stack_check(&mut module, &mut b, stack_check, diagnostic_data.as_ref(), fault_context.unwrap(), i, true);
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
                if let (Some(probes), Some(spill)) = (&root_probes, root_spill) {
                    probes.emit(&mut module, &mut b, i, pc, spill, probe_frame.unwrap(), &stack);
                }
                match op {
                    Op::Int(v) => stack.push(b.ins().iconst(types::I32, i64::from(*v))),
                    Op::Int64(v) => stack.push(b.ins().iconst(types::I64, *v)),
                    Op::Float64 { bits } => stack.push(b.ins().f64const(ir::immediates::Ieee64::with_bits(*bits))),
                    Op::String(_) => {
                        let data = module.declare_data_in_func(literals[&(i, pc)], b.func);
                        let pointer = b.ins().global_value(types::I64, data);
                        if let Some(binding) = text_copy {
                            let owned = copy_text(&mut module, &mut b, binding, pointer, fault_context.unwrap(), site.as_ref());
                            stack.push(owned);
                        } else { stack.push(pointer); }
                    }
                    Op::IsInstance(neoclr::metadata::Type::String) | Op::CastClass(neoclr::metadata::Type::String) if *top() == Ty::Literal => (),
                    Op::IsInstance(target) | Op::CastClass(target) => {
                        let value = pop(&mut stack);
                        let probe = b.create_block();
                        let joined = b.create_block();
                        b.append_block_param(joined, types::I64);
                        let null = b.ins().icmp_imm(IntCC::Equal, value, 0);
                        b.ins().brif(null, joined, &[value.into()], probe, &[]);
                        b.switch_to_block(probe);
                        let zero = b.ins().iconst(types::I64, 0);
                        let target_ty = p.ty(target)?;
                        let cast = if *top() == Ty::Literal {
                            if string_view(&target_ty) { text_object_view(&mut b, value) } else { zero }
                        } else {
                            let text = b.create_block();
                            let object = b.create_block();
                            let merged = b.create_block();
                            b.append_block_param(merged, types::I64);
                            let tagged = b.ins().band_imm(value, 1);
                            b.ins().brif(tagged, text, &[], object, &[]);
                            b.switch_to_block(text);
                            let result = if target_ty == Ty::Literal { b.ins().band_imm(value, -2) }
                                else if string_view(&target_ty) { value } else { zero };
                            b.ins().jump(merged, &[result.into()]);
                            b.switch_to_block(object);
                            let result = if target_ty == Ty::Literal { zero } else {
                                let tag = b.ins().load(types::I64, MemFlags::new(), value, 0);
                                let mut matches = b.ins().iconst(types::I8, 0);
                                for index in p.cast_targets(&target_ty) {
                                    let matched = matches_type(&mut b, &p, tag, index);
                                    matches = b.ins().bor(matches, matched);
                                }
                                b.ins().select(matches, value, zero)
                            };
                            b.ins().jump(merged, &[result.into()]);
                            b.switch_to_block(merged);
                            b.block_params(merged)[0]
                        };
                        b.ins().jump(joined, &[cast.into()]);
                        b.switch_to_block(joined);
                        let result = b.block_params(joined)[0];
                        if matches!(op, Op::CastClass(_)) {
                            let absent = b.ins().icmp_imm(IntCC::Equal, result, 0);
                            let present = b.ins().icmp_imm(IntCC::NotEqual, value, 0);
                            let invalid = b.ins().band(absent, present);
                            let status = b.ins().iconst(types::I32, 3); // Existing interpreter class/interface cast contract
                            return_if_detailed(&mut b, invalid, status, site.as_ref());
                        }
                        stack.push(result);
                    }
                    Op::ReferenceIsNull => {
                        let value = pop(&mut stack);
                        let is_null = b.ins().icmp_imm(IntCC::Equal, value, 0);
                        stack.push(b.ins().uextend(types::I32, is_null));
                    }
                    Op::ReferenceEqual => {
                        let mut right = pop(&mut stack);
                        let mut left = pop(&mut stack);
                        if shape[shape.len()-1] == Ty::Literal { right = text_object_view(&mut b, right); }
                        if shape[shape.len()-2] == Ty::Literal { left = text_object_view(&mut b, left); }
                        let equal = b.ins().icmp(IntCC::Equal, left, right);
                        stack.push(b.ins().uextend(types::I32, equal));
                    }
                    Op::ConvertInt32 | Op::ConvertUInt32 => {
                        if matches!(top(), Ty::Size | Ty::Wide) {
                            let value = pop(&mut stack);
                            stack.push(b.ins().ireduce(types::I32, value));
                        }
                    }
                    Op::ConvertInt64 | Op::ConvertUInt64 | Op::ConvertNativeInt | Op::ConvertNativeUInt => {
                        if matches!(top(), Ty::Int) {
                            let value = pop(&mut stack);
                            stack.push(if matches!(op, Op::ConvertUInt64 | Op::ConvertNativeUInt) {
                                b.ins().uextend(types::I64, value)
                            } else { b.ins().sextend(types::I64, value) });
                        }
                    }
                    Op::ConvertUInt8 | Op::ConvertInt8 | Op::ConvertInt16 | Op::ConvertUInt16 => {
                        let value = pop(&mut stack);
                        let value = if matches!(top(), Ty::Size | Ty::Wide) { b.ins().ireduce(types::I32, value) } else { value };
                        let narrow = b.ins().ireduce(if matches!(op, Op::ConvertUInt8 | Op::ConvertInt8) { types::I8 } else { types::I16 }, value);
                        stack.push(if matches!(op, Op::ConvertInt8 | Op::ConvertInt16) { b.ins().sextend(types::I32, narrow) } else { b.ins().uextend(types::I32, narrow) });
                    }
                    Op::Bool(v) => stack.push(b.ins().iconst(types::I32, i64::from(*v))),
                    Op::Void => stack.push(b.ins().iconst(types::I32, 0)),
                    Op::PackValue(t) => {
                        let payload = pop(&mut stack);
                        let payload = normalize(&mut b, &p, &p.ty(t)?, &[payload])[0];
                        let payload = if matches!(p.ty(t)?, Ty::Literal | Ty::Wide) { payload } else { b.ins().uextend(types::I64, payload) };
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
                            stack.push(if matches!(p.ty(t)?, Ty::Literal | Ty::Wide) { payload } else { b.ins().ireduce(types::I32, payload) });
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
                        if *ty == Ty::ByteValues { check_byte_value_replacement(&mut b, address, value[0], site.as_ref()); }
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
                            lanes(&p, &ty).into_iter().map(|t| zero(&mut b, t)).collect()
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
                        if p.ty(t)? == Ty::ByteValues { check_byte_value_replacement(&mut b, address, value[0], site.as_ref()); }
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
                    Op::NewArray(t) | Op::ReserveArray(t) if crate::selection::scalar_array_element(t) => {
                        let count = pop(&mut stack);
                        let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                        let output = b.ins().stack_addr(types::I64, call_result, 0);
                        let reserved = b.ins().iconst(types::I32, i64::from(matches!(op, Op::ReserveArray(_))));
                        let service = module.declare_func_in_func(scalar_array_service.unwrap(), b.func);
                        let call = b.ins().call(service, &[arena, count, reserved, output]);
                        let status = b.inst_results(call)[0];
                        let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                        return_if_detailed(&mut b, failed, status, site.as_ref());
                        stack.push(b.ins().load(types::I64, MemFlags::new(), output, 0));
                    }
                    Op::NewArray(t) if matches!(t, neoclr::metadata::Type::Named(_)) => {
                        let count = pop(&mut stack);
                        let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                        let output = b.ins().stack_addr(types::I64, call_result, 0);
                        let service = module.declare_func_in_func(default_reference_array_service.unwrap(), b.func);
                        let call = b.ins().call(service, &[arena, count, output]);
                        let status = b.inst_results(call)[0];
                        let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                        return_if_detailed(&mut b, failed, status, site.as_ref());
                        let array = b.ins().load(types::I64, MemFlags::new(), output, 0);
                        if let Some(index) = p.backing_for_array(&p.ty(&neoclr::metadata::Type::ArrayRef(Box::new(t.clone())))?) {
                            let tag = b.ins().iconst(types::I64, (((index as u64 + 1) << 32) | 0x80000005) as i64);
                            b.ins().store(MemFlags::new(), tag, array, 0);
                        }
                        stack.push(array);
                    }
                    Op::ReserveArray(t) if matches!(t, neoclr::metadata::Type::Named(_)) => {
                        let count = pop(&mut stack);
                        let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                        let output = b.ins().stack_addr(types::I64, call_result, 0);
                        let width = b.ins().iconst(types::I32, p.lanes(&p.ty(t)?) as i64);
                        let service = module.declare_func_in_func(record_array_service.unwrap(), b.func);
                        let call = b.ins().call(service, &[arena, count, width, output]);
                        let status = b.inst_results(call)[0];
                        let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                        return_if_detailed(&mut b, failed, status, site.as_ref());
                        let array = b.ins().load(types::I64, MemFlags::new(), output, 0);
                        if let Some(index) = p.backing_for_array(&p.ty(&neoclr::metadata::Type::ArrayRef(Box::new(t.clone())))?) {
                            let tag = b.ins().iconst(types::I64, (((index as u64 + 1) << 32) | 0x80000005) as i64);
                            b.ins().store(MemFlags::new(), tag, array, 0);
                        }
                        stack.push(array);
                    }
                    Op::NewArray(t) | Op::ReserveArray(t) if matches!(t, neoclr::metadata::Type::String | neoclr::metadata::Type::Function(_)) => {
                        let count = pop(&mut stack);
                        let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                        let output = b.ins().stack_addr(types::I64, call_result, 0);
                        let reserved = b.ins().iconst(types::I32, i64::from(matches!(op, Op::ReserveArray(_))));
                        let service = module.declare_func_in_func(reference_array_service.unwrap(), b.func);
                        let call = b.ins().call(service, &[arena, count, reserved, output]);
                        let status = b.inst_results(call)[0];
                        let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                        return_if_detailed(&mut b, failed, status, site.as_ref());
                        stack.push(b.ins().load(types::I64, MemFlags::new(), output, 0));
                    }
                    Op::NewArray(neoclr::metadata::Type::Byte) | Op::ReserveArray(neoclr::metadata::Type::Byte) => {
                        let count = pop(&mut stack);
                        let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                        let output = b.ins().stack_addr(types::I64, call_result, 0);
                        let service = module.declare_func_in_func(if matches!(op, Op::ReserveArray(_)) { reserve_service.unwrap() } else { array_service.unwrap() }, b.func);
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
                    Op::ArrayElement(t) | Op::StoreArrayElement(t) if matches!(t, neoclr::metadata::Type::Named(_)) || crate::selection::scalar_array_element(t) => {
                        let ty = p.ty(t)?;
                        let width = p.lanes(&ty);
                        let value = if matches!(op, Op::StoreArrayElement(_)) {
                            Some(stack.split_off(stack.len() - width))
                        } else { None };
                        let index = pop(&mut stack);
                        let array = pop(&mut stack);
                        null_reference(&mut b, array, site.as_ref());
                        let length = b.ins().load(types::I32, MemFlags::new(), array, 8);
                        let outside = b.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, index, length);
                        let status = b.ins().iconst(types::I32, 8);
                        return_if_detailed(&mut b, outside, status, site.as_ref());
                        let index = b.ins().uextend(types::I64, index);
                        let length = b.ins().uextend(types::I64, length);
                        let offset = b.ins().imul_imm(index, (width * 8) as i64);
                        let data = b.ins().iadd_imm(array, 24);
                        let address = b.ins().iadd(data, offset);
                        let bytes = b.ins().imul_imm(length, (width * 8) as i64);
                        let markers = b.ins().iadd(data, bytes);
                        let marker = b.ins().iadd(markers, index);
                        if let Some(value) = value {
                            // Publish initialization only after the complete snapshot is stored.
                            // There is no collector boundary between these instructions.
                            write_typed(&mut b, &p, &ty, address, &value);
                            let initialized = b.ins().iconst(types::I8, 1);
                            b.ins().store(MemFlags::new(), initialized, marker, 0);
                        } else {
                            let initialized = b.ins().load(types::I8, MemFlags::new(), marker, 0);
                            let unreadable = b.ins().icmp_imm(IntCC::Equal, initialized, 0);
                            let status = b.ins().iconst(types::I32, 3);
                            return_if_detailed(&mut b, unreadable, status, site.as_ref());
                            stack.extend(read(&mut b, &p, &ty, address));
                        }
                    }
                    Op::ArrayElement(t) | Op::StoreArrayElement(t) | Op::ArrayAddress(t) => {
                        let pointers = matches!(t, neoclr::metadata::Type::String | neoclr::metadata::Type::Function(_));
                        let value = if matches!(op, Op::StoreArrayElement(_)) { Some(pop(&mut stack)) } else { None };
                        let index = pop(&mut stack);
                        let array = pop(&mut stack);
                        null_reference(&mut b, array, site.as_ref());
                        let length = b.ins().load(types::I32, MemFlags::new(), array, 8);
                        let outside = b.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, index, length);
                        let status = b.ins().iconst(types::I32, 8);
                        return_if_detailed(&mut b, outside, status, site.as_ref());
                        let offset = b.ins().uextend(types::I64, index);
                        let offset = if pointers { b.ins().ishl_imm(offset, 3) } else { offset };
                        let data = b.ins().iadd_imm(array, 16);
                        let address = b.ins().iadd(data, offset);
                        if reservations {
                            let kind = b.ins().load(types::I64, MemFlags::new(), array, 0);
                            let reserved = b.ins().icmp_imm(IntCC::Equal, kind, if pointers { 0x80000004 } else { 0x80000002 });
                            let check = b.create_block();
                            let ready = b.create_block();
                            b.ins().brif(reserved, check, &[], ready, &[]);
                            b.switch_to_block(check);
                            let length = b.ins().uextend(types::I64, length);
                            let marker = if pointers {
                                let bytes = b.ins().ishl_imm(length, 3);
                                let markers = b.ins().iadd(data, bytes);
                                let index = b.ins().uextend(types::I64, index);
                                b.ins().iadd(markers, index)
                            } else { b.ins().iadd(address, length) };
                            if value.is_some() {
                                let initialized = b.ins().iconst(types::I8, 1);
                                b.ins().store(MemFlags::new(), initialized, marker, 0);
                            } else {
                                let initialized = b.ins().load(types::I8, MemFlags::new(), marker, 0);
                                let unreadable = b.ins().icmp_imm(IntCC::Equal, initialized, 0);
                                let status = b.ins().iconst(types::I32, 3);
                                return_if_detailed(&mut b, unreadable, status, site.as_ref());
                            }
                            b.ins().jump(ready, &[]);
                            b.switch_to_block(ready);
                        }
                        if let Some(value) = value {
                            let value = if pointers { value } else { b.ins().ireduce(types::I8, value) };
                            b.ins().store(MemFlags::new(), value, address, 0);
                        } else if matches!(op, Op::ArrayAddress(_)) {
                            stack.push(address);
                        } else {
                            let value = b.ins().load(if pointers { types::I64 } else { types::I8 }, MemFlags::new(), address, 0);
                            stack.push(if pointers { value } else { b.ins().uextend(types::I32, value) });
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
                            if matches!(top(), Ty::Reference(i) if p.is_array_backing(*i)) {
                                // The verified backing's sole field denotes this same array,
                                // not storage at the length-header offset.
                                stack.push(owner);
                            } else if matches!(op, Op::FieldAddress(_)) {
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
                    Op::BindFunction { target, .. } => {
                        let c = p.callee(target)?;
                        let receiver = if target.instance {
                            let receiver = pop(&mut stack);
                            // Binding uses the interpreter's invalid-receiver fault;
                            // invoking a null function separately uses NullReference.
                            let null = b.ins().icmp_imm(IntCC::Equal, receiver, 0);
                            let invalid = b.ins().iconst(types::I32, 3);
                            return_if_detailed(&mut b, null, invalid, site.as_ref());
                            receiver
                        } else { b.ins().iconst(types::I64, 0) };
                        let service = module.declare_func_in_func(object_service.unwrap(), b.func);
                        let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                        // Private descriptor header, never exposed as a nominal class.
                        let type_id = b.ins().iconst(types::I32, u32::MAX as i64);
                        let bytes = b.ins().iconst(types::I32, 24);
                        let output = b.ins().stack_addr(types::I64, call_result, 0);
                        let call = b.ins().call(service, &[arena, type_id, bytes, output]);
                        let raw = b.inst_results(call)[0];
                        let exhausted = b.ins().icmp_imm(IntCC::Equal, raw, 5);
                        let memory = b.ins().iconst(types::I32, 5);
                        let runtime = b.ins().iconst(types::I32, 3);
                        let status = b.ins().select(exhausted, memory, runtime);
                        let failed = b.ins().icmp_imm(IntCC::NotEqual, raw, 0);
                        return_if_detailed(&mut b, failed, status, site.as_ref());
                        let descriptor = b.ins().load(types::I64, MemFlags::new(), output, 0);
                        let identity = b.ins().iconst(types::I64, (c + 1) as i64);
                        b.ins().store(MemFlags::new(), identity, descriptor, 8);
                        b.ins().store(MemFlags::new(), receiver, descriptor, 16);
                        stack.push(descriptor);
                    }
                    Op::Call(target) | Op::CallVirtual(target) if crate::selection::callable_invoke(target).is_some() => {
                        let signature = crate::selection::callable_invoke(target).unwrap();
                        let owner = target.owner.as_ref().unwrap();
                        let count = signature.parameters.iter().map(|t| p.ty(t).map(|t| p.lanes(&t))).collect::<Result<Vec<_>, _>>()?.into_iter().sum::<usize>();
                        let actual = stack.split_off(stack.len() - count);
                        let descriptor = pop(&mut stack);
                        null_reference(&mut b, descriptor, site.as_ref());
                        let identity = b.ins().load(types::I64, MemFlags::new(), descriptor, 8);
                        let result = b.ins().stack_addr(types::I64, call_result, 0);
                        let result_type = if signature.no_result { None } else { Some(p.ty(&signature.returns)?) };
                        if let (Some(probes), Some(t)) = (&root_probes, &result_type) {
                            let zero = b.ins().iconst(types::I64, 0);
                            for lane in gc_layout::seed_lanes(&p, t) {
                                b.ins().store(MemFlags::new(), zero, result, (lane * 8) as i32);
                            }
                            probes.publish(&mut module, &mut b, i, pc, 3, probe_frame.unwrap(), transient_table.unwrap(), result);
                        }
                        let join = b.create_block();
                        b.append_block_param(join, types::I32);
                        for c in p.callable_targets(owner)? {
                            let matched = b.create_block();
                            let next = b.create_block();
                            let equal = b.ins().icmp_imm(IntCC::Equal, identity, (c + 1) as i64);
                            b.ins().brif(equal, matched, &[], next, &[]);
                            b.switch_to_block(matched);
                            let mut args = Vec::new();
                            if input.functions[c].instance {
                                args.push(b.ins().load(types::I64, MemFlags::new(), descriptor, 16));
                            }
                            args.extend_from_slice(&actual);
                            args.push(result);
                            if let Some(context) = fault_context { args.push(context); }
                            let callee = module.declare_func_in_func(ids[c], b.func);
                            let call = b.ins().call(callee, &args);
                            let status = b.inst_results(call)[0];
                            b.ins().jump(join, &[status.into()]);
                            b.switch_to_block(next);
                        }
                        let invalid = b.ins().iconst(types::I32, 3);
                        b.ins().jump(join, &[invalid.into()]);
                        b.switch_to_block(join);
                        let status = b.block_params(join)[0];
                        let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                        return_if_detailed(&mut b, failed, status, site.as_ref());
                        if let Some(t) = &result_type {
                            if let Some(probes) = &root_probes {
                                probes.publish(&mut module, &mut b, i, pc, 2, probe_frame.unwrap(), transient_table.unwrap(), result);
                            }
                            stack.extend(read(&mut b, &p, t, result));
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
                                let zeros: Vec<_> = lanes(&p, &t).into_iter().map(|t| zero(&mut b, t)).collect();
                                write(&mut b, address, &zeros);
                                address
                            };
                            if let Some(probes) = &root_probes {
                                probes.publish(&mut module, &mut b, i, pc, 1, probe_frame.unwrap(), transient_table.unwrap(), address);
                            }
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
                        if !construct {
                            if let (Some(probes), Some(t)) = (&root_probes, &p.results[c]) {
                                // Seed only traceable scratch lanes; this does not publish
                                // a guest result. The caller retains their addresses while
                                // the callee writes and removes its diagnostic frame.
                                let zero = b.ins().iconst(types::I64, 0);
                                for lane in gc_layout::seed_lanes(&p, t) {
                                    b.ins().store(MemFlags::new(), zero, result, (lane * 8) as i32);
                                }
                                probes.publish(&mut module, &mut b, i, pc, 3, probe_frame.unwrap(), transient_table.unwrap(), result);
                            }
                        }
                        call_args.push(result);
                        if let Some(context) = fault_context { call_args.push(context); }
                        let target = module.declare_func_in_func(ids[c], b.func);
                        let call = b.ins().call(target, &call_args);
                        let status = b.inst_results(call)[0];
                        let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                        return_if_detailed(&mut b, failed, status, site.as_ref());
                        if let Some((t, address)) = constructed {
                            if let Some(probes) = &root_probes {
                                probes.publish(&mut module, &mut b, i, pc, 2, probe_frame.unwrap(), transient_table.unwrap(), address);
                            }
                            stack.extend(read(&mut b, &p, &t, address));
                        } else if let Some(t) = &p.results[c] {
                            if let Some(probes) = &root_probes {
                                probes.publish(&mut module, &mut b, i, pc, 2, probe_frame.unwrap(), transient_table.unwrap(), result);
                            }
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
                    Op::BitNot => {
                        let value = pop(&mut stack);
                        stack.push(b.ins().bnot(value));
                    }
                    Op::BitAnd | Op::BitOr | Op::BitXor => {
                        let right = pop(&mut stack);
                        let left = pop(&mut stack);
                        stack.push(match op {
                            Op::BitAnd => b.ins().band(left, right),
                            Op::BitOr => b.ins().bor(left, right),
                            _ => b.ins().bxor(left, right),
                        });
                    }
                    Op::Add | Op::Sub | Op::Mul | Op::Divide if *top() == Ty::Double => {
                        let r = pop(&mut stack);
                        let l = pop(&mut stack);
                        stack.push(match op {
                            Op::Add => b.ins().fadd(l, r),
                            Op::Sub => b.ins().fsub(l, r),
                            Op::Mul => b.ins().fmul(l, r),
                            _ => b.ins().fdiv(l, r),
                        });
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
                    Op::Equal if *top() == Ty::Literal => {
                        let right = pop(&mut stack);
                        let left = pop(&mut stack);
                        stack.push(string_equal(&mut b, left, right));
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
                        let bool8 = if *top() == Ty::Double { float_compare(&mut b, cc, l, r) } else { b.ins().icmp(cc, l, r) };
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
                        let condition = if *top() == Ty::Double { float_compare(&mut b, cc, l, r) } else { b.ins().icmp(cc, l, r) };
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
        if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
            probes.finish(&mut module, &mut context.func, frame);
        }
        define_checked(&mut module, ids[i], &mut context, stack_budget)?;
    }
    let entry_check = if details.is_some_and(|d| d.native_gc) {
        let mut signature = module.make_signature();
        signature.params.push(AbiParam::new(types::I64));
        signature.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_gc_entry_check_v1", Linkage::Import, &signature)?)
    } else { None };
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
        if let Some(check) = entry_check {
            let check = module.declare_func_in_func(check, b.func);
            let call = b.ins().call(check, &[params[2]]);
            let status = b.inst_results(call)[0];
            let rejected = b.create_block();
            let admitted = b.create_block();
            let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
            b.ins().brif(failed, rejected, &[], admitted, &[]);
            b.switch_to_block(rejected);
            b.ins().store(MemFlags::new(), status, params[2], 0);
            b.ins().return_(&[status]);
            b.switch_to_block(admitted);
        }
        if stack_budget {
            emit_stack_check(&mut module, &mut b, stack_check, diagnostic_data.as_ref(), params[2], root, false);
        }
        if text_arena {
            // Reset only the cursor; capacity/data belong to the host. Previous
            // dynamic text, including fault messages, expires on this next call.
            let zero = b.ins().iconst(types::I64, 0);
            b.ins().store(MemFlags::new(), zero, params[2], 1064);
        }
        let mut call_args = vec![];
        if !p.args[root].is_empty() { call_args.push(params[0]); }
        let unit_result = p.results[root] != Some(Ty::Int);
        let result = if unit_result {
            let storage = slot(&mut b, 8);
            b.ins().stack_addr(types::I64, storage, 0)
        } else { params[1] };
        call_args.push(result);
        if details.is_some() { call_args.push(params[2]); }
        let call = b.ins().call(target, &call_args);
        let status = b.inst_results(call)[0];
        if unit_result {
            let success = b.create_block();
            let done = b.create_block();
            let ok = b.ins().icmp_imm(IntCC::Equal, status, 0);
            b.ins().brif(ok, success, &[], done, &[]);
            b.switch_to_block(success);
            let zero = b.ins().iconst(types::I32, 0);
            b.ins().store(MemFlags::new(), zero, params[1], 0);
            b.ins().jump(done, &[]);
            b.switch_to_block(done);
        }
        b.ins().return_(&[status]);
        b.seal_all_blocks();
        b.finalize();
    }
    define_checked(&mut module, export, &mut context, stack_budget)?;
    if details.is_some_and(|d| d.native_gc) {
        compile_host_callbacks(&mut module, &p, &ids, stack_check, diagnostic_data.as_ref())?;
        if let Some(drain) = details.and_then(|d| d.task_queue_drain) {
            compile_host_queue(&mut module, ids[drain], drain, stack_check, diagnostic_data.as_ref().unwrap())?;
        }
    }
    super::finish(module)
}

// Quiescent host entry for the existing inhabited-Void callback shape. The strong
// handle keeps the descriptor/receiver alive across the callee's collection points.
fn compile_host_callbacks(module: &mut ObjectModule, p: &Profile<'_>, ids: &[cranelift_module::FuncId], stack_check: Option<cranelift_module::FuncId>, diagnostic_data: Option<&crate::fault_details::Data>) -> Result<(), Error> {
    use neoclr::metadata::{FunctionType, Type};
    let shape = Type::Function(Box::new(FunctionType {
        parameters: vec![], returns: Type::Void, no_result: false,
        out_parameters: vec![], out_when_true: vec![],
    }));
    let targets = p.callable_targets(&shape)?;
    if targets.is_empty() { return Ok(()); }
    let mut signature = module.make_signature();
    signature.params.extend([types::I64, types::I64, types::I64].map(AbiParam::new));
    signature.returns.push(AbiParam::new(types::I32));
    let reader = module.declare_function("neoclr_gc_callback_read_v1", Linkage::Import, &signature)?;
    signature.params.pop();
    let export = module.declare_function("neoclr_invoke_void_callback_v1", Linkage::Export, &signature)?;
    let mut context = module.make_context();
    context.func.signature = signature;
    let mut fb = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut context.func, &mut fb);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let params = b.block_params(entry).to_vec();
        let storage = slot(&mut b, 16);
        let output = b.ins().stack_addr(types::I64, storage, 0);
        let reader = module.declare_func_in_func(reader, b.func);
        let call = b.ins().call(reader, &[params[1], params[0], output]);
        let status = b.inst_results(call)[0];
        let rejected = b.create_block();
        b.append_block_param(rejected, types::I32);
        let admitted = b.create_block();
        let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
        b.ins().brif(failed, rejected, &[status.into()], admitted, &[]);
        b.switch_to_block(admitted);
        let descriptor = b.ins().load(types::I64, MemFlags::new(), output, 0);
        let identity = b.ins().load(types::I64, MemFlags::new(), descriptor, 8);
        let result = b.ins().stack_addr(types::I64, storage, 8);
        for c in targets {
            let matched = b.create_block();
            let next = b.create_block();
            let equal = b.ins().icmp_imm(IntCC::Equal, identity, (c + 1) as i64);
            b.ins().brif(equal, matched, &[], next, &[]);
            b.switch_to_block(matched);
            let mut args = vec![];
            if p.input.functions[c].instance {
                args.push(b.ins().load(types::I64, MemFlags::new(), descriptor, 16));
            }
            args.extend([result, params[1]]);
            emit_stack_check(module, &mut b, stack_check, diagnostic_data, params[1], c, false);
            let callee = module.declare_func_in_func(ids[c], b.func);
            let call = b.ins().call(callee, &args);
            let status = b.inst_results(call)[0];
            b.ins().return_(&[status]);
            b.switch_to_block(next);
        }
        let invalid = b.ins().iconst(types::I32, 3);
        b.ins().jump(rejected, &[invalid.into()]);
        b.switch_to_block(rejected);
        let status = b.block_params(rejected)[0];
        b.ins().return_(&[status]);
        b.seal_all_blocks();
        b.finalize();
    }
    define_checked(module, export, &mut context, stack_check.is_some())?;
    Ok(())
}

// Guard after publishing the current frame, before executing guest operations.
// All returns use the existing frame unlink instrumentation, including this fault.
fn emit_stack_check(module: &mut ObjectModule, b: &mut FunctionBuilder<'_>,
    check: Option<cranelift_module::FuncId>, data: Option<&crate::fault_details::Data>,
    context: ir::Value, function: usize, capture_frame: bool) {
    let Some(check) = check else { return; };
    let check = module.declare_func_in_func(check, b.func);
    let call = b.ins().call(check, &[]);
    let status = b.inst_results(call)[0];
    let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
    let mut site = data.unwrap().site(module, b, context, function, 0);
    site.message = None;
    site.capture_frame = capture_frame;
    return_if_detailed(b, failed, status, Some(&site));
}
fn define_checked(module: &mut ObjectModule, id: cranelift_module::FuncId,
    context: &mut cranelift_codegen::Context, stack_budget: bool) -> Result<(), Error> {
    module.define_function(id, context)?;
    if stack_budget {
        // Cranelift 0.121.2 frame_size excludes FP/LR and ephemeral outgoing args.
        // This backend uses fixed I32/I64 arguments only: charge eight bytes for
        // every argument (including register args) and round to the ARM64 alignment.
        let outgoing = context.func.dfg.signatures.values()
            .map(|sig| sig.params.len() as u64 * 8).max().unwrap_or(0);
        let outgoing = (outgoing + 15) & !15;
        let bytes = u64::from(context.compiled_code().ok_or("missing final machine frame")?.frame_size)
            + 16 + outgoing;
        if bytes > 65536 {
            return Err("native stack budget requires final machine frames including call arguments at most 64 KiB".into());
        }
    }
    Ok(())
}

fn compile_host_queue(module: &mut ObjectModule, target: cranelift_module::FuncId,
    drain: usize, stack_check: Option<cranelift_module::FuncId>, data: &crate::fault_details::Data) -> Result<(), Error> {
    let mut sig = module.make_signature();
    sig.params.extend([types::I64, types::I64].map(AbiParam::new));
    sig.returns.push(AbiParam::new(types::I32));
    let reader = module.declare_function("neoclr_task_queue_host_read_v1", Linkage::Import, &sig)?;
    sig.params.pop();
    let export = module.declare_function("neoclr_drain_default_queue_v1", Linkage::Export, &sig)?;
    let mut context = module.make_context();
    context.func.signature = sig;
    let mut fb = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut context.func, &mut fb);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let ctx = b.block_params(entry)[0];
        let storage = slot(&mut b, 16);
        let output = b.ins().stack_addr(types::I64, storage, 0);
        let reader = module.declare_func_in_func(reader, b.func);
        let call = b.ins().call(reader, &[ctx, output]);
        let status = b.inst_results(call)[0];
        let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
        return_if_detailed(&mut b, failed, status, None);
        let queue = b.ins().load(types::I64, MemFlags::new(), output, 0);
        let empty = b.create_block();
        let work = b.create_block();
        let absent = b.ins().icmp_imm(IntCC::Equal, queue, 0);
        b.ins().brif(absent, empty, &[], work, &[]);
        b.switch_to_block(empty);
        let zero = b.ins().iconst(types::I32, 0);
        b.ins().return_(&[zero]);
        b.switch_to_block(work);
        emit_stack_check(module, &mut b, stack_check, Some(data), ctx, drain, false);
        let result = b.ins().stack_addr(types::I64, storage, 8);
        let callee = module.declare_func_in_func(target, b.func);
        let call = b.ins().call(callee, &[queue, result, ctx]);
        let status = b.inst_results(call)[0];
        b.ins().return_(&[status]);
        b.seal_all_blocks(); b.finalize();
    }
    define_checked(module, export, &mut context, stack_check.is_some())
}
