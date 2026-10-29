//! Typed service admission and real guest execution; lifetime tests live with the heap adapter.
use neoclr::{
    Limits, LoadedProgram, RuntimeService, Value, assemble, assembler::parse_function_ref,
};
const SERVICES: &str = include_str!("../runtime/legacy/bootstrap/neoCLR/Runtime/Tls.neoil");
#[test]
fn guest_tls_configuration_reports_errors_and_capabilities() {
    let module = assemble(&format!(".module TlsTests\n{SERVICES}\n.function Run() -> Int32\n.local arrayref<Byte> owner\nldc.i4 0\nnewarr Byte\nstloc owner\nldloc owner\nldc.i4 0\nnewarr Byte\nldc.i4 0\nnewarr Byte\ncall neoCLR.Runtime.TlsCreateClient(arrayref<Byte>,arrayref<Byte>,arrayref<Byte>)\nret\n.end")).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    let entry = parse_function_ref("Run()").unwrap();
    let graph = program
        .analyze_reachability(std::slice::from_ref(&entry), 16)
        .unwrap();
    for service in [
        RuntimeService::Tls,
        RuntimeService::ManagedArrays,
        RuntimeService::ManagedHeap,
    ] {
        assert!(graph.required_services().contains(&service));
    }
    assert_eq!(
        program
            .resolve_function(&entry)
            .unwrap()
            .invoke(vec![], Limits::default())
            .unwrap()
            .value,
        Value::Int32(-14)
    );
}
#[test]
fn signature_changes_do_not_activate_tls_dispatch() {
    for declaration in [
        ".function neoCLR.Runtime.TlsRelease(Int64) -> Int32",
        ".function neoCLR.Runtime.TlsRelease(arrayref<Int32>) -> Int32",
        ".function neoCLR.Runtime.TlsRelease(arrayref<Byte>) -> Boolean",
        ".function neoCLR.Runtime.TlsCreateClient(arrayref<Byte>,arrayref<Byte>) -> Int32",
    ] {
        assert!(
            assemble(&format!(
                ".module Invalid\n{declaration}\n.methodimpl InternalCall\n.end"
            ))
            .is_err()
        );
    }
}
