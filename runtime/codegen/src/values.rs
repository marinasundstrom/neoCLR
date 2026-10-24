//! Inline value/member lowering. Records travel as field snapshots, not owning pointers.
use super::{Error, checked_arithmetic, flow, return_if_detailed};
#[path = "enum_display.rs"]
mod enum_display;
#[path = "gc_layout.rs"]
mod gc_layout;
#[path = "gc_points.rs"]
mod gc_points;
#[path = "gc_probe.rs"]
mod gc_probe;
#[path = "object_upcasts.rs"]
mod object_upcasts;
#[path = "value_profile.rs"]
mod profile;

pub(super) fn trace_layout(
    input: &neoclr::Module,
    details: Option<&crate::fault_details::Options>,
) -> Result<serde_json::Value, Error> {
    let mut p = Profile::new(
        input,
        details.is_some_and(|d| d.reference_arena),
        details.and_then(|d| d.object_base),
        details.and_then(|d| d.array_backing),
        details.map_or(&[], |d| d.reference_array_backings.as_slice()),
        details.map(|d| &d.object_display),
        details.map(|d| &d.string_dispatch),
        details.map(|d| &d.interface_dispatch),
        details.map(|d| d.primitive_receivers.as_slice()),
        details.is_some_and(|d| d.native_stack_budget),
    )?;
    p.boxed_interfaces = details
        .map(|d| d.boxed_interfaces.clone())
        .unwrap_or_default();
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
        Ty::Tuple(parts) => parts.iter().flat_map(|t| lanes(p, t)).collect(),
        Ty::ValueUnion(parts) => std::iter::once(types::I32)
            .chain(parts.iter().flat_map(|t| lanes(p, t)))
            .collect(),
        Ty::PendingUnionScalar(t) => lanes(p, &p.ty(t).expect("admitted union payload")),
        Ty::Double => vec![types::F64],
        Ty::Single => vec![types::F32],
        Ty::Record(i) if !p.input.types[*i].fields.is_empty() => p.input.types[*i]
            .fields
            .iter()
            .flat_map(|f| lanes(p, &p.ty(&f.ty).expect("admitted field")))
            .collect(),
        _ => p
            .pointer_lanes(t)
            .into_iter()
            .map(|pointer| if pointer { types::I64 } else { types::I32 })
            .collect(),
    }
}
// Repack exact alternatives between private union layouts. Numeric bits and
// traced references never share a slot, and inactive payload banks remain zero.
fn widen_union(
    p: &Profile<'_>,
    b: &mut FunctionBuilder<'_>,
    source: &Ty,
    target: &Ty,
    values: Vec<ir::Value>,
) -> Vec<ir::Value> {
    if source == target
        || matches!(
            (source, target),
            (Ty::ScalarUnion(_), Ty::ScalarUnion(_))
                | (Ty::MixedUnion(_), Ty::MixedUnion(_))
                | (Ty::ReferenceUnion(_), Ty::ReferenceUnion(_))
        )
    {
        return values;
    }
    let from = p.union_parts(source).expect("union source");
    let to = p.union_parts(target).expect("union target");
    let mut result: Vec<_> = lanes(p, target).into_iter().map(|ty| zero(b, ty)).collect();
    let mut offset = 1;
    for (index, part) in from.iter().enumerate() {
        let reference = matches!(part, Ty::Reference(_) | Ty::Interface(_));
        let scalar_tag = match part {
            Ty::Int => 1,
            Ty::Wide => 2,
            Ty::Bool => 3,
            _ => 4,
        };
        let (active, payload) = match source {
            Ty::ValueUnion(_) => {
                let active = b.ins().icmp_imm(IntCC::Equal, values[0], index as i64 + 1);
                let payload = values[offset..offset + p.lanes(part)].to_vec();
                offset += p.lanes(part);
                (active, payload)
            }
            Ty::ReferenceUnion(_) => (
                b.ins().icmp_imm(IntCC::NotEqual, values[0], 0),
                vec![values[0]],
            ),
            Ty::ScalarUnion(_) | Ty::MixedUnion(_) => {
                let active = b.ins().icmp_imm(IntCC::Equal, values[0], scalar_tag);
                let bits = values[if reference { 2 } else { 1 }];
                let payload = if matches!(part, Ty::Int | Ty::Bool) {
                    b.ins().ireduce(types::I32, bits)
                } else {
                    bits
                };
                (active, vec![payload])
            }
            _ => unreachable!(),
        };
        let (tag, destination, payload) = match target {
            Ty::ValueUnion(_) => {
                // Reference alternatives share the first bank. Nominal checks
                // inspect the referenced object's identity, not this private tag.
                let index = to
                    .iter()
                    .position(|candidate| {
                        if reference {
                            matches!(candidate, Ty::Reference(_) | Ty::Interface(_))
                        } else {
                            candidate == part
                        }
                    })
                    .expect("included alternative");
                (
                    index as i64 + 1,
                    1 + to[..index].iter().map(|t| p.lanes(t)).sum::<usize>(),
                    payload,
                )
            }
            Ty::ScalarUnion(_) | Ty::MixedUnion(_) => {
                let bits = if matches!(part, Ty::Int | Ty::Bool) {
                    b.ins().uextend(types::I64, payload[0])
                } else {
                    payload[0]
                };
                (scalar_tag, if reference { 2 } else { 1 }, vec![bits])
            }
            _ => unreachable!("inline widening target"),
        };
        let tag = b.ins().iconst(types::I32, tag);
        result[0] = b.ins().select(active, tag, result[0]);
        for (slot, value) in payload.into_iter().enumerate() {
            result[destination + slot] = b.ins().select(active, value, result[destination + slot]);
        }
    }
    result
}

fn zero(b: &mut FunctionBuilder<'_>, ty: ir::Type) -> ir::Value {
    if ty == types::F64 {
        b.ins().f64const(0.0)
    } else if ty == types::F32 {
        b.ins().f32const(0.0)
    } else {
        b.ins().iconst(ty, 0)
    }
}

// Cranelift 0.121's ARM64 lowering lacks unordered relational FloatCCs.
// Invert the complementary ordered test so NaN still follows CLI .un semantics.
fn float_compare(
    b: &mut FunctionBuilder<'_>,
    cc: IntCC,
    left: ir::Value,
    right: ir::Value,
) -> ir::Value {
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
    if invert {
        b.ins().bxor_imm(result, 1)
    } else {
        result
    }
}

fn read(
    b: &mut FunctionBuilder<'_>,
    p: &Profile<'_>,
    t: &Ty,
    pointer: ir::Value,
) -> Vec<ir::Value> {
    lanes(p, t)
        .into_iter()
        .zip(p.narrow_lanes(t))
        .enumerate()
        .map(|(i, (ty, narrow))| {
            let storage = narrow.map_or(
                ty,
                |(bits, _)| if bits == 8 { types::I8 } else { types::I16 },
            );
            let value = b
                .ins()
                .load(storage, MemFlags::new(), pointer, i as i32 * 8);
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
        .map(|(v, narrow)| {
            if let Some((bits, signed)) = narrow {
                let narrow = b
                    .ins()
                    .ireduce(if bits == 8 { types::I8 } else { types::I16 }, *v);
                if signed {
                    b.ins().sextend(types::I32, narrow)
                } else {
                    b.ins().uextend(types::I32, narrow)
                }
            } else {
                *v
            }
        })
        .collect()
}
fn write_typed(
    b: &mut FunctionBuilder<'_>,
    p: &Profile<'_>,
    t: &Ty,
    pointer: ir::Value,
    values: &[ir::Value],
) {
    let values: Vec<_> = normalize(b, p, t, values)
        .into_iter()
        .zip(p.narrow_lanes(t))
        .map(|(value, narrow)| {
            if let Some((bits, _)) = narrow {
                b.ins()
                    .ireduce(if bits == 8 { types::I8 } else { types::I16 }, value)
            } else {
                value
            }
        })
        .collect();
    write(b, pointer, &values);
}
fn slot(b: &mut FunctionBuilder<'_>, bytes: u32) -> ir::StackSlot {
    b.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, bytes, 3))
}
fn args(values: &[ir::Value]) -> Vec<ir::BlockArg> {
    values.iter().copied().map(Into::into).collect()
}

fn matches_type(
    b: &mut FunctionBuilder<'_>,
    p: &Profile<'_>,
    tag: ir::Value,
    index: usize,
) -> ir::Value {
    if p.array_backing == Some(index) {
        let ordinary = b.ins().icmp_imm(IntCC::Equal, tag, 0x80000001);
        let reserved = b.ins().icmp_imm(IntCC::Equal, tag, 0x80000002);
        b.ins().bor(ordinary, reserved)
    } else if p.reference_array_backings.contains(&index) {
        let owner = b.ins().ushr_imm(tag, 32);
        b.ins().icmp_imm(IntCC::Equal, owner, index as i64 + 1)
    } else {
        b.ins().icmp_imm(IntCC::Equal, tag, index as i64)
    }
}

fn tag_array(b: &mut FunctionBuilder<'_>, p: &Profile<'_>, ty: &Ty, array: ir::Value) {
    if let Some(index) = p.backing_for_array(ty) {
        if p.array_backing == Some(index) {
            return;
        }
        let kind = b.ins().load(types::I64, MemFlags::new(), array, 0);
        let kind = b.ins().band_imm(kind, 0xffffffff);
        let tagged = b.ins().bor_imm(kind, (index as i64 + 1) << 32);
        b.ins().store(MemFlags::new(), tagged, array, 0);
    }
}

// Private Object views use the low bit of aligned text pointers. Null stays zero.
// Every dereference of a dynamically typed receiver must distinguish this view first.
fn text_object_view(b: &mut FunctionBuilder<'_>, value: ir::Value) -> ir::Value {
    let present = b.ins().icmp_imm(IntCC::NotEqual, value, 0);
    let tagged = b.ins().bor_imm(value, 1);
    b.ins().select(present, tagged, value)
}

