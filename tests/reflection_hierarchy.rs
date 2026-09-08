use neoclr::{Limits, LoadedProgram, Value, assemble, frontend, metadata::Type};

const SAMPLE: &str = include_str!("../examples/source/reflection-hierarchy.neo");

#[test]
fn source_base_views_and_heterogeneous_heap_list_roundtrip() {
    let module = frontend::compile(SAMPLE).unwrap();
    let module = serde_json::from_str(&serde_json::to_string(&module).unwrap()).unwrap();
    let p = LoadedProgram::new(&module).unwrap();
    p.verify().unwrap();
    let run = p.run(Limits::default()).unwrap();
    assert_eq!(run.value, Value::Int32(3));
    assert_eq!(
        run.output,
        [
            "Age",
            "Counter",
            "Age",
            "System.Reflection.FieldInfo",
            "Read",
            "System.Reflection.MethodInfo",
            "Name",
            "System.Reflection.PropertyInfo"
        ]
    );
}

#[test]
fn inherited_properties_are_accessible_but_queries_remain_declared_only() {
    let source = "record Model(N: int)
func Main() -> int {
    let fields = typeof(Model).GetFields()
    let info = fields[0]
    let view: readonly System.Reflection.MemberInfo& = &info
    let baseType = typeof(System.Reflection.FieldInfo).BaseType
    let inherited = baseType match { Some(let value) => value, None => typeof(Model) }
    if !inherited.Equals(typeof(System.Reflection.MemberInfo)) { return 0 }
    if !inherited.IsAbstract { return 0 }
    if view.GetType().Equals(typeof(System.Reflection.FieldInfo)) {
        let props = typeof(System.Reflection.FieldInfo).GetProperties()
        for i in 0..<props.Length { if props[i].Name.Equals(\"Name\") { return 0 } }
        return inherited.GetProperties().Length
    }
    return 0
}";
    let p = LoadedProgram::new(&frontend::compile(source).unwrap()).unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(2));
}

#[test]
fn temporary_receiver_evaluates_once_and_explicit_method_lookup_uses_base() {
    let source = "import System.Console.*
record Model(N: int)
func Inspect() -> System.Reflection.FieldInfo {
    WriteLine(\"once\")
    return typeof(Model).GetFields()[0]
}
func Main() -> () {
    WriteLine(Inspect().Name)
    let field = Inspect()
    WriteLine(field.get_Name())
}";
    let p = LoadedProgram::new(&frontend::compile(source).unwrap()).unwrap();
    p.verify().unwrap();
    assert_eq!(
        p.run(Limits::default()).unwrap().output,
        ["once", "N", "once", "N"]
    );
}

#[test]
fn reference_collection_retains_complete_owners_after_factory_frames_end() {
    let source = "record Model(N: int)
func Collect() -> System.Collections.ArrayList<readonly System.Reflection.MemberInfo&> {
    let fields = typeof(Model).GetFields()
    let storage = new System.Reflection.FieldInfo[1] { fields[0] }
    var list = System.Collections.ArrayList<readonly System.Reflection.MemberInfo&>.Allocate(1)
    list.Add(&storage[0])
    return list
}
func Main() -> int {
    let list = Collect()
    for i in 0..<10 { new int[1] }
    if list[0].DeclaringType.Equals(typeof(Model)) {
        if list[0].GetType().Equals(typeof(System.Reflection.FieldInfo)) { return 42 }
    }
    return 0
}";
    let p = LoadedProgram::new(&frontend::compile(source).unwrap()).unwrap();
    p.verify().unwrap();
    assert_eq!(
        p.run(Limits {
            heap_objects: 3,
            ..Limits::default()
        })
        .unwrap()
        .value,
        Value::Int32(42)
    );
}

#[test]
fn base_value_slicing_and_abstract_instantiation_remain_rejected() {
    let source = "record Model(N: int)
func Main() -> int {
    let field = typeof(Model).GetFields()[0]
    let copy: System.Reflection.MemberInfo = field
    return 0
}";
    assert!(frontend::compile(source).is_err());
    let text = ".module App\n.entry Main\n.function Main() -> System.Reflection.MemberInfo\nnewobj instance System.Reflection.MemberInfo::.ctor(String,System.Type)\nret\n.end";
    assert!(assemble(text).is_err());
}

#[test]
fn internal_descriptor_constructor_matches_trusted_snapshot_factory() {
    let library = format!("{}\n.function ReflectionProbe(System.Type declaringType) -> System.Reflection.FieldInfo
ldstr \"Age\"
ldarg declaringType
ldtoken Int32
call System.Type::GetTypeFromHandle(System.RuntimeTypeHandle)
ldc.bool true
ldc.bool false
ldc.bool false
ldc.bool false
ldc.i4 0
newobj instance System.Reflection.FieldInfo::.ctor(String,System.Type,System.Type,Boolean,Boolean,Boolean,Boolean,Int32)
ret
.end", neoclr::library::system_source());
    let library = assemble(&library).unwrap();
    let app = ".module App\n.entry Main\n.type Counter\n.field Age Int32\n.end\n.function Main() -> System.Reflection.FieldInfo\nldtoken Counter\ncall System.Type::GetTypeFromHandle(System.RuntimeTypeHandle)\ncall ReflectionProbe(System.Type)\nret\n.end";
    let app =
        neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(app)], &library)
            .unwrap()
            .remove(0);
    let p = LoadedProgram::with_library(&app, &library).unwrap();
    p.verify().unwrap();
    let constructed = p.run(Limits::default()).unwrap().value;
    let query = ".module App\n.entry Main\n.type Counter\n.field Age Int32\n.end\n.function Main() -> System.Reflection.FieldInfo\nldtoken Counter\ncall System.Type::GetTypeFromHandle(System.RuntimeTypeHandle)\ncall instance System.Type::GetFields()\nldc.i4 0\nldelem System.Reflection.FieldInfo\nret\n.end";
    let query =
        neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(query)], &library)
            .unwrap()
            .remove(0);
    let snapshot = LoadedProgram::with_library(&query, &library)
        .unwrap()
        .run(Limits::default())
        .unwrap()
        .value;
    assert_eq!(constructed, snapshot);
    let Value::Object { fields, .. } = snapshot else {
        panic!()
    };
    let layout = library
        .instantiated_fields(&Type::from_name("System.Reflection.FieldInfo"))
        .unwrap();
    assert_eq!(fields.len(), layout.len());
    for (value, field) in fields.iter().zip(layout) {
        assert_eq!(value.ty(), field.ty);
    }
}
