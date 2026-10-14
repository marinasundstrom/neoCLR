//! Test-only reference composition admission using neoCLR metadata and heap services.
//! Not a guest signature, public descriptor, persisted identity or storage ABI.
use crate::{
    Fault, ManagedHeap, Module, Value,
    metadata::{Representation, Type},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Expression {
    Nominal(Type),
    Union(Vec<Expression>),
    Intersection(Vec<Expression>),
}

// Equality is meaningful only inside this borrowed module. Nothing is exposed for
// comparing unbound descriptors from different load contexts.
struct Contract<'a> {
    module: &'a Module,
    expression: Expression,
}

impl<'a> Contract<'a> {
    fn admit(module: &'a Module, expression: &Expression) -> Result<Self, Fault> {
        let mut remaining = 4096;
        let expression = normalize(module, expression, 0, &mut remaining)?;
        Ok(Self { module, expression })
    }

    fn equivalent(&self, other: &Self) -> bool {
        std::ptr::eq(self.module, other.module) && self.expression == other.expression
    }

    fn accepts_type(&self, concrete: &Type) -> bool {
        fn accepts(module: &Module, concrete: &Type, expression: &Expression) -> bool {
            match expression {
                Expression::Nominal(target) => {
                    concrete == target || module.reference_assignable(concrete, target)
                }
                Expression::Union(parts) => parts.iter().any(|p| accepts(module, concrete, p)),
                Expression::Intersection(parts) => {
                    parts.iter().all(|p| accepts(module, concrete, p))
                }
            }
        }
        // Boxed values and intrinsic/array/function references are outside this slice.
        matches!(concrete, Type::Named(_))
            && self
                .module
                .type_definition(concrete)
                .is_some_and(|d| d.is_reference_type && d.representation == Representation::Record)
            && accepts(self.module, concrete, &self.expression)
    }

    fn narrow(&self, heap: &ManagedHeap, value: &Value) -> Result<Value, Fault> {
        let Value::ObjectReference(object) = value else {
            return Err(Fault::new(
                "composition requires a non-null class reference",
            ));
        };
        // Allocation numbers alone do not establish ownership. Validate the cell.
        heap.check_reference_owner(&object.reference)?;
        object.reference.assigned()?;
        if !self.accepts_type(&object.concrete_type()) {
            return Err(Fault::new("object does not satisfy composition"));
        }
        // The compound descriptor is external; no invented Type view or carrier.
        Ok(value.clone())
    }
}

fn normalize(
    module: &Module,
    expression: &Expression,
    depth: usize,
    remaining: &mut usize,
) -> Result<Expression, Fault> {
    if depth > 32 || *remaining == 0 {
        return Err(Fault::new("composition resource limit"));
    }
    *remaining -= 1;
    match expression {
        Expression::Nominal(ty @ Type::Named(_)) => {
            crate::vm::check_type(ty, module)?;
            let definition = module.type_definition(ty).unwrap();
            if !definition.generic_parameters.is_empty()
                || !(definition.representation == Representation::Interface
                    || (definition.representation == Representation::Record
                        && definition.is_reference_type))
            {
                return Err(Fault::new(
                    "composition requires closed class/interface types",
                ));
            }
            Ok(expression.clone())
        }
        Expression::Nominal(_) => Err(Fault::new("unsupported composition constituent")),
        Expression::Union(parts) | Expression::Intersection(parts) => {
            if !(2..=256).contains(&parts.len()) {
                return Err(Fault::new("composition requires 2 to 256 operands"));
            }
            let union = matches!(expression, Expression::Union(_));
            let mut flat = Vec::new();
            for part in parts {
                let child = normalize(module, part, depth + 1, remaining)?;
                match child {
                    Expression::Union(nested) if union => flat.extend(nested),
                    Expression::Intersection(nested) if !union => flat.extend(nested),
                    other => flat.push(other),
                }
            }
            // Private deterministic ordering, not an ABI byte encoding. All atoms
            // are already validated nominal names in one supplied module.
            flat.sort_by_cached_key(|p| serde_json::to_string(p).unwrap());
            flat.dedup();
            if flat.len() == 1 {
                return Ok(flat.pop().unwrap());
            }
            Ok(if union {
                Expression::Union(flat)
            } else {
                Expression::Intersection(flat)
            })
        }
    }
}

