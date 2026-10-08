//! Conservative IL-boundary root plans. No native root publication or scanning.
use super::{
    Error, gc_layout,
    profile::{Profile, Ty},
};
use neoclr::metadata::Instruction as Op;
use serde_json::{Value, json};

pub(super) fn native_body(p: &Profile<'_>, i: usize, details: Option<&crate::fault_details::Options>) -> bool {
    if p.dispatch.contains_key(&i) {
        return true;
    }
    details.is_some_and(|d| {
        d.int32_boxes.contains_key(&i)
            || d.empty_record_boxes.contains_key(&i)
            || [
                &d.console_read_byte,
                &d.console_write_line,
                &d.console_write_bytes,
                &d.console_flush,
                &d.parse_int32,
                &d.task_queue_register,
                &d.task_queue_default,
                &d.task_queue_current,
                &d.socket_accept,
                &d.socket_connect_result,
                &d.socket_cancel,
                &d.socket_receive,
                &d.socket_send,
                &d.socket_transfer_result,
                &d.socket_deadline_after,
                &d.socket_deadline_expired,
                &d.socket_receive_until,
                &d.socket_send_until,
                &d.string_compare_ordinal,
                &d.socket_listen,
                &d.socket_local_port,
                &d.socket_close,
                &d.int32_to_string,
                &d.string_contains_ordinal,
                &d.string_starts_with_ordinal,
                &d.string_ends_with_ordinal,
                &d.string_concat,
                &d.string_join_parts,
                &d.string_byte_count,
                &d.utf8_decode,
                &d.utf8_encode,
                &d.string_slice_utf8,
                &d.char_from_string,
                &d.char_text,
                &d.int64_to_string,
                &d.uint64_to_string,
                &d.native_integer_to64,
            ]
            .iter()
            .any(|indices| indices.contains(&i))
    })
}

pub(super) fn point(p: &Profile<'_>, op: &Op, stack: &[Ty]) -> Result<Option<Value>, Error> {
    let (consumed, kind, result, receiver) = match op {
        Op::BindFunction { function_type, target } => (
            usize::from(target.instance), "callback-bind", Some(p.ty(function_type)?), None,
        ),
        Op::Call(target) | Op::CallVirtual(target) if crate::selection::callable_invoke(target).is_some() => {
            let shape = crate::selection::callable_invoke(target).unwrap();
            (shape.parameters.len() + 1, "callback-invoke",
                if shape.no_result { None } else { Some(p.ty(&shape.returns)?) }, None)
        }
        Op::Call(target) | Op::CallVirtual(target) | Op::Construct(target) => {
            let callee = p.callee(target)?;
            let construct = matches!(op, Op::Construct(_));
            let result = if construct {
                Some(p.ty(target.owner.as_ref().ok_or("constructor owner missing")?)?)
            } else {
                p.results[callee].clone()
            };
            (
                p.args[callee].len() - usize::from(construct),
                if construct { "construct" } else { "call" },
                result,
                construct.then(|| p.args[callee][0].clone()),
            )
        }
        Op::NewArray(t) | Op::ReserveArray(t) => (
            1,
            "array-allocation",
            Some(p.ty(&neoclr::metadata::Type::ArrayRef(Box::new(t.clone())))?),
            None,
        ),
        // Literal materialization is conservative: identity-observable text can
        // allocate even though other literals remain image data.
        Op::String(_) => (0, "text-materialization", Some(Ty::Literal), None),
        _ => return Ok(None),
    };
    let split = stack
        .len()
        .checked_sub(consumed)
        .ok_or("root plan stack underflow")?;
    let mut lane_base = 0;
    let mut roots = vec![];
    let mut spill_lanes = vec![];
    for (value, ty) in stack.iter().enumerate() {
        spill_lanes.extend(
            gc_layout::seed_lanes(p, ty)
                .into_iter()
                .map(|lane| lane_base + lane),
        );
        let layout = gc_layout::layout(p, ty);
        if !layout["traceSlots"].as_array().unwrap().is_empty() {
            roots.push(json!({"stackValue": value, "laneBase": lane_base,
                "role": if value < split { "caller-retained" } else { "pending-operand" },
                "layout": layout}));
        }
        lane_base += p.lanes(ty);
    }
    Ok(Some(
        json!({"kind": kind, "stackValues": stack.len(), "stackLanes": lane_base,
        "consumedValues": consumed, "roots": roots, "requiredSpillLanes": spill_lanes,
        "resultAfterSuccess": result.as_ref().map(|t| gc_layout::layout(p, t)),
        "constructorReceiverBeforeCall": receiver.as_ref().map(|t| gc_layout::layout(p, t)),
        "resultActiveBeforeOperation": false}),
    ))
}