// Copy immutable bytes into a fresh String owner when identity is observable.
fn copy_text(
    module: &mut impl Module,
    b: &mut FunctionBuilder<'_>,
    binding: (cranelift_module::FuncId, cranelift_module::DataId),
    pointer: ir::Value,
    context: ir::Value,
    site: Option<&crate::fault_details::Site>,
) -> ir::Value {
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
    b.ins()
        .brif(equal, scan, &[zero.into()], done, &[no.into()]);
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

fn null_reference(
    b: &mut FunctionBuilder<'_>,
    pointer: ir::Value,
    site: Option<&crate::fault_details::Site>,
) {
    let null = b.ins().icmp_imm(IntCC::Equal, pointer, 0);
    let status = b.ins().iconst(types::I32, 6);
    return_if_detailed(b, null, status, site);
}

// Value-array slots have fixed extent after their first assignment, matching the
// interpreter even though immutable snapshots can share their backing bytes.
fn check_value_array_replacement(
    b: &mut FunctionBuilder<'_>,
    address: ir::Value,
    value: ir::Value,
    site: Option<&crate::fault_details::Site>,
) {
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

pub(super) fn compile(
    input: &neoclr::Module,
    root: &str,
    details: Option<&crate::fault_details::Options>,
    platform_target: super::Target,
) -> Result<Vec<u8>, Error> {
    let mut flags = settings::builder();
    flags.set("is_pic", "true")?;
    platform_target.configure_stack(&mut flags)?;
    let isa = isa::lookup(platform_target.triple().parse().unwrap())?
        .finish(settings::Flags::new(flags))?;
    let mut module = ObjectModule::new(ObjectBuilder::new(
        isa,
        "neoclr_values",
        cranelift_module::default_libcall_names(),
    )?);
    compile_into(input, root, details, platform_target, &mut module, false)?;
    super::finish(module)
}

pub(crate) fn compile_into(
    input: &neoclr::Module,
    root: &str,
    details: Option<&crate::fault_details::Options>,
    platform_target: super::Target,
    module: &mut impl Module,
    long_range_calls: bool,
) -> Result<(), Error> {
    let live_stores = details.is_some_and(|d| d.experimental_live_gc_stores);
    if live_stores && !details.is_some_and(|d| d.native_gc && d.probe_stack_roots
        && d.reference_arena && !d.native_worker_safepoints && !d.native_host_bootstrap
        && !d.entry_host_io && d.entry_task_drain.is_empty()) {
        return Err("experimental live stores require owned published frames without host bootstrap/services".into());
    }
    if details.is_some_and(|d| !d.native_gc && !d.string_intern.is_empty()) {
        return Err(
            "native String.Intern requires --native-gc for invocation-scoped strong ownership"
                .into(),
        );
    }
    if details.is_some_and(|d| d.native_worker_safepoints)
        && (!details.is_some_and(|d| {
            d.native_gc && d.native_host_bootstrap && d.reference_arena && d.probe_stack_roots
        }) || platform_target != super::Target::MacosArm64)
    {
        return Err("worker safepoints require the macOS ARM64 native-GC bootstrap profile".into());
    }
    let reservations = input
        .functions
        .iter()
        .any(|f| f.body.iter().any(|op| matches!(op, Op::ReserveArray(_))));
    let references = details.is_some_and(|d| d.reference_arena);
    if details.is_some_and(|d| d.native_gc && (!d.reference_arena || !d.probe_stack_roots)) {
        return Err(
            "native GC requires an admitted reference-arena profile and published roots".into(),
        );
    }
    let stack_budget = details.is_some_and(|d| d.native_stack_budget);
    if stack_budget && !details.is_some_and(|d| d.native_gc) {
        return Err("native stack budget requires GC frame publication".into());
    }
    let mut p = Profile::new(
        input,
        references,
        details.and_then(|d| d.object_base),
        details.and_then(|d| d.array_backing),
        details.map_or(&[], |d| d.reference_array_backings.as_slice()),
        details.map(|d| &d.object_display),
        details.map(|d| &d.string_dispatch),
        details.map(|d| &d.interface_dispatch),
        details.map(|d| d.primitive_receivers.as_slice()),
        details.is_some_and(|d| d.native_stack_budget),
    )?;
    p.boxed_interfaces = details
        .map(|d| d.boxed_interfaces.clone())
        .unwrap_or_default();
    if live_stores && input.functions.iter().any(|f| f.body.iter().any(|op|
        matches!(op, Op::StoreArrayElement(neoclr::metadata::Type::Function(_))))) {
        return Err("live stores require qualification for callable arrays".into());
    }
    if live_stores && input.functions.iter().enumerate().any(|(i,f)|
        f.is_internal_call() || f.pinvoke.is_some() || gc_points::native_body(&p,i,details)) {
        return Err("experimental live stores reject unaudited native replacement bodies".into());
    }
    let bootstrap = details.is_some_and(|d| d.native_host_bootstrap);
    if bootstrap && !details.is_some_and(|d| d.native_gc) {
        return Err("native host bootstrap requires native GC".into());
    }
    let root = p.root(root, bootstrap)?;
    let entry_arguments = p.args[root] == [Ty::StringArray];
    if live_stores && entry_arguments { return Err("live stores do not admit process-argument providers".into()); }
    if entry_arguments && !details.is_some_and(|d| d.native_gc) {
        return Err("String[] entry requires native GC and a process argument host".into());
    }
    // Identity is scoped to this linked image. No pointer or persistent numeric
    // value is exposed by the guest RuntimeTypeHandle contract.
    let mut type_tokens = Vec::new();
    for f in &input.functions {
        for op in &f.body {
            if let Op::LoadTypeToken(t) = op
                && !type_tokens.contains(t)
            {
                type_tokens.push(t.clone());
            }
        }
    }
    let mut metadata_tokens = Vec::new();
    for op in input.functions.iter().flat_map(|f| &f.body) {
        if let Op::LoadMetadataToken(index) = op {
            neoclr::native_metadata::descriptor(input, *index).map_err(|e| e.to_string())?;
            if !metadata_tokens.contains(index) {
                metadata_tokens.push(*index);
            }
        }
    }
    let flows: Vec<_> = (0..input.functions.len())
        .map(|i| p.analyze(i))
        .collect::<Result<_, _>>()?;
    let string_variance = p.dispatch.iter().any(|(contract, targets)| {
        targets.iter().any(|(_, target)| {
            p.args[*contract]
                .iter()
                .zip(&p.args[*target])
                .any(|(from, to)| *from == Ty::Literal && from != to)
                || p.results[*target] == Some(Ty::Literal)
                    && p.results[*contract] != Some(Ty::Literal)
        })
    });
    let text_identity = string_variance
        || input.functions.iter().enumerate().any(|(i, f)| {
            f.body.iter().enumerate().any(|(pc, op)| {
                let Some(shape) = flows[i].get(pc).and_then(Option::as_ref) else {
                    return false;
                };
                match op {
                    Op::CastClass(t) | Op::IsInstance(t) => {
                        shape.last() == Some(&Ty::Literal) && *t != neoclr::metadata::Type::String
                            || *t == neoclr::metadata::Type::String
                                && shape.last() != Some(&Ty::Literal)
                    }
                    Op::ReferenceEqual => shape.iter().rev().take(2).any(|t| *t == Ty::Literal),
                    _ => false,
                }
            })
        });
    let string_interfaces = details.and_then(|d| d.string_interfaces.as_ref());
    if text_identity {
        let interface_casts = input.functions.iter().any(|f| f.body.iter().any(|op| {
            matches!(op, Op::CastClass(t) | Op::IsInstance(t) if matches!(p.ty(t), Ok(Ty::Interface(_) | Ty::Intersection(_) | Ty::ReferenceUnion(_))))
        }));
        if interface_casts && string_interfaces.is_none() {
            return Err("String interface views require verified load-set conformance".into());
        }
    }
    let nominal_string_view = |ty: &Ty| {
        p.is_object_base(ty)
            || matches!(ty, Ty::Interface(i) if string_interfaces.is_some_and(|indices| indices.contains(i)))
    };
    let string_view = |ty: &Ty| match ty {
        Ty::Intersection(parts) => parts.iter().all(nominal_string_view),
        Ty::ReferenceUnion(parts) => parts.iter().any(nominal_string_view),
        _ => nominal_string_view(ty),
    };
    // The backend's narrow shape analysis is additional admission, not a replacement
    // for type/member identity, accessibility, initialization or byref lifetime checks.
    object_upcasts::verify(&p, &flows)?;
    // Immutable length-prefixed UTF-8. Strings originate from these image literals
    // or explicit, bounded invocation-arena producers. When identity is observable,
    // literal evaluations receive fresh arena storage; image data remains a template.
    // String Object views retain that pointer; no arbitrary host/escaping String is admitted.
    // A default neoCLR Char is the single NUL grapheme, never a null pointer.
    let default_character =
        module.declare_data("neoclr_default_character", Linkage::Local, false, false)?;
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
                let id = module.declare_data(
                    &format!("neoclr_literal_{i}_{pc}"),
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
                literals.insert((i, pc), id);
            }
        }
    }
    let enum_display = enum_display::Display::prepare(input, details, module)?;
    // Extra integral boxes use the existing signed/unsigned wide text services.
    // Width changes are register conversions; nominal tags and box storage stay intact.
    let integral_display = if details.is_some_and(|d| {
        !d.integral_boxes.is_empty()
            && d.object_display
                .keys()
                .any(|i| !d.object_value_kinds.contains_key(i))
    }) {
        let rooted = details.unwrap().native_gc;
        let mut sig = module.make_signature();
        sig.params
            .extend([types::I64, types::I64, types::I64].map(AbiParam::new));
        sig.returns.push(AbiParam::new(types::I32));
        Some((
            module.declare_function(
                if rooted {
                    "neoclr_int64_to_string_rooted_v1"
                } else {
                    "neoclr_int64_to_string_v1"
                },
                Linkage::Import,
                &sig,
            )?,
            module.declare_function(
                if rooted {
                    "neoclr_uint64_to_string_rooted_v1"
                } else {
                    "neoclr_uint64_to_string_v1"
                },
                Linkage::Import,
                &sig,
            )?,
        ))
    } else {
        None
    };

    let mut floating_display = vec![];
    if let Some(d) = details.filter(|d| {
        !d.floating_boxes.is_empty()
            && d.object_display
                .keys()
                .any(|i| !d.object_value_kinds.contains_key(i))
    }) {
        for (ty, symbol) in [
            (
                types::F32,
                if d.native_gc {
                    "neoclr_single_format_rooted_v1"
                } else {
                    "neoclr_single_format_v1"
                },
            ),
            (
                types::F64,
                if d.native_gc {
                    "neoclr_double_format_rooted_v1"
                } else {
                    "neoclr_double_format_v1"
                },
            ),
        ] {
            let mut sig = module.make_signature();
            sig.params.extend(
                [
                    ty,
                    types::I32,
                    types::I32,
                    types::I32,
                    types::I64,
                    types::I64,
                ]
                .map(AbiParam::new),
            );
            sig.returns.push(AbiParam::new(types::I32));
            floating_display.push(module.declare_function(symbol, Linkage::Import, &sig)?);
        }
    }
    let mut default_display_literals = vec![];
    if let Some(details) = details {
        for (index, text) in &details.object_default_display {
            let id = module.declare_data(
                &format!("neoclr_default_display_{index}"),
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
            default_display_literals.push((*index, id));
        }
    }
    let mut boolean_display_literals = vec![];
    if details.is_some_and(|d| d.boxed_boolean_display) {
        for text in ["False", "True"] {
            let id = module.declare_data(
                &format!("neoclr_boolean_display_{text}"),
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
            boolean_display_literals.push(id);
        }
    }
    let text_copy = if text_identity {
        let id = module.declare_data("neoclr_identity_empty", Linkage::Local, false, false)?;
        let mut data = DataDescription::new();
        data.define(vec![0; 8].into_boxed_slice());
        data.set_align(8);
        module.define_data(id, &data)?;
        let mut sig = module.make_signature();
        sig.params.extend([types::I64; 4].map(AbiParam::new));
        sig.returns.push(AbiParam::new(types::I32));
        Some((
            module.declare_function(
                if details.is_some_and(|d| d.native_gc) {
                    "neoclr_string_concat_rooted_v1"
                } else {
                    "neoclr_string_concat_v1"
                },
                Linkage::Import,
                &sig,
            )?,
            id,
        ))
    } else {
        None
    };
    let diagnostic_data = details
        .map(|options| crate::fault_details::Data::new(module, input, options))
        .transpose()?;
    let mut terminal_services = Vec::new();
    if details.is_some_and(|d| !d.console_terminal.is_empty()) {
        for (symbol, count) in [
            ("neoclr_terminal_scope_open_v1", 1),
            ("neoclr_terminal_scope_read_byte_v1", 2),
            ("neoclr_terminal_scope_close_v1", 1),
        ] {
            let mut sig = module.make_signature();
            sig.params.push(AbiParam::new(types::I64));
            for _ in 0..count {
                sig.params.push(AbiParam::new(types::I32));
            }
            sig.returns.push(AbiParam::new(types::I32));
            terminal_services.push(module.declare_function(symbol, Linkage::Import, &sig)?);
        }
    }
    let input_service = if details.is_some_and(|d| !d.console_read_byte.is_empty()) {
        let mut sig = module.make_signature();
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_console_read_byte_v1", Linkage::Import, &sig)?)
    } else {
        None
    };
    let output_service = if details.is_some_and(|d| !d.console_write_line.is_empty()) {
        let mut sig = module.make_signature();
        sig.params
            .extend([AbiParam::new(types::I64), AbiParam::new(types::I64)]);
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_console_write_line_utf8_v1", Linkage::Import, &sig)?)
    } else {
        None
    };
    let cryptography_service = if details.is_some_and(|d| !d.cryptography.is_empty()) {
        let mut sig = module.make_signature();
        sig.params.extend(
            [
                types::I32,
                types::I64,
                types::I64,
                types::I64,
                types::I64,
                types::I64,
            ]
            .map(AbiParam::new),
        );
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_cryptography_slices_v1", Linkage::Import, &sig)?)
    } else {
        None
    };
    let tls_service = if details.is_some_and(|d| !d.tls.is_empty()) {
        let mut sig = module.make_signature();
        sig.params.extend(
            [
                types::I32,
                types::I64,
                types::I64,
                types::I64,
                types::I64,
                types::I64,
                types::I64,
                types::I64,
            ]
            .map(AbiParam::new),
        );
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_tls_managed_v2", Linkage::Import, &sig)?)
    } else {
        None
    };
    let console_memory_access = if details.is_some_and(|d| !d.console_write_memory.is_empty()) {
        let mut sig = module.make_signature();
        sig.params
            .extend([types::I64, types::I64, types::I32, types::I64].map(AbiParam::new));
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_byte_view_access_v1", Linkage::Import, &sig)?)
    } else {
        None
    };
    let mut stream_services = std::collections::HashMap::new();
    if let Some(d) = details {
        for (indices, symbol, parameters) in [
            (
                &d.console_write_bytes,
                "neoclr_console_write_bytes_v1",
                vec![types::I32, types::I64, types::I64, types::I32, types::I32],
            ),
            (
                &d.console_flush,
                "neoclr_console_flush_v1",
                vec![types::I32],
            ),
        ] {
            if indices.is_empty() {
                continue;
            }
            let mut sig = module.make_signature();
            sig.params.extend(parameters.into_iter().map(AbiParam::new));
            sig.returns.push(AbiParam::new(types::I32));
            let service = module.declare_function(symbol, Linkage::Import, &sig)?;
            for index in indices {
                stream_services.insert(*index, service);
            }
        }
    }
    let entry_drain_services = if details.is_some_and(|d| !d.entry_task_drain.is_empty()) {
        let mut sig = module.make_signature();
        sig.params.extend(
            [types::I64, types::I32, types::I32, types::I32, types::I64].map(AbiParam::new),
        );
        sig.returns.push(AbiParam::new(types::I32));
        let begin =
            module.declare_function("neoclr_entry_tasks_begin_v1", Linkage::Import, &sig)?;
        sig.params.truncate(1);
        sig.returns.clear();
        let end = module.declare_function("neoclr_entry_tasks_end_v1", Linkage::Import, &sig)?;
        sig.params = [types::I64, types::I32, types::I64]
            .map(AbiParam::new)
            .to_vec();
        sig.returns.push(AbiParam::new(types::I32));
        let poll = module.declare_function("neoclr_entry_tasks_poll_v1", Linkage::Import, &sig)?;
        sig.params = [types::I64, types::I64].map(AbiParam::new).to_vec();
        let callback =
            module.declare_function("neoclr_invoke_entry_callback_v1", Linkage::Export, &sig)?;
        let queue =
            module.declare_function("neoclr_task_queue_default_v1", Linkage::Import, &sig)?;
        Some((begin, end, poll, callback, queue))
    } else {
        None
    };
    let runtime_async_callbacks = if details.is_some_and(|d| {
        !d.runtime_async_start.is_empty() || !d.runtime_async_startmanaged.is_empty()
    }) {
        let mut sig = module.make_signature();
        sig.params
            .extend([types::I64, types::I64].map(AbiParam::new));
        sig.returns.push(AbiParam::new(types::I32));
        let callback =
            module.declare_function("neoclr_invoke_void_callback_v1", Linkage::Export, &sig)?;
        sig.params.pop();
        let drain =
            module.declare_function("neoclr_drain_default_queue_v1", Linkage::Export, &sig)?;
        sig.params = [types::I64, types::I64, types::I64]
            .map(AbiParam::new)
            .to_vec();
        let pump =
            module.declare_function("neoclr_runtime_async_pump_v1", Linkage::Import, &sig)?;
        Some((callback, drain, pump))
    } else {
        None
    };
    let mut socket_services = std::collections::HashMap::new();
    if let Some(d) = details {
        for (indices, symbol, parameters) in [
            (
                &d.task_queue_register,
                "neoclr_task_queue_register_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.task_queue_default,
                "neoclr_task_queue_default_v1",
                vec![types::I64, types::I64],
            ),
            (
                &d.task_queue_current,
                "neoclr_task_queue_current_v1",
                vec![types::I64, types::I32, types::I32, types::I64],
            ),
            (
                &d.scheduler_register,
                "neoclr_scheduler_register_managed_v1",
                vec![types::I64, types::I64, types::I64, types::I64],
            ),
            (
                &d.scheduler_activate,
                "neoclr_scheduler_activate_managed_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.scheduler_cancel,
                "neoclr_scheduler_cancel_managed_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.scheduler_capture,
                "neoclr_scheduler_capture_managed_v1",
                vec![types::I64, types::I64],
            ),
            (
                &d.scheduler_submit_to,
                "neoclr_scheduler_submit_to_managed_v1",
                vec![types::I64, types::I64, types::I64, types::I64],
            ),
            (
                &d.task_schedule,
                "neoclr_schedule_task_managed_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.scheduler_submit,
                "neoclr_scheduler_submit_managed_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.stack_trace_capture,
                "neoclr_stack_trace_capture_v1",
                vec![types::I32, types::I64, types::I64, types::I64, types::I32],
            ),
            (
                &d.gc_collect,
                "neoclr_gc_collect_managed_v1",
                vec![types::I64, types::I64],
            ),
            (
                &d.runtime_async_create,
                "neoclr_runtime_async_create_v1",
                vec![types::I64, types::I64],
            ),
            (
                &d.runtime_async_startmanaged,
                "neoclr_runtime_async_startmanaged_v1",
                vec![types::I64; 5],
            ),
            (
                &d.runtime_async_createowned,
                "neoclr_runtime_async_createowned_v1",
                vec![types::I64; 3],
            ),
            (
                &d.runtime_async_currentowner,
                "neoclr_runtime_async_currentowner_v1",
                vec![types::I64; 2],
            ),
            (
                &d.runtime_async_start,
                "neoclr_runtime_async_start_v1",
                vec![types::I64, types::I64, types::I64, types::I64],
            ),
            (
                &d.runtime_async_completeunit,
                "neoclr_runtime_async_completeunit_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.runtime_async_awaitunit,
                "neoclr_runtime_async_awaitunit_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.runtime_async_release,
                "neoclr_runtime_async_release_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.runtime_async_iscompleted,
                "neoclr_runtime_async_iscompleted_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.runtime_async_waitcompletion,
                "neoclr_runtime_async_waitcompletion_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.timer_start,
                "neoclr_timer_start_v1",
                vec![types::I32, types::I64, types::I64, types::I64],
            ),
            (
                &d.timer_cancel,
                "neoclr_timer_cancel_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.socket_deadline_after,
                "neoclr_socket_deadline_after_v1",
                vec![types::I32, types::I64, types::I64],
            ),
            (
                &d.socket_deadline_expired,
                "neoclr_socket_deadline_expired_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.socket_receive_until,
                "neoclr_socket_receive_until_v1",
                vec![
                    types::I64,
                    types::I64,
                    types::I32,
                    types::I32,
                    types::I64,
                    types::I64,
                    types::I64,
                    types::I64,
                ],
            ),
            (
                &d.socket_send_until,
                "neoclr_socket_send_until_v1",
                vec![
                    types::I64,
                    types::I64,
                    types::I32,
                    types::I32,
                    types::I64,
                    types::I64,
                    types::I64,
                    types::I64,
                ],
            ),
            (
                &d.socket_receive,
                "neoclr_socket_receive_v1",
                vec![
                    types::I64,
                    types::I64,
                    types::I32,
                    types::I32,
                    types::I64,
                    types::I64,
                    types::I64,
                ],
            ),
            (
                &d.socket_send,
                "neoclr_socket_send_v1",
                vec![
                    types::I64,
                    types::I64,
                    types::I32,
                    types::I32,
                    types::I64,
                    types::I64,
                    types::I64,
                ],
            ),
            (
                &d.socket_transfer_result,
                "neoclr_socket_transfer_result_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.dns_lookup,
                "neoclr_dns_lookup_v1",
                vec![types::I64, types::I64, types::I64, types::I64],
            ),
            (
                &d.dns_lookup_until,
                "neoclr_dns_lookup_until_v1",
                vec![types::I64, types::I64, types::I64, types::I64, types::I64],
            ),
            (
                &d.dns_cancel,
                "neoclr_dns_cancel_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.dns_result,
                "neoclr_dns_result_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.dns_addresses,
                "neoclr_dns_addresses_v1",
                vec![types::I32, types::I64, types::I64, types::I64],
            ),
            (
                &d.socket_connect,
                "neoclr_socket_connect_v1",
                vec![types::I64, types::I32, types::I64, types::I64, types::I64],
            ),
            (
                &d.socket_connect_addresses,
                "neoclr_socket_connect_addresses_v1",
                vec![types::I64, types::I32, types::I64, types::I64, types::I64],
            ),
            (
                &d.socket_connect_addresses_until,
                "neoclr_socket_connect_addresses_until_v1",
                vec![
                    types::I64,
                    types::I32,
                    types::I64,
                    types::I64,
                    types::I64,
                    types::I64,
                ],
            ),
            (
                &d.socket_accept,
                "neoclr_socket_accept_v1",
                vec![types::I64, types::I64, types::I64, types::I64],
            ),
            (
                &d.socket_connect_result,
                "neoclr_socket_connect_result_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.socket_cancel,
                "neoclr_socket_cancel_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.socket_listen,
                "neoclr_socket_listen_v1",
                vec![types::I64, types::I32, types::I32, types::I64, types::I64],
            ),
            (
                &d.socket_local_port,
                "neoclr_socket_local_port_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.socket_close,
                "neoclr_socket_close_v1",
                vec![types::I64, types::I64, types::I64],
            ),
        ] {
            if indices.is_empty() {
                continue;
            }
            let mut sig = module.make_signature();
            sig.params.extend(parameters.into_iter().map(AbiParam::new));
            sig.returns.push(AbiParam::new(types::I32));
            for index in indices {
                let symbol = if d.socket_memory.contains(index) {
                    symbol.replace("_v1", "_memory_v1")
                } else {
                    symbol.to_owned()
                };
                let service = module.declare_function(&symbol, Linkage::Import, &sig)?;
                socket_services.insert(*index, service);
            }
        }
    }
    let mut utf8_services = std::collections::HashMap::new();
    if let Some(d) = details {
        let mut parsers: Vec<_> = d.primitive_parsers.iter().collect();
        parsers.sort_by_key(|(index, _)| **index);
        for (index, symbol) in parsers {
            let mut sig = module.make_signature();
            sig.params
                .extend([AbiParam::new(types::I64), AbiParam::new(types::I64)]);
            sig.returns.push(AbiParam::new(types::I32));
            utf8_services.insert(
                *index,
                module.declare_function(symbol, Linkage::Import, &sig)?,
            );
        }
    }
    if let Some(d) = details {
        for (indices, symbol, parameters) in [
            (
                &d.file_input,
                if d.native_gc {
                    "neoclr_file_read_utf8_rooted_v1"
                } else {
                    "neoclr_file_read_utf8_v1"
                },
                vec![types::I64, types::I32, types::I64, types::I64],
            ),
            (
                &d.file_output,
                "neoclr_file_write_utf8_v1",
                vec![types::I64, types::I64, types::I32, types::I64],
            ),
            (
                &d.path_combine,
                if d.native_gc {
                    "neoclr_path_combine_unix_rooted_v1"
                } else {
                    "neoclr_path_combine_unix_v1"
                },
                vec![types::I64, types::I64, types::I64, types::I64],
            ),
            (
                &d.path_file_name,
                if d.native_gc {
                    "neoclr_path_file_name_unix_rooted_v1"
                } else {
                    "neoclr_path_file_name_unix_v1"
                },
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.string_compare_ordinal,
                "neoclr_string_compare_ordinal_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.char_is_single_grapheme,
                "neoclr_char_is_single_grapheme_v1",
                vec![types::I64, types::I64],
            ),
            (
                &d.string_contains_canonical,
                "neoclr_string_contains_canonical_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.string_starts_with_canonical,
                "neoclr_string_starts_with_canonical_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.string_ends_with_canonical,
                "neoclr_string_ends_with_canonical_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.string_contains_ignore_case,
                "neoclr_string_contains_ignore_case_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.string_starts_with_ignore_case,
                "neoclr_string_starts_with_ignore_case_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.string_ends_with_ignore_case,
                "neoclr_string_ends_with_ignore_case_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.string_contains_ordinal,
                "neoclr_string_contains_ordinal_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.string_starts_with_ordinal,
                "neoclr_string_starts_with_ordinal_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.string_ends_with_ordinal,
                "neoclr_string_ends_with_ordinal_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.parse_int64,
                "neoclr_parse_int64_v1",
                vec![types::I64, types::I64],
            ),
            (
                &d.parse_int32,
                "neoclr_parse_int32_v1",
                vec![types::I64, types::I64],
            ),
            (
                &d.utf8_encode,
                if d.native_gc {
                    "neoclr_utf8_encode_rooted_v1"
                } else {
                    "neoclr_utf8_encode_v1"
                },
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.utf8_decode_slice,
                if d.native_gc {
                    "neoclr_utf8_decode_slice_rooted_v1"
                } else {
                    "neoclr_utf8_decode_slice_v1"
                },
                vec![types::I64, types::I64, types::I32, types::I32, types::I64],
            ),
            (
                &d.utf8_decode,
                if d.native_gc {
                    "neoclr_utf8_decode_rooted_v1"
                } else {
                    "neoclr_utf8_decode_v1"
                },
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.string_join_parts,
                if d.native_gc {
                    "neoclr_string_join_parts_rooted_v1"
                } else {
                    "neoclr_string_join_parts_v1"
                },
                vec![
                    types::I64,
                    types::I32,
                    types::I64,
                    types::I32,
                    types::I64,
                    types::I64,
                ],
            ),
            (
                &d.string_replace_ordinal,
                if d.native_gc {
                    "neoclr_string_replace_ordinal_rooted_v1"
                } else {
                    "neoclr_string_replace_ordinal_v1"
                },
                vec![types::I64, types::I64, types::I64, types::I64, types::I64],
            ),
            (
                &d.string_replace_line_endings,
                if d.native_gc {
                    "neoclr_string_replace_line_endings_rooted_v1"
                } else {
                    "neoclr_string_replace_line_endings_v1"
                },
                vec![types::I64, types::I64, types::I64, types::I64],
            ),
            (
                &d.string_concat,
                if d.native_gc {
                    "neoclr_string_concat_rooted_v1"
                } else {
                    "neoclr_string_concat_v1"
                },
                vec![types::I64, types::I64, types::I64, types::I64],
            ),
            (
                &d.string_grapheme_count,
                "neoclr_string_grapheme_count_v1",
                vec![types::I64, types::I64],
            ),
            (
                &d.string_grapheme_at,
                if d.native_gc {
                    "neoclr_string_grapheme_at_rooted_v1"
                } else {
                    "neoclr_string_grapheme_at_v1"
                },
                vec![types::I64, types::I32, types::I64, types::I64],
            ),
            (
                &d.string_graphemes,
                if d.native_gc {
                    "neoclr_string_graphemes_rooted_v1"
                } else {
                    "neoclr_string_graphemes_v1"
                },
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.time_zone_exists,
                "neoclr_time_zone_exists_v1",
                vec![types::I64, types::I64],
            ),
            (
                &d.time_zone_offset,
                "neoclr_time_zone_offset_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.time_zone_version,
                if d.native_gc {
                    "neoclr_time_zone_version_rooted_v1"
                } else {
                    "neoclr_time_zone_version_v1"
                },
                vec![types::I64, types::I64],
            ),
            (
                &d.system_culture,
                if d.native_gc {
                    "neoclr_system_culture_name_rooted_v1"
                } else {
                    "neoclr_system_culture_name_v1"
                },
                vec![types::I64, types::I64],
            ),
            (
                &d.environment_platform,
                if d.native_gc {
                    "neoclr_environment_platform_rooted_v1"
                } else {
                    "neoclr_environment_platform_v1"
                },
                vec![types::I32, types::I64, types::I64],
            ),
            (
                &d.string_scalars,
                if d.native_gc {
                    "neoclr_string_scalars_rooted_v1"
                } else {
                    "neoclr_string_scalars_v1"
                },
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.string_from_chars,
                if d.native_gc {
                    "neoclr_string_from_chars_rooted_v1"
                } else {
                    "neoclr_string_from_chars_v1"
                },
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.string_upper,
                if d.native_gc {
                    "neoclr_string_upper_rooted_v1"
                } else {
                    "neoclr_string_upper_v1"
                },
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.string_lower,
                if d.native_gc {
                    "neoclr_string_lower_rooted_v1"
                } else {
                    "neoclr_string_lower_v1"
                },
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.string_compare_ignore_case,
                "neoclr_string_compare_ignore_case_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.string_compare_canonical,
                "neoclr_string_compare_canonical_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.string_hash_ordinal,
                "neoclr_string_hash_ordinal_v1",
                vec![types::I64, types::I64],
            ),
            (
                &d.string_hash_ignore_case,
                "neoclr_string_hash_ignore_case_v1",
                vec![types::I64, types::I64],
            ),
            (
                &d.string_hash_canonical,
                "neoclr_string_hash_canonical_v1",
                vec![types::I64, types::I64],
            ),
            (
                &d.string_intern,
                "neoclr_string_intern_rooted_v1",
                vec![types::I64, types::I64, types::I64],
            ),
            (
                &d.string_byte_count,
                "neoclr_string_byte_count_v1",
                vec![types::I64, types::I64],
            ),
            (
                &d.string_slice_graphemes,
                if d.native_gc {
                    "neoclr_string_slice_graphemes_rooted_v1"
                } else {
                    "neoclr_string_slice_graphemes_v1"
                },
                vec![types::I64, types::I32, types::I32, types::I64, types::I64],
            ),
            (
                &d.string_slice_utf8,
                if d.native_gc {
                    "neoclr_string_slice_utf8_rooted_v1"
                } else {
                    "neoclr_string_slice_utf8_v1"
                },
                vec![types::I64, types::I32, types::I32, types::I64, types::I64],
            ),
        ] {
            if indices.is_empty() {
                continue;
            }
            let mut sig = module.make_signature();
            sig.params.extend(parameters.into_iter().map(AbiParam::new));
            sig.returns.push(AbiParam::new(types::I32));
            let service = module.declare_function(symbol, Linkage::Import, &sig)?;
            for index in indices {
                utf8_services.insert(*index, service);
            }
        }
    }
    let text_arena = references
        || details.is_some_and(|d| {
            !d.int32_to_string.is_empty()
                || !d.int64_to_string.is_empty()
                || !d.uint64_to_string.is_empty()
        });
    let format_service =
        if details.is_some_and(|d| !d.int32_to_string.is_empty() || d.boxed_int32_display) {
            let mut sig = module.make_signature();
            sig.params.extend([
                AbiParam::new(types::I32),
                AbiParam::new(types::I64),
                AbiParam::new(types::I64),
            ]);
            sig.returns.push(AbiParam::new(types::I32));
            Some(module.declare_function(
                if details.is_some_and(|d| d.native_gc) {
                    "neoclr_int32_to_string_rooted_v1"
                } else {
                    "neoclr_int32_to_string_v1"
                },
                Linkage::Import,
                &sig,
            )?)
        } else {
            None
        };
    let boxed_wide_format_service = if details.is_some_and(|d| d.boxed_int64_display) {
        let mut sig = module.make_signature();
        sig.params
            .extend([types::I64, types::I64, types::I64].map(AbiParam::new));
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function(
            if details.is_some_and(|d| d.native_gc) {
                "neoclr_int64_to_string_rooted_v1"
            } else {
                "neoclr_int64_to_string_v1"
            },
            Linkage::Import,
            &sig,
        )?)
    } else {
        None
    };
    let byte_view_utf8 = if details.is_some_and(|d| !d.byte_view_utf8.is_empty()) {
        let mut sig = module.make_signature();
        sig.params.extend([types::I64; 3].map(AbiParam::new));
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_byte_view_utf8_v1", Linkage::Import, &sig)?)
    } else {
        None
    };
    let byte_memory_create = if details.is_some_and(|d| !d.byte_memory_create.is_empty()) {
        let mut sig = module.make_signature();
        sig.params.extend([types::I64; 4].map(AbiParam::new));
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_byte_view_create_all_v1", Linkage::Import, &sig)?)
    } else {
        None
    };
    let byte_view_create_window = if details.is_some_and(|d| !d.byte_memory_create.is_empty()) {
        let mut sig = module.make_signature();
        sig.params.extend(
            [
                types::I64,
                types::I64,
                types::I32,
                types::I32,
                types::I64,
                types::I64,
            ]
            .map(AbiParam::new),
        );
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_byte_view_create_v1", Linkage::Import, &sig)?)
    } else {
        None
    };
    let mut byte_view_access = [None, None];
    for (index, symbol, enabled) in [
        (
            0,
            "neoclr_byte_view_get_v1",
            details.is_some_and(|d| !d.byte_view_get.is_empty()),
        ),
        (
            1,
            "neoclr_byte_view_set_v1",
            details.is_some_and(|d| !d.byte_view_set.is_empty()),
        ),
    ] {
        if enabled {
            let mut sig = module.make_signature();
            sig.params.extend(
                [
                    types::I64,
                    types::I64,
                    types::I32,
                    if index == 0 { types::I64 } else { types::I8 },
                ]
                .map(AbiParam::new),
            );
            sig.returns.push(AbiParam::new(types::I32));
            byte_view_access[index] =
                Some(module.declare_function(symbol, Linkage::Import, &sig)?);
        }
    }
    let byte_memory_slice = if details.is_some_and(|d| !d.byte_memory_slice.is_empty()) {
        let mut sig = module.make_signature();
        sig.params.extend(
            [types::I64, types::I64, types::I32, types::I32, types::I64].map(AbiParam::new),
        );
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_byte_view_slice_v1", Linkage::Import, &sig)?)
    } else {
        None
    };
    let mut wide_format_services = std::collections::HashMap::new();
    if let Some(d) = details {
        for (indices, symbol) in [
            (
                &d.int64_to_string,
                if d.native_gc {
                    "neoclr_int64_to_string_rooted_v1"
                } else {
                    "neoclr_int64_to_string_v1"
                },
            ),
            (
                &d.uint64_to_string,
                if d.native_gc {
                    "neoclr_uint64_to_string_rooted_v1"
                } else {
                    "neoclr_uint64_to_string_v1"
                },
            ),
        ] {
            if indices.is_empty() {
                continue;
            }
            let mut sig = module.make_signature();
            sig.params
                .extend([types::I64, types::I64, types::I64].map(AbiParam::new));
            sig.returns.push(AbiParam::new(types::I32));
            let service = module.declare_function(symbol, Linkage::Import, &sig)?;
            for index in indices {
                wide_format_services.insert(*index, service);
            }
        }
    }
    let mut float_format_services = std::collections::HashMap::new();
    if let Some(d) = details {
        for (indices, ty, symbol) in [
            (
                &d.single_format,
                types::F32,
                if d.native_gc {
                    "neoclr_single_format_rooted_v1"
                } else {
                    "neoclr_single_format_v1"
                },
            ),
            (
                &d.double_format,
                types::F64,
                if d.native_gc {
                    "neoclr_double_format_rooted_v1"
                } else {
                    "neoclr_double_format_v1"
                },
            ),
        ] {
            if indices.is_empty() {
                continue;
            }
            let mut sig = module.make_signature();
            sig.params.extend(
                [
                    ty,
                    types::I32,
                    types::I32,
                    types::I32,
                    types::I64,
                    types::I64,
                ]
                .map(AbiParam::new),
            );
            sig.returns.push(AbiParam::new(types::I32));
            let service = module.declare_function(symbol, Linkage::Import, &sig)?;
            for index in indices {
                float_format_services.insert(*index, service);
            }
        }
    }
    let character_service = if details.is_some_and(|d| !d.char_from_string.is_empty()) {
        let mut sig = module.make_signature();
        sig.params
            .extend([types::I64, types::I64].map(AbiParam::new));
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_is_single_grapheme_v1", Linkage::Import, &sig)?)
    } else {
        None
    };
    let static_service = if input.functions.iter().any(|f| {
        f.name.ends_with("..cctor")
            || f.body
                .iter()
                .any(|op| matches!(op, Op::StaticField { .. } | Op::SetStaticField { .. }))
    }) {
        if !references {
            return Err("static storage requires a native context".into());
        }
        let mut sig = module.make_signature();
        sig.params.extend(
            [types::I64, types::I32, types::I32, types::I32, types::I64].map(AbiParam::new),
        );
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function(
            if live_stores {
                "experiment_gc_static_storage_rooted_v1"
            } else if details.is_some_and(|d| d.native_gc) {
                "neoclr_static_storage_rooted_v1"
            } else {
                "neoclr_static_storage_v1"
            },
            Linkage::Import,
            &sig,
        )?)
    } else {
        None
    };
    let object_service = if references
        && (input.functions.iter().any(|f| {
            f.body
                .iter()
                .any(|op| matches!(op, Op::BindFunction { .. }))
        }) || input
            .types
            .iter()
            .any(|t| t.is_reference_type && !t.is_abstract && !crate::selection::static_owner(t))
            || details.is_some_and(|d| {
                !d.empty_record_boxes.is_empty()
                    || !d.int32_boxes.is_empty()
                    || !d.int64_boxes.is_empty()
                    || !d.boolean_boxes.is_empty()
            })) {
        let mut sig = module.make_signature();
        sig.params.extend([
            AbiParam::new(types::I64),
            AbiParam::new(types::I32),
            AbiParam::new(types::I32),
            AbiParam::new(types::I64),
        ]);
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function(
            if details.is_some_and(|d| d.native_gc) {
                "neoclr_allocate_object_rooted_v1"
            } else {
                "neoclr_allocate_object_v1"
            },
            Linkage::Import,
            &sig,
        )?)
    } else {
        None
    };
    let array_service = if references
        && input.functions.iter().any(|f| {
            f.body
                .iter()
                .any(|op| matches!(op, Op::NewArray(neoclr::metadata::Type::Byte)))
        }) {
        let mut sig = module.make_signature();
        sig.params.extend([
            AbiParam::new(types::I64),
            AbiParam::new(types::I32),
            AbiParam::new(types::I64),
        ]);
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function(
            if details.is_some_and(|d| d.native_gc) {
                "neoclr_allocate_bytes_rooted_v1"
            } else {
                "neoclr_allocate_bytes_v1"
            },
            Linkage::Import,
            &sig,
        )?)
    } else {
        None
    };
    let reserve_service = if references
        && input.functions.iter().any(|f| {
            f.body
                .iter()
                .any(|op| matches!(op, Op::ReserveArray(neoclr::metadata::Type::Byte)))
        }) {
        let mut sig = module.make_signature();
        sig.params
            .extend([types::I64, types::I32, types::I64].map(AbiParam::new));
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function(
            if details.is_some_and(|d| d.native_gc) {
                "neoclr_reserve_bytes_rooted_v1"
            } else {
                "neoclr_reserve_bytes_v1"
            },
            Linkage::Import,
            &sig,
        )?)
    } else {
        None
    };
    let scalar_array_service = if references && input.functions.iter().any(|f| f.body.iter().any(|op|
        matches!(op, Op::NewArray(t) | Op::ReserveArray(t) if crate::selection::scalar_array_element(t)))) {
        let mut sig = module.make_signature();
        sig.params.extend([types::I64, types::I32, types::I32, types::I64].map(AbiParam::new));
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function(if details.is_some_and(|d| d.native_gc) { "neoclr_allocate_scalars_rooted_v1" } else { "neoclr_allocate_scalars_v1" }, Linkage::Import, &sig)?)
    } else { None };
    let default_reference_array_service = if references
        && input.functions.iter().any(|f| {
            f.body.iter().any(|op| {
                matches!(
                    op,
                    Op::NewArray(
                        neoclr::metadata::Type::Named(_) | neoclr::metadata::Type::ArrayRef(_)
                    )
                )
            })
        }) {
        let mut sig = module.make_signature();
        sig.params
            .extend([types::I64, types::I32, types::I64].map(AbiParam::new));
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function(
            if details.is_some_and(|d| d.native_gc) {
                "neoclr_allocate_references_rooted_v1"
            } else {
                "neoclr_allocate_references_v1"
            },
            Linkage::Import,
            &sig,
        )?)
    } else {
        None
    };
    let record_array_service = if references
        && input.functions.iter().any(|f| {
            f.body.iter().any(|op| {
                matches!(
                    op,
                    Op::ReserveArray(
                        neoclr::metadata::Type::Named(_)
                            | neoclr::metadata::Type::Tuple(_)
                            | neoclr::metadata::Type::Union(_)
                            | neoclr::metadata::Type::Intersection(_)
                            | neoclr::metadata::Type::ArrayRef(_)
                    )
                )
            })
        }) {
        let mut sig = module.make_signature();
        sig.params
            .extend([types::I64, types::I32, types::I32, types::I64].map(AbiParam::new));
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function(
            if details.is_some_and(|d| d.native_gc) {
                "neoclr_reserve_records_rooted_v1"
            } else {
                "neoclr_reserve_records_v1"
            },
            Linkage::Import,
            &sig,
        )?)
    } else {
        None
    };
    let reference_array_service = if references && input.functions.iter().any(|f| f.body.iter().any(|op|
        matches!(op, Op::NewArray(t) | Op::ReserveArray(t) if matches!(t, neoclr::metadata::Type::String | neoclr::metadata::Type::Char | neoclr::metadata::Type::Function(_))))) {
        let mut sig = module.make_signature();
        sig.params.extend([types::I64, types::I32, types::I32, types::I64].map(AbiParam::new));
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function(if details.is_some_and(|d| d.native_gc) { "neoclr_allocate_strings_rooted_v1" } else { "neoclr_allocate_strings_v1" }, Linkage::Import, &sig)?)
    } else { None };
    let initialized_service = if reservations
        && details.is_some_and(|d| !d.console_write_bytes.is_empty())
    {
        let mut sig = module.make_signature();
        sig.params
            .extend([types::I64, types::I32, types::I32].map(AbiParam::new));
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_check_bytes_initialized_v1", Linkage::Import, &sig)?)
    } else {
        None
    };
    let mut ids = vec![];
    for (i, _) in input.functions.iter().enumerate() {
        let mut sig = module.make_signature();
        for t in &p.args[i] {
            sig.params
                .extend(lanes(&p, t).into_iter().map(AbiParam::new));
        }
        sig.params.push(AbiParam::new(types::I64)); // caller-owned result storage
        if details.is_some() {
            sig.params.push(AbiParam::new(types::I64));
        }
        sig.returns.push(AbiParam::new(types::I32)); // Fault status, same as scalar profile
        ids.push(module.declare_function(&format!("neoclr_value_{i}"), Linkage::Local, &sig)?);
    }
    let stack_check = if stack_budget {
        let mut sig = module.make_signature();
        sig.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_native_stack_check_v1", Linkage::Import, &sig)?)
    } else {
        None
    };
    let mut live_bodies = Vec::new();
    let initialization_guards = compile_initialization_guards(
        module,
        &p,
        &ids,
        static_service,
        stack_budget,
        long_range_calls,
        if live_stores { Some(&mut live_bodies) } else { None },
    )?;
    if details.is_some_and(|d| d.native_worker_safepoints) && !initialization_guards.is_empty() {
        return Err("worker safepoints do not yet support type initializers".into());
    }
    let root_probes = if details.is_some_and(|d| d.probe_stack_roots) {
        Some(gc_probe::Probes::prepare(module, &p, &flows, details)?)
    } else {
        None
    };
    for (i, f) in input.functions.iter().enumerate() {
        let mut context = module.make_context();
        context.func.signature = module
            .declarations()
            .get_function_decl(ids[i])
            .signature
            .clone();
        let mut fb = FunctionBuilderContext::new();
        let mut probe_frame = None;
        let mut mutation_frame_ready = None;
        {
            let mut b = FunctionBuilder::new(&mut context.func, &mut fb);
            let entry = b.create_block();
            b.append_block_params_for_function_params(entry);
            b.switch_to_block(entry);
            let parameters = b.block_params(entry).to_vec();
            let output = parameters[parameters.len() - 1 - usize::from(details.is_some())];
            let fault_context = details.map(|_| *parameters.last().unwrap());
            if gc_points::native_body(&p, i, details) {
                if crate::type_initialization::entry_owner(input, f)
                    .and_then(|owner| owner.definition_name())
                    .is_some_and(|name| initialization_guards.contains_key(name))
                {
                    return Err(
                        "type initialization on a native replacement body is not supported".into(),
                    );
                }
                if let Some(probes) = &root_probes {
                    // Native wrappers keep typed argument copies visible across service
                    // calls/dispatch. These frames do not enter the guest fault trace.
                    let table_bytes = probes.table_bytes(i);
                    let mut frame_bytes =
                        probes.frame_bytes() as usize + 7 + table_bytes as usize + 7;
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
                    let frame = slot(&mut b, probes.frame_bytes());
                    let table = slot(&mut b, table_bytes);
                    mutation_frame_ready = Some(probes.enter(
                        module,
                        &mut b,
                        frame,
                        fault_context.unwrap(),
                        i,
                        table,
                        &arguments,
                        &[],
                    ));
                    probe_frame = Some(frame);
                    emit_stack_check(
                        module,
                        &mut b,
                        stack_check,
                        diagnostic_data.as_ref(),
                        fault_context.unwrap(),
                        i,
                        false,
                    );
                }
            }
            if details.is_some_and(|d| d.memory_view_bounds_fault.contains(&i)) {
                let status = b.ins().iconst(types::I32, 8);
                let mut site = diagnostic_data.as_ref().unwrap().site(
                    module,
                    &mut b,
                    fault_context.unwrap(),
                    i,
                    0,
                );
                site.capture_frame = false;
                site.record(&mut b, status);
                b.ins().return_(&[status]);
                b.seal_all_blocks();
                b.finalize();
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(module, &mut context.func, frame);
                }
                define_checked(module, ids[i], &mut context, stack_budget, long_range_calls)?;
                continue;
            }
            if details.is_some_and(|d| {
                d.byte_view_get.contains(&i)
                    || d.byte_view_set.contains(&i)
                    || d.byte_memory_create.contains(&i)
                    || d.byte_view_utf8.contains(&i)
                    || d.byte_memory_length.contains(&i)
                    || d.byte_memory_slice.contains(&i)
                    || d.byte_memory_readonly.contains(&i)
            }) {
                if details.unwrap().byte_memory_readonly.contains(&i) {
                    let ([Ty::ByteMemory(source)], Some(Ty::ByteMemory(target))) =
                        (p.args[i].as_slice(), p.results[i].as_ref())
                    else {
                        return Err(
                            "byte-memory readonly conversion requires exact persistent views"
                                .into(),
                        );
                    };
                    let source_type =
                        neoclr::metadata::Type::Named(input.types[*source].name.clone());
                    let target_type =
                        neoclr::metadata::Type::Named(input.types[*target].name.clone());
                    if !crate::memory_views::conversion(input, &source_type, &target_type, false)
                        && !crate::memory_views::conversion(input, &source_type, &target_type, true)
                    {
                        return Err(
                            "byte-view conversion cannot grant writable access or extend lifetime"
                                .into(),
                        );
                    }
                    // Exact binding restricts the nominal capability; owner and
                    // bounds retain the same representation and GC root lane.
                    write(&mut b, output, &parameters[..3]);
                } else if details.unwrap().byte_view_get.contains(&i)
                    || details.unwrap().byte_view_set.contains(&i)
                {
                    let Ty::ByteMemory(view) = p.args[i][0] else {
                        return Err("byte-view access requires a byte view".into());
                    };
                    let set = details.unwrap().byte_view_set.contains(&i);
                    let kind = match input.types[view].memory_view {
                        Some(neoclr::metadata::MemoryViewKind::Slice) => 0,
                        Some(neoclr::metadata::MemoryViewKind::ReadOnlySlice) if !set => 1,
                        _ => {
                            return Err(
                                "byte-view access requires the matching borrowed capability".into(),
                            );
                        }
                    };
                    let storage = slot(&mut b, 24);
                    let address = b.ins().stack_addr(types::I64, storage, 0);
                    b.ins().store(MemFlags::new(), parameters[0], address, 0);
                    b.ins().store(MemFlags::new(), parameters[1], address, 8);
                    b.ins().store(MemFlags::new(), parameters[2], address, 12);
                    let kind = b.ins().iconst(types::I64, kind);
                    b.ins().store(MemFlags::new(), kind, address, 16);
                    let result_slot = slot(&mut b, 8);
                    let result_address = b.ins().stack_addr(types::I64, result_slot, 0);
                    let last = if set {
                        b.ins().ireduce(types::I8, parameters[4])
                    } else {
                        result_address
                    };
                    let service = module
                        .declare_func_in_func(byte_view_access[usize::from(set)].unwrap(), b.func);
                    let call = b.ins().call(
                        service,
                        &[fault_context.unwrap(), address, parameters[3], last],
                    );
                    let status = b.inst_results(call)[0];
                    let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                    let mut site = diagnostic_data.as_ref().unwrap().site(
                        module,
                        &mut b,
                        fault_context.unwrap(),
                        i,
                        0,
                    );
                    site.capture_frame = false;
                    return_if_detailed(&mut b, failed, status, Some(&site));
                    if !set {
                        let byte = b.ins().load(types::I8, MemFlags::new(), result_address, 0);
                        let result = b.ins().uextend(types::I32, byte);
                        write(&mut b, output, &[result]);
                    }
                } else if !details.unwrap().byte_memory_length.contains(&i) {
                    let Ty::ByteMemory(owner) =
                        p.results[i].as_ref().ok_or("missing memory result")?
                    else {
                        return Err("byte-memory factory requires a persistent byte result".into());
                    };
                    let kind = match input.types[*owner].memory_view {
                        Some(neoclr::metadata::MemoryViewKind::Slice) => 0,
                        Some(neoclr::metadata::MemoryViewKind::ReadOnlySlice) => 1,
                        Some(neoclr::metadata::MemoryViewKind::Memory) => 2,
                        Some(neoclr::metadata::MemoryViewKind::ReadOnlyMemory) => 3,
                        _ => return Err("invalid persistent memory kind".into()),
                    };
                    let kind = b.ins().iconst(types::I64, kind);
                    let storage = slot(&mut b, 24);
                    let address = b.ins().stack_addr(types::I64, storage, 0);
                    let call = if details.unwrap().byte_view_utf8.contains(&i) {
                        let service = module.declare_func_in_func(byte_view_utf8.unwrap(), b.func);
                        b.ins()
                            .call(service, &[fault_context.unwrap(), parameters[0], address])
                    } else if details.unwrap().byte_memory_create.contains(&i) {
                        if p.args[i].len() == 3 {
                            let service = module
                                .declare_func_in_func(byte_view_create_window.unwrap(), b.func);
                            b.ins().call(
                                service,
                                &[
                                    fault_context.unwrap(),
                                    parameters[0],
                                    parameters[1],
                                    parameters[2],
                                    kind,
                                    address,
                                ],
                            )
                        } else {
                            let service =
                                module.declare_func_in_func(byte_memory_create.unwrap(), b.func);
                            b.ins().call(
                                service,
                                &[fault_context.unwrap(), parameters[0], kind, address],
                            )
                        }
                    } else {
                        b.ins().store(MemFlags::new(), parameters[0], address, 0);
                        b.ins().store(MemFlags::new(), parameters[1], address, 8);
                        b.ins().store(MemFlags::new(), parameters[2], address, 12);
                        b.ins().store(MemFlags::new(), kind, address, 16);
                        let service =
                            module.declare_func_in_func(byte_memory_slice.unwrap(), b.func);
                        let count = if p.args[i].len() == 2 {
                            // An unsigned comparison rejects negative starts too.
                            // Subtract only after proving 0 <= start <= length.
                            let invalid = b.ins().icmp(
                                IntCC::UnsignedGreaterThan,
                                parameters[3],
                                parameters[2],
                            );
                            let status = b.ins().iconst(types::I32, 8);
                            let mut site = diagnostic_data.as_ref().unwrap().site(
                                module,
                                &mut b,
                                fault_context.unwrap(),
                                i,
                                0,
                            );
                            site.capture_frame = false;
                            return_if_detailed(&mut b, invalid, status, Some(&site));
                            b.ins().isub(parameters[2], parameters[3])
                        } else {
                            parameters[4]
                        };
                        b.ins().call(
                            service,
                            &[
                                fault_context.unwrap(),
                                address,
                                parameters[3],
                                count,
                                address,
                            ],
                        )
                    };
                    let status = b.inst_results(call)[0];
                    let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                    let mut site = diagnostic_data.as_ref().unwrap().site(
                        module,
                        &mut b,
                        fault_context.unwrap(),
                        i,
                        0,
                    );
                    site.capture_frame = false;
                    return_if_detailed(&mut b, failed, status, Some(&site));
                    // C kernel packs two bounds into one word. Private generated
                    // storage has one eight-byte slot per logical lane, no kind word.
                    let owner = b.ins().load(types::I64, MemFlags::new(), address, 0);
                    let offset = b.ins().load(types::I32, MemFlags::new(), address, 8);
                    let length = b.ins().load(types::I32, MemFlags::new(), address, 12);
                    write(&mut b, output, &[owner, offset, length]);
                } else {
                    if !matches!(p.args[i].as_slice(), [Ty::ByteMemory(_)]) {
                        return Err("byte-memory length requires one persistent byte view".into());
                    }
                    write(&mut b, output, &[parameters[2]]);
                }
                let zero = b.ins().iconst(types::I32, 0);
                b.ins().return_(&[zero]);
                b.seal_all_blocks();
                b.finalize();
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(module, &mut context.func, frame);
                }
                define_checked(module, ids[i], &mut context, stack_budget, long_range_calls)?;
                continue;
            }
            if let Some(targets) = details.and_then(|d| d.object_type_queries.get(&i)) {
                let receiver = parameters[0];
                let null = b.ins().icmp_imm(IntCC::Equal, receiver, 0);
                let mut site = diagnostic_data.as_ref().unwrap().site(
                    module,
                    &mut b,
                    fault_context.unwrap(),
                    i,
                    0,
                );
                site.capture_frame = false;
                let status = b.ins().iconst(types::I32, 6);
                return_if_detailed(&mut b, null, status, Some(&site));
                let text_block = b.create_block();
                let object_block = b.create_block();
                let bits = b.ins().band_imm(receiver, 1);
                let text = b.ins().icmp_imm(IntCC::NotEqual, bits, 0);
                b.ins().brif(text, text_block, &[], object_block, &[]);
                b.switch_to_block(text_block);
                let identity = type_tokens
                    .iter()
                    .position(|t| *t == neoclr::metadata::Type::String)
                    .expect("String token producer")
                    + 1;
                let value = b.ins().iconst(types::I64, identity as i64);
                write(&mut b, output, &[value]);
                let zero = b.ins().iconst(types::I32, 0);
                b.ins().return_(&[zero]);
                b.switch_to_block(object_block);
                let tag = b.ins().load(types::I64, MemFlags::new(), receiver, 0);
                if let Some(tokens) = details.and_then(|d| d.object_function_queries.get(&i)) {
                    let function = b.create_block();
                    let ordinary = b.create_block();
                    let callable = b.ins().icmp_imm(IntCC::Equal, tag, u32::MAX as i64);
                    b.ins().brif(callable, function, &[], ordinary, &[]);
                    b.switch_to_block(function);
                    let target = b.ins().load(types::I64, MemFlags::new(), receiver, 8);
                    for token in tokens {
                        let mut exact = b.ins().iconst(types::I8, 0);
                        for index in p.callable_targets(token)? {
                            let equal = b.ins().icmp_imm(IntCC::Equal, target, (index + 1) as i64);
                            exact = b.ins().bor(exact, equal);
                        }
                        let matched = b.create_block();
                        let next = b.create_block();
                        b.ins().brif(exact, matched, &[], next, &[]);
                        b.switch_to_block(matched);
                        let identity = type_tokens
                            .iter()
                            .position(|t| t == token)
                            .ok_or("missing function token producer")?
                            + 1;
                        let value = b.ins().iconst(types::I64, identity as i64);
                        write(&mut b, output, &[value]);
                        let zero = b.ins().iconst(types::I32, 0);
                        b.ins().return_(&[zero]);
                        b.switch_to_block(next);
                    }
                    b.ins().jump(ordinary, &[]);
                    b.switch_to_block(ordinary);
                }
                for (index, token) in targets {
                    let matched = b.create_block();
                    let next = b.create_block();
                    let exact = matches_type(&mut b, &p, tag, *index);
                    b.ins().brif(exact, matched, &[], next, &[]);
                    b.switch_to_block(matched);
                    let identity = type_tokens
                        .iter()
                        .position(|t| t == token)
                        .expect("object token producer")
                        + 1;
                    let value = b.ins().iconst(types::I64, identity as i64);
                    write(&mut b, output, &[value]);
                    let zero = b.ins().iconst(types::I32, 0);
                    b.ins().return_(&[zero]);
                    b.switch_to_block(next);
                }
                let mut site = diagnostic_data.as_ref().unwrap().site(
                    module,
                    &mut b,
                    fault_context.unwrap(),
                    i,
                    1,
                );
                site.capture_frame = false;
                let status = b.ins().iconst(types::I32, 3);
                site.record(&mut b, status);
                b.ins().return_(&[status]);
                b.seal_all_blocks();
                b.finalize();
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(module, &mut context.func, frame);
                }
                define_checked(module, ids[i], &mut context, stack_budget, long_range_calls)?;
                continue;
            }
            if let Some(&(tag_index, unbox)) = details.and_then(|d| d.scalar_box_queries.get(&i)) {
                let receiver = parameters[0];
                let null = b.ins().icmp_imm(IntCC::Equal, receiver, 0);
                if unbox {
                    let mut site = diagnostic_data.as_ref().unwrap().site(
                        module,
                        &mut b,
                        fault_context.unwrap(),
                        i,
                        0,
                    );
                    site.capture_frame = false;
                    let status = b.ins().iconst(types::I32, 6);
                    return_if_detailed(&mut b, null, status, Some(&site));
                }
                let read_tag = b.create_block();
                let matched = b.create_block();
                let absent = b.create_block();
                let bits = b.ins().band_imm(receiver, 1);
                let text = b.ins().icmp_imm(IntCC::NotEqual, bits, 0);
                let invalid = b.ins().bor(null, text);
                b.ins().brif(invalid, absent, &[], read_tag, &[]);
                b.switch_to_block(read_tag);
                let tag = b.ins().load(types::I64, MemFlags::new(), receiver, 0);
                let valid = if let Some(index) = tag_index {
                    matches_type(&mut b, &p, tag, index)
                } else {
                    b.ins().iconst(types::I8, 0)
                };
                b.ins().brif(valid, matched, &[], absent, &[]);
                b.switch_to_block(matched);
                if unbox {
                    let payload = b.ins().iadd_imm(receiver, 8);
                    let result = p.results[i].as_ref().unwrap();
                    let values = if matches!(result, Ty::Record(index) if input.types[*index].fields.is_empty())
                    {
                        lanes(&p, result)
                            .into_iter()
                            .map(|ty| zero(&mut b, ty))
                            .collect()
                    } else {
                        read(&mut b, &p, result, payload)
                    };
                    write(&mut b, output, &values);
                } else {
                    write(&mut b, output, &[receiver]);
                }
                let zero = b.ins().iconst(types::I32, 0);
                b.ins().return_(&[zero]);
                b.switch_to_block(absent);
                if unbox {
                    let mut site = diagnostic_data.as_ref().unwrap().site(
                        module,
                        &mut b,
                        fault_context.unwrap(),
                        i,
                        1,
                    );
                    site.capture_frame = false;
                    let status = b.ins().iconst(types::I32, 11);
                    site.record(&mut b, status);
                    b.ins().return_(&[status]);
                } else {
                    let pointer = b.ins().iconst(types::I64, 0);
                    write(&mut b, output, &[pointer]);
                    let zero = b.ins().iconst(types::I32, 0);
                    b.ins().return_(&[zero]);
                }
                b.seal_all_blocks();
                b.finalize();
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(module, &mut context.func, frame);
                }
                define_checked(module, ids[i], &mut context, stack_budget, long_range_calls)?;
                continue;
            }
            if let Some(&record) = details.and_then(|d| {
                d.empty_record_boxes
                    .get(&i)
                    .or_else(|| d.int32_boxes.get(&i))
                    .or_else(|| d.int64_boxes.get(&i))
                    .or_else(|| d.boolean_boxes.get(&i))
            }) {
                let service = module.declare_func_in_func(object_service.unwrap(), b.func);
                let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                let tag = b.ins().iconst(types::I32, record as i64);
                let primitive = details.unwrap().int32_boxes.contains_key(&i)
                    || details.unwrap().int64_boxes.contains_key(&i)
                    || details.unwrap().boolean_boxes.contains_key(&i);
                let payload = if primitive {
                    8
                } else if input.types[record].fields.is_empty()
                    && input.types[record].memory_view.is_none()
                {
                    0
                } else {
                    p.bytes(&p.args[i][0])
                };
                let bytes = b.ins().iconst(types::I32, (8 + payload) as i64);
                let call = b.ins().call(service, &[arena, tag, bytes, output]);
                let raw = b.inst_results(call)[0];
                let exhausted = b.ins().icmp_imm(IntCC::Equal, raw, 5);
                let memory = b.ins().iconst(types::I32, 5);
                let runtime = b.ins().iconst(types::I32, 3);
                let status = b.ins().select(exhausted, memory, runtime);
                let failed = b.ins().icmp_imm(IntCC::NotEqual, raw, 0);
                let mut site = diagnostic_data.as_ref().unwrap().site(
                    module,
                    &mut b,
                    fault_context.unwrap(),
                    i,
                    0,
                );
                site.capture_frame = false;
                return_if_detailed(&mut b, failed, status, Some(&site));
                if payload != 0 {
                    let pointer = b.ins().load(types::I64, MemFlags::new(), output, 0);
                    let pointer = b.ins().iadd_imm(pointer, 8);
                    write_typed(
                        &mut b,
                        &p,
                        &p.args[i][0],
                        pointer,
                        &parameters[..p.lanes(&p.args[i][0])],
                    );
                }
                let zero = b.ins().iconst(types::I32, 0);
                b.ins().return_(&[zero]);
                b.seal_all_blocks();
                b.finalize();
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(module, &mut context.func, frame);
                }
                define_checked(module, ids[i], &mut context, stack_budget, long_range_calls)?;
                continue;
            }
            if let Some(targets) = p.dispatch.get(&i) {
                // Caller checks null. Forward the original receiver, result slot and
                // context; no synthetic interface frame enters the managed trace.
                let value_kind = details.and_then(|d| d.object_value_kinds.get(&i));
                let display = details.is_some_and(|d| d.object_display.contains_key(&i))
                    && value_kind.is_none();
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
                            let mut site = d.site(module, &mut b, fault_context.unwrap(), i, 0);
                            site.capture_frame = false;
                            site.record(&mut b, status);
                        }
                        b.ins().return_(&[status]);
                    }
                    b.switch_to_block(object);
                }
                let tag = b.ins().load(types::I64, MemFlags::new(), parameters[0], 0);
                if display
                    && details.is_some_and(|d| d.boxed_int32_display || d.boxed_int64_display)
                {
                    for (type_index, wide) in details
                        .unwrap()
                        .int32_boxes
                        .values()
                        .map(|i| (*i, false))
                        .chain(details.unwrap().int64_boxes.values().map(|i| (*i, true)))
                    {
                        let matched = b.create_block();
                        let next = b.create_block();
                        let equal = matches_type(&mut b, &p, tag, type_index);
                        b.ins().brif(equal, matched, &[], next, &[]);
                        b.switch_to_block(matched);
                        let value = b.ins().load(
                            if wide { types::I64 } else { types::I32 },
                            MemFlags::new(),
                            parameters[0],
                            8,
                        );
                        let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                        let service = module.declare_func_in_func(
                            if wide {
                                boxed_wide_format_service.unwrap()
                            } else {
                                format_service.unwrap()
                            },
                            b.func,
                        );
                        let call = b.ins().call(service, &[value, arena, output]);
                        let raw = b.inst_results(call)[0];
                        let exhausted = b.ins().icmp_imm(IntCC::Equal, raw, 5);
                        let memory = b.ins().iconst(types::I32, 5);
                        let runtime = b.ins().iconst(types::I32, 3);
                        let status = b.ins().select(exhausted, memory, runtime);
                        let failed = b.ins().icmp_imm(IntCC::NotEqual, raw, 0);
                        let mut site = diagnostic_data.as_ref().unwrap().site(
                            module,
                            &mut b,
                            fault_context.unwrap(),
                            i,
                            0,
                        );
                        site.capture_frame = false;
                        return_if_detailed(&mut b, failed, status, Some(&site));
                        let zero = b.ins().iconst(types::I32, 0);
                        b.ins().return_(&[zero]);
                        b.switch_to_block(next);
                    }
                }
                if display && details.is_some_and(|d| d.boxed_boolean_display) {
                    for &type_index in details.unwrap().boolean_boxes.values() {
                        let matched = b.create_block();
                        let next = b.create_block();
                        let equal = matches_type(&mut b, &p, tag, type_index);
                        b.ins().brif(equal, matched, &[], next, &[]);
                        b.switch_to_block(matched);
                        let value = b.ins().load(types::I32, MemFlags::new(), parameters[0], 8);
                        let no = module.declare_data_in_func(boolean_display_literals[0], b.func);
                        let yes = module.declare_data_in_func(boolean_display_literals[1], b.func);
                        let no = b.ins().global_value(types::I64, no);
                        let yes = b.ins().global_value(types::I64, yes);
                        let text = b.ins().select(value, yes, no);
                        write(&mut b, output, &[text]);
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
                    let mut adapted = parameters.clone();
                    if input.functions[target].receiver_byref
                        || details.is_some_and(|d| d.primitive_receivers.contains(&target))
                    {
                        // A value override borrows the payload of the rooted box.
                        adapted[0] = b.ins().iadd_imm(parameters[0], 8);
                    }
                    let mut offset = 0;
                    for (from, to) in p.args[i].iter().zip(&p.args[target]) {
                        if *from == Ty::Literal && from != to {
                            if !string_view(to) {
                                return Err(
                                    "variant String argument requires a verified reference view"
                                        .into(),
                                );
                            }
                            adapted[offset] = text_object_view(&mut b, adapted[offset]);
                        }
                        offset += p.lanes(from);
                    }
                    let string_result =
                        p.results[target] == Some(Ty::Literal) && p.results[i] != Some(Ty::Literal);
                    if string_result && !p.results[i].as_ref().is_some_and(&string_view) {
                        return Err(
                            "variant String result requires a verified reference view".into()
                        );
                    }
                    let target = module.declare_func_in_func(ids[target], b.func);
                    let call = b.ins().call(target, &adapted);
                    let status = b.inst_results(call)[0];
                    if string_result {
                        // A failed call has no initialized result to read or convert.
                        let success = b.create_block();
                        let failed = b.create_block();
                        b.ins().brif(status, failed, &[], success, &[]);
                        b.switch_to_block(failed);
                        b.ins().return_(&[status]);
                        b.switch_to_block(success);
                        let value = b.ins().load(types::I64, MemFlags::new(), output, 0);
                        let view = text_object_view(&mut b, value);
                        b.ins().store(MemFlags::new(), view, output, 0);
                    }
                    b.ins().return_(&[status]);
                    b.switch_to_block(next);
                }
                if display {
                    for &(index, wide) in &details.unwrap().floating_boxes {
                        let matched = b.create_block();
                        let next = b.create_block();
                        let equal = matches_type(&mut b, &p, tag, index);
                        b.ins().brif(equal, matched, &[], next, &[]);
                        b.switch_to_block(matched);
                        let value = b.ins().load(
                            if wide { types::F64 } else { types::F32 },
                            MemFlags::new(),
                            parameters[0],
                            8,
                        );
                        let code = b.ins().iconst(types::I32, 71);
                        let precision = b.ins().iconst(types::I32, -1);
                        let swedish = b.ins().iconst(types::I32, 0);
                        let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                        let service = module
                            .declare_func_in_func(floating_display[usize::from(wide)], b.func);
                        let call = b
                            .ins()
                            .call(service, &[value, code, precision, swedish, arena, output]);
                        let status = b.inst_results(call)[0];
                        let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                        let mut site = diagnostic_data.as_ref().unwrap().site(
                            module,
                            &mut b,
                            fault_context.unwrap(),
                            i,
                            0,
                        );
                        site.capture_frame = false;
                        return_if_detailed(&mut b, failed, status, Some(&site));
                        b.ins().return_(&[status]);
                        b.switch_to_block(next);
                    }
                    if let Some((signed_service, unsigned_service)) = integral_display {
                        for &(index, wide, signed) in &details.unwrap().integral_boxes {
                            let matched = b.create_block();
                            let next = b.create_block();
                            let equal = matches_type(&mut b, &p, tag, index);
                            b.ins().brif(equal, matched, &[], next, &[]);
                            b.switch_to_block(matched);
                            let payload = b.ins().iadd_imm(parameters[0], 8);
                            let scalar = p.ty(&input.types[index].fields[0].ty)?;
                            let value = read(&mut b, &p, &scalar, payload)[0];
                            let value = if wide {
                                value
                            } else if signed {
                                b.ins().sextend(types::I64, value)
                            } else {
                                b.ins().uextend(types::I64, value)
                            };
                            let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                            let service = module.declare_func_in_func(
                                if signed {
                                    signed_service
                                } else {
                                    unsigned_service
                                },
                                b.func,
                            );
                            let call = b.ins().call(service, &[value, arena, output]);
                            let status = b.inst_results(call)[0];
                            let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                            let mut site = diagnostic_data.as_ref().unwrap().site(
                                module,
                                &mut b,
                                fault_context.unwrap(),
                                i,
                                0,
                            );
                            site.capture_frame = false;
                            return_if_detailed(&mut b, failed, status, Some(&site));
                            b.ins().return_(&[status]);
                            b.switch_to_block(next);
                        }
                    }
                    if let Some(enums) = &enum_display {
                        for &index in enums.types() {
                            let matched = b.create_block();
                            let next = b.create_block();
                            let equal = matches_type(&mut b, &p, tag, index);
                            b.ins().brif(equal, matched, &[], next, &[]);
                            b.switch_to_block(matched);
                            let value = b.ins().load(types::I32, MemFlags::new(), parameters[0], 8);
                            let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                            let status = enums.emit(module, &mut b, index, value, arena, output);
                            let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                            let mut site = diagnostic_data.as_ref().unwrap().site(
                                module,
                                &mut b,
                                fault_context.unwrap(),
                                i,
                                0,
                            );
                            site.capture_frame = false;
                            return_if_detailed(&mut b, failed, status, Some(&site));
                            b.ins().return_(&[status]);
                            b.switch_to_block(next);
                        }
                    }
                    for &(index, literal) in &default_display_literals {
                        let matched = b.create_block();
                        let next = b.create_block();
                        let equal = matches_type(&mut b, &p, tag, index);
                        b.ins().brif(equal, matched, &[], next, &[]);
                        b.switch_to_block(matched);
                        let data = module.declare_data_in_func(literal, b.func);
                        let pointer = b.ins().global_value(types::I64, data);
                        write(&mut b, output, &[pointer]);
                        let zero = b.ins().iconst(types::I32, 0);
                        b.ins().return_(&[zero]);
                        b.switch_to_block(next);
                    }
                }
                if let Some(kind) = value_kind {
                    let d = details.unwrap();
                    // Persistent views share region identity across writable/readonly
                    // projections. Their payload is the existing owner/offset/length ABI.
                    let memory_boxes: Vec<_> = d
                        .empty_record_boxes
                        .values()
                        .copied()
                        .filter(|index| {
                            input.types[*index]
                                .memory_view
                                .is_some_and(|k| !k.is_borrowed())
                        })
                        .collect();
                    for &index in &memory_boxes {
                        let matched = b.create_block();
                        let next = b.create_block();
                        let equal = matches_type(&mut b, &p, tag, index);
                        b.ins().brif(equal, matched, &[], next, &[]);
                        b.switch_to_block(matched);
                        let left_owner =
                            b.ins().load(types::I64, MemFlags::new(), parameters[0], 8);
                        let left_start =
                            b.ins().load(types::I32, MemFlags::new(), parameters[0], 16);
                        let left_length =
                            b.ins().load(types::I32, MemFlags::new(), parameters[0], 24);
                        let value = if kind == "Equals" {
                            let inspect = b.create_block();
                            let compare = b.create_block();
                            let different = b.create_block();
                            let done = b.create_block();
                            b.append_block_param(done, types::I32);
                            let nonnull = b.ins().icmp_imm(IntCC::NotEqual, parameters[1], 0);
                            let low = b.ins().band_imm(parameters[1], 1);
                            let untagged = b.ins().icmp_imm(IntCC::Equal, low, 0);
                            let boxed = b.ins().band(nonnull, untagged);
                            b.ins().brif(boxed, inspect, &[], different, &[]);
                            b.switch_to_block(inspect);
                            let right_tag =
                                b.ins().load(types::I64, MemFlags::new(), parameters[1], 0);
                            // Keep element identity explicit even though the current profile
                            // admits Byte views only. Access mode does not affect the region.
                            let mut compatible = b.ins().iconst(types::I8, 0);
                            for &candidate in &memory_boxes {
                                if neoclr::native_metadata::memory_view_element(
                                    &input.types[candidate],
                                ) != neoclr::native_metadata::memory_view_element(
                                    &input.types[index],
                                ) {
                                    continue;
                                }
                                let matches = matches_type(&mut b, &p, right_tag, candidate);
                                compatible = b.ins().bor(compatible, matches);
                            }
                            b.ins().brif(compatible, compare, &[], different, &[]);
                            b.switch_to_block(compare);
                            let right_owner =
                                b.ins().load(types::I64, MemFlags::new(), parameters[1], 8);
                            let right_start =
                                b.ins().load(types::I32, MemFlags::new(), parameters[1], 16);
                            let right_length =
                                b.ins().load(types::I32, MemFlags::new(), parameters[1], 24);
                            let owner = b.ins().icmp(IntCC::Equal, left_owner, right_owner);
                            let start = b.ins().icmp(IntCC::Equal, left_start, right_start);
                            let length = b.ins().icmp(IntCC::Equal, left_length, right_length);
                            let same = b.ins().band(owner, start);
                            let same = b.ins().band(same, length);
                            let same = b.ins().uextend(types::I32, same);
                            b.ins().jump(done, &[same.into()]);
                            b.switch_to_block(different);
                            let no = b.ins().iconst(types::I32, 0);
                            b.ins().jump(done, &[no.into()]);
                            b.switch_to_block(done);
                            b.block_params(done)[0]
                        } else {
                            // The current collector does not move payload addresses.
                            // Hashes are process-local and need not match interpreter numbers.
                            let high = b.ins().ushr_imm(left_owner, 32);
                            let bits = b.ins().bxor(left_owner, high);
                            let hash = b.ins().ireduce(types::I32, bits);
                            let hash = b.ins().imul_imm(hash, 31);
                            let hash = b.ins().bxor(hash, left_start);
                            let hash = b.ins().imul_imm(hash, 31);
                            let hash = b.ins().bxor(hash, left_length);
                            let empty = b.ins().icmp_imm(IntCC::Equal, left_owner, 0);
                            let zero = b.ins().iconst(types::I32, 0);
                            b.ins().select(empty, zero, hash)
                        };
                        write(&mut b, output, &[value]);
                        let status = b.ins().iconst(types::I32, 0);
                        b.ins().return_(&[status]);
                        b.switch_to_block(next);
                    }
                    // Primitive and enum boxes preserve value semantics through the ordinary
                    // Object slots, including calls made from generic code.
                    for (index, wide, floating) in d
                        .int32_boxes
                        .values()
                        .chain(d.boolean_boxes.values())
                        .chain(
                            d.empty_record_boxes
                                .values()
                                .filter(|index| input.types[**index].enum_info.is_some()),
                        )
                        .map(|index| (*index, false, false))
                        .chain(d.int64_boxes.values().map(|index| (*index, true, false)))
                        .chain(
                            d.integral_boxes
                                .iter()
                                .map(|(index, wide, _)| (*index, *wide, false)),
                        )
                        .chain(
                            d.floating_boxes
                                .iter()
                                .map(|(index, wide)| (*index, *wide, true)),
                        )
                    {
                        let matched = b.create_block();
                        let next = b.create_block();
                        let equal = matches_type(&mut b, &p, tag, index);
                        b.ins().brif(equal, matched, &[], next, &[]);
                        b.switch_to_block(matched);
                        let payload_type = match (floating, wide) {
                            (true, true) => types::F64,
                            (true, false) => types::F32,
                            (false, true) => types::I64,
                            (false, false) => types::I32,
                        };
                        let integral_ty = d
                            .integral_boxes
                            .iter()
                            .any(|(i, _, _)| *i == index)
                            .then(|| p.ty(&input.types[index].fields[0].ty))
                            .transpose()?;
                        let left = if let Some(scalar) = &integral_ty {
                            let payload = b.ins().iadd_imm(parameters[0], 8);
                            read(&mut b, &p, scalar, payload)[0]
                        } else {
                            b.ins()
                                .load(payload_type, MemFlags::new(), parameters[0], 8)
                        };
                        let result = if kind == "Equals" {
                            let compare = b.create_block();
                            let different = b.create_block();
                            let same_type = b.create_block();
                            let result = b.create_block();
                            b.append_block_param(result, types::I32);
                            // Null and tagged String views have no box header.
                            let nonnull = b.ins().icmp_imm(IntCC::NotEqual, parameters[1], 0);
                            let tagged = b.ins().band_imm(parameters[1], 1);
                            let untagged = b.ins().icmp_imm(IntCC::Equal, tagged, 0);
                            let boxed = b.ins().band(nonnull, untagged);
                            b.ins().brif(boxed, compare, &[], different, &[]);
                            b.switch_to_block(compare);
                            let right_tag =
                                b.ins().load(types::I64, MemFlags::new(), parameters[1], 0);
                            let same = matches_type(&mut b, &p, right_tag, index);
                            b.ins().brif(same, same_type, &[], different, &[]);
                            b.switch_to_block(same_type);
                            let right = if let Some(scalar) = &integral_ty {
                                let payload = b.ins().iadd_imm(parameters[1], 8);
                                read(&mut b, &p, scalar, payload)[0]
                            } else {
                                b.ins()
                                    .load(payload_type, MemFlags::new(), parameters[1], 8)
                            };
                            let equal = if floating {
                                let equal = b.ins().fcmp(FloatCC::Equal, left, right);
                                let left_nan = b.ins().fcmp(FloatCC::Unordered, left, left);
                                let right_nan = b.ins().fcmp(FloatCC::Unordered, right, right);
                                let both_nan = b.ins().band(left_nan, right_nan);
                                b.ins().bor(equal, both_nan)
                            } else {
                                b.ins().icmp(IntCC::Equal, left, right)
                            };
                            let equal = b.ins().uextend(types::I32, equal);
                            b.ins().jump(result, &[equal.into()]);
                            b.switch_to_block(different);
                            let no = b.ins().iconst(types::I32, 0);
                            b.ins().jump(result, &[no.into()]);
                            b.switch_to_block(result);
                            b.block_params(result)[0]
                        } else if floating {
                            let integer = if wide { types::I64 } else { types::I32 };
                            let bits = b.ins().bitcast(integer, MemFlags::new(), left);
                            let magnitude = b
                                .ins()
                                .band_imm(bits, if wide { i64::MAX } else { i32::MAX as i64 });
                            let is_zero = b.ins().icmp_imm(IntCC::Equal, magnitude, 0);
                            let zero = b.ins().iconst(integer, 0);
                            let bits = b.ins().select(is_zero, zero, bits);
                            let nan = b.ins().fcmp(FloatCC::Unordered, left, left);
                            let canonical = b.ins().iconst(
                                integer,
                                if wide {
                                    0x7ff0_0000_0000_0000
                                } else {
                                    0x7f80_0000
                                },
                            );
                            let bits = b.ins().select(nan, canonical, bits);
                            if wide {
                                let high = b.ins().ushr_imm(bits, 32);
                                let mixed = b.ins().bxor(bits, high);
                                b.ins().ireduce(types::I32, mixed)
                            } else {
                                bits
                            }
                        } else if wide {
                            let high = b.ins().ushr_imm(left, 32);
                            let mixed = b.ins().bxor(left, high);
                            b.ins().ireduce(types::I32, mixed)
                        } else {
                            left
                        };
                        write(&mut b, output, &[result]);
                        let status = b.ins().iconst(types::I32, 0);
                        b.ins().return_(&[status]);
                        b.switch_to_block(next);
                    }
                    // Other value boxes still need a verified override; never
                    // silently apply reference identity to them.
                    let mut boxes: Vec<_> = d
                        .int32_boxes
                        .values()
                        .chain(d.int64_boxes.values())
                        .chain(d.boolean_boxes.values())
                        .chain(d.empty_record_boxes.values())
                        .copied()
                        .collect();
                    boxes.sort_unstable();
                    boxes.dedup();
                    for index in boxes {
                        let boxed = matches_type(&mut b, &p, tag, index);
                        let status = b.ins().iconst(types::I32, 3);
                        let mut site = diagnostic_data.as_ref().unwrap().site(
                            module,
                            &mut b,
                            fault_context.unwrap(),
                            i,
                            0,
                        );
                        site.capture_frame = false;
                        return_if_detailed(&mut b, boxed, status, Some(&site));
                    }
                    let value = if kind == "Equals" {
                        let equal = b.ins().icmp(IntCC::Equal, parameters[0], parameters[1]);
                        b.ins().uextend(types::I32, equal)
                    } else {
                        // The native collector is nonmoving: this hash is stable
                        // for the object's lifetime, with no cross-process promise.
                        let high = b.ins().ushr_imm(parameters[0], 32);
                        let mixed = b.ins().bxor(parameters[0], high);
                        b.ins().ireduce(types::I32, mixed)
                    };
                    write(&mut b, output, &[value]);
                    let status = b.ins().iconst(types::I32, 0);
                    b.ins().return_(&[status]);
                } else {
                    let status = b.ins().iconst(types::I32, 3);
                    if let Some(d) = &diagnostic_data {
                        let mut site = d.site(module, &mut b, fault_context.unwrap(), i, 0);
                        site.capture_frame = false;
                        site.record(&mut b, status);
                    }
                    b.ins().return_(&[status]);
                }
                b.seal_all_blocks();
                b.finalize();
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(module, &mut context.func, frame);
                }
                define_checked(module, ids[i], &mut context, stack_budget, long_range_calls)?;
                continue;
            }
            if details.is_some_and(|d| d.entry_task_drain.contains(&i))
                && let Some((callback, drain, pump)) = runtime_async_callbacks
            {
                let callback = module.declare_func_in_func(callback, b.func);
                let drain = module.declare_func_in_func(drain, b.func);
                let callback = b.ins().func_addr(types::I64, callback);
                let drain = b.ins().func_addr(types::I64, drain);
                let pump = module.declare_func_in_func(pump, b.func);
                let call = b
                    .ins()
                    .call(pump, &[fault_context.unwrap(), callback, drain]);
                let status = b.inst_results(call)[0];
                let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                let mut site = diagnostic_data.as_ref().unwrap().site(
                    module,
                    &mut b,
                    fault_context.unwrap(),
                    i,
                    0,
                );
                site.capture_frame = false;
                return_if_detailed(&mut b, failed, status, Some(&site));
                let zero = b.ins().iconst(types::I32, 0);
                b.ins().store(MemFlags::new(), zero, output, 0);
                b.ins().return_(&[zero]);
                b.seal_all_blocks();
                b.finalize();
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(module, &mut context.func, frame);
                }
                define_checked(module, ids[i], &mut context, stack_budget, long_range_calls)?;
                continue;
            }
            if details.is_some_and(|d| d.entry_task_drain.contains(&i)) {
                let d = details.unwrap();
                let ctx = fault_context.unwrap();
                let (begin, end, poll, callback, queue_read) = entry_drain_services.unwrap();
                let begin = module.declare_func_in_func(begin, b.func);
                let end = module.declare_func_in_func(end, b.func);
                let entry = b.ins().iconst(types::I32, root as i64);
                let run = b
                    .ins()
                    .iconst(types::I32, d.task_queue_run.map_or(-1, |i| i as i64));
                let drain = b
                    .ins()
                    .iconst(types::I32, d.task_queue_drain.map_or(-1, |i| i as i64));
                let storage = slot(&mut b, 16);
                let queue_output = b.ins().stack_addr(types::I64, storage, 0);
                let call = b.ins().call(begin, &[ctx, entry, run, drain, queue_output]);
                let status = b.inst_results(call)[0];
                let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                let mut site = diagnostic_data
                    .as_ref()
                    .unwrap()
                    .site(module, &mut b, ctx, i, 0);
                site.capture_frame = false;
                return_if_detailed(&mut b, failed, status, Some(&site));
                let done = b.create_block();
                b.append_block_param(done, types::I32);
                let zero = b.ins().iconst(types::I32, 0);
                let cycle = b.create_block();
                let poll_work = b.create_block();
                b.ins().jump(cycle, &[]);
                b.switch_to_block(cycle);
                if let Some(drain) = d.task_queue_drain {
                    let reader = module.declare_func_in_func(queue_read, b.func);
                    let call = b.ins().call(reader, &[ctx, queue_output]);
                    let status = b.inst_results(call)[0];
                    let loaded = b.create_block();
                    let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                    b.ins().brif(failed, done, &[status.into()], loaded, &[]);
                    b.switch_to_block(loaded);
                    let queue = b.ins().load(types::I64, MemFlags::new(), queue_output, 0);
                    let absent = b.ins().icmp_imm(IntCC::Equal, queue, 0);
                    let work = b.create_block();
                    b.ins().brif(absent, poll_work, &[], work, &[]);
                    b.switch_to_block(work);
                    // Ordinary managed Drain publishes its own frame; startup remains rooted.
                    let result = b.ins().stack_addr(types::I64, storage, 8);
                    let callee = module.declare_func_in_func(ids[drain], b.func);
                    let call = b.ins().call(callee, &[queue, result, ctx]);
                    let status = b.inst_results(call)[0];
                    let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                    b.ins().brif(failed, done, &[status.into()], poll_work, &[]);
                } else {
                    b.ins().jump(poll_work, &[]);
                }
                b.switch_to_block(poll_work);
                let poll = module.declare_func_in_func(poll, b.func);
                let required = b.ins().iconst(types::I32, i64::from(d.entry_host_io));
                let call = b.ins().call(poll, &[ctx, required, queue_output]);
                let ready = b.inst_results(call)[0];
                let pending = b.create_block();
                let idle = b.ins().icmp_imm(IntCC::Equal, ready, 0);
                b.ins().brif(idle, done, &[zero.into()], pending, &[]);
                b.switch_to_block(pending);
                let dispatch = b.create_block();
                let waiting = b.ins().icmp_imm(IntCC::Equal, ready, 2);
                b.ins().brif(waiting, cycle, &[], dispatch, &[]);
                b.switch_to_block(dispatch);
                let invoke = b.create_block();
                let fault = b.create_block();
                let available = b.ins().icmp_imm(IntCC::Equal, ready, 1);
                b.ins().brif(available, invoke, &[], fault, &[]);
                b.switch_to_block(fault);
                let error = b.ins().iconst(types::I32, 3);
                site.record(&mut b, error);
                b.ins().jump(done, &[error.into()]);
                b.switch_to_block(invoke);
                let handle = b.ins().load(types::I64, MemFlags::new(), queue_output, 0);
                let callback = module.declare_func_in_func(callback, b.func);
                let call = b.ins().call(callback, &[handle, ctx]);
                let status = b.inst_results(call)[0];
                let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                b.ins().brif(failed, done, &[status.into()], cycle, &[]);
                b.switch_to_block(done);
                let status = b.block_params(done)[0];
                b.ins().call(end, &[ctx]);
                let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                return_if_detailed(&mut b, failed, status, None);
                b.ins().store(MemFlags::new(), zero, output, 0);
                b.ins().return_(&[zero]);
                b.seal_all_blocks();
                b.finalize();
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(module, &mut context.func, frame);
                }
                define_checked(module, ids[i], &mut context, stack_budget, long_range_calls)?;
                continue;
            }
            if let Some(service) = socket_services.get(&i) {
                let service = module.declare_func_in_func(*service, b.func);
                let argument_lanes: usize = p.args[i].iter().map(|t| lanes(&p, t).len()).sum();
                let mut args = parameters[..argument_lanes].to_vec();
                args.push(fault_context.unwrap());
                if details.unwrap().task_queue_current.contains(&i) {
                    let run = b.ins().iconst(
                        types::I32,
                        details.unwrap().task_queue_run.map_or(-1, |i| i as i64),
                    );
                    let drain = b.ins().iconst(
                        types::I32,
                        details.unwrap().task_queue_drain.map_or(-1, |i| i as i64),
                    );
                    args.extend([run, drain]);
                }
                args.push(output);
                if details.unwrap().stack_trace_capture.contains(&i) {
                    // Unlike interpreter intrinsics, the native service wrapper publishes
                    // its own probe frame. Hide it without changing managed skip semantics.
                    // Preserve negative input for validation and saturate at Int32::MAX.
                    let valid = b
                        .ins()
                        .icmp_imm(IntCC::SignedGreaterThanOrEqual, args[0], 0);
                    let bounded = b
                        .ins()
                        .icmp_imm(IntCC::SignedLessThan, args[0], i32::MAX as i64);
                    let increment = b.ins().band(valid, bounded);
                    let next = b.ins().iadd_imm(args[0], 1);
                    args[0] = b.ins().select(increment, next, args[0]);
                    args.extend(
                        diagnostic_data
                            .as_ref()
                            .unwrap()
                            .stack_names(module, &mut b),
                    );
                }
                if details.unwrap().runtime_async_start.contains(&i)
                    || details.unwrap().runtime_async_startmanaged.contains(&i)
                {
                    let callback =
                        module.declare_func_in_func(runtime_async_callbacks.unwrap().0, b.func);
                    args.push(b.ins().func_addr(types::I64, callback));
                }
                let call = b.ins().call(service, &args);
                let raw = b.inst_results(call)[0];
                let failed = b.ins().icmp_imm(IntCC::NotEqual, raw, 0);
                let exhausted = b.ins().icmp_imm(IntCC::Equal, raw, 5);
                let memory = b.ins().iconst(types::I32, 5);
                let runtime = b.ins().iconst(types::I32, 3);
                let status = b.ins().select(exhausted, memory, runtime);
                let mut site = diagnostic_data.as_ref().unwrap().site(
                    module,
                    &mut b,
                    fault_context.unwrap(),
                    i,
                    0,
                );
                site.capture_frame = false;
                return_if_detailed(&mut b, failed, status, Some(&site));
                if let Some(ty @ Ty::StringArray) = &p.results[i] {
                    let array = b.ins().load(types::I64, MemFlags::new(), output, 0);
                    tag_array(&mut b, &p, ty, array);
                }
                let zero = b.ins().iconst(types::I32, 0);
                b.ins().return_(&[zero]);
                b.seal_all_blocks();
                b.finalize();
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(module, &mut context.func, frame);
                }
                define_checked(module, ids[i], &mut context, stack_budget, long_range_calls)?;
                continue;
            }
            if let Some(operation) = details.and_then(|d| d.cryptography.get(&i)) {
                let service = module.declare_func_in_func(cryptography_service.unwrap(), b.func);
                let op = b.ins().iconst(types::I32, i64::from(*operation));
                let null = b.ins().iconst(types::I64, 0);
                let mut args = vec![op, fault_context.unwrap()];
                let arity = match operation {
                    1 => 2,
                    4 => 1,
                    _ => 3,
                };
                for position in 0..arity {
                    let storage = slot(&mut b, 24);
                    let address = b.ins().stack_addr(types::I64, storage, 0);
                    let start = position * 3;
                    b.ins()
                        .store(MemFlags::new(), parameters[start], address, 0);
                    b.ins()
                        .store(MemFlags::new(), parameters[start + 1], address, 8);
                    b.ins()
                        .store(MemFlags::new(), parameters[start + 2], address, 12);
                    let writable = *operation == 4 || *operation != 3 && position == arity - 1;
                    let kind = b.ins().iconst(types::I64, if writable { 0 } else { 1 });
                    b.ins().store(MemFlags::new(), kind, address, 16);
                    args.push(address);
                }
                while args.len() < 5 {
                    args.push(null);
                }
                args.push(output);
                let call = b.ins().call(service, &args);
                let status = b.inst_results(call)[0];
                let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                let mut site = diagnostic_data.as_ref().unwrap().site(
                    module,
                    &mut b,
                    fault_context.unwrap(),
                    i,
                    0,
                );
                site.capture_frame = false;
                return_if_detailed(&mut b, failed, status, Some(&site));
                let zero = b.ins().iconst(types::I32, 0);
                b.ins().return_(&[zero]);
                b.seal_all_blocks();
                b.finalize();
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(module, &mut context.func, frame);
                }
                define_checked(module, ids[i], &mut context, stack_budget, long_range_calls)?;
                continue;
            }
            if let Some(operation) = details.and_then(|d| d.tls.get(&i)) {
                let service = module.declare_func_in_func(tls_service.unwrap(), b.func);
                let op = b.ins().iconst(types::I32, i64::from(*operation));
                let null = b.ins().iconst(types::I64, 0);
                let mut args = vec![op, fault_context.unwrap()];
                let arity = match operation {
                    2 => 4,
                    4 | 5 => 1,
                    6 => 5,
                    _ => 3,
                };
                args.extend(parameters.iter().take(arity).copied());
                while args.len() < 7 {
                    args.push(null);
                }
                args.push(output);
                let call = b.ins().call(service, &args);
                let status = b.inst_results(call)[0];
                let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                let mut site = diagnostic_data.as_ref().unwrap().site(
                    module,
                    &mut b,
                    fault_context.unwrap(),
                    i,
                    0,
                );
                site.capture_frame = false;
                return_if_detailed(&mut b, failed, status, Some(&site));
                let zero = b.ins().iconst(types::I32, 0);
                b.ins().return_(&[zero]);
                b.seal_all_blocks();
                b.finalize();
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(module, &mut context.func, frame);
                }
                define_checked(module, ids[i], &mut context, stack_budget, long_range_calls)?;
                continue;
            }
            if let Some(service) = stream_services.get(&i) {
                let service = module.declare_func_in_func(*service, b.func);
                let mut call_args = vec![parameters[0]];
                if details.unwrap().console_write_memory.contains(&i) {
                    // Resolve a checked synchronous window. The provider neither
                    // retains this pointer nor calls back into managed code.
                    let storage = slot(&mut b, 24);
                    let view = b.ins().stack_addr(types::I64, storage, 0);
                    b.ins().store(MemFlags::new(), parameters[1], view, 0);
                    b.ins().store(MemFlags::new(), parameters[2], view, 8);
                    b.ins().store(MemFlags::new(), parameters[3], view, 12);
                    let borrowed = input
                        .type_definition(&input.functions[i].parameters[1])
                        .is_some_and(|t| {
                            t.memory_view == Some(neoclr::metadata::MemoryViewKind::ReadOnlySlice)
                        });
                    let kind = b.ins().iconst(types::I64, if borrowed { 1 } else { 3 });
                    b.ins().store(MemFlags::new(), kind, view, 16);
                    let storage = slot(&mut b, 24);
                    let access = b.ins().stack_addr(types::I64, storage, 0);
                    let readonly = b.ins().iconst(types::I32, 0);
                    let check = module.declare_func_in_func(console_memory_access.unwrap(), b.func);
                    let call = b
                        .ins()
                        .call(check, &[fault_context.unwrap(), view, readonly, access]);
                    let status = b.inst_results(call)[0];
                    let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                    let mut site = diagnostic_data.as_ref().unwrap().site(
                        module,
                        &mut b,
                        fault_context.unwrap(),
                        i,
                        0,
                    );
                    site.capture_frame = false;
                    return_if_detailed(&mut b, failed, status, Some(&site));
                    let bytes = b.ins().load(types::I64, MemFlags::new(), access, 0);
                    let count = b.ins().load(types::I32, MemFlags::new(), access, 16);
                    let length = b.ins().uextend(types::I64, count);
                    let zero = b.ins().iconst(types::I32, 0);
                    call_args.extend([bytes, length, zero, count]);
                } else if details.unwrap().console_write_bytes.contains(&i) {
                    let array = parameters[1];
                    let null = b.ins().icmp_imm(IntCC::Equal, array, 0);
                    let status = b.ins().iconst(types::I32, 3);
                    let mut site = diagnostic_data.as_ref().unwrap().site(
                        module,
                        &mut b,
                        fault_context.unwrap(),
                        i,
                        0,
                    );
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
                } else {
                    b.ins().icmp_imm(IntCC::NotEqual, raw, 0)
                };
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
                    probes.finish(module, &mut context.func, frame);
                }
                define_checked(module, ids[i], &mut context, stack_budget, long_range_calls)?;
                continue;
            }
            if let Some((_, kind)) =
                details.and_then(|d| d.console_terminal.iter().find(|(index, _)| *index == i))
            {
                let service = module.declare_func_in_func(terminal_services[*kind], b.func);
                let mut args = vec![fault_context.unwrap()];
                for parameter in parameters.iter().take(f.parameters.len()) {
                    let value = if b.func.dfg.value_type(*parameter) == types::I32 {
                        *parameter
                    } else {
                        b.ins().uextend(types::I32, *parameter)
                    };
                    args.push(value);
                }
                let call = b.ins().call(service, &args);
                let value = b.inst_results(call)[0];
                write(&mut b, output, &[value]);
                let zero = b.ins().iconst(types::I32, 0);
                b.ins().return_(&[zero]);
                b.seal_all_blocks();
                b.finalize();
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(module, &mut context.func, frame);
                }
                define_checked(module, ids[i], &mut context, stack_budget, long_range_calls)?;
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
                    probes.finish(module, &mut context.func, frame);
                }
                define_checked(module, ids[i], &mut context, stack_budget, long_range_calls)?;
                continue;
            }
            if details.is_some_and(|d| d.console_write_line.contains(&i)) {
                let service = module.declare_func_in_func(output_service.unwrap(), b.func);
                // Preserve the interpreter's invalid-native-argument RuntimeError
                // for default/null String instead of dereferencing address zero.
                let null = b.ins().icmp_imm(IntCC::Equal, parameters[0], 0);
                let null_status = b.ins().iconst(types::I32, 3);
                let mut null_site = diagnostic_data.as_ref().unwrap().site(
                    module,
                    &mut b,
                    fault_context.unwrap(),
                    i,
                    0,
                );
                null_site.capture_frame = false;
                return_if_detailed(&mut b, null, null_status, Some(&null_site));
                let length = b.ins().load(types::I64, MemFlags::new(), parameters[0], 0);
                let bytes = b.ins().iadd_imm(parameters[0], 8);
                let call = b.ins().call(service, &[bytes, length]);
                let result = b.inst_results(call)[0];
                let failed = b.ins().icmp_imm(IntCC::NotEqual, result, 0);
                let status = b.ins().iconst(types::I32, 3); // RuntimeError, same as interpreter
                let mut site = diagnostic_data.as_ref().unwrap().site(
                    module,
                    &mut b,
                    fault_context.unwrap(),
                    i,
                    0,
                );
                site.capture_frame = false; // InternalCall has no managed frame
                return_if_detailed(&mut b, failed, status, Some(&site));
                let zero = b.ins().iconst(types::I32, 0);
                if !f.no_result {
                    write(&mut b, output, &[zero]);
                }
                b.ins().return_(&[zero]);
                b.seal_all_blocks();
                b.finalize();
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(module, &mut context.func, frame);
                }
                define_checked(module, ids[i], &mut context, stack_budget, long_range_calls)?;
                continue;
            }
            if details.is_some_and(|d| d.char_from_string.contains(&i) || d.char_text.contains(&i))
            {
                let pointer = parameters[0];
                let mut site = diagnostic_data.as_ref().unwrap().site(
                    module,
                    &mut b,
                    fault_context.unwrap(),
                    i,
                    0,
                );
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
                        copy_text(
                            module,
                            &mut b,
                            binding,
                            pointer,
                            fault_context.unwrap(),
                            Some(&site),
                        )
                    } else {
                        pointer
                    }
                } else {
                    pointer
                };
                write(&mut b, output, &[pointer]);
                let zero = b.ins().iconst(types::I32, 0);
                b.ins().return_(&[zero]);
                b.seal_all_blocks();
                b.finalize();
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(module, &mut context.func, frame);
                }
                define_checked(module, ids[i], &mut context, stack_budget, long_range_calls)?;
                continue;
            }
            if details.is_some_and(|d| d.native_integer_to64.contains(&i)) {
                write(&mut b, output, &[parameters[0]]);
                let zero = b.ins().iconst(types::I32, 0);
                b.ins().return_(&[zero]);
                b.seal_all_blocks();
                b.finalize();
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(module, &mut context.func, frame);
                }
                define_checked(module, ids[i], &mut context, stack_budget, long_range_calls)?;
                continue;
            }
            if let Some(service) = utf8_services.get(&i) {
                let service = module.declare_func_in_func(*service, b.func);
                let args = if details.unwrap().utf8_decode_slice.contains(&i) {
                    vec![
                        fault_context.unwrap(),
                        parameters[0],
                        parameters[1],
                        parameters[2],
                        output,
                    ]
                } else if details.unwrap().file_input.contains(&i) {
                    let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                    vec![parameters[0], parameters[1], arena, output]
                } else if details.unwrap().file_output.contains(&i) {
                    vec![parameters[0], parameters[1], parameters[2], output]
                } else if details.unwrap().string_join_parts.contains(&i) {
                    let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                    vec![
                        parameters[0],
                        parameters[1],
                        parameters[2],
                        parameters[3],
                        arena,
                        output,
                    ]
                } else if details.unwrap().string_slice_graphemes.contains(&i)
                    || details.unwrap().string_slice_utf8.contains(&i)
                    || details.unwrap().string_replace_ordinal.contains(&i)
                {
                    let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                    vec![parameters[0], parameters[1], parameters[2], arena, output]
                } else if details.unwrap().system_culture.contains(&i)
                    || details.unwrap().time_zone_version.contains(&i)
                {
                    let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                    vec![arena, output]
                } else if details.unwrap().string_grapheme_at.contains(&i)
                    || details.unwrap().string_replace_line_endings.contains(&i)
                    || details.unwrap().string_concat.contains(&i)
                    || details.unwrap().path_combine.contains(&i)
                {
                    let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                    vec![parameters[0], parameters[1], arena, output]
                } else if details.unwrap().string_graphemes.contains(&i)
                    || details.unwrap().string_scalars.contains(&i)
                    || details.unwrap().string_from_chars.contains(&i)
                    || details.unwrap().string_upper.contains(&i)
                    || details.unwrap().string_lower.contains(&i)
                    || details.unwrap().string_intern.contains(&i)
                    || details.unwrap().utf8_encode.contains(&i)
                    || details.unwrap().utf8_decode.contains(&i)
                    || details.unwrap().path_file_name.contains(&i)
                    || details.unwrap().environment_platform.contains(&i)
                {
                    let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                    vec![parameters[0], arena, output]
                } else if details.unwrap().time_zone_offset.contains(&i)
                    || details.unwrap().string_compare_canonical.contains(&i)
                    || details.unwrap().string_compare_ignore_case.contains(&i)
                    || details.unwrap().string_compare_ordinal.contains(&i)
                    || details.unwrap().string_contains_canonical.contains(&i)
                    || details.unwrap().string_starts_with_canonical.contains(&i)
                    || details.unwrap().string_ends_with_canonical.contains(&i)
                    || details.unwrap().string_contains_ignore_case.contains(&i)
                    || details.unwrap().string_starts_with_ignore_case.contains(&i)
                    || details.unwrap().string_ends_with_ignore_case.contains(&i)
                    || details.unwrap().string_contains_ordinal.contains(&i)
                    || details.unwrap().string_starts_with_ordinal.contains(&i)
                    || details.unwrap().string_ends_with_ordinal.contains(&i)
                {
                    vec![parameters[0], parameters[1], output]
                } else {
                    vec![parameters[0], output]
                };
                let call = b.ins().call(service, &args);
                let raw = b.inst_results(call)[0];
                let exhausted = b.ins().icmp_imm(IntCC::Equal, raw, 5);
                let memory = b.ins().iconst(types::I32, 5);
                let runtime = b.ins().iconst(types::I32, 3);
                let status = b.ins().select(exhausted, memory, runtime);
                let status = if details.unwrap().utf8_encode.contains(&i)
                    || details.unwrap().string_graphemes.contains(&i)
                    || details.unwrap().string_scalars.contains(&i)
                {
                    let limited = b.ins().icmp_imm(IntCC::Equal, raw, 7);
                    let array_limit = b.ins().iconst(types::I32, 7);
                    b.ins().select(limited, array_limit, status)
                } else {
                    status
                };
                let status = if details.unwrap().string_grapheme_at.contains(&i)
                    || details.unwrap().string_slice_graphemes.contains(&i)
                {
                    let bounded = b.ins().icmp_imm(IntCC::Equal, raw, 8);
                    let index = b.ins().iconst(types::I32, 8);
                    b.ins().select(bounded, index, status)
                } else if details.unwrap().string_intern.contains(&i) {
                    let limited = b.ins().icmp_imm(IntCC::Equal, raw, 10);
                    let intern = b.ins().iconst(types::I32, 10);
                    b.ins().select(limited, intern, status)
                } else {
                    status
                };
                let failed = b.ins().icmp_imm(IntCC::NotEqual, raw, 0);
                let mut site = diagnostic_data.as_ref().unwrap().site(
                    module,
                    &mut b,
                    fault_context.unwrap(),
                    i,
                    0,
                );
                site.capture_frame = false;
                return_if_detailed(&mut b, failed, status, Some(&site));
                if let Some(ty) = &p.results[i] {
                    // The legacy helper produces an immutable value snapshot. A native
                    // Byte[] result owns this fresh allocation as an ordinary array.
                    if *ty == Ty::ByteArray && details.unwrap().utf8_encode.contains(&i) {
                        let array = b.ins().load(types::I64, MemFlags::new(), output, 0);
                        let kind = b.ins().iconst(types::I64, 0x80000001);
                        b.ins().store(MemFlags::new(), kind, array, 0);
                    }
                    if p.backing_for_array(ty).is_some() {
                        let array = b.ins().load(types::I64, MemFlags::new(), output, 0);
                        tag_array(&mut b, &p, ty, array);
                    }
                }
                let zero = b.ins().iconst(types::I32, 0);
                b.ins().return_(&[zero]);
                b.seal_all_blocks();
                b.finalize();
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(module, &mut context.func, frame);
                }
                define_checked(module, ids[i], &mut context, stack_budget, long_range_calls)?;
                continue;
            }
            if details.is_some_and(|d| d.int32_to_string.contains(&i))
                || wide_format_services.contains_key(&i)
                || float_format_services.contains_key(&i)
            {
                let service = module.declare_func_in_func(
                    float_format_services
                        .get(&i)
                        .or_else(|| wide_format_services.get(&i))
                        .copied()
                        .or(format_service)
                        .unwrap(),
                    b.func,
                );
                let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                let args = if float_format_services.contains_key(&i) {
                    let swedish = parameters[3];
                    vec![
                        parameters[0],
                        parameters[1],
                        parameters[2],
                        swedish,
                        arena,
                        output,
                    ]
                } else {
                    vec![parameters[0], arena, output]
                };
                let call = b.ins().call(service, &args);
                let raw = b.inst_results(call)[0];
                // Only success and resource exhaustion are service outcomes. Other
                // failures become RuntimeError; adapters cannot manufacture UserFault.
                let exhausted = b.ins().icmp_imm(IntCC::Equal, raw, 5);
                let memory = b.ins().iconst(types::I32, 5);
                let runtime = b.ins().iconst(types::I32, 3);
                let status = b.ins().select(exhausted, memory, runtime);
                let failed = b.ins().icmp_imm(IntCC::NotEqual, raw, 0);
                let mut site = diagnostic_data.as_ref().unwrap().site(
                    module,
                    &mut b,
                    fault_context.unwrap(),
                    i,
                    0,
                );
                site.capture_frame = false;
                return_if_detailed(&mut b, failed, status, Some(&site));
                let zero = b.ins().iconst(types::I32, 0);
                b.ins().return_(&[zero]);
                b.seal_all_blocks();
                b.finalize();
                if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                    probes.finish(module, &mut context.func, frame);
                }
                define_checked(module, ids[i], &mut context, stack_budget, long_range_calls)?;
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
            let root_spill = root_probes
                .as_ref()
                .and_then(|probes| probes.storage_bytes(i))
                .map(|bytes| {
                    frame_bytes += bytes as usize + 7;
                    slot(&mut b, bytes)
                });
            let storage_table = root_probes.as_ref().map(|probes| {
                let bytes = probes.table_bytes(i);
                frame_bytes += probes.frame_bytes() as usize + 7 + bytes as usize + 7;
                probe_frame = Some(slot(&mut b, probes.frame_bytes()));
                slot(&mut b, bytes)
            });
            let transient_table = root_probes
                .as_ref()
                .and_then(|probes| probes.transient_bytes(i))
                .map(|bytes| {
                    frame_bytes += bytes as usize + 7;
                    slot(&mut b, bytes)
                });
            if frame_bytes > 65536 {
                return Err("value profile frame storage exceeds 64 KiB".into());
            }
            if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
                mutation_frame_ready = Some(probes.enter(
                    module,
                    &mut b,
                    frame,
                    fault_context.unwrap(),
                    i,
                    storage_table.unwrap(),
                    &arguments,
                    &locals,
                ));
                emit_stack_check(
                    module,
                    &mut b,
                    stack_check,
                    diagnostic_data.as_ref(),
                    fault_context.unwrap(),
                    i,
                    true,
                );
            }
            if let Some(guard) = crate::type_initialization::entry_owner(input, f)
                .and_then(|owner| owner.definition_name())
                .and_then(|name| initialization_guards.get(name))
            {
                if let (Some(probes), Some(spill)) = (&root_probes, root_spill) {
                    probes.emit(module, &mut b, i, 0, spill, probe_frame.unwrap(), &[]);
                }
                let callee = module.declare_func_in_func(*guard, b.func);
                let call = b.ins().call(callee, &[fault_context.unwrap()]);
                let status = b.inst_results(call)[0];
                let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                let site = diagnostic_data
                    .as_ref()
                    .map(|d| d.site(module, &mut b, fault_context.unwrap(), i, 0));
                return_if_detailed(&mut b, failed, status, site.as_ref());
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
                let mut site = diagnostic_data
                    .as_ref()
                    .map(|d| d.site(module, &mut b, fault_context.unwrap(), i, pc));
                if let Some(&adapter_pc) =
                    details.and_then(|d| d.reflection_array_sites.get(&(i, pc)))
                    && let Some(site) = site.as_mut()
                {
                    site.remap_instruction(adapter_pc);
                }
                if matches!(op, Op::Construct(_))
                    && details.is_some_and(|d| d.reflection_construction.contains(&i))
                {
                    // The interpreter's checked construction adapter is newobj/ret.
                    if let Some(site) = site.as_mut() {
                        site.remap_instruction(0);
                    }
                }

                if (!matches!(op, Op::StaticField { .. } | Op::SetStaticField { .. })
                    || details.is_some_and(|d| d.native_worker_safepoints))
                    && let (Some(probes), Some(spill)) = (&root_probes, root_spill)
                {
                    probes.emit(module, &mut b, i, pc, spill, probe_frame.unwrap(), &stack);
                }
                if !initialization_guards.is_empty() {
                    let initialization_owner = match op {
                        Op::StaticField { owner, .. } | Op::SetStaticField { owner, .. } => {
                            Some(owner)
                        }
                        Op::Call(target) | Op::CallVirtual(target) | Op::Construct(target)
                            if crate::selection::callable_invoke(target).is_none() =>
                        {
                            crate::type_initialization::entry_owner(
                                input,
                                &input.functions[p.callee(target)?],
                            )
                        }
                        _ => None,
                    };
                    if let Some(guard) = initialization_owner
                        .and_then(|owner| owner.definition_name())
                        .and_then(|name| initialization_guards.get(name))
                    {
                        if matches!(op, Op::StaticField { .. } | Op::SetStaticField { .. })
                            && let (Some(probes), Some(spill)) = (&root_probes, root_spill)
                        {
                            probes.emit(module, &mut b, i, pc, spill, probe_frame.unwrap(), &stack);
                        }
                        let callee = module.declare_func_in_func(*guard, b.func);
                        let call = b.ins().call(callee, &[fault_context.unwrap()]);
                        let status = b.inst_results(call)[0];
                        let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                        return_if_detailed(&mut b, failed, status, site.as_ref());
                    }
                }
                match op {
                    Op::Int(v) => stack.push(b.ins().iconst(types::I32, i64::from(*v))),
                    Op::Int64(v) => stack.push(b.ins().iconst(types::I64, *v)),
                    Op::Float32 { bits } => {
                        stack.push(b.ins().f32const(ir::immediates::Ieee32::with_bits(*bits)))
                    }
                    Op::ConvertFloat32 | Op::ConvertFloat64 | Op::ConvertFloatUnsigned => {
                        let value = pop(&mut stack);
                        let target = if matches!(op, Op::ConvertFloat32) {
                            types::F32
                        } else {
                            types::F64
                        };
                        let converted = match top() {
                            Ty::Single if target == types::F64 => b.ins().fpromote(target, value),
                            Ty::Double if target == types::F32 => b.ins().fdemote(target, value),
                            Ty::Single | Ty::Double => value,
                            _ if matches!(op, Op::ConvertFloatUnsigned) => {
                                b.ins().fcvt_from_uint(target, value)
                            }
                            _ => b.ins().fcvt_from_sint(target, value),
                        };
                        stack.push(converted);
                    }
                    Op::Float64 { bits } => {
                        stack.push(b.ins().f64const(ir::immediates::Ieee64::with_bits(*bits)))
                    }
                    Op::String(_) => {
                        let data = module.declare_data_in_func(literals[&(i, pc)], b.func);
                        let pointer = b.ins().global_value(types::I64, data);
                        if let Some(binding) = text_copy {
                            let owned = copy_text(
                                module,
                                &mut b,
                                binding,
                                pointer,
                                fault_context.unwrap(),
                                site.as_ref(),
                            );
                            stack.push(owned);
                        } else {
                            stack.push(pointer);
                        }
                    }
                    Op::IsInstance(neoclr::metadata::Type::String)
                    | Op::CastClass(neoclr::metadata::Type::String)
                        if *top() == Ty::Literal => {}
                    Op::IsInstance(target) | Op::UnboxAny(target)
                        if crate::scalar_unions::value_only(input, target) =>
                    {
                        let start = stack.len() - p.lanes(top());
                        let tag = stack[start];
                        if matches!(op, Op::IsInstance(_)) {
                            stack.truncate(start);
                            let present = b.ins().icmp_imm(IntCC::NotEqual, tag, 0);
                            stack.push(b.ins().uextend(types::I32, present));
                        }
                        // Identity unboxing preserves the default/null union, matching
                        // the interpreter; extracting a concrete value still faults.
                    }
                    Op::BoxValue(_) => {
                        // The adjacent admitted union cast consumes this scalar;
                        // no Object box exists or escapes in the native program.
                    }
                    Op::CastClass(target)
                        if crate::scalar_unions::inline(input, target)
                            && p.widens_union(top(), &p.ty(target)?) =>
                    {
                        let values = stack.split_off(stack.len() - p.lanes(top()));
                        stack.extend(widen_union(&p, &mut b, top(), &p.ty(target)?, values));
                    }
                    Op::CastClass(target)
                        if matches!(p.ty(target)?, Ty::ValueUnion(_))
                            && (matches!(
                                top(),
                                Ty::PendingUnionScalar(_) | Ty::Callable(_) | Ty::Literal
                            ) || *top() == p.ty(target)?) =>
                    {
                        if matches!(
                            top(),
                            Ty::PendingUnionScalar(_) | Ty::Callable(_) | Ty::Literal
                        ) {
                            let payload_type = match top() {
                                Ty::PendingUnionScalar(t) => p.ty(t)?,
                                other => other.clone(),
                            };
                            let Ty::ValueUnion(parts) = p.ty(target)? else {
                                unreachable!()
                            };
                            let payload = stack.split_off(stack.len() - p.lanes(&payload_type));
                            let active = parts
                                .iter()
                                .position(|part| *part == payload_type)
                                .expect("admitted exact payload");
                            let tag = b.ins().iconst(types::I32, active as i64 + 1);
                            let tag = if matches!(payload_type, Ty::Callable(_) | Ty::Literal) {
                                let null = b.ins().icmp_imm(IntCC::Equal, payload[0], 0);
                                let empty = b.ins().iconst(types::I32, 0);
                                b.ins().select(null, empty, tag)
                            } else {
                                tag
                            };
                            stack.push(tag);
                            for (index, part) in parts.iter().enumerate() {
                                if index == active {
                                    stack.extend(payload.iter().copied());
                                } else {
                                    stack.extend(
                                        lanes(&p, part).into_iter().map(|ty| zero(&mut b, ty)),
                                    );
                                }
                            }
                        }
                    }
                    Op::IsInstance(
                        target @ (neoclr::metadata::Type::Function(_)
                        | neoclr::metadata::Type::String),
                    )
                    | Op::CastClass(
                        target @ (neoclr::metadata::Type::Function(_)
                        | neoclr::metadata::Type::String),
                    ) if matches!(top(), Ty::ValueUnion(_)) => {
                        let Ty::ValueUnion(parts) = top() else {
                            unreachable!()
                        };
                        let values = stack.split_off(stack.len() - p.lanes(top()));
                        let target_type = p.ty(target)?;
                        let active = parts.iter().position(|part| *part == target_type);
                        let expected = active.map_or(-1, |i| i as i64 + 1);
                        let matched = b.ins().icmp_imm(IntCC::Equal, values[0], expected);
                        let null = b.ins().iconst(types::I64, 0);
                        if matches!(op, Op::CastClass(_)) {
                            let occupied = b.ins().icmp_imm(IntCC::NotEqual, values[0], 0);
                            let mismatch = b.ins().icmp_imm(IntCC::NotEqual, values[0], expected);
                            let invalid = b.ins().band(occupied, mismatch);
                            let status = b.ins().iconst(types::I32, 11);
                            return_if_detailed(&mut b, invalid, status, site.as_ref());
                        }
                        let payload = if let Some(index) = active {
                            let offset = 1 + parts[..index]
                                .iter()
                                .map(|part| p.lanes(part))
                                .sum::<usize>();
                            values[offset]
                        } else {
                            null
                        };
                        stack.push(b.ins().select(matched, payload, null));
                    }
                    Op::IsInstance(target) | Op::UnboxAny(target)
                        if matches!(top(), Ty::ValueUnion(_))
                            && (crate::scalar_unions::tag(target).is_some()
                                || crate::scalar_unions::bank_scalar(target)
                                || crate::scalar_unions::record(input, target)) =>
                    {
                        let Ty::ValueUnion(parts) = top() else {
                            unreachable!()
                        };
                        let values = stack.split_off(stack.len() - p.lanes(top()));
                        let target_type = p.ty(target)?;
                        let active = parts.iter().position(|part| *part == target_type);
                        let expected = active.map_or(-1, |i| i as i64 + 1);
                        if matches!(op, Op::IsInstance(_)) {
                            let matched = b.ins().icmp_imm(IntCC::Equal, values[0], expected);
                            stack.push(b.ins().uextend(types::I32, matched));
                        } else {
                            let null = b.ins().icmp_imm(IntCC::Equal, values[0], 0);
                            let status = b.ins().iconst(types::I32, 6);
                            return_if_detailed(&mut b, null, status, site.as_ref());
                            let mismatch = b.ins().icmp_imm(IntCC::NotEqual, values[0], expected);
                            let status = b.ins().iconst(types::I32, 11);
                            return_if_detailed(&mut b, mismatch, status, site.as_ref());
                            if let Some(index) = active {
                                let offset = 1 + parts[..index]
                                    .iter()
                                    .map(|part| p.lanes(part))
                                    .sum::<usize>();
                                stack.extend_from_slice(
                                    &values[offset..offset + p.lanes(&target_type)],
                                );
                            } else {
                                stack.extend(
                                    lanes(&p, &target_type)
                                        .into_iter()
                                        .map(|ty| zero(&mut b, ty)),
                                );
                            }
                        }
                    }
                    Op::CastClass(target)
                        if crate::scalar_unions::supported(target)
                            && (matches!(
                                top(),
                                Ty::PendingUnionScalar(_) | Ty::ScalarUnion(_)
                            ) || *top() == p.ty(target)?) =>
                    {
                        if let Ty::PendingUnionScalar(scalar) = top() {
                            let value = pop(&mut stack);
                            let payload = if b.func.dfg.value_type(value) == types::I64 {
                                value
                            } else {
                                b.ins().uextend(types::I64, value)
                            };
                            stack.push(
                                b.ins()
                                    .iconst(types::I32, crate::scalar_unions::tag(scalar).unwrap()),
                            );
                            stack.push(payload);
                            if matches!(p.ty(target)?, Ty::MixedUnion(_)) {
                                stack.push(b.ins().iconst(types::I64, 0));
                            }
                        }
                    }
                    Op::IsInstance(target) if crate::scalar_unions::tag(target).is_some() => {
                        if matches!(top(), Ty::MixedUnion(_)) {
                            pop(&mut stack);
                        }
                        pop(&mut stack); // Numeric payload is not needed for the test.
                        let tag = pop(&mut stack);
                        let matched = b.ins().icmp_imm(
                            IntCC::Equal,
                            tag,
                            crate::scalar_unions::tag(target).unwrap(),
                        );
                        stack.push(b.ins().uextend(types::I32, matched));
                    }
                    Op::UnboxAny(target) => {
                        if matches!(top(), Ty::MixedUnion(_)) {
                            pop(&mut stack);
                        }
                        let payload = pop(&mut stack);
                        let tag = pop(&mut stack);
                        let null = b.ins().icmp_imm(IntCC::Equal, tag, 0);
                        let null_status = b.ins().iconst(types::I32, 6);
                        return_if_detailed(&mut b, null, null_status, site.as_ref());
                        let mismatch = b.ins().icmp_imm(
                            IntCC::NotEqual,
                            tag,
                            crate::scalar_unions::tag(target).unwrap(),
                        );
                        let cast_status = b.ins().iconst(types::I32, 11);
                        return_if_detailed(&mut b, mismatch, cast_status, site.as_ref());
                        stack.push(if *target == neoclr::metadata::Type::Int64 {
                            payload
                        } else {
                            b.ins().ireduce(types::I32, payload)
                        });
                    }
                    Op::IsInstance(target) | Op::CastClass(target) => {
                        let (value, record_tag) = if let Ty::ValueUnion(parts) = top() {
                            let values = stack.split_off(stack.len() - p.lanes(top()));
                            let mut reference = b.ins().iconst(types::I64, 0);
                            let mut offset = 1;
                            for part in parts {
                                if matches!(part, Ty::Reference(_) | Ty::Interface(_)) {
                                    reference = b.ins().bor(reference, values[offset]);
                                }
                                offset += p.lanes(part);
                            }
                            (reference, Some(values[0]))
                        } else {
                            (pop(&mut stack), None)
                        };
                        let source_tag = if matches!(top(), Ty::MixedUnion(_)) {
                            pop(&mut stack); // Scalar bits never become an object address.
                            Some(pop(&mut stack))
                        } else {
                            record_tag
                        };
                        let probe = b.create_block();
                        let joined = b.create_block();
                        b.append_block_param(joined, types::I64);
                        let null = b.ins().icmp_imm(IntCC::Equal, value, 0);
                        b.ins().brif(null, joined, &[value.into()], probe, &[]);
                        b.switch_to_block(probe);
                        let null_reference = b.ins().iconst(types::I64, 0);
                        let target_ty = p.ty(target)?;
                        let cast = if *top() == Ty::Literal {
                            if string_view(&target_ty) {
                                text_object_view(&mut b, value)
                            } else {
                                null_reference
                            }
                        } else {
                            let text = b.create_block();
                            let object = b.create_block();
                            let merged = b.create_block();
                            b.append_block_param(merged, types::I64);
                            let tagged = b.ins().band_imm(value, 1);
                            b.ins().brif(tagged, text, &[], object, &[]);
                            b.switch_to_block(text);
                            let result = if target_ty == Ty::Literal {
                                b.ins().band_imm(value, -2)
                            } else if string_view(&target_ty) {
                                value
                            } else {
                                null_reference
                            };
                            b.ins().jump(merged, &[result.into()]);
                            b.switch_to_block(object);
                            let result = if target_ty == Ty::Literal {
                                null_reference
                            } else {
                                let tag = b.ins().load(types::I64, MemFlags::new(), value, 0);
                                let mut matches = b.ins().iconst(types::I8, 0);
                                let callable = b.ins().icmp_imm(IntCC::Equal, tag, u32::MAX as i64);
                                if p.is_object_base(&target_ty) {
                                    matches = callable;
                                }
                                if let Ty::Callable(shape) = &target_ty {
                                    // Check the descriptor header before reading its target slot:
                                    // an ordinary empty Object need not have payload storage.
                                    let function = b.create_block();
                                    let done = b.create_block();
                                    b.append_block_param(done, types::I8);
                                    b.ins()
                                        .brif(callable, function, &[], done, &[matches.into()]);
                                    b.switch_to_block(function);
                                    let identity =
                                        b.ins().load(types::I64, MemFlags::new(), value, 8);
                                    for index in p.callable_targets(shape)? {
                                        let equal = b.ins().icmp_imm(
                                            IntCC::Equal,
                                            identity,
                                            (index + 1) as i64,
                                        );
                                        matches = b.ins().bor(matches, equal);
                                    }
                                    b.ins().jump(done, &[matches.into()]);
                                    b.switch_to_block(done);
                                    matches = b.block_params(done)[0];
                                }
                                for index in p.cast_targets(&target_ty) {
                                    let matched = matches_type(&mut b, &p, tag, index);
                                    matches = b.ins().bor(matches, matched);
                                }
                                b.ins().select(matches, value, null_reference)
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
                            let present =
                                b.ins()
                                    .icmp_imm(IntCC::NotEqual, source_tag.unwrap_or(value), 0);
                            let invalid = b.ins().band(absent, present);
                            // Every failed checked reference cast has InvalidCast semantics,
                            // including nominal targets reached from structural views.
                            let status = b.ins().iconst(types::I32, 11);
                            return_if_detailed(&mut b, invalid, status, site.as_ref());
                        }
                        if matches!(target_ty, Ty::MixedUnion(_)) {
                            let present = b.ins().icmp_imm(IntCC::NotEqual, result, 0);
                            let reference_tag = b.ins().iconst(types::I32, 4);
                            let null_tag = b.ins().iconst(types::I32, 0);
                            stack.push(b.ins().select(present, reference_tag, null_tag));
                            stack.push(b.ins().iconst(types::I64, 0));
                        }
                        if let Ty::ValueUnion(parts) = &target_ty {
                            // All reference alternatives share the first reference bank;
                            // nominal membership is checked against the actual object.
                            let active = parts
                                .iter()
                                .position(|part| {
                                    matches!(part, Ty::Reference(_) | Ty::Interface(_))
                                })
                                .expect("reference alternative");
                            let present = b.ins().icmp_imm(IntCC::NotEqual, result, 0);
                            let tag = b.ins().iconst(types::I32, active as i64 + 1);
                            let null = b.ins().iconst(types::I32, 0);
                            stack.push(b.ins().select(present, tag, null));
                            for (index, part) in parts.iter().enumerate() {
                                if index == active {
                                    stack.push(result);
                                } else {
                                    stack.extend(
                                        lanes(&p, part).into_iter().map(|ty| zero(&mut b, ty)),
                                    );
                                }
                            }
                        } else {
                            stack.push(result);
                        }
                    }
                    Op::ReferenceIsNull => {
                        if matches!(top(), Ty::ValueUnion(_)) {
                            stack.truncate(stack.len() - p.lanes(top()) + 1);
                        }
                        if matches!(top(), Ty::MixedUnion(_)) {
                            pop(&mut stack);
                        }
                        if matches!(top(), Ty::ScalarUnion(_) | Ty::MixedUnion(_)) {
                            pop(&mut stack);
                        }
                        let value = pop(&mut stack);
                        let is_null = b.ins().icmp_imm(IntCC::Equal, value, 0);
                        stack.push(b.ins().uextend(types::I32, is_null));
                    }
                    Op::ReferenceEqual => {
                        let mut right = pop(&mut stack);
                        let mut left = pop(&mut stack);
                        if shape[shape.len() - 1] == Ty::Literal {
                            right = text_object_view(&mut b, right);
                        }
                        if shape[shape.len() - 2] == Ty::Literal {
                            left = text_object_view(&mut b, left);
                        }
                        let equal = b.ins().icmp(IntCC::Equal, left, right);
                        stack.push(b.ins().uextend(types::I32, equal));
                    }
                    _ if crate::checked_conversion::spec(op).is_some() => {
                        let value = pop(&mut stack);
                        stack.push(crate::checked_conversion::emit(
                            &mut b,
                            op,
                            value,
                            site.as_ref(),
                        ));
                    }
                    Op::ConvertInt32 | Op::ConvertUInt32 => {
                        if matches!(top(), Ty::Size | Ty::Wide) {
                            let value = pop(&mut stack);
                            stack.push(b.ins().ireduce(types::I32, value));
                        }
                    }
                    Op::ConvertInt64
                    | Op::ConvertUInt64
                    | Op::ConvertNativeInt
                    | Op::ConvertNativeUInt => {
                        if matches!(top(), Ty::Int) {
                            let value = pop(&mut stack);
                            stack.push(
                                if matches!(op, Op::ConvertUInt64 | Op::ConvertNativeUInt) {
                                    b.ins().uextend(types::I64, value)
                                } else {
                                    b.ins().sextend(types::I64, value)
                                },
                            );
                        }
                    }
                    Op::ConvertUInt8 | Op::ConvertInt8 | Op::ConvertInt16 | Op::ConvertUInt16 => {
                        let value = pop(&mut stack);
                        let value = if matches!(top(), Ty::Size | Ty::Wide) {
                            b.ins().ireduce(types::I32, value)
                        } else {
                            value
                        };
                        let narrow = b.ins().ireduce(
                            if matches!(op, Op::ConvertUInt8 | Op::ConvertInt8) {
                                types::I8
                            } else {
                                types::I16
                            },
                            value,
                        );
                        stack.push(if matches!(op, Op::ConvertInt8 | Op::ConvertInt16) {
                            b.ins().sextend(types::I32, narrow)
                        } else {
                            b.ins().uextend(types::I32, narrow)
                        });
                    }
                    Op::Bool(v) => stack.push(b.ins().iconst(types::I32, i64::from(*v))),
                    Op::Void => stack.push(b.ins().iconst(types::I32, 0)),
                    Op::LoadMetadataToken(index) => {
                        let identity = type_tokens.len()
                            + metadata_tokens
                                .iter()
                                .position(|i| i == index)
                                .expect("collected metadata token")
                            + 1;
                        stack.push(b.ins().iconst(types::I64, identity as i64));
                    }
                    Op::LoadTypeToken(t) => {
                        let identity = type_tokens
                            .iter()
                            .position(|v| v == t)
                            .expect("collected token")
                            + 1;
                        stack.push(b.ins().iconst(types::I64, identity as i64));
                    }
                    Op::PackValue(t) => {
                        let payload = pop(&mut stack);
                        let payload = normalize(&mut b, &p, &p.ty(t)?, &[payload])[0];
                        let payload = match p.ty(t)? {
                            Ty::Literal | Ty::Wide => payload,
                            Ty::Double => b.ins().bitcast(types::I64, MemFlags::new(), payload),
                            Ty::Single => {
                                let bits = b.ins().bitcast(types::I32, MemFlags::new(), payload);
                                b.ins().uextend(types::I64, bits)
                            }
                            _ => b.ins().uextend(types::I64, payload),
                        };
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
                            let value = match p.ty(t)? {
                                Ty::Literal | Ty::Wide => payload,
                                Ty::Double => b.ins().bitcast(types::F64, MemFlags::new(), payload),
                                Ty::Single => {
                                    let bits = b.ins().ireduce(types::I32, payload);
                                    b.ins().bitcast(types::F32, MemFlags::new(), bits)
                                }
                                _ => b.ins().ireduce(types::I32, payload),
                            };
                            stack.push(normalize(&mut b, &p, &p.ty(t)?, &[value])[0]);
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
                        if details.is_some_and(|d| d.boxed_property_receivers.contains(&(i, pc))) {
                            // The validated adapter has checked exact box identity. Its
                            // Object argument stays rooted while the getter borrows payload.
                            stack.push(b.ins().iadd_imm(parameters[2], 8));
                        } else {
                            let s = if matches!(op, Op::ArgumentAddress(_)) {
                                arguments[*n]
                            } else {
                                locals[*n]
                            };
                            stack.push(b.ins().stack_addr(types::I64, s, 0));
                        }
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
                        if matches!(ty, Ty::ByteValues | Ty::CharacterValues) {
                            check_value_array_replacement(&mut b, address, value[0], site.as_ref());
                        }
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
                            lanes(&p, &ty)
                                .into_iter()
                                .map(|t| zero(&mut b, t))
                                .collect()
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
                        if matches!(p.ty(t)?, Ty::ByteValues | Ty::CharacterValues) {
                            check_value_array_replacement(&mut b, address, value[0], site.as_ref());
                        }
                        write_typed(&mut b, &p, &p.ty(t)?, address, &value);
                    }
                    Op::New(t) => {
                        let ty = p.ty(t)?;
                        // Structural elements and nominal fields are ordered snapshots.
                        if matches!(&ty, Ty::Record(owner) if input.types[*owner].fields.is_empty())
                        {
                            stack.push(b.ins().iconst(types::I32, 0));
                        }
                        let value = stack.split_off(stack.len() - p.lanes(&ty));
                        stack.extend(normalize(&mut b, &p, &ty, &value));
                    }
                    Op::NewArray(t) | Op::ReserveArray(t)
                        if crate::selection::scalar_array_element(t) =>
                    {
                        let count = pop(&mut stack);
                        let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                        let output = b.ins().stack_addr(types::I64, call_result, 0);
                        let reserved = b
                            .ins()
                            .iconst(types::I32, i64::from(matches!(op, Op::ReserveArray(_))));
                        let service =
                            module.declare_func_in_func(scalar_array_service.unwrap(), b.func);
                        let call = b.ins().call(service, &[arena, count, reserved, output]);
                        let status = b.inst_results(call)[0];
                        let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                        return_if_detailed(&mut b, failed, status, site.as_ref());
                        let array = b.ins().load(types::I64, MemFlags::new(), output, 0);
                        tag_array(
                            &mut b,
                            &p,
                            &p.ty(&neoclr::metadata::Type::ArrayRef(Box::new(t.clone())))?,
                            array,
                        );
                        stack.push(array);
                    }
                    Op::NewArray(t)
                        if matches!(
                            t,
                            neoclr::metadata::Type::Named(_) | neoclr::metadata::Type::ArrayRef(_)
                        ) =>
                    {
                        let count = pop(&mut stack);
                        let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                        let output = b.ins().stack_addr(types::I64, call_result, 0);
                        let service = module
                            .declare_func_in_func(default_reference_array_service.unwrap(), b.func);
                        let call = b.ins().call(service, &[arena, count, output]);
                        let status = b.inst_results(call)[0];
                        let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                        return_if_detailed(&mut b, failed, status, site.as_ref());
                        let array = b.ins().load(types::I64, MemFlags::new(), output, 0);
                        if let Some(index) = p.backing_for_array(
                            &p.ty(&neoclr::metadata::Type::ArrayRef(Box::new(t.clone())))?,
                        ) {
                            let tag = b.ins().iconst(
                                types::I64,
                                (((index as u64 + 1) << 32) | 0x80000005) as i64,
                            );
                            b.ins().store(MemFlags::new(), tag, array, 0);
                        }
                        stack.push(array);
                    }
                    Op::ReserveArray(t)
                        if matches!(
                            t,
                            neoclr::metadata::Type::Named(_)
                                | neoclr::metadata::Type::Tuple(_)
                                | neoclr::metadata::Type::Union(_)
                                | neoclr::metadata::Type::Intersection(_)
                                | neoclr::metadata::Type::ArrayRef(_)
                        ) =>
                    {
                        let count = pop(&mut stack);
                        let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                        let output = b.ins().stack_addr(types::I64, call_result, 0);
                        let width = b.ins().iconst(types::I32, p.lanes(&p.ty(t)?) as i64);
                        let service =
                            module.declare_func_in_func(record_array_service.unwrap(), b.func);
                        let call = b.ins().call(service, &[arena, count, width, output]);
                        let status = b.inst_results(call)[0];
                        let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                        return_if_detailed(&mut b, failed, status, site.as_ref());
                        let array = b.ins().load(types::I64, MemFlags::new(), output, 0);
                        if let Some(index) = p.backing_for_array(
                            &p.ty(&neoclr::metadata::Type::ArrayRef(Box::new(t.clone())))?,
                        ) {
                            let tag = b.ins().iconst(
                                types::I64,
                                (((index as u64 + 1) << 32) | 0x80000005) as i64,
                            );
                            b.ins().store(MemFlags::new(), tag, array, 0);
                        }
                        stack.push(array);
                    }
                    Op::NewArray(t) | Op::ReserveArray(t)
                        if matches!(
                            t,
                            neoclr::metadata::Type::String
                                | neoclr::metadata::Type::Char
                                | neoclr::metadata::Type::Function(_)
                        ) =>
                    {
                        let count = pop(&mut stack);
                        let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                        let output = b.ins().stack_addr(types::I64, call_result, 0);
                        let reserved = b
                            .ins()
                            .iconst(types::I32, i64::from(matches!(op, Op::ReserveArray(_))));
                        let service =
                            module.declare_func_in_func(reference_array_service.unwrap(), b.func);
                        let call = b.ins().call(service, &[arena, count, reserved, output]);
                        let status = b.inst_results(call)[0];
                        let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                        return_if_detailed(&mut b, failed, status, site.as_ref());
                        let array = b.ins().load(types::I64, MemFlags::new(), output, 0);
                        tag_array(
                            &mut b,
                            &p,
                            &p.ty(&neoclr::metadata::Type::ArrayRef(Box::new(t.clone())))?,
                            array,
                        );
                        if matches!(op, Op::NewArray(neoclr::metadata::Type::Char)) {
                            let data = module.declare_data_in_func(default_character, b.func);
                            let character = b.ins().global_value(types::I64, data);
                            let loop_block = b.create_block();
                            b.append_block_param(loop_block, types::I32);
                            let body = b.create_block();
                            let done = b.create_block();
                            let zero = b.ins().iconst(types::I32, 0);
                            b.ins().jump(loop_block, &[zero.into()]);
                            b.switch_to_block(loop_block);
                            let index = b.block_params(loop_block)[0];
                            let more = b.ins().icmp(IntCC::SignedLessThan, index, count);
                            b.ins().brif(more, body, &[], done, &[]);
                            b.switch_to_block(body);
                            let wide = b.ins().uextend(types::I64, index);
                            let offset = b.ins().imul_imm(wide, 8);
                            let slot = b.ins().iadd(array, offset);
                            b.ins().store(MemFlags::new(), character, slot, 16);
                            let next = b.ins().iadd_imm(index, 1);
                            b.ins().jump(loop_block, &[next.into()]);
                            b.switch_to_block(done);
                        }
                        stack.push(array);
                    }
                    Op::NewArray(neoclr::metadata::Type::Byte)
                    | Op::ReserveArray(neoclr::metadata::Type::Byte) => {
                        let count = pop(&mut stack);
                        let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                        let output = b.ins().stack_addr(types::I64, call_result, 0);
                        let service = module.declare_func_in_func(
                            if matches!(op, Op::ReserveArray(_)) {
                                reserve_service.unwrap()
                            } else {
                                array_service.unwrap()
                            },
                            b.func,
                        );
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
                    Op::ArrayElement(t) | Op::StoreArrayElement(t)
                        if matches!(
                            t,
                            neoclr::metadata::Type::Named(_)
                                | neoclr::metadata::Type::Tuple(_)
                                | neoclr::metadata::Type::Union(_)
                                | neoclr::metadata::Type::Intersection(_)
                                | neoclr::metadata::Type::ArrayRef(_)
                        ) || crate::selection::scalar_array_element(t) =>
                    {
                        let ty = p.ty(t)?;
                        let width = p.lanes(&ty);
                        let value = if matches!(op, Op::StoreArrayElement(_)) {
                            Some(stack.split_off(stack.len() - width))
                        } else {
                            None
                        };
                        let index = pop(&mut stack);
                        let array = pop(&mut stack);
                        null_reference(&mut b, array, site.as_ref());
                        let length = b.ins().load(types::I32, MemFlags::new(), array, 8);
                        let outside =
                            b.ins()
                                .icmp(IntCC::UnsignedGreaterThanOrEqual, index, length);
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
                            if live_stores && !crate::selection::scalar_array_element(t) {
                                let snapshot = slot(&mut b, (width * 8) as u32);
                                let source = b.ins().stack_addr(types::I64, snapshot, 0);
                                // Narrow lanes must not copy uninitialized stack padding.
                                let zero = b.ins().iconst(types::I64, 0);
                                for lane in 0..width {
                                    b.ins().stack_store(zero, snapshot, (lane * 8) as i32);
                                }
                                write_typed(&mut b, &p, &ty, source, &value);
                                let mut signature = module.make_signature();
                                signature.params.extend(
                                    [types::I64, types::I64, types::I32, types::I64, types::I32]
                                        .map(AbiParam::new),
                                );
                                signature.returns.push(AbiParam::new(types::I32));
                                let helper = module.declare_function(
                                    "experiment_gc_store_record", Linkage::Import, &signature,
                                )?;
                                let helper = module.declare_func_in_func(helper, b.func);
                                let ordinal = b.ins().ireduce(types::I32, index);
                                let lanes = b.ins().iconst(types::I32, width as i64);
                                let call = b.ins().call(helper, &[
                                    fault_context.unwrap(), array, ordinal, source, lanes,
                                ]);
                                let status = b.inst_results(call)[0];
                                let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                                return_if_detailed(&mut b, failed, status, site.as_ref());
                            } else {
                                write_typed(&mut b, &p, &ty, address, &value);
                                let initialized = b.ins().iconst(types::I8, 1);
                                b.ins().store(MemFlags::new(), initialized, marker, 0);
                            }
                        } else {
                            let initialized = b.ins().load(types::I8, MemFlags::new(), marker, 0);
                            let unreadable = b.ins().icmp_imm(IntCC::Equal, initialized, 0);
                            let status = b.ins().iconst(types::I32, 3);
                            return_if_detailed(&mut b, unreadable, status, site.as_ref());
                            stack.extend(read(&mut b, &p, &ty, address));
                        }
                    }
                    Op::ArrayElement(t) | Op::StoreArrayElement(t) | Op::ArrayAddress(t) => {
                        let pointers = matches!(
                            t,
                            neoclr::metadata::Type::String
                                | neoclr::metadata::Type::Char
                                | neoclr::metadata::Type::Function(_)
                        );
                        let value = if matches!(op, Op::StoreArrayElement(_)) {
                            Some(pop(&mut stack))
                        } else {
                            None
                        };
                        let index = pop(&mut stack);
                        let array = pop(&mut stack);
                        null_reference(&mut b, array, site.as_ref());
                        let length = b.ins().load(types::I32, MemFlags::new(), array, 8);
                        let outside =
                            b.ins()
                                .icmp(IntCC::UnsignedGreaterThanOrEqual, index, length);
                        let status = b.ins().iconst(types::I32, 8);
                        return_if_detailed(&mut b, outside, status, site.as_ref());
                        let offset = b.ins().uextend(types::I64, index);
                        let offset = if pointers {
                            b.ins().ishl_imm(offset, 3)
                        } else {
                            offset
                        };
                        let data = b.ins().iadd_imm(array, 16);
                        let address = b.ins().iadd(data, offset);
                        if live_stores && pointers && value.is_some() {
                            let mut signature = module.make_signature();
                            signature.params.extend(
                                [types::I64, types::I64, types::I32, types::I64].map(AbiParam::new),
                            );
                            signature.returns.push(AbiParam::new(types::I32));
                            let helper = module.declare_function(
                                "experiment_gc_store_text", Linkage::Import, &signature,
                            )?;
                            let helper = module.declare_func_in_func(helper, b.func);
                            let call = b.ins().call(helper, &[
                                fault_context.unwrap(), array, index, value.unwrap(),
                            ]);
                            let status = b.inst_results(call)[0];
                            let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                            return_if_detailed(&mut b, failed, status, site.as_ref());
                        } else {
                            if reservations {
                                let kind = b.ins().load(types::I64, MemFlags::new(), array, 0);
                                let kind = b.ins().band_imm(kind, 0xffffffff);
                                let reserved = b.ins().icmp_imm(
                                    IntCC::Equal,
                                    kind,
                                    if pointers { 0x80000004 } else { 0x80000002 },
                                );
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
                                } else {
                                    b.ins().iadd(address, length)
                                };
                                if value.is_some() {
                                    let initialized = b.ins().iconst(types::I8, 1);
                                    b.ins().store(MemFlags::new(), initialized, marker, 0);
                                } else {
                                    let initialized =
                                        b.ins().load(types::I8, MemFlags::new(), marker, 0);
                                    let unreadable = b.ins().icmp_imm(IntCC::Equal, initialized, 0);
                                    let status = b.ins().iconst(types::I32, 3);
                                    return_if_detailed(&mut b, unreadable, status, site.as_ref());
                                }
                                b.ins().jump(ready, &[]);
                                b.switch_to_block(ready);
                            }
                            if let Some(value) = value {
                                let value = if pointers {
                                    value
                                } else {
                                    b.ins().ireduce(types::I8, value)
                                };
                                b.ins().store(MemFlags::new(), value, address, 0);
                            } else if matches!(op, Op::ArrayAddress(_)) {
                                stack.push(address);
                            } else {
                                let value = b.ins().load(
                                    if pointers { types::I64 } else { types::I8 },
                                    MemFlags::new(),
                                    address,
                                    0,
                                );
                                stack.push(if pointers {
                                    value
                                } else {
                                    b.ins().uextend(types::I32, value)
                                });
                            }
                        }
                    }
                    Op::StaticField { owner, index } | Op::SetStaticField { owner, index } => {
                        let (owner_index, ty, offset, bytes) = p.static_field(owner, *index)?;
                        // Dense static-owner ordinals avoid retaining/scanning empty
                        // slots for every unrelated selected type on each collection.
                        let type_index = input.types[..owner_index]
                            .iter()
                            .filter(|t| crate::type_initialization::needs_storage(input, t))
                            .count();
                        let static_count = input
                            .types
                            .iter()
                            .filter(|t| crate::type_initialization::needs_storage(input, t))
                            .count();
                        // Warm static access neither allocates nor calls managed code.
                        // Publish the pending operands only on the cold allocation path.
                        let cold = b.create_block();
                        let lookup = b.create_block();
                        let ready = b.create_block();
                        b.append_block_param(ready, types::I64);
                        let table =
                            b.ins()
                                .load(types::I64, MemFlags::new(), fault_context.unwrap(), 1072);
                        let missing = b.ins().icmp_imm(IntCC::Equal, table, 0);
                        b.ins().brif(missing, cold, &[], lookup, &[]);
                        b.switch_to_block(lookup);
                        let cached = b.ins().load(
                            types::I64,
                            MemFlags::new(),
                            table,
                            ((type_index + 1) * 8) as i32,
                        );
                        let missing = b.ins().icmp_imm(IntCC::Equal, cached, 0);
                        b.ins().brif(missing, cold, &[], ready, &[cached.into()]);
                        b.switch_to_block(cold);
                        if let (Some(probes), Some(spill)) = (&root_probes, root_spill) {
                            probes.emit(module, &mut b, i, pc, spill, probe_frame.unwrap(), &stack);
                        }
                        let output_slot = slot(&mut b, 8);
                        let output = b.ins().stack_addr(types::I64, output_slot, 0);
                        let service = module.declare_func_in_func(static_service.unwrap(), b.func);
                        let type_id = b.ins().iconst(types::I32, type_index as i64);
                        let count = b.ins().iconst(types::I32, static_count as i64);
                        let bytes = b.ins().iconst(types::I32, bytes as i64);
                        let call = b.ins().call(
                            service,
                            &[fault_context.unwrap(), type_id, count, bytes, output],
                        );
                        let status = b.inst_results(call)[0];
                        let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                        return_if_detailed(&mut b, failed, status, site.as_ref());
                        let storage = b.ins().load(types::I64, MemFlags::new(), output, 0);
                        b.ins().jump(ready, &[storage.into()]);
                        b.switch_to_block(ready);
                        let storage = b.block_params(ready)[0];
                        let address = b.ins().iadd_imm(storage, offset as i64);
                        if matches!(op, Op::SetStaticField { .. }) {
                            let value = stack.split_off(stack.len() - p.lanes(top()));
                            let value = normalize(&mut b, &p, &ty, &value);
                            write_typed(&mut b, &p, &ty, address, &value);
                        } else {
                            stack.extend(read(&mut b, &p, &ty, address));
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
                            } else {
                                0
                            };
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
                            } else {
                                0
                            };
                            let address = b.ins().iadd_imm(owner, (header + offset * 8) as i64);
                            write_typed(
                                &mut b,
                                &p,
                                &p.field(&shape[shape.len() - 2], *n)?,
                                address,
                                &value,
                            );
                            if !is_reference {
                                stack.push(b.ins().iconst(types::I32, 0));
                            } // borrowed value store returns Void
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
                        } else {
                            b.ins().iconst(types::I64, 0)
                        };
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
                    Op::Call(target) | Op::CallVirtual(target)
                        if crate::selection::callable_invoke(target).is_some() =>
                    {
                        let signature = crate::selection::callable_invoke(target).unwrap();
                        let owner = target.owner.as_ref().unwrap();
                        let count = signature
                            .parameters
                            .iter()
                            .map(|t| p.ty(t).map(|t| p.lanes(&t)))
                            .collect::<Result<Vec<_>, _>>()?
                            .into_iter()
                            .sum::<usize>();
                        let actual = stack.split_off(stack.len() - count);
                        let descriptor = pop(&mut stack);
                        null_reference(&mut b, descriptor, site.as_ref());
                        let identity = b.ins().load(types::I64, MemFlags::new(), descriptor, 8);
                        let result = b.ins().stack_addr(types::I64, call_result, 0);
                        let result_type = if signature.no_result {
                            None
                        } else {
                            Some(p.ty(&signature.returns)?)
                        };
                        if let (Some(probes), Some(t)) = (&root_probes, &result_type) {
                            let zero = b.ins().iconst(types::I64, 0);
                            for lane in gc_layout::seed_lanes(&p, t) {
                                b.ins()
                                    .store(MemFlags::new(), zero, result, (lane * 8) as i32);
                            }
                            probes.publish(
                                module,
                                &mut b,
                                i,
                                pc,
                                3,
                                probe_frame.unwrap(),
                                transient_table.unwrap(),
                                result,
                            );
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
                                args.push(b.ins().load(
                                    types::I64,
                                    MemFlags::new(),
                                    descriptor,
                                    16,
                                ));
                            }
                            args.extend_from_slice(&actual);
                            args.push(result);
                            if let Some(context) = fault_context {
                                args.push(context);
                            }
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
                                probes.publish(
                                    module,
                                    &mut b,
                                    i,
                                    pc,
                                    2,
                                    probe_frame.unwrap(),
                                    transient_table.unwrap(),
                                    result,
                                );
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
                                let service =
                                    module.declare_func_in_func(object_service.unwrap(), b.func);
                                let arena = b.ins().iadd_imm(fault_context.unwrap(), 1048);
                                let type_id = b.ins().iconst(types::I32, index as i64);
                                let bytes =
                                    b.ins().iconst(types::I32, p.object_bytes(index) as i64);
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
                                let zeros: Vec<_> =
                                    lanes(&p, &t).into_iter().map(|t| zero(&mut b, t)).collect();
                                write(&mut b, address, &zeros);
                                address
                            };
                            if let Some(probes) = &root_probes {
                                probes.publish(
                                    module,
                                    &mut b,
                                    i,
                                    pc,
                                    1,
                                    probe_frame.unwrap(),
                                    transient_table.unwrap(),
                                    address,
                                );
                            }
                            call_args.insert(0, receiver);
                            Some((t, address))
                        } else {
                            None
                        };
                        if matches!(op, Op::CallVirtual(_))
                            || details.is_some_and(|d| d.string_virtual_calls.contains(&(i, pc)))
                        {
                            // Unlike direct call, callvirt checks null before entering
                            // even a nonvirtual target; capture the caller's site.
                            null_reference(&mut b, call_args[0], site.as_ref());
                        }
                        let result = b.ins().stack_addr(types::I64, call_result, 0);
                        if !construct && let (Some(probes), Some(t)) = (&root_probes, &p.results[c])
                        {
                            // Seed only traceable scratch lanes; this does not publish
                            // a guest result. The caller retains their addresses while
                            // the callee writes and removes its diagnostic frame.
                            let zero = b.ins().iconst(types::I64, 0);
                            for lane in gc_layout::seed_lanes(&p, t) {
                                b.ins()
                                    .store(MemFlags::new(), zero, result, (lane * 8) as i32);
                            }
                            probes.publish(
                                module,
                                &mut b,
                                i,
                                pc,
                                3,
                                probe_frame.unwrap(),
                                transient_table.unwrap(),
                                result,
                            );
                        }
                        call_args.push(result);
                        if let Some(context) = fault_context {
                            call_args.push(context);
                        }
                        let target = module.declare_func_in_func(ids[c], b.func);
                        let call = b.ins().call(target, &call_args);
                        let status = b.inst_results(call)[0];
                        let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                        return_if_detailed(&mut b, failed, status, site.as_ref());
                        if let Some((t, address)) = constructed {
                            if let Some(probes) = &root_probes {
                                probes.publish(
                                    module,
                                    &mut b,
                                    i,
                                    pc,
                                    2,
                                    probe_frame.unwrap(),
                                    transient_table.unwrap(),
                                    address,
                                );
                            }
                            stack.extend(read(&mut b, &p, &t, address));
                        } else if let Some(t) = &p.results[c] {
                            if let Some(probes) = &root_probes {
                                probes.publish(
                                    module,
                                    &mut b,
                                    i,
                                    pc,
                                    2,
                                    probe_frame.unwrap(),
                                    transient_table.unwrap(),
                                    result,
                                );
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
                        let descriptor_query =
                            details.is_some_and(|d| d.descriptor_queries.contains(&i));
                        if descriptor_query {
                            // InternalCall errors belong to the managed call site.
                            if let Some(site) = site.as_mut() {
                                site.capture_frame = false;
                            }
                        }
                        let status = b
                            .ins()
                            .iconst(types::I32, if descriptor_query { 3 } else { 4 });
                        if let Some(site) = &site {
                            site.record(&mut b, status);
                        }
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
                    Op::Negate => {
                        let value = pop(&mut stack);
                        stack.push(if matches!(top(), Ty::Single | Ty::Double) {
                            b.ins().fneg(value)
                        } else {
                            b.ins().ineg(value)
                        });
                    }
                    Op::BitNot => {
                        let value = pop(&mut stack);
                        stack.push(b.ins().bnot(value));
                    }
                    Op::ShiftLeft | Op::ShiftRight | Op::ShiftRightUnsigned => {
                        let count = pop(&mut stack);
                        let value = pop(&mut stack);
                        let mask = b.func.dfg.value_type(value).bits() - 1;
                        let count = b.ins().band_imm(count, i64::from(mask));
                        stack.push(match op {
                            Op::ShiftLeft => b.ins().ishl(value, count),
                            Op::ShiftRight => b.ins().sshr(value, count),
                            _ => b.ins().ushr(value, count),
                        });
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
                    Op::Add | Op::Sub | Op::Mul | Op::Divide
                        if matches!(top(), Ty::Single | Ty::Double) =>
                    {
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
                        let bool8 = if matches!(top(), Ty::Single | Ty::Double) {
                            float_compare(&mut b, cc, l, r)
                        } else {
                            b.ins().icmp(cc, l, r)
                        };
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
                        let condition = if matches!(top(), Ty::Single | Ty::Double) {
                            float_compare(&mut b, cc, l, r)
                        } else {
                            b.ins().icmp(cc, l, r)
                        };
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
        if live_stores {
            let entry = context.func.layout.entry_block().unwrap();
            let owner = *context.func.dfg.block_params(entry).last().unwrap();
            crate::mutation_store::lower_framed(
                module,
                &mut context.func,
                owner,
                mutation_frame_ready.ok_or("missing live-store frame initialization")?,
            )?;
        }
        if let (Some(probes), Some(frame)) = (&root_probes, probe_frame) {
            probes.finish(module, &mut context.func, frame);
        }
        if live_stores {
            live_bodies.push((ids[i], context));
        } else {
            define_checked(module, ids[i], &mut context, stack_budget, long_range_calls)?;
        }
    }
    if live_stores {
        if live_bodies.len() != ids.len() + initialization_guards.len() {
            return Err("live stores reject unqualified replacement emission".into());
        }
        let bodies: Vec<_> = live_bodies.iter().map(|(id, ctx)| (*id, &ctx.func)).collect();
        crate::mutation_admission::check_call_closure(
            module, &bodies, &crate::mutation_admission::root_providers(module),
        )?;
        for (id, mut context) in live_bodies {
            define_checked(module, id, &mut context, stack_budget, long_range_calls)?;
        }
    }
    let entry_check = if details.is_some_and(|d| d.native_gc) {
        let mut signature = module.make_signature();
        signature.params.push(AbiParam::new(types::I64));
        signature.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function("neoclr_gc_entry_check_v1", Linkage::Import, &signature)?)
    } else {
        None
    };
    let argument_service = if entry_arguments {
        let mut signature = module.make_signature();
        signature.params = [types::I32, types::I64, types::I64, types::I64]
            .map(AbiParam::new)
            .to_vec();
        signature.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function(
            if details.is_some_and(|d| d.native_gc) {
                "neoclr_process_arguments_rooted_v1"
            } else {
                "neoclr_process_arguments_v1"
            },
            Linkage::Import,
            &signature,
        )?)
    } else {
        None
    };
    let complete_invocation = if details.is_some_and(|d| d.native_gc) && !bootstrap {
        let mut signature = module.make_signature();
        signature.params.push(AbiParam::new(types::I64));
        signature.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function(
            "neoclr_gc_complete_invocation_v1",
            Linkage::Import,
            &signature,
        )?)
    } else {
        None
    };
    let export_root = if bootstrap {
        let mut signature = module.make_signature();
        signature.params = vec![AbiParam::new(types::I64); 3];
        signature.returns.push(AbiParam::new(types::I32));
        Some(module.declare_function(
            "neoclr_gc_host_root_create_v1",
            Linkage::Import,
            &signature,
        )?)
    } else {
        None
    };
    // Only the stable experiment C entry is exported. Private aggregate signatures
    // are intentionally not a public ARM64 struct ABI.
    let mut context = module.make_context();
    context.func.signature.params = vec![AbiParam::new(types::I32), AbiParam::new(types::I64)];
    if details.is_some() {
        context
            .func
            .signature
            .params
            .push(AbiParam::new(types::I64));
    }
    if entry_arguments {
        context
            .func
            .signature
            .params
            .push(AbiParam::new(types::I64));
    }
    context
        .func
        .signature
        .returns
        .push(AbiParam::new(types::I32));
    let export = module.declare_function(
        if entry_arguments {
            "neoclr_entry_args_v1"
        } else if bootstrap {
            "neoclr_bootstrap_callback_v1"
        } else if text_arena {
            "neoclr_entry_v4"
        } else if details.is_some() {
            "neoclr_entry_v3"
        } else {
            "neoclr_entry_v2"
        },
        Linkage::Export,
        &context.func.signature,
    )?;
    let target = module.declare_func_in_func(ids[root], &mut context.func);
    let mut fb = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut context.func, &mut fb);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let params = b.block_params(entry).to_vec();
        if bootstrap {
            let rejected = b.create_block();
            let admitted = b.create_block();
            let null_output = b.ins().icmp_imm(IntCC::Equal, params[1], 0);
            let null_context = b.ins().icmp_imm(IntCC::Equal, params[2], 0);
            let invalid = b.ins().bor(null_output, null_context);
            b.ins().brif(invalid, rejected, &[], admitted, &[]);
            b.switch_to_block(rejected);
            let status = b.ins().iconst(types::I32, 3);
            b.ins().return_(&[status]);
            b.switch_to_block(admitted);
        }
        if details.is_some() && !bootstrap {
            crate::fault_details::reset(&mut b, params[2]);
        }
        if let Some(check) = entry_check {
            let check = module.declare_func_in_func(check, b.func);
            let call = b.ins().call(check, &[params[2]]);
            let status = b.inst_results(call)[0];
            let rejected = b.create_block();
            let admitted = b.create_block();
            let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
            b.ins().brif(failed, rejected, &[], admitted, &[]);
            b.switch_to_block(rejected);
            if !bootstrap {
                b.ins().store(MemFlags::new(), status, params[2], 0);
            }
            b.ins().return_(&[status]);
            b.switch_to_block(admitted);
        }
        if bootstrap {
            crate::fault_details::reset(&mut b, params[2]);
        }
        if stack_budget {
            emit_stack_check(
                module,
                &mut b,
                stack_check,
                diagnostic_data.as_ref(),
                params[2],
                root,
                false,
            );
        }
        if text_arena {
            // Reset only the cursor; capacity/data belong to the host. Previous
            // dynamic text, including fault messages, expires on this next call.
            let zero = b.ins().iconst(types::I64, 0);
            b.ins().store(MemFlags::new(), zero, params[2], 1064);
            b.ins().store(MemFlags::new(), zero, params[2], 1072);
        }
        let mut call_args = vec![];
        if let Some(service) = argument_service {
            // ABI: argc, result, context, argv. Construct after the admitted heap
            // reset. Allocation cannot collect; the callee publishes argument roots.
            let storage = slot(&mut b, 8);
            let output = b.ins().stack_addr(types::I64, storage, 0);
            let service = module.declare_func_in_func(service, b.func);
            let call = b
                .ins()
                .call(service, &[params[0], params[3], params[2], output]);
            let status = b.inst_results(call)[0];
            let ready = b.create_block();
            let failed = b.create_block();
            let ok = b.ins().icmp_imm(IntCC::Equal, status, 0);
            b.ins().brif(ok, ready, &[], failed, &[]);
            b.switch_to_block(failed);
            b.ins().store(MemFlags::new(), status, params[2], 0);
            b.ins().return_(&[status]);
            b.switch_to_block(ready);
            call_args.push(b.ins().load(types::I64, MemFlags::new(), output, 0));
        } else if !p.args[root].is_empty() {
            call_args.push(params[0]);
        }
        let unit_result = p.results[root] != Some(Ty::Int);
        let result = if unit_result {
            let storage = slot(&mut b, 8);
            b.ins().stack_addr(types::I64, storage, 0)
        } else {
            params[1]
        };
        call_args.push(result);
        if details.is_some() {
            call_args.push(params[2]);
        }
        let call = b.ins().call(target, &call_args);
        let status = b.inst_results(call)[0];
        if bootstrap {
            let success = b.create_block();
            let failed = b.create_block();
            let ok = b.ins().icmp_imm(IntCC::Equal, status, 0);
            b.ins().brif(ok, success, &[], failed, &[]);
            b.switch_to_block(failed);
            b.ins().return_(&[status]);
            b.switch_to_block(success);
            // Guest frames have unwound. Creating a host root does not collect;
            // ownership is published atomically before control returns to C.
            let callback = b.ins().load(types::I64, MemFlags::new(), result, 0);
            let nonnull = b.create_block();
            let invalid = b.create_block();
            let is_null = b.ins().icmp_imm(IntCC::Equal, callback, 0);
            b.ins().brif(is_null, invalid, &[], nonnull, &[]);
            b.switch_to_block(invalid);
            let rejected = b.ins().iconst(types::I32, 3);
            b.ins().return_(&[rejected]);
            b.switch_to_block(nonnull);
            let create = module.declare_func_in_func(export_root.unwrap(), b.func);
            let call = b.ins().call(create, &[params[2], callback, params[1]]);
            let status = b.inst_results(call)[0];
            b.ins().return_(&[status]);
        } else {
            let status = if let Some(complete) = complete_invocation {
                let complete = module.declare_func_in_func(complete, b.func);
                let call = b.ins().call(complete, &[params[2]]);
                let completed = b.inst_results(call)[0];
                let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
                b.ins().select(failed, status, completed)
            } else {
                if text_arena {
                    let zero = b.ins().iconst(types::I64, 0);
                    b.ins().store(MemFlags::new(), zero, params[2], 1072);
                }
                status
            };
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
        }
        b.seal_all_blocks();
        b.finalize();
    }
    define_checked(module, export, &mut context, stack_budget, long_range_calls)?;
    // Entry's prologue runs before its first budget check; keep it below one
    // page. Hosts still owe a usable stack to call the ABI at all.
    if platform_target == super::Target::WindowsX64 && frame_charge(&context, true)? > 4096 {
        return Err("Windows stack experiment entry frame exceeds one page".into());
    }
    if details.is_some_and(|d| d.native_gc) {
        compile_host_callbacks(
            module,
            &p,
            &ids,
            stack_check,
            diagnostic_data.as_ref(),
            false,
            long_range_calls,
        )?;
        if details.is_some_and(|d| !d.entry_task_drain.is_empty()) {
            compile_host_callbacks(
                module,
                &p,
                &ids,
                stack_check,
                diagnostic_data.as_ref(),
                true,
                long_range_calls,
            )?;
        }
        if let Some(drain) = details.and_then(|d| d.task_queue_drain) {
            compile_host_queue(
                module,
                ids[drain],
                drain,
                stack_check,
                diagnostic_data.as_ref().unwrap(),
                long_range_calls,
            )?;
        }
    }
    Ok(())
}

