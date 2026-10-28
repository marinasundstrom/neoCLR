use neoclr::{Limits, LoadedProgram, Value, assemble};

#[test]
fn generic_interface_dispatch_preserves_method_state_and_closed_graph() {
    let source = include_str!("../examples/generic-interfaces.neoil");
    let module = assemble(source).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
    let graph = program
        .analyze_reachability(
            &[neoclr::assembler::parse_function_ref("Main()").unwrap()],
            100,
        )
        .unwrap();
    assert!(
        graph
            .functions
            .iter()
            .any(|f| f.target.name == "Identity.Apply"
                && f.target.generic_arguments == [neoclr::metadata::Type::Int32])
    );
}

#[test]
fn generic_interface_conformance_rejects_different_arity() {
    let source = include_str!("../examples/generic-interfaces.neoil");
    for replacement in ["Apply<U,V>(U value) -> U", "Apply(T value) -> T"] {
        let index = source.find(".type class Identity").unwrap();
        let broken = format!(
            "{}{}",
            &source[..index],
            source[index..].replace("Apply<T>(T value) -> T", replacement)
        );
        assert!(assemble(&broken).is_err());
    }
}

#[test]
fn multiple_method_instantiations_keep_independent_state() {
    let source = include_str!("../examples/generic-interfaces.neoil").replace("ldc.i4 42\ncallvirt", "ldstr \"hello\"\ncallvirt instance Echo::Apply<String>(String)\npop\nnewobj Identity\ncastclass Echo\nldc.i4 42\ncallvirt");
    let module = assemble(&source).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn generic_owner_and_inherited_contract_keep_separate_parameter_spaces() {
    let source = include_str!("../examples/generic-interfaces.neoil")
        .replace(".interface Echo", ".interface Echo<U>")
        .replace(
            ".type class Identity",
            ".interface Derived<V>\n.implements Echo<V>\n.end\n.type class Identity<W>",
        )
        .replace(".implements Echo\n", ".implements Derived<W>\n")
        .replace("newobj Identity", "newobj Identity<String>")
        .replace("castclass Echo", "castclass Echo<String>")
        .replace("Echo::Apply", "Echo<String>::Apply");
    let module = assemble(&source).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn unsupported_generic_dispatch_categories_are_still_rejected() {
    let valid = assemble(include_str!("../examples/generic-interfaces.neoil")).unwrap();
    let contract = valid
        .functions
        .iter()
        .position(|f| f.name == "Echo.Apply")
        .unwrap();
    let implementation = valid
        .functions
        .iter()
        .position(|f| f.name == "Identity.Apply")
        .unwrap();
    let mut virtual_class = valid.clone();
    virtual_class.functions[implementation].is_virtual = true;
    assert!(neoclr::verify(&virtual_class).is_err());
    let mut constrained = valid.clone();
    constrained.functions[contract]
        .generic_constraints
        .push(neoclr::metadata::GenericConstraint {
            parameter: 0,
            kind: neoclr::metadata::ConstraintKind::ReferenceType,
        });
    assert!(neoclr::verify(&constrained).is_err());
    let mut stronger_implementation = valid.clone();
    stronger_implementation.functions[implementation]
        .generic_constraints
        .push(neoclr::metadata::GenericConstraint {
            parameter: 0,
            kind: neoclr::metadata::ConstraintKind::ReferenceType,
        });
    assert!(neoclr::verify(&stronger_implementation).is_err());
    let mut static_contract = valid;
    static_contract.functions[contract].instance = false;
    assert!(neoclr::verify(&static_contract).is_err());
}
