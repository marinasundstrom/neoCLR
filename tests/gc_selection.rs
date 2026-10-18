use neoclr::{ExecutionOptions, GarbageCollector, LoadedProgram, Value, assemble};
use std::process::Command;

#[test]
fn explicit_collector_is_execution_local_and_preserves_collection() {
    let module = assemble(".module App\n.entry Main\n.function Main() -> Int32\nldc.i4 42\nheap.new\nldobj Int32\nret\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    for collector in GarbageCollector::available() {
        let execution = program
            .run(ExecutionOptions {
                garbage_collector: *collector,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(execution.value, Value::Int32(42));
        assert_eq!(execution.heap.collector(), *collector);
        assert!(execution.heap.is_empty());
        assert_eq!(execution.heap.statistics().reclaimed_objects, 1);
    }
}

#[test]
fn cli_selects_current_collector_and_rejects_missing_or_unavailable_choices() {
    let run = |flags: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_neoclr"))
            .args(["run", "examples/hello.neoil", "--execution", "interpreter"])
            .args(flags)
            .output()
            .unwrap()
    };
    let default = run(&[]);
    let selected = run(&["--gc", "mark-sweep"]);
    assert!(default.status.success() && selected.status.success());
    assert_eq!(default.stdout, selected.stdout);
    assert_eq!(default.stderr, selected.stderr);
    for flags in [
        vec!["--gc"],
        vec!["--gc", "concurrent"],
        vec!["--gc", "none"],
        vec!["--gc", "mark-sweep", "--gc", "mark-sweep"],
    ] {
        let result = run(&flags);
        assert!(!result.status.success(), "{flags:?}");
        assert!(result.stdout.is_empty());
    }
    assert_eq!(GarbageCollector::default().name(), "mark-sweep");
    assert!(
        "concurrent"
            .parse::<GarbageCollector>()
            .unwrap_err()
            .contains("available: mark-sweep")
    );
}