// Quiescent host entry for inhabited-Void and no-result unit callback shapes. The strong
// handle keeps the descriptor/receiver alive across the callee's collection points.
fn compile_host_callbacks(
    module: &mut impl Module,
    p: &Profile<'_>,
    ids: &[cranelift_module::FuncId],
    stack_check: Option<cranelift_module::FuncId>,
    diagnostic_data: Option<&crate::fault_details::Data>,
    entry_pump: bool,
    long_range_calls: bool,
) -> Result<(), Error> {
    use neoclr::metadata::{FunctionType, Type};
    let shape = Type::Function(Box::new(FunctionType {
        parameters: vec![],
        returns: Type::Void,
        no_result: false,
        out_parameters: vec![],
        out_when_true: vec![],
    }));
    let mut targets = p.callable_targets(&shape)?;
    let no_result = Type::Function(Box::new(FunctionType {
        parameters: vec![],
        returns: Type::Void,
        no_result: true,
        out_parameters: vec![],
        out_when_true: vec![],
    }));
    for target in p.callable_targets(&no_result)? {
        if !targets.contains(&target) {
            targets.push(target);
        }
    }
    // Callable targets are bounded by selected functions. Their union
    // needs no separate cap: this dispatcher emits branches, not a fixed-size
    // host table. Mixed unit/no-result continuations can exceed either shape.
    // Hosts link this ABI even when no guest callback is reachable. Keep the
    // rejecting dispatcher so synchronous task-only programs can link as well.
    let mut signature = module.make_signature();
    signature
        .params
        .extend([types::I64, types::I64, types::I64].map(AbiParam::new));
    signature.returns.push(AbiParam::new(types::I32));
    let reader = module.declare_function(
        if entry_pump {
            "neoclr_entry_callback_read_v1"
        } else {
            "neoclr_gc_callback_read_v1"
        },
        Linkage::Import,
        &signature,
    )?;
    signature.params.pop();
    let export = module.declare_function(
        if entry_pump {
            "neoclr_invoke_entry_callback_v1"
        } else {
            "neoclr_invoke_void_callback_v1"
        },
        Linkage::Export,
        &signature,
    )?;
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
        b.ins()
            .brif(failed, rejected, &[status.into()], admitted, &[]);
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
            emit_stack_check(
                module,
                &mut b,
                stack_check,
                diagnostic_data,
                params[1],
                c,
                false,
            );
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
    define_checked(
        module,
        export,
        &mut context,
        stack_check.is_some(),
        long_range_calls,
    )?;
    Ok(())
}

