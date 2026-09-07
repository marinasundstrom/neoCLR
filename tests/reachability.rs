use neoclr::{
    FunctionImplementation, LoadedProgram, assemble,
    assembler::{assemble_modules, parse_function_ref},
    library,
};

#[test]
fn hello_graph_follows_library_wrappers_to_runtime_imports_without_execution() {
    let module = assemble(include_str!("../examples/hello.neoil")).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let root = parse_function_ref("Main()").unwrap();
    let graph = program
        .analyze_reachability(&[root.clone(), root], 3)
        .unwrap();
    assert_eq!(graph.roots, [0, 0]);
    assert_eq!(graph.functions.len(), 3);
    assert_eq!(graph.functions[0].calls[0].instruction, 1);
    assert_eq!(graph.functions[0].calls[0].target, 1);
    assert_eq!(graph.functions[1].target.name, "System.Console.WriteLine");
    assert_eq!(graph.functions[1].calls[0].target, 2);
    assert_eq!(graph.functions[2].target.name, "neoCLR.Runtime.WriteLine");
    assert!(matches!(
        graph.functions[2].implementation,
        FunctionImplementation::InternalCall
    ));
    assert!(graph.functions[2].calls.is_empty());
    assert_eq!(
        graph.functions[2]
            .target
            .definition
            .as_ref()
            .unwrap()
            .module,
        "System"
    );
}

