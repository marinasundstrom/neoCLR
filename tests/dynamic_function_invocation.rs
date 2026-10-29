use neoclr::{Limits, LoadedProgram, Value, assemble};
const SOURCE: &str = include_str!("../docs/validation/dynamic-function-invocation/Main.neoil");
fn run(source: &str) -> Result<neoclr::Execution, neoclr::Fault> {
    static LIBRARY: std::sync::OnceLock<neoclr::Module> = std::sync::OnceLock::new();
    let library = LIBRARY.get_or_init(|| {
        let output = std::process::Command::new("python3").current_dir(env!("CARGO_MANIFEST_DIR"))
            .args(["-c", "import runpy; m=runpy.run_path('docs/experiments/raven-target/collection_library.py'); print(m['build'](m['ROOT'] / 'runtime/legacy/bootstrap/System.neoil'))"])
            .output().unwrap();
        assert!(output.status.success());
        assemble(std::str::from_utf8(&output.stdout).unwrap()).unwrap()
    });
    let mut modules = neoclr::assembler::read_modules(
        &[neoclr::assembler::ModuleInput::Source(source)],
        library,
    )?;
    let program = LoadedProgram::with_library(&modules.remove(0), library)?;
    program.verify()?;
    program.run(Limits::default())
}
#[test]
fn bound_receiver_and_boxed_result() {
    let result = run(SOURCE).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.heap.len(), 0);
}
#[test]
fn wrong_count_and_wrong_box_reject_before_call() {
    let wrong_count = SOURCE.replace("ldc.i4 1\narray.reserve", "ldc.i4 2\narray.reserve");
    assert!(
        run(&wrong_count)
            .unwrap_err()
            .message
            .contains("count mismatch")
    );
    let wrong_box = SOURCE.replace("ldc.i4 2\nbox Int32", "ldc.bool true\nbox Boolean");
    assert_eq!(
        run(&wrong_box).unwrap_err().code,
        neoclr::FaultCode::InvalidCast
    );
}

#[test]
fn null_callback_and_null_arguments_reject() {
    let null_function = SOURCE.replace(
        "ldloc callback\nldloc arguments",
        "ldloca callback\ninitobj System.Object\nldloc callback\nldloc arguments",
    );
    assert!(
        run(&null_function)
            .unwrap_err()
            .message
            .contains("non-null function")
    );
    let null_arguments = SOURCE.replace(
        "ldloc callback\nldloc arguments",
        "ldloca arguments\ninitobj arrayref<System.Object>\nldloc callback\nldloc arguments",
    );
    assert!(
        run(&null_arguments)
            .unwrap_err()
            .message
            .contains("non-null Object array")
    );
}

#[test]
fn target_fault_is_not_wrapped() {
    let source = SOURCE.replace("ldarg 1\nadd\nret", "fault \"dynamic target entered\"");
    assert!(
        run(&source)
            .unwrap_err()
            .message
            .contains("dynamic target entered")
    );
}

#[test]
fn invocation_check_returns_errors_without_entering_target() {
    let prefix = SOURCE.split(".function Main()").next().unwrap();
    let header = ".function neoCLR.Runtime.InvokeFunctionCheck(System.Object, arrayref<System.Object>) -> Int32\n.methodimpl InternalCall\n.end\n";
    for (count, value, expected) in [
        (1, "ldc.i4 2\nbox Int32", 0),
        (2, "ldc.i4 2\nbox Int32", 8),
        (1, "ldc.bool true\nbox Boolean", 8),
    ] {
        let source = format!("{prefix}{header}.function Main() -> Int32\n.local arrayref<System.Object> args\nldc.i4 {count}\narray.reserve System.Object\nstloc args\nldloc args\nldc.i4 0\n{value}\nstelem System.Object\nldc.i4 40\nnewobj instance Counter::.ctor(Int32)\nfunction.bind fn<Int32,Int32> = instance Counter::Add(Int32)\ncastclass System.Object\nldloc args\ncall neoCLR.Runtime.InvokeFunctionCheck(System.Object,arrayref<System.Object>)\nret\n.end")
            .replace("ldarg 1\nadd\nret", "fault \"check must not enter target\"");
        assert_eq!(run(&source).unwrap().value, Value::Int32(expected));
    }
}
