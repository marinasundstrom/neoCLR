use neoclr::{Limits, LoadedProgram, Value, assemble, assembler::parse_function_ref, load};

const CONTRACT: &str = ".interface Read\n.method instance Get() -> Int32\n.end\n.end\n";
const CELL: &str = ".type Cell\n.implements Read\n.field Value Int32\n.method instance Get() -> Int32\nldarg this\nldfld Cell::Value\nret\n.end\n.end\n";

fn program(extra: &str, body: &str) -> Result<LoadedProgram, neoclr::Fault> {
    let module = assemble(&format!(
        ".module App\n.entry Main\n{CONTRACT}{extra}\n.function Main() -> Int32\n{body}\nret\n.end"
    ))?;
    LoadedProgram::new(&load(&serde_json::to_string(&module).unwrap())?)
}

fn receiver(allocation: &str) -> String {
    format!(".local Cell* p\n{allocation}\nstloc p\nldloc p\nldc.i4 42\nnewobj Cell\nstobj Cell\n")
}

#[test]
fn sample_roundtrips_dispatches_generic_contract_and_releases_storage() {
    let module = assemble(include_str!("../examples/interfaces.neoil")).unwrap();
    let program =
        LoadedProgram::new(&load(&serde_json::to_string(&module).unwrap()).unwrap()).unwrap();
    program.verify().unwrap();
    let result = program.run(Limits::default()).unwrap();
    assert_eq!(result.output, ["42", "2"]);
    assert_eq!(result.memory.live_allocations(), 0);
    let graph = program
        .analyze_reachability(&[parse_function_ref("Main()").unwrap()], 100)
        .unwrap();
    assert!(
        graph
            .required_services()
            .contains(&neoclr::RuntimeService::InterfaceDispatch)
    );
    assert!(
        graph
            .functions
            .iter()
            .any(|f| f.target.name == "System.Collections.ArrayList.get_Item")
    );
    assert!(
        !graph
            .functions
            .iter()
            .any(|f| f.target.name == "System.Collections.List.get_Item")
    );
}

#[test]
fn stack_and_heap_receivers_dispatch_without_allocating_a_box() {
    for allocation in [
        "sizeof Cell\nlocalloc\nptr.cast Cell",
        "ldc.i4 1\nheap.alloc Cell",
    ] {
        let cleanup = if allocation.contains("heap") {
            "ldloc p\nheap.free\npop"
        } else {
            ""
        };
        let body = format!(
            "{}ldloc p\ninterface.borrow Read\ncallvirt instance Read::Get()\n{cleanup}",
            receiver(allocation)
        );
        let p = program(CELL, &body).unwrap();
        p.verify().unwrap();
        let result = p.run(Limits::default()).unwrap();
        assert_eq!(result.value, Value::Int32(42));
        assert_eq!(result.memory.live_allocations(), 0);
    }
}

#[test]
fn borrowed_view_does_not_keep_heap_or_frame_storage_alive() {
    let heap = format!(
        ".local InterfaceRef<Read> view\n{}ldloc p\ninterface.borrow Read\nstloc view\nldloc p\nheap.free\npop\nldloc view\ncallvirt instance Read::Get()",
        receiver("ldc.i4 1\nheap.alloc Cell")
    );
    let escape = format!(
        "{CELL}.function Escape() -> InterfaceRef<Read>\n{}ldloc p\ninterface.borrow Read\nret\n.end",
        receiver("sizeof Cell\nlocalloc\nptr.cast Cell")
    );
    for (extra, body) in [
        (CELL.to_string(), heap),
        (
            escape,
            "call Escape()\ncallvirt instance Read::Get()".into(),
        ),
    ] {
        let p = program(&extra, &body).unwrap();
        p.verify().unwrap();
        let fault = p.run(Limits::default()).unwrap_err();
        assert!(
            fault.message.contains("free") || fault.message.contains("expired"),
            "{fault}"
        );
    }
}

