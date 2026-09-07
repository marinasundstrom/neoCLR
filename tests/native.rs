use neoclr::{
    Limits, Value, assemble, library, load,
    metadata::{INTERNAL_CALL, Instruction},
    run_with_library,
};

#[test]
fn methodimpl_internalcall_round_trips_as_clr_flag() {
    let compiled = assemble(include_str!("../runtime/System.neoil")).unwrap();
    let json = serde_json::to_value(&compiled).unwrap();
    let declared = json["functions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "neoCLR.Runtime.WriteLine")
        .unwrap();
    assert_eq!(declared["impl_flags"], 0x1000);
    assert_eq!(declared["body"], serde_json::json!([]));
    let loaded = load(&json.to_string()).unwrap();
    assert_eq!(
        loaded
            .functions
            .iter()
            .filter(|f| f.impl_flags == INTERNAL_CALL)
            .count(),
        8
    );
}

#[test]
fn native_registry_checks_full_signature_and_implementation_shape() {
    for declaration in [
        ".function Unknown() -> Void\n.methodimpl InternalCall",
        ".function neoCLR.Runtime.WriteLine(int32) -> Void\n.methodimpl InternalCall",
        ".function neoCLR.Runtime.WriteLine(string) -> Int32\n.methodimpl InternalCall",
        ".function neoCLR.Runtime.WriteLine(string) -> Void\n.methodimpl InternalCall\nldvoid\nret",
        ".function neoCLR.Runtime.WriteLine(string) -> Void\n.local Void\n.methodimpl InternalCall",
        ".function neoCLR.Runtime.WriteLine(string) -> Void\n.methodimpl InternalCall\n.methodimpl InternalCall",
        ".function neoCLR.Runtime.WriteLine(string) -> Void\n.methodimpl Native",
    ] {
        assert!(
            assemble(&format!(".module System\n{declaration}\n.end")).is_err(),
            "{declaration}"
        );
    }
}

#[test]
fn a_native_name_without_the_flag_executes_its_il_body() {
    let mut system = library::system().unwrap().clone();
    let function = system
        .functions
        .iter_mut()
        .find(|f| f.name == "neoCLR.Runtime.WriteLine")
        .unwrap();
    function.impl_flags = 0;
    function.body = vec![Instruction::Void, Instruction::Return];
    let loaded = load(&serde_json::to_string(&system).unwrap()).unwrap();
    let hello = assemble(include_str!("../examples/hello.neoil")).unwrap();
    let execution = run_with_library(&hello, &loaded, Limits::default()).unwrap();
    assert!(execution.output.is_empty());
    assert_eq!(execution.value, Value::Void);
}

#[test]
fn missing_native_declarations_are_not_resolved_by_name() {
    let mut system = library::system().unwrap().clone();
    system
        .functions
        .retain(|f| f.name != "neoCLR.Runtime.WriteLine");
    // Rebuild row identities after changing the definition table.
    for (index, function) in system.functions.iter_mut().enumerate() {
        function.definition.as_mut().unwrap().index = index as u32;
    }
    assert!(
        load(&serde_json::to_string(&system).unwrap())
            .unwrap_err()
            .message
            .contains("unknown function overload")
    );
}

#[test]
fn unknown_implementation_flags_are_rejected_by_loader() {
    let base = serde_json::to_value(library::system().unwrap()).unwrap();
    for flags in [1, 3, 0x1001, 0x8000] {
        let mut json = base.clone();
        json["functions"][0]["impl_flags"] = flags.into();
        assert!(
            load(&json.to_string())
                .unwrap_err()
                .message
                .contains("unsupported method implementation flags")
        );
    }
}