pub(super) fn report(
    p: &Profile<'_>,
    details: Option<&crate::fault_details::Options>,
) -> Result<Value, Error> {
    let mut functions = vec![];
    for (i, f) in p.input.functions.iter().enumerate() {
        if native_body(p, i, details) {
            functions.push(json!({"index": i, "name": f.name, "coverage": "native-body-requires-separate-plan",
                "diagnosticFrame": "typed-arguments-only", "serviceInternalRoots": "uncovered"}));
            continue;
        }
        let shapes = p.analyze(i)?;
        let mut points = vec![];
        for (pc, op) in f.body.iter().enumerate() {
            let Some(shape) = &shapes[pc] else {
                continue;
            };
            if let Some(mut plan) = point(p, op, shape)? {
                plan["instruction"] = json!(pc);
                points.push(plan);
            }
        }
        functions.push(json!({"index": i, "name": f.name, "coverage": "managed-il-boundaries", "points": points}));
    }
    Ok(
        json!({"schema": "neoclr-pre-operation-roots-v1", "functions": functions,
        "collection": if details.is_some_and(|d| d.native_gc) { "nonmoving-conservative-pre-operation" } else { "disabled" },
        "notice": if details.is_some_and(|d| d.native_gc) {
            "Collection runs only at published pre-operation boundaries; allocation services never collect. In-buffer descriptors recover interior owners, caller storage covers stack borrows, and fault contexts retain messages. Conservative candidates can over-retain. General host handles and concurrent collection are unsupported."
        } else {
            "Conservative reachable IL boundaries, not native safepoints. Opt-in diagnostic frames expose typed storage and call-result handoff; native wrappers expose arguments only. This diagnostic-only mode forbids collection."
        }}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_plans_stage_constructor_receivers_arrays_and_output_borrows() {
        let source = r#".module Stages
.entry Main
.type class Box
.field Text String
.method instance .ctor(String text) -> noresult
ldarg 0
ldarg text
stfld 0
ret
.end
.end
.function Fill(out String& text) -> noresult
ldarg text
ldstr "filled"
stobj String
ret
.end
.function Main() -> Int32
.local String result
ldstr "retained"
ldstr "argument"
newobj.ctor instance Box::.ctor(String)
pop
ldc.i4 2
newarr String
pop
ldloca result
call Fill(String&)
pop
ldc.i4 0
ret
.end
"#;
        let module = neoclr::assemble(source).unwrap();
        let details = crate::fault_details::Options {
            reference_arena: true,
            ..Default::default()
        };
        super::super::compile(&module, "Main", Some(&details)).unwrap();
        let p = Profile::new(&module, true, None, None, None, None, None, false).unwrap();
        let report = report(&p, Some(&details)).unwrap();
        let functions = report["functions"].as_array().unwrap();
        let main = functions.iter().find(|f| f["name"] == "Main").unwrap();
        let points = main["points"].as_array().unwrap();
        let ctor = points.iter().find(|p| p["kind"] == "construct").unwrap();
        assert_eq!(
            ctor["consumedValues"], 1,
            "receiver is not yet on the input stack"
        );
        assert_eq!(ctor["roots"][0]["role"], "caller-retained");
        assert_eq!(ctor["roots"][1]["role"], "pending-operand");
        assert_eq!(
            ctor["constructorReceiverBeforeCall"]["traceSlots"][0]["trace"]["kind"],
            "object-view"
        );
        let array = points
            .iter()
            .find(|p| p["kind"] == "array-allocation")
            .unwrap();
        assert_eq!(array["stackValues"], 2);
        assert_eq!(
            array["roots"].as_array().unwrap().len(),
            1,
            "length is not a root"
        );
        assert_eq!(
            array["resultAfterSuccess"]["traceSlots"][0]["trace"]["kind"],
            "string-array"
        );
        let call = points.iter().find(|p| p["kind"] == "call").unwrap();
        assert_eq!(
            call["roots"][1]["layout"]["traceSlots"][0]["trace"]["kind"],
            "managed-borrow"
        );
        assert!(
            call["resultAfterSuccess"].is_null(),
            "no-result calls must not invent roots"
        );
    }
    #[test]
    fn spill_selection_keeps_discriminators_and_excludes_wide_integers() {
        let module =
            neoclr::assemble(".module Spill\n.function Main() -> Int32\nldc.i4 0\nret\n.end")
                .unwrap();
        let p = Profile::new(&module, false, None, None, None, None, None, false).unwrap();
        let plan = point(
            &p,
            &Op::String("next".into()),
            &[Ty::Wide, Ty::Erased, Ty::Literal],
        )
        .unwrap()
        .unwrap();
        assert_eq!(plan["stackLanes"], 4);
        assert_eq!(plan["requiredSpillLanes"], json!([1, 2, 3]));
        assert_eq!(plan["roots"][0]["laneBase"], 1);
        assert_eq!(plan["roots"][1]["laneBase"], 3);
        let details = crate::fault_details::Options {
            console_write_line: vec![0],
            ..Default::default()
        };
        let report = report(&p, Some(&details)).unwrap();
        assert_eq!(
            report["functions"][0]["coverage"],
            "native-body-requires-separate-plan"
        );
        assert!(report["functions"][0]["points"].is_null());
    }
}
