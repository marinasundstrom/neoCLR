use neoclr::{FaultCode, Limits, LoadedProgram, Module, Value};
use std::{process::Command, sync::OnceLock};

fn library() -> &'static Module {
    static LIBRARY: OnceLock<Module> = OnceLock::new();
    LIBRARY.get_or_init(|| {
        let output = Command::new("python3").current_dir(env!("CARGO_MANIFEST_DIR"))
            .args(["-c", "import runpy; m=runpy.run_path('docs/experiments/raven-target/collection_library.py'); print(m['build'](m['ROOT'] / 'runtime/System.neoil'))"])
            .output().unwrap();
        assert!(output.status.success());
        neoclr::assemble(std::str::from_utf8(&output.stdout).unwrap()).unwrap()
    })
}
fn assemble(source: &str) -> Result<Module, neoclr::Fault> {
    neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(source)], library())
        .map(|mut modules| modules.remove(0))
}
fn run(module: &Module, limits: Limits) -> Result<neoclr::Execution, neoclr::Fault> {
    LoadedProgram::with_library(module, library())?.run(limits)
}

fn app(body: &str) -> neoclr::Module {
    assemble(&format!(".module Arrays\n.entry Main\n.function Main() -> Int32\n.local arrayref<System.Object> values\n.local System.Object result\n{body}\nret\n.end")).unwrap()
}
const SOURCE: &str = "ldc.i4 1\narray.reserve System.Object\nstloc values\nldloc values\nldc.i4 0\nldc.i4 42\nbox Int32\nstelem System.Object\n";
const CREATE: &str = "ldc.i4 0\nnewarr Int32\ncastclass System.Object\ncall instance System.Object::GetType()\nldloc values\ncall neoCLR.Runtime.ReflectionArrayCreate(System.Introspection.TypeInfo,arrayref<System.Object>)\nstloc result\n";

#[test]
fn dynamic_array_round_trip_uses_checked_boxing_and_releases_roots() {
    let module = app(&format!(
        "{SOURCE}{CREATE}ldloc result\ncall neoCLR.Runtime.ReflectionArrayLength(System.Object)\nldloc result\nldc.i4 0\ncall neoCLR.Runtime.ReflectionArrayGet(System.Object,Int32)\nunbox.any Int32\nadd"
    ));
    let result = run(&module, Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(43));
    assert_eq!(result.heap.len(), 0);
}

#[test]
fn dynamic_array_rejects_wrong_box_type_and_out_of_range_access() {
    let wrong = app(&format!(
        "{SOURCE}{}ldc.i4 0",
        CREATE.replace("newarr Int32", "newarr Boolean")
    ));
    assert_eq!(
        run(&wrong, Limits::default()).unwrap_err().code,
        FaultCode::InvalidCast
    );
    let out_of_range = app(&format!(
        "{SOURCE}{CREATE}ldloc result\nldc.i4 1\ncall neoCLR.Runtime.ReflectionArrayGet(System.Object,Int32)\nunbox.any Int32"
    ));
    assert!(run(&out_of_range, Limits::default()).is_err());
}

#[test]
fn array_services_require_exact_declared_signatures() {
    assert!(assemble(".module Bad\n.entry neoCLR.Runtime.ReflectionArrayLength\n.function neoCLR.Runtime.ReflectionArrayLength(Int32) -> Int32\n.methodimpl InternalCall\n.end").is_err());
}