#[test]
fn invalid_receiver_access_faults_at_dispatch() {
    for body in [
        "ptr.null Cell\ninterface.borrow Read\ncallvirt instance Read::Get()",
        "sizeof Cell\nlocalloc\nptr.cast Cell\ninterface.borrow Read\ncallvirt instance Read::Get()",
    ] {
        let p = program(CELL, body).unwrap();
        p.verify().unwrap();
        assert!(p.run(Limits::default()).is_err());
    }
}

#[test]
fn metadata_requires_complete_exact_public_instance_implementation() {
    assert!(program(CELL, "ldc.i4 0").is_ok());
    for invalid in [
        CELL.replace(".implements Read", ".implements Read\n.implements Read"),
        CELL.replace(
            ".method instance Get() -> Int32",
            ".method private instance Get() -> Int32",
        ),
        CELL.replace(
            ".method instance Get() -> Int32",
            ".method static Get() -> Int32",
        ),
        CELL.replace("Get() -> Int32", "Get() -> Void"),
        CELL.replace("Get() -> Int32", "Other() -> Int32"),
        CELL.replace(".implements Read", ".implements Cell"),
        ".interface Bad\n.field Value Int32\n.end".into(),
        ".interface Bad\n.implements Read\n.end".into(),
        ".interface Bad\n.method instance Get() -> Int32\nldc.i4 0\nret\n.end\n.end".into(),
    ] {
        assert!(program(&invalid, "ldc.i4 0").is_err(), "accepted {invalid}");
    }
}

#[test]
fn calls_require_explicit_matching_borrowed_interface_receiver() {
    for body in [
        "ldc.i4 42\nnewobj Cell\ncall instance Read::Get()",
        "ldc.i4 42\nnewobj Cell\ncallvirt instance Cell::Get()",
        "ldc.i4 42\nnewobj Cell\ncallvirt instance Read::Get()",
        "ldc.i4 42\ninterface.borrow Read\ncallvirt instance Read::Get()",
        "ptr.null Int32\ninterface.borrow Read\ncallvirt instance Read::Get()",
        "sizeof InterfaceRef<Read>",
    ] {
        let outcome = program(CELL, body).and_then(|p| {
            p.verify()?;
            p.run(Limits::default())
        });
        assert!(outcome.is_err(), "accepted {body}");
    }
}

#[test]
fn graph_includes_all_loaded_concrete_dispatch_targets() {
    let extra = format!(
        "{CELL}{}\n.function Invoke(InterfaceRef<Read> view) -> Int32\nldarg view\ncallvirt instance Read::Get()\nret\n.end",
        CELL.replace("Cell", "Other")
    );
    let p = program(&extra, "ldc.i4 0").unwrap();
    p.verify().unwrap();
    let graph = p
        .analyze_reachability(
            &[parse_function_ref("Invoke(InterfaceRef<Read>)").unwrap()],
            10,
        )
        .unwrap();
    assert_eq!(graph.functions[0].calls.len(), 2);
    assert_eq!(
        graph.functions[0].calls[0].instruction,
        graph.functions[0].calls[1].instruction
    );
    assert_eq!(graph.functions.len(), 3);
}

#[test]
fn graph_rejects_uninferable_generic_implementation_instead_of_omitting_it() {
    let extra = ".type Holder<T>\n.implements Read\n.method instance Get() -> Int32\nldc.i4 42\nret\n.end\n.end\n.function Invoke(InterfaceRef<Read> view) -> Int32\nldarg view\ncallvirt instance Read::Get()\nret\n.end";
    let p = program(extra, "ldc.i4 0").unwrap();
    p.verify().unwrap();
    let fault = p
        .analyze_reachability(
            &[parse_function_ref("Invoke(InterfaceRef<Read>)").unwrap()],
            10,
        )
        .unwrap_err();
    assert!(fault.message.contains("cannot infer"), "{fault}");
}