// Guard after publishing the current frame, before executing guest operations.
// All returns use the existing frame unlink instrumentation, including this fault.
fn emit_stack_check(
    module: &mut impl Module,
    b: &mut FunctionBuilder<'_>,
    check: Option<cranelift_module::FuncId>,
    data: Option<&crate::fault_details::Data>,
    context: ir::Value,
    function: usize,
    capture_frame: bool,
) {
    let Some(check) = check else {
        return;
    };
    let check = module.declare_func_in_func(check, b.func);
    let call = b.ins().call(check, &[]);
    let status = b.inst_results(call)[0];
    let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
    let mut site = data.unwrap().site(module, b, context, function, 0);
    site.message = None;
    site.capture_frame = capture_frame;
    return_if_detailed(b, failed, status, Some(&site));
}
fn define_checked(
    module: &mut impl Module,
    id: cranelift_module::FuncId,
    context: &mut cranelift_codegen::Context,
    stack_budget: bool,
    long_range_calls: bool,
) -> Result<(), Error> {
    super::prepare_calls(context, long_range_calls);
    module.define_function(id, context)?;
    if stack_budget {
        let windows =
            module.isa().default_call_conv() == cranelift_codegen::isa::CallConv::WindowsFastcall;
        let bytes = frame_charge(context, windows)?;
        if bytes > 65536 {
            return Err("native stack budget requires final machine frames including call arguments at most 64 KiB".into());
        }
    }
    Ok(())
}