#[test]
fn recursive_calls_deduplicate_but_syntactically_unreachable_calls_are_retained() {
    let module = assemble(".module App\n.function A() -> Void\ncall B()\nret\ncall Native()\n.end\n.function B() -> Void\ncall A()\nret\n.end\n.function Native() -> Void\n.pinvoke \"nonexistent_reachability_library\" \"entry\" cdecl\n.end\n.function Unused() -> Void\nfault \"do not execute\"\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let graph = program
        .analyze_reachability(&[parse_function_ref("A()").unwrap()], 3)
        .unwrap();
    assert_eq!(
        graph
            .functions
            .iter()
            .map(|f| f.target.name.as_str())
            .collect::<Vec<_>>(),
        ["A", "B", "Native"]
    );
    assert_eq!(graph.functions[1].calls[0].target, 0);
    assert_eq!(graph.functions[0].calls[1].instruction, 2);
    let FunctionImplementation::NativeImport(import) = &graph.functions[2].implementation else {
        panic!("expected import")
    };
    assert_eq!(import.library, "nonexistent_reachability_library");
    assert_eq!(import.entry_point, "entry");
}

#[test]
fn bound_generic_overloads_keep_definition_identity_after_specialization() {
    let module = assemble(include_str!("../examples/member-identities.neoil")).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let graph = program
        .analyze_reachability(&[parse_function_ref("Main()").unwrap()], 16)
        .unwrap();
    let descriptions = graph
        .functions
        .iter()
        .filter(|f| f.target.name == "Choice.Describe")
        .collect::<Vec<_>>();
    assert_eq!(descriptions.len(), 2);
    assert_eq!(
        descriptions[0].target.parameters,
        descriptions[1].target.parameters
    );
    assert_ne!(
        descriptions[0].target.definition,
        descriptions[1].target.definition
    );
    let forward = graph
        .functions
        .iter()
        .find(|f| f.target.name == "Choice.Forward")
        .unwrap();
    let selected = &graph.functions[forward.calls[0].target];
    assert_eq!(selected.target.definition.as_ref().unwrap().index, 0);
}

#[test]
fn closed_owners_are_distinct_and_open_roots_are_rejected() {
    let module = assemble(include_str!("../examples/instance_invocation.neoil")).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let roots = ["instance Box<Int32>::Get()", "instance Box<String>::Get()"]
        .map(|s| parse_function_ref(s).unwrap());
    let graph = program.analyze_reachability(&roots, 2).unwrap();
    assert_eq!(graph.roots, [0, 1]);
    assert_eq!(
        graph.functions[0].target.definition,
        graph.functions[1].target.definition
    );
    assert_ne!(
        graph.functions[0].target.owner,
        graph.functions[1].target.owner
    );
    assert_ne!(graph.functions[0].returns, graph.functions[1].returns);
    assert!(
        program
            .analyze_reachability(&[parse_function_ref("instance Box<!0>::Get()").unwrap()], 2)
            .is_err()
    );
}

#[test]
fn transitive_calls_use_their_declaring_module_references_and_revisions() {
    let sources = [
        ".module App\n.references (Helpers#r1)\n.function Main() -> Void\ncall Help()\nret\n.end",
        ".module Helpers\n.revision r1\n.references (Hidden#r2)\n.function Help() -> Void\ncall HiddenCall()\nret\n.end",
        ".module Hidden\n.revision r2\n.references ()\n.function HiddenCall() -> Void\nldvoid\nret\n.end",
    ];
    let modules = assemble_modules(&sources).unwrap();
    let program =
        LoadedProgram::with_modules(&modules[0], library::system().unwrap(), &modules[1..])
            .unwrap();
    let graph = program
        .analyze_reachability(&[parse_function_ref("Main()").unwrap()], 3)
        .unwrap();
    assert_eq!(
        graph.functions[2]
            .target
            .definition
            .as_ref()
            .unwrap()
            .revision
            .as_deref(),
        Some("r2")
    );
    assert!(
        program
            .analyze_reachability(&[parse_function_ref("HiddenCall()").unwrap()], 3)
            .unwrap_err()
            .message
            .contains("does not reference")
    );
    assert!(
        program
            .analyze_reachability(
                &[parse_function_ref("Help() @ Helpers#stale:0").unwrap()],
                3
            )
            .is_err()
    );
    let reversed = LoadedProgram::with_modules(
        &modules[0],
        library::system().unwrap(),
        &[modules[2].clone(), modules[1].clone()],
    )
    .unwrap();
    let other = reversed
        .analyze_reachability(&[parse_function_ref("Main()").unwrap()], 3)
        .unwrap();
    assert_eq!(
        graph
            .functions
            .iter()
            .map(|f| (&f.target, &f.calls))
            .collect::<Vec<_>>(),
        other
            .functions
            .iter()
            .map(|f| (&f.target, &f.calls))
            .collect::<Vec<_>>()
    );
}

#[test]
fn expanding_generic_calls_fail_at_the_bound_without_returning_a_partial_graph() {
    let module = assemble(".module App\n.type Grow<T>\n.method static Next() -> Void\ncall Grow<Grow<T>>::Next()\nret\n.end\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let root = parse_function_ref("Grow<Int32>::Next()").unwrap();
    let fault = program.analyze_reachability(&[root], 4).unwrap_err();
    assert!(fault.message.contains("function limit"));
    assert_eq!(fault.function.as_deref(), Some("Grow.Next"));
    assert_eq!(fault.instruction, Some(0));
    assert!(
        program
            .analyze_reachability(&[], 0)
            .unwrap()
            .functions
            .is_empty()
    );
    assert!(
        program
            .analyze_reachability(&[parse_function_ref("Grow<Int32>::Next()").unwrap()], 0)
            .is_err()
    );
}

#[test]
fn pointer_signatures_and_native_roots_do_not_require_host_input_schemas() {
    let module = assemble(".module App\n.function Identity(Int32* value) -> Int32*\nldarg value\nret\n.end\n.function Native() -> Void\n.pinvoke \"never_loaded\" \"entry\" cdecl\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let graph = program
        .analyze_reachability(
            &[
                parse_function_ref("Identity(Int32*)").unwrap(),
                parse_function_ref("Native()").unwrap(),
            ],
            2,
        )
        .unwrap();
    assert_eq!(graph.functions.len(), 2);
    assert!(matches!(
        graph.functions[1].implementation,
        FunctionImplementation::NativeImport(_)
    ));
}