fn nominal(name: &str) -> Expression {
    Expression::Nominal(Type::from_name(name))
}

fn fixture_with_body(body: &str) -> Module {
    crate::assemble(&format!(
        ".module CompositionProbe\n.entry Main\n\
         .interface Left\n.end\n.interface Right\n.end\n\
         .type class Both\n.implements Left\n.implements Right\n.field Number Int32\n.end\n\
         .type class LeftOnly\n.implements Left\n.end\n\
         .type class Neither\n.end\n.type ValueCell\n.end\n{body}"
    ))
    .unwrap()
}

fn fixture() -> Module {
    fixture_with_body(".function Main() -> Int32\nldc.i4 0\nret\n.end")
}

fn allocate(heap: &mut ManagedHeap, name: &str) -> Value {
    let id = heap
        .allocate(Value::Object {
            ty: Type::from_name(name),
            fields: if name == "Both" {
                vec![Value::Int32(42)]
            } else {
                vec![]
            },
        })
        .unwrap();
    Value::ObjectReference(crate::value::ObjectReference {
        reference: heap.address(id).unwrap(),
        view: None,
    })
}

#[test]
fn canonical_identity_flattens_sorts_deduplicates_and_collapses_singletons() {
    let module = fixture();
    for union in [true, false] {
        let compose = |parts| {
            if union {
                Expression::Union(parts)
            } else {
                Expression::Intersection(parts)
            }
        };
        let a =
            Contract::admit(&module, &compose(vec![nominal("Left"), nominal("Right")])).unwrap();
        let b = Contract::admit(
            &module,
            &compose(vec![
                nominal("Right"),
                compose(vec![nominal("Left"), nominal("Right")]),
            ]),
        )
        .unwrap();
        assert!(a.equivalent(&b));
        let singleton =
            Contract::admit(&module, &compose(vec![nominal("Left"), nominal("Left")])).unwrap();
        assert!(singleton.equivalent(&Contract::admit(&module, &nominal("Left")).unwrap()));
        let roundtrip: Expression =
            serde_json::from_str(&serde_json::to_string(&a.expression).unwrap()).unwrap();
        assert!(a.equivalent(&Contract::admit(&module, &roundtrip).unwrap()));
    }
    let other = fixture();
    assert!(
        !Contract::admit(&module, &nominal("Left"))
            .unwrap()
            .equivalent(&Contract::admit(&other, &nominal("Left")).unwrap())
    );
}

#[test]
fn actual_runtime_nominal_relations_define_either_and_all_membership() {
    let module = fixture();
    let union = Contract::admit(
        &module,
        &Expression::Union(vec![nominal("Left"), nominal("Right")]),
    )
    .unwrap();
    let intersection = Contract::admit(
        &module,
        &Expression::Intersection(vec![nominal("Left"), nominal("Right")]),
    )
    .unwrap();
    assert!(!union.equivalent(&intersection));
    for (name, either, all) in [
        ("Both", true, true),
        ("LeftOnly", true, false),
        ("Neither", false, false),
    ] {
        assert_eq!(union.accepts_type(&Type::from_name(name)), either);
        assert_eq!(intersection.accepts_type(&Type::from_name(name)), all);
    }
}

#[test]
fn narrowing_preserves_one_object_and_existing_interface_view_across_collection() {
    let module = fixture();
    let contract = Contract::admit(
        &module,
        &Expression::Intersection(vec![nominal("Left"), nominal("Right")]),
    )
    .unwrap();
    let mut heap = ManagedHeap::default();
    let mut value = allocate(&mut heap, "Both");
    if let Value::ObjectReference(object) = &mut value {
        object.view = Some(Type::from_name("Left"));
    }
    let projected = contract.narrow(&heap, &value).unwrap();
    assert!(crate::object_identity::reference_equals(&value, &projected).unwrap());
    assert_eq!(projected.ty(), Type::from_name("Left"));
    allocate(&mut heap, "Neither");
    let roots = heap.value_roots(&projected).unwrap();
    heap.collect(roots, crate::CollectionReason::ExplicitRequest)
        .unwrap();
    assert_eq!(heap.len(), 1);
    assert_eq!(heap.reclaimed_objects(), 1);
    assert!(contract.narrow(&heap, &projected).is_ok());
    let Value::ObjectReference(object) = projected else {
        unreachable!()
    };
    let Value::Object { fields, .. } = heap.read_reference(&object.reference).unwrap() else {
        unreachable!()
    };
    assert_eq!(fields, [Value::Int32(42)]);
}