#[test]
fn inline_receiver_fields_keep_existing_copy_semantics() {
    let extra = ".interface Change\n.method instance Set(Int32 value) -> Void\n.end\n.end\n.type Counter\n.implements Change\n.field Value Int32\n.method instance Set(Int32 value) -> Void\nldarg this\nldarg value\nstfld Counter::Value\nstarg this\nldvoid\nret\n.end\n.end";
    let body = ".local Counter* p\nsizeof Counter\nlocalloc\nptr.cast Counter\nstloc p\nldloc p\nldc.i4 7\nnewobj Counter\nstobj Counter\nldloc p\ninterface.borrow Change\nldc.i4 42\ncallvirt instance Change::Set(Int32)\npop\nldloc p\nldobj Counter\nldfld Counter::Value";
    let p = program(extra, body).unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(7));
}

#[test]
fn interface_indexer_setter_updates_shared_list_storage() {
    // include_str! preserves checkout line endings. Normalize before editing,
    // then exercise both input encodings so this test also covers Windows.
    let sample = include_str!("../examples/interfaces.neoil").replace("\r\n", "\n");
    let marker =
        "    ldloca owner\n    interface.borrow System.Collections.List<Int32>\n    call Sum";
    assert_eq!(
        sample.matches(marker).count(),
        1,
        "sample edit marker changed"
    );
    let source = sample.replace(
        marker,
        "    ldloca owner\n    interface.borrow System.Collections.List<Int32>\n    ldc.i4 0\n    ldc.i4 30\n    callvirt instance System.Collections.List<Int32>::set_Item(Int32,Int32)\n    pop\n    ldloca owner\n    interface.borrow System.Collections.List<Int32>\n    call Sum"
    );
    for newline in ["\n", "\r\n"] {
        let source = source.replace('\n', newline);
        let p = LoadedProgram::new(&assemble(&source).unwrap()).unwrap();
        p.verify().unwrap();
        assert_eq!(p.run(Limits::default()).unwrap().output, ["52", "2"]);
    }
}

#[test]
fn exact_interface_identity_is_checked_even_for_identical_signatures() {
    let extra = format!("{CELL}{}", CONTRACT.replace("Read", "Other"));
    let body = format!(
        "{}ldloc p\ninterface.borrow Read\ncallvirt instance Other::Get()",
        receiver("sizeof Cell\nlocalloc\nptr.cast Cell")
    );
    let p = program(&extra, &body).unwrap();
    assert!(p.verify().is_err());
    assert!(p.run(Limits::default()).is_err());
}

#[test]
fn overloads_dispatch_using_the_full_parameter_signature() {
    let extra = ".interface Choose\n.method instance Get(Int32 value) -> Int32\n.end\n.method instance Get(Boolean value) -> Int32\n.end\n.end\n.type Choice\n.implements Choose\n.method instance Get(Int32 value) -> Int32\nldarg value\nret\n.end\n.method instance Get(Boolean value) -> Int32\nldc.i4 9\nret\n.end\n.end";
    let body = ".local InterfaceRef<Choose> view\n.local Choice* p\nsizeof Choice\nlocalloc\nptr.cast Choice\nstloc p\nldloc p\nnewobj Choice\nstobj Choice\nldloc p\ninterface.borrow Choose\nstloc view\nldloc view\nldc.i4 33\ncallvirt instance Choose::Get(Int32)\nldloc view\nldc.bool true\ncallvirt instance Choose::Get(Boolean)\nadd";
    let p = program(extra, body).unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}

#[test]
fn faults_report_concrete_callee_without_an_abstract_frame() {
    let extra = CELL.replace("ldarg this\nldfld Cell::Value\nret", "fault \"broken\"");
    let body = format!(
        "{}ldloc p\ninterface.borrow Read\ncallvirt instance Read::Get()",
        receiver("sizeof Cell\nlocalloc\nptr.cast Cell")
    );
    let p = program(&extra, &body).unwrap();
    p.verify().unwrap();
    let fault = p.run(Limits::default()).unwrap_err();
    assert_eq!(fault.message, "broken");
    let frames = fault.stack_trace.unwrap().frames;
    assert_eq!(
        frames
            .iter()
            .map(|f| f.function.name.as_str())
            .collect::<Vec<_>>(),
        ["Cell.Get", "Main"]
    );
}