fn frame_charge(context: &cranelift_codegen::Context, windows: bool) -> Result<u64, Error> {
    // Cranelift 0.121.2 frame_size excludes linkage and ephemeral outgoing args.
    // Charge every argument, including register args, plus Windows shadow space.
    // Both targets require 16-byte alignment; this intentionally overestimates.
    let outgoing = context
        .func
        .dfg
        .signatures
        .values()
        .map(|sig| sig.params.len() as u64 * 8)
        .max()
        .unwrap_or(0);
    Ok(u64::from(
        context
            .compiled_code()
            .ok_or("missing final machine frame")?
            .frame_size,
    ) + 16
        + ((outgoing + 15) & !15)
        + if windows { 32 } else { 0 })
}

fn compile_host_queue(
    module: &mut impl Module,
    target: cranelift_module::FuncId,
    drain: usize,
    stack_check: Option<cranelift_module::FuncId>,
    data: &crate::fault_details::Data,
    long_range_calls: bool,
) -> Result<(), Error> {
    let mut sig = module.make_signature();
    sig.params
        .extend([types::I64, types::I64].map(AbiParam::new));
    sig.returns.push(AbiParam::new(types::I32));
    let reader =
        module.declare_function("neoclr_task_queue_host_read_v1", Linkage::Import, &sig)?;
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
        b.seal_all_blocks();
        b.finalize();
    }
    define_checked(
        module,
        export,
        &mut context,
        stack_check.is_some(),
        long_range_calls,
    )
}

