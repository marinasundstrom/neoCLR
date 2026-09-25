//! Ownership and identity gates across VM conversions and teardown.
use super::StringValue;
use crate::{
    Execution, Fault, Limits, LoadedProgram, Module, Value, assembler::parse_function_ref,
};
use std::{
    process::Command,
    sync::{Arc, OnceLock},
};

fn library() -> &'static Module {
    static LIBRARY: OnceLock<Module> = OnceLock::new();
    LIBRARY.get_or_init(|| {
        let output = Command::new("python3")
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .args(["-c", "import runpy; m=runpy.run_path('docs/experiments/raven-target/collection_library.py'); print(m['build'](m['ROOT'] / 'runtime/System.neoil'))"])
            .output().unwrap();
        assert!(output.status.success());
        crate::assemble(std::str::from_utf8(&output.stdout).unwrap()).unwrap()
    })
}

pub(super) fn program(body: &str, declarations: &str, returns: &str) -> LoadedProgram {
    let source = format!(
        ".module StringOwnership\n{declarations}\n.function Test(String input) -> {returns}\n{body}\n.end"
    );
    let module = crate::assembler::read_modules(
        &[crate::assembler::ModuleInput::Source(&source)],
        library(),
    )
    .unwrap()
    .remove(0);
    // Exercise the persisted IL metadata path too.
    let module = serde_json::from_str(&serde_json::to_string(&module).unwrap()).unwrap();
    let program = LoadedProgram::with_library(&module, library()).unwrap();
    program.verify().unwrap();
    program
}

fn invoke(program: &LoadedProgram, text: StringValue) -> Result<Execution, Fault> {
    program
        .resolve_function(&parse_function_ref("Test(String)").unwrap())
        .unwrap()
        .invoke(
            vec![Value::String(text)],
            Limits {
                heap_objects: 4,
                ..Default::default()
            },
        )
}

fn assert_owner(program: &LoadedProgram) {
    for source in ["", "a\0b", "e\u{301}", "hello 👩‍💻"] {
        let original = StringValue::from(source);
        let observer = Arc::downgrade(&original.0);
        let execution = invoke(program, original.clone()).unwrap();
        let Value::String(returned) = execution.value.clone() else {
            panic!("expected text");
        };
        assert!(
            Arc::ptr_eq(&original.0, &returned.0),
            "owner changed for {source:?}"
        );
        assert_eq!(returned.as_str(), source);
        assert!(
            crate::object_identity::reference_equals(
                &Value::String(original.clone()),
                &execution.value
            )
            .unwrap()
        );
        assert_eq!(
            crate::object_identity::hash(&Value::String(original.clone())).unwrap(),
            crate::object_identity::hash(&execution.value).unwrap()
        );
        assert!(execution.heap.is_empty());
        let retained_hash = original.identity_hash();
        drop(original);
        drop(execution);
        // Host results survive the originating execution and its heap.
        assert_eq!(returned.as_str(), source);
        assert_eq!(returned.identity_hash(), retained_hash);
        assert!(observer.upgrade().is_some());
        drop(returned);
        assert!(observer.upgrade().is_none(), "owner leaked for {source:?}");
    }
}

#[test]
fn string_ownership_round_trips_object_interface_and_display() {
    for body in [
        "ldarg input\ncastclass System.Object\ncastclass String\nret",
        "ldarg input\nisinst System.Object\nisinst String\nret",
        "ldarg input\ncastclass System.EquatableTo<String>\ncastclass System.Object\ncastclass String\nret",
        "ldarg input\nisinst System.EquatableTo<String>\ncastclass String\nret",
        "ldarg input\ncastclass System.Object\ncallvirt instance System.Object::ToString()\nret",
    ] {
        assert_owner(&program(body, "", "String"));
    }
}

#[test]
fn string_ownership_survives_fields_arrays_erasure_and_byrefs() {
    let declarations =
        ".type Payload\n.field Text String\n.end\n.type class Holder\n.field Text String\n.end";
    for body in [
        ".local String text\nldarg input\nstloc text\nldloca text\nldobj String\nret",
        "ldarg input\nnewobj Payload\nldfld Payload::Text\nret",
        "ldarg input\nnewobj Holder\nldfld Holder::Text\nret",
        ".local arrayref<String> items\nldc.i4 1\nnewarr String\nstloc items\nldloc items\nldc.i4 0\nldarg input\nstelem String\nldloc items\nldc.i4 0\nldelem String\nret",
        "ldarg input\nvalue.pack String\nvalue.unpack String\nret",
    ] {
        assert_owner(&program(body, declarations, "String"));
    }
}

#[test]
fn string_ownership_survives_stack_only_wrapper_roots_under_pressure() {
    for opcode in ["castclass", "isinst"] {
        let mut body = format!("ldarg input\n{opcode} System.Object\n");
        for _ in 0..20 {
            body.push_str(&format!(
                "ldstr \"temporary\"\n{opcode} System.Object\npop\n"
            ));
        }
        body.push_str("castclass String\nret");
        let program = program(&body, "", "String");
        let original = StringValue::from("retained");
        let execution = invoke(&program, original.clone()).unwrap();
        assert!(execution.heap.collections() > 1);
        assert_eq!(execution.heap.statistics().reclaimed_objects, 21);
        let Value::String(returned) = &execution.value else {
            panic!("expected text");
        };
        assert!(Arc::ptr_eq(&original.0, &returned.0));
    }
}

#[test]
fn string_ownership_releases_cycles_on_completion_and_fault() {
    let declarations = ".type class Node\n.field Text String\n.field Next Node\n.end";
    let body = ".local Node empty\n.local Node node\nldloca empty\ninitobj Node\nldarg input\nldloc empty\nnewobj Node\nstloc node\nldloc node\nldloc node\nstfld Node::Next\n";
    for ending in ["ldvoid\nret", "fault \"intentional\""] {
        let program = program(&format!("{body}{ending}"), declarations, "Void");
        let text = StringValue::from("cycle-owned text");
        let observer = Arc::downgrade(&text.0);
        let result = invoke(&program, text);
        if ending.starts_with("fault") {
            assert!(result.unwrap_err().message.contains("intentional"));
        } else {
            let execution = result.unwrap();
            assert!(execution.heap.is_empty());
            assert_eq!(execution.heap.statistics().reclaimed_objects, 1);
        }
        assert!(
            observer.upgrade().is_none(),
            "cycle retained text after {ending}"
        );
    }
}

#[test]
fn string_ownership_does_not_intern_separate_equal_inputs() {
    let program = program(
        "ldarg input\ncastclass System.Object\ncastclass String\nret",
        "",
        "String",
    );
    // Compare Arc owners, not data pointers: empty strings can share a dangling buffer pointer.
    for source in ["", "same", "👩‍💻"] {
        let left = invoke(&program, source.into()).unwrap();
        let right = invoke(&program, source.into()).unwrap();
        let (Value::String(left), Value::String(right)) = (&left.value, &right.value) else {
            panic!("expected text");
        };
        assert_eq!(left, right);
        assert!(!Arc::ptr_eq(&left.0, &right.0));
        assert!(
            !crate::object_identity::reference_equals(
                &Value::String(left.clone()),
                &Value::String(right.clone())
            )
            .unwrap()
        );
    }
}
