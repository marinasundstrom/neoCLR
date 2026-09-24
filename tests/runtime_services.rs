use neoclr::{LoadedProgram, RuntimeService as Service, assemble, assembler::parse_function_ref};

#[test]
fn hello_requires_console_service_at_the_bound_import_declaration() {
    let module = assemble(include_str!("../examples/hello.neoil")).unwrap();
    let graph = LoadedProgram::new(&module)
        .unwrap()
        .analyze_reachability(&[parse_function_ref("Main()").unwrap()], 8)
        .unwrap();
    assert_eq!(graph.required_services(), [Service::ConsoleOutput]);
    let missing = graph.missing_services(&[]);
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].function, 2);
    assert_eq!(missing[0].instruction, None);
    assert_eq!(graph.functions[2].target.name, "neoCLR.Runtime.WriteLine");
    assert!(graph.missing_services(&[Service::ConsoleOutput]).is_empty());
}

#[test]
fn scalar_and_record_value_operations_do_not_imply_an_allocator() {
    let module = assemble(".module App\n.type Point\n.field X Int32\n.end\n.function Main() -> Point\nldc.i4 20\nldc.i4 22\nadd\nnewobj Point\nret\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    let graph = program
        .analyze_reachability(&[parse_function_ref("Main()").unwrap()], 1)
        .unwrap();
    assert!(graph.required_services().is_empty());
    assert!(graph.missing_services(&[]).is_empty());
}

#[test]
fn generic_memory_calls_report_each_instantiation_and_exact_use_location() {
    let module = assemble(".module App\n.type Storage<T>\n.method static Create() -> T*\nldc.i4 1\nheap.alloc T\nret\n.end\n.method static Clear(T* value) -> Void\nldarg value\ninitobj T\nldvoid\nret\n.end\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let roots = [
        "Storage<Int32>::Create()",
        "Storage<Byte>::Create()",
        "Storage<Int32>::Clear(Int32*)",
    ]
    .map(|name| parse_function_ref(name).unwrap());
    let graph = program.analyze_reachability(&roots, 3).unwrap();
    assert_eq!(
        graph.required_services(),
        [
            Service::NativeAllocation,
            Service::PointerMemory,
            Service::SlotReferences
        ]
    );
    let missing = graph.missing_services(&[Service::PointerMemory, Service::PointerMemory]);
    assert_eq!(
        missing
            .iter()
            .map(|m| (m.function, m.instruction, m.service))
            .collect::<Vec<_>>(),
        [
            (0, Some(1), Service::NativeAllocation),
            (1, Some(1), Service::NativeAllocation),
            (2, Some(1), Service::SlotReferences)
        ]
    );
    assert_eq!(graph.functions[2].services[0].instruction, Some(1));
    assert_eq!(
        graph.functions[2].services[0].service,
        Service::PointerMemory
    );
}

#[test]
fn frame_storage_and_managed_heap_share_reference_services() {
    let module = assemble(".module App\n.function Scratch() -> Void\nldc.i4 8\nlocalloc\npop\nldc.i4 1\nheap.new\nldobj Int32\npop\nldvoid\nret\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    let graph = program
        .analyze_reachability(&[parse_function_ref("Scratch()").unwrap()], 1)
        .unwrap();
    assert_eq!(
        graph.required_services(),
        [
            Service::FrameAllocation,
            Service::PointerMemory,
            Service::ManagedHeap,
            Service::SlotReferences
        ]
    );
    assert_eq!(
        graph
            .missing_services(&[Service::FrameAllocation, Service::PointerMemory])
            .iter()
            .map(|m| m.instruction)
            .collect::<Vec<_>>(),
        [Some(4), Some(4), Some(5)]
    );
    assert!(
        !graph
            .required_services()
            .contains(&Service::NativeAllocation)
    );
}

#[test]
fn services_follow_validated_bindings_rather_than_similar_names() {
    let module = assemble(
        ".module App\n.function neoCLR.Runtime.WriteLine(Int32 value) -> Void\nldvoid\nret\n.end",
    )
    .unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let roots = [
        "neoCLR.Runtime.WriteLine(Int32)",
        "neoCLR.Runtime.ParseInt32(String)",
        "neoCLR.Runtime.Int32ToString(Int32)",
    ]
    .map(|name| parse_function_ref(name).unwrap());
    let graph = program.analyze_reachability(&roots, 3).unwrap();
    assert!(graph.functions[0].services.is_empty());
    assert_eq!(
        graph.required_services(),
        [
            Service::ParseInt32,
            Service::FormatInt32,
            Service::ValueStorage
        ]
    );
    assert!(
        graph.functions[1..]
            .iter()
            .all(|f| f.services[0].instruction.is_none())
    );
}

#[test]
fn unreachable_memory_operations_and_native_imports_remain_conservative_requirements() {
    let module = assemble(".module App\n.function Main() -> Void\nldvoid\nret\nptr.null Int32\nldind.i4\npop\ncall Foreign()\n.end\n.function Foreign() -> Void\n.pinvoke \"nonexistent_service_fixture\" \"entry\" cdecl\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let graph = program
        .analyze_reachability(&[parse_function_ref("Main()").unwrap()], 2)
        .unwrap();
    assert_eq!(
        graph.required_services(),
        [Service::PointerMemory, Service::NativeInterop]
    );
    let missing = graph.missing_services(&[]);
    assert_eq!(missing[0].instruction, Some(3));
    assert_eq!(missing[1].instruction, None);
    assert_eq!(missing[1].function, 1);
    assert!(
        program
            .analyze_reachability(&[], 0)
            .unwrap()
            .required_services()
            .is_empty()
    );
}

#[test]
fn worker_notification_requires_worker_and_dispatch_services() {
    let module = assemble(concat!(
        ".module System\n",
        include_str!("../runtime/raven/generated/Func.methods.neoil"),
        include_str!("../runtime/neoCLR/Runtime/Workers.neoil")
    ))
    .unwrap();
    let graph = LoadedProgram::new(&module)
        .unwrap()
        .analyze_reachability(
            &[parse_function_ref("neoCLR.Runtime.NotifyWorker(Int32,System.Func<Void>)").unwrap()],
            1,
        )
        .unwrap();
    assert_eq!(
        graph.required_services(),
        [Service::TaskDispatch, Service::IsolatedWorkers]
    );
    for name in ["RequestWorkerCancellation", "JoinWorkerResult"] {
        let graph = LoadedProgram::new(&module).unwrap().analyze_reachability(
            &[parse_function_ref(&format!("neoCLR.Runtime.{name}(Int32)")).unwrap()], 1,
        ).unwrap();
        assert_eq!(graph.required_services(), if name == "JoinWorkerResult" {
            vec![Service::ValueStorage, Service::IsolatedWorkers]
        } else {
            vec![Service::IsolatedWorkers]
        });
    }
}

#[test]
fn socket_submission_requires_socket_and_dispatch_services() {
    let module = assemble(concat!(
        ".module System\n",
        include_str!("../runtime/raven/generated/Func.methods.neoil"),
        include_str!("../runtime/neoCLR/Runtime/Sockets.neoil")
    )).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    for signature in ["SocketConnectAddresses(arrayref<String>,Int32,System.Func<Void>)", "SocketAccept(Int64,System.Func<Void>)", "SocketConnect(String,Int32,System.Func<Void>)", "SocketReceive(Int64,arrayref<Byte>,Int32,Int32,System.Func<Void>)", "SocketSend(Int64,arrayref<Byte>,Int32,Int32,System.Func<Void>)"] {
        let graph = program.analyze_reachability(&[parse_function_ref(&format!("neoCLR.Runtime.{signature}")).unwrap()], 1).unwrap();
        assert!(graph.required_services().contains(&Service::SocketIo));
        assert!(graph.required_services().contains(&Service::TaskDispatch));
        assert!(!graph.required_services().contains(&Service::IsolatedWorkers));
    }
    let graph = program.analyze_reachability(&[parse_function_ref("neoCLR.Runtime.SocketClose(Int64)").unwrap()], 1).unwrap();
    assert!(graph.required_services().contains(&Service::SocketIo));
    assert!(!graph.required_services().contains(&Service::TaskDispatch));
}

#[test]
fn dns_uses_host_resolution_and_dispatch_without_guest_workers() {
    let module = assemble(concat!(
        ".module System\n",
        include_str!("../runtime/raven/generated/Func.methods.neoil"),
        include_str!("../runtime/neoCLR/Runtime/Dns.neoil")
    ))
    .unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let graph = program
        .analyze_reachability(
            &[parse_function_ref("neoCLR.Runtime.DnsLookup(String,System.Func<Void>)").unwrap()],
            1,
        )
        .unwrap();
    assert!(graph.required_services().contains(&Service::NameResolution));
    assert!(graph.required_services().contains(&Service::TaskDispatch));
    assert!(
        !graph
            .required_services()
            .contains(&Service::IsolatedWorkers)
    );
    assert!(!graph.required_services().contains(&Service::SocketIo));
    let graph = program
        .analyze_reachability(
            &[parse_function_ref("neoCLR.Runtime.DnsResult(Int64)").unwrap()],
            1,
        )
        .unwrap();
    assert!(graph.required_services().contains(&Service::NameResolution));
    assert!(!graph.required_services().contains(&Service::TaskDispatch));
}