#[cfg(test)]
mod windows_frame_tests {
    use super::*;

    #[test]
    fn windows_final_frame_limit_includes_machine_linkage_and_call_space() {
        for (size, accepted) in [(64000, true), (65536, false)] {
            let mut flags = settings::builder();
            flags.set("enable_probestack", "true").unwrap();
            flags.set("probestack_strategy", "inline").unwrap();
            let isa = isa::lookup("x86_64-pc-windows-msvc".parse().unwrap())
                .unwrap()
                .finish(settings::Flags::new(flags))
                .unwrap();
            let mut module = ObjectModule::new(
                ObjectBuilder::new(
                    isa,
                    "frame_boundary",
                    cranelift_module::default_libcall_names(),
                )
                .unwrap(),
            );
            let mut context = module.make_context();
            context
                .func
                .signature
                .returns
                .push(AbiParam::new(types::I32));
            let id = module
                .declare_function("boundary", Linkage::Export, &context.func.signature)
                .unwrap();
            let mut fb = FunctionBuilderContext::new();
            {
                let mut b = FunctionBuilder::new(&mut context.func, &mut fb);
                let entry = b.create_block();
                b.switch_to_block(entry);
                let storage = slot(&mut b, size);
                let value = b.ins().iconst(types::I32, 42);
                b.ins().stack_store(value, storage, 0);
                let value = b.ins().stack_load(types::I32, storage, 0);
                b.ins().return_(&[value]);
                b.seal_all_blocks();
                b.finalize();
            }
            let result = define_checked(&mut module, id, &mut context, true, false);
            if accepted {
                result.unwrap();
            } else {
                assert!(
                    result
                        .unwrap_err()
                        .to_string()
                        .contains("final machine frames")
                );
            }
        }
    }
}