#[test]
fn interface_views_cannot_be_reimported_through_host_erasure() {
    let extra = format!(
        "{CELL}\n.function Create() -> InterfaceRef<Read>\nptr.null Cell\ninterface.borrow Read\nret\n.end\n.function Echo(System.Value value) -> System.Value\nldarg value\nret\n.end\n.function Direct(InterfaceRef<Read> value) -> InterfaceRef<Read>\nldarg value\nret\n.end"
    );
    let p = program(&extra, "ldc.i4 0").unwrap();
    let view = p
        .resolve_function(&parse_function_ref("Create()").unwrap())
        .unwrap()
        .invoke(vec![], Limits::default())
        .unwrap()
        .value;
    let echo = p
        .resolve_function(&parse_function_ref("Echo(System.Value)").unwrap())
        .unwrap();
    assert!(
        echo.invoke(vec![Value::Erased(Box::new(view))], Limits::default())
            .is_err()
    );
    assert!(
        p.resolve_function(&parse_function_ref("Direct(InterfaceRef<Read>)").unwrap())
            .is_err()
    );
    assert!(
        p.resolve_function(&parse_function_ref("instance Read::Get()").unwrap())
            .is_err()
    );
}

#[test]
fn module_references_and_internal_contract_visibility_are_enforced() {
    fn prepare(app: &str, contract: &str) -> Result<LoadedProgram, neoclr::Fault> {
        let modules = neoclr::assembler::assemble_modules(&[app, contract])?;
        LoadedProgram::with_modules(&modules[0], neoclr::library::system()?, &modules[1..])
    }
    let contract = format!(".module Contracts\n{CONTRACT}");
    let app = format!(".module App\n.references (Contracts)\n{CELL}");
    prepare(&app, &contract).unwrap().verify().unwrap();
    assert!(
        prepare(&app.replace("(Contracts)", "()"), &contract)
            .unwrap_err()
            .message
            .contains("does not reference")
    );
    assert!(
        prepare(
            &app,
            &contract.replace(".interface Read", ".interface internal Read")
        )
        .is_err()
    );
    let consumer = ".module App\n.references (Contracts)\n.function Invoke(InterfaceRef<[Contracts]Read> view) -> Int32\nldarg view\ncallvirt instance [Contracts]Read::Get()\nret\n.end";
    prepare(consumer, &contract).unwrap().verify().unwrap();
    assert!(prepare(&consumer.replace("(Contracts)", "()"), &contract).is_err());
}

#[test]
fn managed_interface_receiver_mutates_inline_fields_through_forwarded_calls() {
    let extra = ".interface Change\n.method instance byref Set(Int32 value) -> Void\n.end\n.end\n.type Counter\n.implements Change\n.field Value Int32\n.method instance byref Set(Int32 value) -> Void\nldarg this\nldarg value\nnewobj Counter\nstobj Counter\nldvoid\nret\n.end\n.end\n.function Forward(Change& view) -> Void\nldarg view\nldc.i4 42\ncallvirt instance Change::Set(Int32)\nret\n.end";
    let p = program(extra, ".local Counter value\nldc.i4 7\nnewobj Counter\nstloc value\nldloca value\ninterface.borrow Change\ncall Forward(Change&)\npop\nldloc value\nldfld Counter::Value").unwrap();
    p.verify().unwrap();
    let result = p.run(Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.memory.live_allocations(), 0);
    assert!(
        program(
            &extra.replace(
                ".method instance byref Set(Int32 value) -> Void\nldarg this",
                ".method instance Set(Int32 value) -> Void\nldarg this"
            ),
            "ldc.i4 0"
        )
        .is_err()
    );
}

