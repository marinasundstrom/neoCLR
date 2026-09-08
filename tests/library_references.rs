use neoclr::{Limits, LoadedProgram, Value, assemble, frontend};

fn run_source_helper(receiver: &str, heap: bool) -> neoclr::Execution {
    let source = format!(
        "func AddAndCount(values: {receiver}&) -> int {{\n values.Add(40)\n values.Add(2)\n return values.Count\n}}\nfunc Main() -> int {{ return 0 }}"
    );
    let il = frontend::lower_to_il(&source).unwrap();
    let helpers = &il[..il.rfind(".function Main(").unwrap()];
    let (local_type, allocation, address) = if heap {
        (
            "System.Collections.ArrayList<Int32>&",
            "heap.new\n",
            "ldloc owner",
        )
    } else {
        ("System.Collections.ArrayList<Int32>", "", "ldloca owner")
    };
    let view = if receiver.contains("ArrayList") {
        ""
    } else {
        "interface.borrow System.Collections.List<Int32>\n"
    };
    let module = assemble(&format!(
        "{helpers}\n.function Main() -> Int32\n.local {local_type} owner\nldc.i4 0\ncall System.Collections.ArrayList<Int32>::Allocate(Int32)\n{allocation}stloc owner\n{address}\n{view}call AddAndCount({receiver}&)\nret\n.end"
    )).unwrap();
    // The IL entry exercises both frame and heap wrapper receivers.
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    program.run(Limits::default()).unwrap()
}

#[test]
fn neo_library_methods_and_properties_use_frame_and_heap_reference_receivers() {
    for receiver in [
        "System.Collections.ArrayList<Int32>",
        "System.Collections.List<Int32>",
    ] {
        for heap in [false, true] {
            let execution = run_source_helper(receiver, heap);
            assert_eq!(execution.value, Value::Int32(2));
            assert_eq!(execution.memory.live_allocations(), 0);
            assert_eq!(
                execution.heap.statistics().allocated_objects,
                usize::from(heap) + 3
            );
        }
    }
}

#[test]
fn list_contract_requires_a_managed_receiver_and_retains_value_elements() {
    let system = neoclr::library::system().unwrap();
    for method in system.functions.iter().filter(|f| {
        f.instance
            && (f.name.starts_with("System.Collections.List.")
                || f.name.starts_with("System.Collections.ArrayList."))
    }) {
        assert!(method.receiver_byref, "{}", method.name);
    }
    let module = assemble(".module Invalid\n.entry Main\n.function Main() -> Int32\nldc.i4 0\ncall System.Collections.ArrayList<Int32>::Allocate(Int32)\ncall instance System.Collections.ArrayList<Int32>::get_Count()\nret\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    assert!(program.verify().is_err());
    assert!(program.run(Limits::default()).is_err());
}

#[test]
fn neo_borrows_addressable_library_values_and_rejects_immutable_mutation() {
    let source = "func Use(values: System.Collections.ArrayList<int>) -> int { var copy = values; copy.Add(42); return copy.Count }\nfunc Main() -> int { return 0 }";
    let il = frontend::lower_to_il(source).unwrap();
    assert!(il.contains("ldloca"));
    let immutable = "func Use(values: System.Collections.ArrayList<int>&) -> () { let copy: System.Collections.ArrayList<int> = values; copy.Add(42) }\nfunc Main() -> int { return 0 }";
    assert!(
        frontend::compile(immutable)
            .unwrap_err()
            .message
            .contains("immutable")
    );
}

#[test]
fn neo_keeps_interface_views_for_readonly_equality_dispatch() {
    let il = frontend::lower_to_il("func Equal(value: System.Equatable<int>&) -> bool { return value.Equals(42) }\nfunc Main() -> bool { return false }").unwrap();
    let helpers = &il[..il.rfind(".function Main(").unwrap()];
    let module = assemble(&format!("{helpers}\n.function Main() -> Boolean\n.local Int32 value\nldc.i4 42\nstloc value\nldloca value\ninterface.borrow System.Equatable<Int32>\ncall Equal(System.Equatable<Int32>&)\nret\n.end")).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Boolean(true)
    );
}