#[test]
fn narrowing_rejects_missing_membership_null_values_and_foreign_heap_handles() {
    let module = fixture();
    let contract = Contract::admit(
        &module,
        &Expression::Intersection(vec![nominal("Left"), nominal("Right")]),
    )
    .unwrap();
    let mut heap = ManagedHeap::default();
    let left = allocate(&mut heap, "LeftOnly");
    assert!(contract.narrow(&heap, &left).is_err());
    assert!(
        contract
            .narrow(&heap, &Value::NullObjectReference(Type::from_name("Both")))
            .is_err()
    );
    assert!(contract.narrow(&heap, &Value::Int32(42)).is_err());
    let mut foreign_heap = ManagedHeap::default();
    let foreign = allocate(&mut foreign_heap, "Both");
    assert!(contract.narrow(&heap, &foreign).is_err());
    let boxed = allocate(&mut heap, "ValueCell");
    assert!(contract.narrow(&heap, &boxed).is_err());
}

#[test]
fn admission_rejects_unknown_value_generic_byref_and_malformed_inputs() {
    let module = fixture();
    let bad = [
        nominal("Missing"),
        nominal("ValueCell"),
        nominal("Int32"),
        Expression::Nominal(Type::TypeParameter(0)),
        Expression::Nominal(Type::ByRef(Box::new(Type::from_name("Both")))),
        Expression::Nominal(Type::ReadOnlyByRef(Box::new(Type::from_name("Both")))),
        Expression::Nominal(Type::ArrayRef(Box::new(Type::Int32))),
        Expression::Union(vec![]),
        Expression::Intersection(vec![nominal("Both")]),
        Expression::Union(vec![nominal("Both"); 257]),
    ];
    for expression in bad {
        assert!(
            Contract::admit(&module, &expression).is_err(),
            "{expression:?}"
        );
    }
    let mut deep = nominal("Both");
    for _ in 0..34 {
        deep = Expression::Union(vec![nominal("Both"), deep]);
    }
    assert!(Contract::admit(&module, &deep).is_err());
    let wide = Expression::Union(vec![Expression::Union(vec![nominal("Both"); 256]); 17]);
    assert!(Contract::admit(&module, &wide).is_err());
}

#[test]
fn ordinary_interpreter_projections_preserve_identity_and_mutation() {
    let module = fixture_with_body(
        ".function Main() -> Boolean\n\
         .local Both cell\n.local Left left\n.local Right right\n\
         ldc.i4 1\nnewobj Both\nstloc cell\n\
         ldloc cell\ncastclass Left\nstloc left\n\
         ldloc cell\ncastclass Right\nstloc right\n\
         ldloc left\ncastclass Both\nldc.i4 42\nstfld Both::Number\n\
         ldloc right\ncastclass Both\nldfld Both::Number\nldc.i4 42\nceq\n\
         ldloc left\nldloc right\nref.eq\nand\nret\n.end",
    );
    let artifact = crate::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let program = crate::LoadedProgram::new(&artifact).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(crate::Limits::default()).unwrap().value,
        Value::Boolean(true)
    );
}

#[test]
fn interpreter_returned_object_satisfies_compound_without_a_wrapper() {
    let module = fixture_with_body(".function Main() -> Both\nldc.i4 42\nnewobj Both\nret\n.end");
    let program = crate::LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    let result = program.run(crate::Limits::default()).unwrap();
    let contract = Contract::admit(
        &module,
        &Expression::Intersection(vec![nominal("Left"), nominal("Right")]),
    )
    .unwrap();
    let narrowed = contract.narrow(&result.heap, &result.value).unwrap();
    assert!(crate::object_identity::reference_equals(&result.value, &narrowed).unwrap());
    assert_eq!(result.heap.len(), 1);
}
