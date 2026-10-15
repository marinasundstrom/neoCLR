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
    ReferenceArray(usize),
    RecordArray(usize),
    ScalarArray(neoclr::metadata::Type),
    Object(usize),
    Interface(usize),
    Intersection,
    UnionReferencePayload,
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
        Ty::ReferenceArray(i) => Trace::ReferenceArray(*i),
        Ty::RecordArray(i) => Trace::RecordArray(*i),
        Ty::ScalarArray(t) => Trace::ScalarArray(t.clone()),
        Ty::Literal | Ty::Character => Trace::Text,
        Ty::Reference(i) if p.is_array_backing(*i) => {
            return slots(
                p,
                &p.ty(&p.input.types[*i].fields[0].ty)
                    .expect("verified array backing"),
            );
        }
        Ty::Reference(i) => Trace::Object(*i),
        Ty::Interface(i) => Trace::Interface(*i),
        Ty::Intersection(_) => Trace::Intersection,
        Ty::ReferenceUnion(_) => Trace::UnionReferencePayload,
        Ty::MixedUnion(_) => {
            return vec![Slot {
                lane: 2,
                trace: Trace::UnionReferencePayload,
            }];
        }
        Ty::ByteArray => Trace::ByteArray,
        Ty::StringArray | Ty::CharacterArray | Ty::CharacterValues => Trace::StringArray,
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
        Ty::Tuple(parts) | Ty::ValueUnion(parts) => {
            let mut result = vec![];
            let mut offset = usize::from(matches!(ty, Ty::ValueUnion(_)));
            for ty in parts {
                for mut slot in slots(p, ty) {
                    slot.lane += offset;
                    if let Trace::ErasedText { tag_lane } = &mut slot.trace {
                        *tag_lane += offset;
                    }
                    result.push(slot);
                }
                offset += p.lanes(ty);
            }
            return result;
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
        Ty::PendingUnionScalar(t) => return slots(p, &p.ty(t).expect("admitted union payload")),
        Ty::ScalarUnion(_) | Ty::ScalarUnionTest => return vec![],
        Ty::Int
        | Ty::Byte
        | Ty::SByte
        | Ty::Short
        | Ty::UShort
        | Ty::Bool
        | Ty::Unit
        | Ty::Size
        | Ty::Wide
        | Ty::Single
        | Ty::Double
        | Ty::TypeToken => return vec![],
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
            Trace::ReferenceArray(i) => json!({"kind": "reference-array", "typeIndex": i, "elements": "initialized-strong-reference-slots"}),
            Trace::RecordArray(i) => json!({"kind": "record-array", "typeIndex": i, "elements": "initialized-value-snapshots"}),
            Trace::ScalarArray(t) => json!({"kind": "scalar-array", "elementType": t, "elements": "atomic-initialized-scalars"}),
            Trace::CallableArray => json!({"kind": "callable-array", "elements": "initialized-callback-slots"}),
            Trace::Callable => json!({"kind": "callable", "receiver": "strong-heap-owner"}),
            Trace::Text => json!({"kind": "text", "storage": "image-or-arena"}),
            Trace::Object(i) => json!({"kind": "object-view", "typeIndex": i, "stringTagMask": 1}),
            Trace::UnionReferencePayload => json!({"kind": "union-reference-payload", "stringTagMask": 1}),
            Trace::Intersection => json!({"kind": "intersection-view", "stringTagMask": 1}),
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
            !p.is_array_backing(*i)
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
        Profile::new(module, true, None, None, &[], None, None, None, false).unwrap()
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
        for ty in [Ty::Int, Ty::Size, Ty::Wide, Ty::Double, Ty::Unit, Ty::Bool] {
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
    fn mixed_union_traces_only_its_separate_reference_lane() {
        let module = module();
        let p = profile(&module);
        let ty = Ty::MixedUnion(vec![Ty::Int, Ty::Interface(0)]);
        assert_eq!(p.lanes(&ty), 3);
        assert_eq!(seed_lanes(&p, &ty), vec![2]);
        assert_eq!(
            slots(&p, &ty),
            vec![Slot {
                lane: 2,
                trace: Trace::UnionReferencePayload
            }]
        );
        let nested = Ty::Tuple(vec![Ty::Wide, ty]);
        assert_eq!(
            slots(&p, &nested),
            vec![Slot {
                lane: 3,
                trace: Trace::UnionReferencePayload
            }]
        );
    }

    #[test]
    fn union_reference_payload_uses_one_strong_slot_without_a_carrier() {
        let module = module();
        let p = profile(&module);
        let ty = Ty::ReferenceUnion(vec![Ty::Interface(0), Ty::Interface(1)]);
        assert_eq!(p.lanes(&ty), 1);
        assert_eq!(p.bytes(&ty), 8);
        assert_eq!(seed_lanes(&p, &ty), vec![0]);
        assert_eq!(
            slots(&p, &ty),
            vec![Slot {
                lane: 0,
                trace: Trace::UnionReferencePayload
            }]
        );
    }

    #[test]
    fn intersection_views_use_one_strong_reference_slot() {
        let module = module();
        let p = profile(&module);
        let ty = Ty::Intersection(vec![Ty::Interface(0), Ty::Interface(1)]);
        assert_eq!(p.lanes(&ty), 1);
        assert_eq!(p.bytes(&ty), 8);
        assert_eq!(seed_lanes(&p, &ty), vec![0]);
        assert_eq!(
            slots(&p, &ty),
            vec![Slot {
                lane: 0,
                trace: Trace::Intersection
            }]
        );
    }

    #[test]
    fn structural_tuple_tracing_preserves_nested_offsets_and_excludes_numeric_lanes() {
        let module = module();
        let p = profile(&module);
        let ty = Ty::Tuple(vec![
            Ty::Wide,
            Ty::Tuple(vec![Ty::Double, Ty::Literal]),
            Ty::Int,
        ]);
        assert_eq!(
            slots(&p, &ty),
            vec![Slot {
                lane: 2,
                trace: Trace::Text
            }]
        );
        assert_eq!(seed_lanes(&p, &ty), vec![2]);
        assert_eq!(p.lanes(&ty), 4);
        assert_eq!(p.bytes(&ty), 32);
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
        let p = Profile::new(&module, true, None, Some(0), &[], None, None, None, false).unwrap();
        assert_eq!(slots(&p, &Ty::Reference(0)), slots(&p, &Ty::ByteArray));
        assert!(report(&p)["objects"].as_array().unwrap().is_empty());
        let mut element = module.types[0].clone();
        element.name = "Element".into();
        element.fields.clear();
        module.types[0].fields[0].ty = neoclr::metadata::Type::ArrayRef(Box::new(
            neoclr::metadata::Type::Named(element.name.clone()),
        ));
        module.types.push(element);
        let p = Profile::new(&module, true, None, None, &[0], None, None, None, false).unwrap();
        assert_eq!(
            slots(&p, &Ty::Reference(0)),
            slots(&p, &Ty::ReferenceArray(1))
        );
        let objects = report(&p)["objects"].as_array().unwrap().clone();
        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0]["typeIndex"], 1);
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
    fn value_union_payload_regions_preserve_record_roots_and_bounds() {
        use neoclr::metadata::Type;
        let mut module = module();
        let mut cell = module.types[0].clone();
        cell.name = "Cell".into();
        cell.is_reference_type = true;
        cell.fields.clear();
        let mut payload = module.types[0].clone();
        payload.name = "Payload".into();
        let mut number = payload.fields[0].clone();
        number.name = "Number".into();
        number.ty = Type::Int64;
        let mut reference = number.clone();
        reference.name = "Reference".into();
        reference.ty = Type::Named("Cell".into());
        payload.fields = vec![number.clone(), reference];
        let mut other = payload.clone();
        other.name = "OtherPayload".into();
        module.types = vec![cell, payload, other];
        module.functions.clear();
        let p = profile(&module);
        let union = p
            .ty(&Type::Union(vec![
                Type::Int32,
                Type::Named("Payload".into()),
                Type::Named("OtherPayload".into()),
            ]))
            .unwrap();
        assert_eq!(p.lanes(&union), 6);
        assert_eq!(seed_lanes(&p, &union), vec![3, 5]);
        assert_eq!(
            slots(&p, &union),
            vec![
                Slot {
                    lane: 3,
                    trace: Trace::Object(0)
                },
                Slot {
                    lane: 5,
                    trace: Trace::Object(0)
                }
            ]
        );
        assert_eq!(
            seed_lanes(&p, &Ty::Tuple(vec![Ty::Wide, union])),
            vec![4, 6]
        );
        assert_eq!(
            slots(&p, &Ty::PendingUnionScalar(Type::Named("Payload".into()))),
            vec![Slot {
                lane: 1,
                trace: Trace::Object(0)
            }]
        );
        for fields in [14, 15] {
            let mut chunk = module.types[1].clone();
            chunk.name = "Chunk".into();
            chunk.fields = (0..16)
                .map(|i| {
                    let mut field = number.clone();
                    field.name = format!("F{i}");
                    field
                })
                .collect();
            let mut tail = chunk.clone();
            tail.name = "Tail".into();
            tail.fields.truncate(fields);
            module.types.truncate(3);
            module.types.push(chunk);
            module.types.push(tail);
            module.types[1].fields = (0..16)
                .map(|i| {
                    let mut field = number.clone();
                    field.name = format!("F{i}");
                    field.ty = Type::Named(if i == 15 { "Tail" } else { "Chunk" }.into());
                    field
                })
                .collect();
            let p = profile(&module);
            let union = p.ty(&Type::Union(vec![
                Type::Int32,
                Type::Named("Payload".into()),
            ]));
            if fields == 14 {
                assert_eq!(p.lanes(&union.unwrap()), 256);
            } else {
                assert!(union.unwrap_err().to_string().contains("inline lane limit"));
            }
        }
    }

    #[test]
    fn scalar_union_payloads_are_numeric_and_never_roots() {
        let module = module();
        let p = profile(&module);
        let ty = Ty::ScalarUnion(vec![
            neoclr::metadata::Type::Int32,
            neoclr::metadata::Type::Int64,
        ]);
        assert!(slots(&p, &Ty::ScalarUnionTest).is_empty());
        assert_eq!(p.lanes(&Ty::ScalarUnionTest), 1);
        assert_eq!(p.lanes(&ty), 2);
        assert_eq!(p.bytes(&ty), 16);
        assert!(slots(&p, &ty).is_empty());
        assert!(seed_lanes(&p, &ty).is_empty());
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
