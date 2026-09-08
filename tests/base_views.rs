use neoclr::{Limits, LoadedProgram, Value, assemble, frontend};

const DEFS: &str =
    ".type Base\n.field X Int32\n.end\n.type Child\n.extends Base\n.field Y Int32\n.end\n";
fn il(body: &str) -> LoadedProgram {
    LoadedProgram::new(&assemble(&format!(".module App\n.entry Main\n{DEFS}\n.function Main() -> Int32\n.local Child child\n.local Base& view\nldc.i4 40\nldc.i4 2\nnewobj Child\nstloc child\n{body}\nret\n.end")).unwrap()).unwrap()
}
#[test]
fn source_and_artifact_keep_owner_identity_type_and_field_access() {
    let m = frontend::compile(include_str!("../examples/source/base-views.neo")).unwrap();
    let m = serde_json::from_str(&serde_json::to_string(&m).unwrap()).unwrap();
    let p = LoadedProgram::new(&m).unwrap();
    p.verify().unwrap();
    let result = p.run(Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.heap.statistics().allocated_objects, 1);
}
#[test]
fn whole_value_reads_writes_defaulting_and_out_reject_base_views_unchecked() {
    for body in [
        "ldloca child\ncastclass Base\nldobj Base\nldfld 0",
        "ldloca child\ncastclass Base\nldc.i4 9\nnewobj Base\nstobj Base\nldc.i4 0",
        "ldloca child\ncastclass Base\ninitobj Base\nldc.i4 0",
    ] {
        let p = il(body);
        assert!(
            p.run(Limits::default())
                .unwrap_err()
                .message
                .contains("slice")
        );
    }
    let source = "record Base(X: int)\nrecord Child(Y: int): Base\nfunc Fill(out b: Base&) -> () { b = Base(0) }\nfunc Main() -> int { var child = Child(40,2); let view: Base& = &child; Fill(out view); return child.X }";
    let m = frontend::compile(source).unwrap();
    assert!(
        LoadedProgram::new(&m)
            .unwrap()
            .run(Limits::default())
            .is_err()
    );
}
#[test]
fn ancestor_projection_cannot_downcast_or_expose_derived_fields() {
    let p = il("ldloca child\ncastclass Base\ncastclass Child\npop\nldc.i4 0");
    assert!(p.verify().is_err());
    assert!(p.run(Limits::default()).is_err());
    let p = il("ldloca child\ncastclass Base\nldfld 1");
    assert!(p.verify().is_err());
    assert!(p.run(Limits::default()).is_err());
}
#[test]
fn field_addresses_and_complete_owner_replacement_keep_views_live() {
    let p = il(
        "ldloca child\ncastclass Base\nstloc view\nldloc view\nldflda Base::X\nldc.i4 41\nstobj Int32\nldc.i4 42\nldc.i4 3\nnewobj Child\nstloc child\nldloc view\nldfld Base::X",
    );
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}
#[test]
fn projected_heap_owner_survives_collection_and_frame_owner_cannot_escape() {
    let source = "record Base(X: int)\nrecord Child(Data: int[]&): Base\nfunc Make() -> Base& { return new Child(42, new int[1]) }\nfunc Discard() -> () { let a = new int[1] }\nfunc Main() -> int { let b = Make(); Discard(); Discard(); Discard(); return b.X }";
    let p = LoadedProgram::new(&frontend::compile(source).unwrap()).unwrap();
    assert_eq!(
        p.run(Limits {
            heap_objects: 3,
            ..Limits::default()
        })
        .unwrap()
        .value,
        Value::Int32(42)
    );
    let source = "record Base(X: int)\nrecord Child(Y: int): Base\nfunc Escape() -> Base& { var c = Child(40,2); return &c }\nfunc Main() -> int { let b = Escape(); return b.X }";
    let p = LoadedProgram::new(&frontend::compile(source).unwrap()).unwrap();
    assert!(p.run(Limits::default()).is_err());
}
#[test]
fn readonly_projection_cannot_upgrade_even_without_verification() {
    let source = "record Base(X: int)\nrecord Child(Y: int): Base\nfunc Observe(readonly c: Child&) -> readonly Base& { return c }\nfunc Main() -> int { let c = new Child(40,2); let b = Observe(c); return b.X }";
    let text = frontend::lower_to_il(source)
        .unwrap()
        .replace("-> readonly Base&", "-> Base&");
    let p = LoadedProgram::new(&assemble(&text).unwrap()).unwrap();
    assert!(p.verify().is_err());
    assert!(
        p.run(Limits::default())
            .unwrap_err()
            .message
            .contains("readonly")
    );
}
#[test]
fn explicit_source_cast_and_nested_value_projection_work() {
    let source = "record Base(X: int)\nrecord Child(Y: int): Base\nrecord Holder(Value: Child)\nfunc Main() -> int { var h = Holder(Child(40,2)); let b = &h.Value as Base&; b.X = 42; return h.Value.X }";
    let p = LoadedProgram::new(&frontend::compile(source).unwrap()).unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}
#[test]
fn generic_base_conversion_uses_substituted_ancestor_identity() {
    let text = ".module App\n.entry Main\n.type Base<T>\n.field X T\n.end\n.type Child<T>\n.extends Base<T>\n.end\n.function Main() -> Int32\n.local Child<Int32> c\nldc.i4 42\nnewobj Child<Int32>\nstloc c\nldloca c\ncastclass Base<Int32>\nldfld Base<Int32>::X\nret\n.end";
    let p = LoadedProgram::new(&assemble(text).unwrap()).unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    let p = LoadedProgram::new(
        &assemble(&text.replace("castclass Base<Int32>", "castclass Base<Boolean>")).unwrap(),
    )
    .unwrap();
    assert!(p.verify().is_err());
    assert!(p.run(Limits::default()).is_err());
}
#[test]
fn cast_requires_initialized_managed_storage() {
    for body in [
        ".local Child c\nldloca c\ncastclass Base",
        "ptr.null Child\ncastclass Base",
    ] {
        let text = format!(
            ".module App\n.entry Main\n{DEFS}\n.function Main() -> Int32\n{body}\npop\nldc.i4 0\nret\n.end"
        );
        let p = LoadedProgram::new(&assemble(&text).unwrap()).unwrap();
        assert!(p.verify().is_err());
        assert!(p.run(Limits::default()).is_err());
    }
}