#[test]
fn managed_interface_views_support_non_native_records_but_cannot_escape() {
    let extra = ".interface Length\n.method instance Count() -> Int32\n.end\n.end\n.type Text\n.implements Length\n.field Value String\n.method instance Count() -> Int32\nldarg this\nldfld Text::Value\ncall instance System.String::GetUtf8ByteCount()\nret\n.end\n.end";
    let prefix = ".local Text value\nldstr \"abc\"\nnewobj Text\nstloc value\nldloca value\ninterface.borrow Length\n";
    let p = program(extra, &format!("{prefix}callvirt instance Length::Count()")).unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(3));
    for suffix in [
        "heap.new\npop\nldc.i4 0",
        "value.pack Length&\npop\nldc.i4 0",
    ] {
        assert!(
            program(extra, &format!("{prefix}{suffix}"))
                .and_then(|p| p.run(Limits::default()))
                .is_err()
        );
    }
    let uninit = program(extra, ".local Text value\nldloca value\ninterface.borrow Length\ncallvirt instance Length::Count()").unwrap();
    assert!(uninit.verify().is_err());
    assert!(uninit.run(Limits::default()).is_err());
}

#[test]
fn raw_interface_views_cannot_supply_managed_reference_receivers() {
    let extra = ".interface Change\n.method instance byref Set() -> Int32\n.end\n.end\n.type Counter\n.implements Change\n.method instance byref Set() -> Int32\nldc.i4 42\nret\n.end\n.end";
    let p = program(
        extra,
        "ptr.null Counter\ninterface.borrow Change\ncallvirt instance Change::Set()",
    )
    .unwrap();
    assert!(p.verify().is_err());
    assert!(
        p.run(Limits::default())
            .unwrap_err()
            .message
            .contains("managed slot")
    );
}

#[test]
fn managed_dispatch_preserves_conditional_output_contracts_and_receiver_mutation() {
    let extra = r#"
.interface Source
    .method instance byref Try(out(true) Int32& destination, Boolean success) -> Boolean
    .end
.end
.type Counter
    .implements Source
    .field Value Int32
    .method instance byref Try(out(true) Int32& destination, Boolean success) -> Boolean
        ldarg success
        brfalse Miss
        ldarg destination
        ldarg this
        ldobj Counter
        ldfld Counter::Value
        stobj Int32
        ldarg this
        ldc.i4 7
        newobj Counter
        stobj Counter
        ldc.bool true
        ret
    Miss:
        ldc.bool false
        ret
    .end
.end
"#;
    for success in [true, false] {
        let body = format!(
            ".local Counter counter\n.local Int32 output\nldc.i4 42\nnewobj Counter\nstloc counter\nldloca counter\ninterface.borrow Source\nldloca output\nldc.bool {success}\ncallvirt instance Source::Try(Int32&,Boolean)\nbrfalse Miss\nldloc output\nldloc counter\nldfld Counter::Value\nadd\nret\nMiss:\nldloc counter\nldfld Counter::Value"
        );
        let p = program(extra, &body).unwrap();
        p.verify().unwrap();
        assert_eq!(
            p.run(Limits::default()).unwrap().value,
            Value::Int32(if success { 49 } else { 42 })
        );
    }
    // Neither ordinary ref nor unconditional out matches conditional out.
    for mode in ["", "out "] {
        let mismatched = extra.replacen("Try(out(true) Int32&", &format!("Try({mode}Int32&"), 1);
        assert!(program(&mismatched, "ldc.i4 0").is_err());
    }
    let unwritten = extra.replace("        stobj Int32", "        pop\n        pop");
    let p = program(&unwritten, ".local Counter counter\n.local Int32 output\nldc.i4 42\nnewobj Counter\nstloc counter\nldloca counter\ninterface.borrow Source\nldloca output\nldc.bool true\ncallvirt instance Source::Try(Int32&,Boolean)\npop\nldc.i4 0").unwrap();
    p.verify().unwrap();
    let fault = p.run(Limits::default()).unwrap_err();
    assert!(fault.message.contains("out parameter"), "{fault}");
    assert_eq!(fault.function.as_deref(), Some("Counter.Try"));
}
