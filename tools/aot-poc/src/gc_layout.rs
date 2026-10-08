//! Private tracing recipes for admitted storage. These are not registered GC roots.
use super::profile::{Profile, Ty};
use serde_json::{Value, json};

// Sparse, typed recipes deliberately do not use Profile::pointer_lanes: integer
// ABI lanes can be 64 bits without containing a managed reference.
#[derive(Debug, PartialEq, Eq)]
enum Trace {
    Text,
    Callable,
    CallableArray,
    RecordArray(usize),
    Object(usize),
    Interface(usize),
    ByteArray,
    StringArray,
    ByteValues,
    Borrow(Vec<Slot>),
    ErasedText { tag_lane: usize },
}
#[derive(Debug, PartialEq, Eq)]
struct Slot {
    lane: usize,
    trace: Trace,
}
fn slots(p: &Profile<'_>, ty: &Ty) -> Vec<Slot> {
    let trace = match ty {
        Ty::Callable(_) => Trace::Callable,
        Ty::CallableArray(_) => Trace::CallableArray,
        Ty::RecordArray(i) => Trace::RecordArray(*i),
        Ty::Literal | Ty::Character => Trace::Text,
        Ty::Reference(i) if p.array_backing == Some(*i) => Trace::ByteArray,
        Ty::Reference(i) => Trace::Object(*i),
        Ty::Interface(i) => Trace::Interface(*i),
        Ty::ByteArray => Trace::ByteArray,
        Ty::StringArray => Trace::StringArray,
        Ty::ByteValues => Trace::ByteValues,
        // Even a borrow of an integer can keep its owning object/array alive.
        // Tracing the pointee alone is insufficient for an interior address.
        Ty::Address(inner) => Trace::Borrow(slots(p, inner)),
        Ty::Erased => {
            return vec![Slot {
                lane: 1,
                trace: Trace::ErasedText { tag_lane: 0 },
            }];
        }
        Ty::Record(i) => {
            let mut result = vec![];
            let mut offset = 0;
            for field in &p.input.types[*i].fields {
                let ty = p.ty(&field.ty).expect("admitted field");
                for mut slot in slots(p, &ty) {
                    slot.lane += offset;
                    if let Trace::ErasedText { tag_lane } = &mut slot.trace {
                        *tag_lane += offset;
                    }
                    result.push(slot);
                }
                offset += p.lanes(&ty);
            }
            return result;
        }
        Ty::Int
        | Ty::Byte
        | Ty::SByte
        | Ty::Short
        | Ty::UShort
        | Ty::Bool
        | Ty::Unit
        | Ty::Size
        | Ty::Wide => return vec![],
    };
    vec![Slot { lane: 0, trace }]
}
// Clear both the pointer and its discriminator before the local can appear in a
// future root frame. This does not mark the guest local definitely assigned.
pub(super) fn seed_lanes(p: &Profile<'_>, ty: &Ty) -> Vec<usize> {
    let mut lanes = vec![];
    for slot in slots(p, ty) {
        lanes.push(slot.lane);
        if let Trace::ErasedText { tag_lane } = slot.trace {
            lanes.push(tag_lane);
        }
    }
    lanes.sort_unstable();
    lanes.dedup();
    lanes
}
fn encode(slots: Vec<Slot>) -> Vec<Value> {
    slots.into_iter().map(|slot| {
        let recipe = match slot.trace {
            Trace::RecordArray(i) => json!({"kind": "record-array", "typeIndex": i, "elements": "initialized-value-snapshots"}),
            Trace::CallableArray => json!({"kind": "callable-array", "elements": "initialized-callback-slots"}),
            Trace::Callable => json!({"kind": "callable", "receiver": "strong-heap-owner"}),
            Trace::Text => json!({"kind": "text", "storage": "image-or-arena"}),
            Trace::Object(i) => json!({"kind": "object-view", "typeIndex": i, "stringTagMask": 1}),
            Trace::Interface(i) => json!({"kind": "interface-view", "typeIndex": i, "stringTagMask": 1}),
            Trace::ByteArray => json!({"kind": "byte-array"}),
            Trace::StringArray => json!({"kind": "string-array", "elements": "initialized-text-slots"}),
            Trace::ByteValues => json!({"kind": "byte-values"}),
            Trace::Borrow(target) => json!({"kind": "managed-borrow", "requiresOwner": true, "pointeeSlots": encode(target)}),
            Trace::ErasedText { tag_lane } => json!({"kind": "conditional-text", "tagLane": tag_lane, "tagValue": super::profile::erased_tag(&neoclr::metadata::Type::String).expect("admitted erased String")}),
        };
        json!({"lane": slot.lane, "byteOffset": slot.lane * 8, "trace": recipe})
    }).collect()
}
pub(super) fn layout(p: &Profile<'_>, ty: &Ty) -> Value {
    json!({"lanes": p.lanes(ty), "storageBytes": p.bytes(ty), "traceSlots": encode(slots(p, ty))})
}
pub(super) fn report(p: &Profile<'_>) -> Value {
    let types: Vec<_> = p
        .input
        .types
        .iter()
        .enumerate()
        .map(|(i, t)| {
            json!({"index": i, "name": t.name, "representation": t.representation,
            "referenceType": t.is_reference_type})
        })
        .collect();
    let objects: Vec<_> = p
        .input
        .types
        .iter()
        .enumerate()
        .filter(|(i, t)| {
            p.array_backing != Some(*i)
                && !t.is_abstract
                && t.is_reference_type
                && t.representation == neoclr::metadata::Representation::Record
                && !crate::selection::static_owner(t)
        })
        .map(|(i, t)| {
            json!({"typeIndex": i, "name": t.name, "headerBytes": 8,
            "allocationBytes": p.object_bytes(i), "payload": layout(p, &Ty::Record(i))})
        })
        .collect();
    let functions: Vec<_> = p
        .input
        .functions
        .iter()
        .enumerate()
        .map(|(i, f)| {
            json!({"index": i, "name": f.name,
            "arguments": p.args[i].iter().map(|t| layout(p, t)).collect::<Vec<_>>(),
            "locals": p.locals[i].iter().map(|t| layout(p, t)).collect::<Vec<_>>(),
            "localSeedLanes": p.locals[i].iter().map(|t| seed_lanes(p, t)).collect::<Vec<_>>(),
            "result": p.results[i].as_ref().map(|t| layout(p, t))})
        })
        .collect();
    json!({"schema": "neoclr-native-trace-layout-v1", "types": types, "objects": objects, "functions": functions,
        "notice": "Private storage recipes only; not initialized/live root maps, safepoints, emitted descriptors or a collector."})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn profile(module: &neoclr::Module) -> Profile<'_> {
        Profile::new(module, true, None, None, None, None).unwrap()
    }
    fn module() -> neoclr::Module {
        neoclr::metadata_container::decode(include_bytes!(
            "../../../docs/experiments/aot-values/Counter.pe"
        ))
        .unwrap()
    }
    #[test]
    fn wide_integers_are_not_roots_but_scalar_borrows_keep_owners() {
        let module = module();
        let p = profile(&module);
        for ty in [Ty::Int, Ty::Size, Ty::Wide, Ty::Unit, Ty::Bool] {
            assert!(slots(&p, &ty).is_empty());
            assert_eq!(
                slots(&p, &ty.address()),
                vec![Slot {
                    lane: 0,
                    trace: Trace::Borrow(vec![])
                }]
            );
        }
        assert_eq!(
            slots(&p, &Ty::Erased),
            vec![Slot {
                lane: 1,
                trace: Trace::ErasedText { tag_lane: 0 }
            }]
        );
        assert_eq!(
            slots(&p, &Ty::StringArray),
            vec![Slot {
                lane: 0,
                trace: Trace::StringArray
            }]
        );
    }
    #[test]
    fn nested_records_preserve_reference_offsets_and_stop_at_reference_edges() {
        let mut module = module();
        let mut leaf = module.types[0].clone();
        leaf.name = "Leaf".into();
        let mut field = leaf.fields[0].clone();
        field.ty = neoclr::metadata::Type::UInt64;
        leaf.fields = vec![field.clone()];
        field.ty = neoclr::metadata::Type::String;
        leaf.fields.push(field.clone());
        let mut parent = leaf.clone();
        parent.name = "Parent".into();
        parent.fields[1].ty = neoclr::metadata::Type::Named("Leaf".into());
        module.types = vec![leaf, parent];
        module.functions.clear();
        let p = profile(&module);
        assert_eq!(
            slots(&p, &Ty::Record(1)),
            vec![Slot {
                lane: 2,
                trace: Trace::Text
            }]
        );
        assert_eq!(layout(&p, &Ty::Record(1))["storageBytes"], 24);
        assert_eq!(
            slots(&p, &Ty::Reference(1)),
            vec![Slot {
                lane: 0,
                trace: Trace::Object(1)
            }]
        );
        assert_eq!(
            slots(&p, &Ty::Record(1).address()),
            vec![Slot {
                lane: 0,
                trace: Trace::Borrow(vec![Slot {
                    lane: 2,
                    trace: Trace::Text
                }])
            }]
        );
    }
    #[test]
    fn nominal_array_backing_is_not_an_object_containing_itself() {
        let mut module = module();
        let mut backing = module.types[0].clone();
        backing.is_reference_type = true;
        backing.fields.truncate(1);
        backing.fields[0].ty =
            neoclr::metadata::Type::ArrayRef(Box::new(neoclr::metadata::Type::Byte));
        module.types = vec![backing];
        module.functions.clear();
        let p = Profile::new(&module, true, None, Some(0), None, None).unwrap();
        assert_eq!(slots(&p, &Ty::Reference(0)), slots(&p, &Ty::ByteArray));
        assert!(report(&p)["objects"].as_array().unwrap().is_empty());
    }
    #[test]
    fn cyclic_objects_describe_edges_without_recursive_expansion() {
        let mut module = module();
        let mut node = module.types[0].clone();
        node.is_reference_type = true;
        node.fields.truncate(1);
        node.fields[0].ty = neoclr::metadata::Type::Named(node.name.clone());
        module.types = vec![node];
        module.functions.clear();
        let p = profile(&module);
        let objects = report(&p)["objects"].clone();
        assert_eq!(objects[0]["allocationBytes"], 16);
        assert_eq!(objects[0]["payload"]["traceSlots"][0]["byteOffset"], 0);
        assert_eq!(
            objects[0]["payload"]["traceSlots"][0]["trace"]["typeIndex"],
            0
        );
        assert_eq!(slots(&p, &Ty::Character), slots(&p, &Ty::Literal));
        assert!(matches!(
            slots(&p, &Ty::Interface(0))[0].trace,
            Trace::Interface(0)
        ));
        assert!(matches!(
            slots(&p, &Ty::ByteValues)[0].trace,
            Trace::ByteValues
        ));
    }
    #[test]
    fn local_seeding_clears_erased_tags_and_roots_but_not_integer_lanes() {
        let mut module = module();
        let mut record = module.types[0].clone();
        let mut field = record.fields[0].clone();
        field.ty = neoclr::metadata::Type::UInt64;
        record.fields = vec![field.clone()];
        field.ty = neoclr::metadata::Type::String;
        record.fields.push(field);
        module.types = vec![record];
        module.functions.clear();
        let p = profile(&module);
        assert_eq!(seed_lanes(&p, &Ty::Record(0)), vec![1]);
        assert_eq!(seed_lanes(&p, &Ty::Erased), vec![0, 1]);
        assert_eq!(seed_lanes(&p, &Ty::ByteValues), vec![0]);
        assert_eq!(seed_lanes(&p, &Ty::Int.address()), vec![0]);
        for ty in [Ty::Wide, Ty::Size, Ty::Int, Ty::Unit] {
            assert!(seed_lanes(&p, &ty).is_empty());
        }
    }
}