#[cfg(all(test, target_os = "macos", target_arch = "aarch64"))]
mod structural_jit_tests {
    use super::*;
    use cranelift_jit::{JITBuilder, JITModule};
    use cranelift_module::FuncOrDataId;
    use std::time::Instant;

    struct OwnedCode(Option<JITModule>);
    impl Drop for OwnedCode {
        fn drop(&mut self) {
            if let Some(module) = self.0.take() {
                // SAFETY: this test invokes code synchronously; no address escapes.
                unsafe { module.free_memory() };
            }
        }
    }

    #[test]
    #[ignore = "explicit native performance qualification; runs millions of calls"]
    fn structural_tuple_jit_costs_match_nominal_records() {
        type Entry = unsafe extern "C" fn(i32, *mut i32) -> i32;
        for (elements, structural, nominal) in [
            (
                2,
                include_str!(
                    "../../../docs/experiments/native-structural-types/structural-pair.neoil"
                ),
                include_str!(
                    "../../../docs/experiments/native-structural-types/nominal-pair.neoil"
                ),
            ),
            (
                16,
                include_str!(
                    "../../../docs/experiments/native-structural-types/structural-16.neoil"
                ),
                include_str!("../../../docs/experiments/native-structural-types/nominal-16.neoil"),
            ),
        ] {
            let mut cases = Vec::new();
            for source in [structural, nominal] {
                let input = neoclr::assemble(source).unwrap();
                let started = Instant::now();
                let mut flags = settings::builder();
                flags.set("is_pic", "false").unwrap();
                flags.set("use_colocated_libcalls", "false").unwrap();
                let target = super::super::Target::MacosArm64;
                target.configure_stack(&mut flags).unwrap();
                let isa = isa::lookup(target.triple().parse().unwrap())
                    .unwrap()
                    .finish(settings::Flags::new(flags))
                    .unwrap();
                let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
                let mut owner = OwnedCode(Some(JITModule::new(builder)));
                let module = owner.0.as_mut().unwrap();
                compile_into(&input, "Main", None, target, module, true).unwrap();
                module.finalize_definitions().unwrap();
                let Some(FuncOrDataId::Func(id)) = module.get_name("neoclr_entry_v2") else {
                    panic!("missing entry")
                };
                // SAFETY: compile_into emits this exact entry ABI. The code owner
                // stays alive throughout every invocation and is released last.
                let entry: Entry =
                    unsafe { std::mem::transmute(module.get_finalized_function(id)) };
                cases.push((owner, entry, started.elapsed().as_secs_f64(), Vec::new()));
            }
            let iterations = 1_000_000;
            for sample in 0..10 {
                for index in if sample % 2 == 0 { [0, 1] } else { [1, 0] } {
                    let (_, entry, _, samples) = &mut cases[index];
                    let start = Instant::now();
                    let mut sum = 0i64;
                    for i in 0..iterations {
                        let mut value = 0;
                        // SAFETY: entry is owned by the live module in this case.
                        let status = unsafe { entry(i & 1023, &mut value) };
                        assert_eq!(status, 0);
                        sum += i64::from(value);
                    }
                    let ns = start.elapsed().as_nanos() as f64 / f64::from(iterations);
                    assert_eq!(
                        sum,
                        (0..iterations)
                            .map(|i| i64::from((i & 1023) + 2))
                            .sum::<i64>()
                    );
                    if sample != 0 {
                        samples.push(ns);
                    }
                }
            }
            println!(
                "STRUCTURAL_JIT_BENCH {}",
                serde_json::json!({
                    "elements": elements, "iterations": iterations, "harness_debug_assertions": cfg!(debug_assertions),
                    "structural": { "compile_seconds": cases[0].2, "samples_ns_per_call": cases[0].3 },
                    "nominal": { "compile_seconds": cases[1].2, "samples_ns_per_call": cases[1].3 },
                    "limitations": ["Rust timing harness includes status checks and checksum", "no managed references or GC", "one host", "no timing threshold assertion"]
                })
            );
        }
    }
}

// Static storage is context-owned and thread-affine. Its private eight-byte
// header reserves the high word for initialization: 0 unstarted, 1 running,
// 2 complete, 3 + terminal fault code failed. No guest address exposes this header.
fn compile_initialization_guards(
    module: &mut impl Module,
    p: &Profile<'_>,
    ids: &[cranelift_module::FuncId],
    allocator: Option<cranelift_module::FuncId>,
    stack_budget: bool,
    long_range_calls: bool,
    mut live_bodies: Option<&mut Vec<(cranelift_module::FuncId, cranelift_codegen::Context)>>,
) -> Result<std::collections::HashMap<String, cranelift_module::FuncId>, Error> {
    let mut guards = std::collections::HashMap::new();
    let owners: Vec<_> = p
        .input
        .types
        .iter()
        .filter(|t| crate::type_initialization::needs_storage(p.input, t))
        .collect();
    for (ordinal, owner) in owners.iter().enumerate() {
        let ty = crate::type_initialization::owner_type(owner);
        let Some(initializer) = crate::type_initialization::initializer(p.input, &ty) else {
            continue;
        };
        let bytes = if owner.static_fields.is_empty() {
            8
        } else {
            p.static_field(&ty, owner.static_fields.len() - 1)?.3
        };
        let mut signature = module.make_signature();
        signature.params.push(AbiParam::new(types::I64));
        signature.returns.push(AbiParam::new(types::I32));
        let id = module.declare_function(
            &format!("neoclr_type_init_{ordinal}"),
            Linkage::Local,
            &signature,
        )?;
        guards.insert(owner.name.clone(), id);
        let mut context = module.make_context();
        context.func.signature = signature;
        let mut fb = FunctionBuilderContext::new();
        {
            let mut b = FunctionBuilder::new(&mut context.func, &mut fb);
            let entry = b.create_block();
            b.append_block_params_for_function_params(entry);
            b.switch_to_block(entry);
            let ctx = b.block_params(entry)[0];
            let cold = b.create_block();
            let lookup = b.create_block();
            let ready = b.create_block();
            b.append_block_param(ready, types::I64);
            let table = b.ins().load(types::I64, MemFlags::new(), ctx, 1072);
            let missing = b.ins().icmp_imm(IntCC::Equal, table, 0);
            b.ins().brif(missing, cold, &[], lookup, &[]);
            b.switch_to_block(lookup);
            let cached = b.ins().load(
                types::I64,
                MemFlags::new(),
                table,
                ((ordinal + 1) * 8) as i32,
            );
            let missing = b.ins().icmp_imm(IntCC::Equal, cached, 0);
            b.ins().brif(missing, cold, &[], ready, &[cached.into()]);
            b.switch_to_block(cold);
            let output_slot = slot(&mut b, 8);
            let output = b.ins().stack_addr(types::I64, output_slot, 0);
            let allocator = module.declare_func_in_func(
                allocator.ok_or("initializer requires native static storage")?,
                b.func,
            );
            let index = b.ins().iconst(types::I32, ordinal as i64);
            let count = b.ins().iconst(types::I32, owners.len() as i64);
            let size = b.ins().iconst(types::I32, bytes as i64);
            let call = b.ins().call(allocator, &[ctx, index, count, size, output]);
            let status = b.inst_results(call)[0];
            let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
            return_if_detailed(&mut b, failed, status, None);
            let storage = b.ins().load(types::I64, MemFlags::new(), output, 0);
            b.ins().jump(ready, &[storage.into()]);
            b.switch_to_block(ready);
            let storage = b.block_params(ready)[0];
            let state = b.ins().load(types::I32, MemFlags::new(), storage, 4);
            let start = b.create_block();
            let previous = b.create_block();
            let missing = b.ins().icmp_imm(IntCC::Equal, state, 0);
            b.ins().brif(missing, start, &[], previous, &[]);
            b.switch_to_block(previous);
            let failed = b.ins().icmp_imm(IntCC::UnsignedGreaterThan, state, 2);
            let status = b.ins().iadd_imm(state, -3);
            let zero = b.ins().iconst(types::I32, 0);
            let status = b.ins().select(failed, status, zero);
            b.ins().return_(&[status]);
            b.switch_to_block(start);
            let running = b.ins().iconst(types::I32, 1);
            b.ins().store(MemFlags::new(), running, storage, 4);
            let scratch = slot(&mut b, 8);
            let scratch = b.ins().stack_addr(types::I64, scratch, 0);
            let callee = module.declare_func_in_func(ids[initializer], b.func);
            let call = b.ins().call(callee, &[scratch, ctx]);
            let status = b.inst_results(call)[0];
            let failed = b.ins().icmp_imm(IntCC::NotEqual, status, 0);
            let cached_failure = b.ins().iadd_imm(status, 3);
            let complete = b.ins().iconst(types::I32, 2);
            let state = b.ins().select(failed, cached_failure, complete);
            b.ins().store(MemFlags::new(), state, storage, 4);
            b.ins().return_(&[status]);
            b.seal_all_blocks();
            b.finalize();
        }
        if let Some(bodies) = &mut live_bodies {
            // The caller has published its frame. This guard retains only the
            // context-rooted static owner across the initializer call; its result
            // scratch is Void. It adds no independently live managed reference.
            let entry = context.func.layout.entry_block().unwrap();
            let owner = context.func.dfg.block_params(entry)[0];
            crate::mutation_store::lower(module, &mut context.func, owner)?;
            bodies.push((id, context));
        } else {
            define_checked(module, id, &mut context, stack_budget, long_range_calls)?;
        }
    }
    Ok(guards)
}
