use neoclr::{Limits, LoadedProgram, RuntimeService, Value, assemble, frontend};

const MODEL: &str = r#"
.type Box<T>
.field public Item T
.field private Hidden Int32
.property instance Value() -> T
.get instance Box<T>::Read()
.set instance Box<T>::Write(T)
.end
.method private instance Read() -> T
fault "getter must not execute"
.end
.method instance byref Write(T value) -> Void
ldvoid
ret
.end
.method static Create(T value) -> T
ldarg value
ret
.end
.method static Create(Int32 extra, T value) -> T
ldarg value
ret
.end
.method static Fill(out Int32& destination) -> Void
ldarg destination
ldc.i4 1
stobj Int32
ldvoid
ret
.end
.end
"#;

fn program(body: &str, returns: &str) -> LoadedProgram {
    let source = format!(
        ".module App\n{MODEL}\n.entry Main\n.function Main() -> {returns}\n{body}\nret\n.end"
    );
    let module = assemble(&source).unwrap();
    let loaded = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let program = LoadedProgram::new(&loaded).unwrap();
    program.verify().unwrap();
    program
}
fn query(member: &str, flags: Option<i32>) -> String {
    let prefix =
        "ldtoken Box<Int32&>\ncall System.Type::GetTypeFromHandle(System.RuntimeTypeHandle)";
    match flags {
        Some(n) => format!(
            "{prefix}\nldc.i4 {n}\ncall System.Reflection.BindingFlags::FromValue(Int32)\ncall instance System.Type::{member}(System.Reflection.BindingFlags)"
        ),
        None => format!("{prefix}\ncall instance System.Type::{member}()"),
    }
}
fn array(value: &Value) -> &[Value] {
    let Value::Array { elements, .. } = value else {
        panic!("expected array: {value:?}")
    };
    elements
}
fn fields(value: &Value) -> &[Value] {
    let Value::Object { fields, .. } = value else {
        panic!("expected record: {value:?}")
    };
    fields
}
fn type_name(value: &Value) -> &str {
    let Value::RuntimeTypeHandle(handle) = &fields(value)[0] else {
        panic!("expected type")
    };
    &handle.name
}

#[test]
fn fields_preserve_closed_reference_types_and_filter_visibility() {
    for (flags, names) in [
        (None, vec!["Item"]),
        (Some(36), vec!["Hidden"]),
        (Some(54), vec!["Item", "Hidden"]),
        (Some(24), vec![]),
        (Some(0), vec![]),
        (Some(16), vec![]),
    ] {
        let result = program(&query("GetFields", flags), "System.Reflection.FieldInfo[]")
            .run(Limits::default())
            .unwrap();
        let entries = array(&result.value);
        assert_eq!(
            entries
                .iter()
                .map(|e| fields(e)[0].clone())
                .collect::<Vec<_>>(),
            names
                .into_iter()
                .map(|s| Value::String(s.into()))
                .collect::<Vec<_>>()
        );
        if flags.is_none() {
            assert_eq!(type_name(&fields(&entries[0])[2]), "System.Int32&");
            assert_eq!(fields(&entries[0])[7], Value::Int32(0));
        }
    }
}

#[test]
fn methods_distinguish_overloads_receivers_and_output_contracts() {
    let result = program(&query("GetMethods", None), "System.Reflection.MethodInfo[]")
        .run(Limits::default())
        .unwrap();
    let methods = array(&result.value);
    assert_eq!(methods.len(), 4); // private Read and free Main excluded
    let write = fields(&methods[0]);
    assert_eq!(write[0], Value::String("Write".into()));
    assert_eq!(write[7], Value::Boolean(true));
    let parameter = fields(&array(&write[9])[0]);
    assert_eq!(parameter[0], Value::String("value".into()));
    assert_eq!(type_name(&parameter[2]), "System.Int32&");
    assert_ne!(fields(&methods[1])[8], fields(&methods[2])[8]);
    let fill = fields(&array(&fields(&methods[3])[9])[0]);
    assert_eq!(fill[3], Value::Boolean(true));
    assert_eq!(fill[4], Value::Boolean(false));
}

#[test]
fn properties_report_accessors_without_executing_them() {
    let prefix = format!(
        ".local System.Reflection.PropertyInfo descriptor\n{}\nldc.i4 0\nldelem System.Reflection.PropertyInfo\nstloc descriptor\nldloca descriptor",
        query("GetProperties", None)
    );
    for (method, argument, expected) in [
        ("GetGetMethod", "", false),
        ("GetGetMethod", "ldc.bool true\n", true),
        ("GetSetMethod", "", true),
    ] {
        let signature = if argument.is_empty() { "" } else { "Boolean" };
        let body = format!(
            "{prefix}\n{argument}call instance System.Reflection.PropertyInfo::{method}({signature})\ncall instance System.Option<System.Reflection.MethodInfo>::get_IsSome()"
        );
        assert_eq!(
            program(&body, "Boolean")
                .run(Limits::default())
                .unwrap()
                .value,
            Value::Boolean(expected)
        );
    }
    let body = format!(
        "{prefix}\ncall instance System.Reflection.PropertyInfo::get_PropertyType()\ncall instance System.Type::get_Name()"
    );
    assert_eq!(
        program(&body, "String")
            .run(Limits::default())
            .unwrap()
            .value,
        Value::String("System.Int32&".into())
    );
    let body = format!("{}\nldlen\nconv.i4", query("GetProperties", Some(36)));
    assert_eq!(
        program(&body, "Int32")
            .run(Limits::default())
            .unwrap()
            .value,
        Value::Int32(0)
    );
}

#[test]
fn invalid_flags_and_descriptor_array_budgets_fault() {
    for flags in [1, 64, -1] {
        let fault = program(
            &query("GetFields", Some(flags)),
            "System.Reflection.FieldInfo[]",
        )
        .run(Limits::default())
        .unwrap_err();
        assert!(fault.message.contains("unsupported BindingFlags"));
    }
    for limits in [
        Limits {
            array_elements: 0,
            ..Limits::default()
        },
        Limits {
            array_bytes: 1,
            ..Limits::default()
        },
    ] {
        assert!(
            program(&query("GetMethods", None), "System.Reflection.MethodInfo[]")
                .run(limits)
                .unwrap_err()
                .message
                .contains("array")
        );
    }
}

#[test]
fn source_example_and_artifact_round_trip_use_metadata_only() {
    let module = frontend::compile(include_str!("../examples/source/reflection.neo")).unwrap();
    let loaded = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let program = LoadedProgram::new(&loaded).unwrap();
    program.verify().unwrap();
    let result = program.run(Limits::default()).unwrap();
    assert_eq!(
        &result.output[..4],
        ["Age", "System.Int32", "Add", "System.Int32"]
    );
    assert_eq!(result.output.last().unwrap(), "Managed reference signature");
    assert_eq!(result.heap.statistics().allocated_objects, 0);
    let graph = program
        .analyze_reachability(
            &neoclr::assembler::parse_function_ref("Main()")
                .map(|r| vec![r])
                .unwrap(),
            300,
        )
        .unwrap();
    assert!(
        graph
            .required_services()
            .contains(&RuntimeService::TypeInspection)
    );
    assert!(
        graph
            .required_services()
            .contains(&RuntimeService::ManagedArrays)
    );
}

#[test]
fn shape_queries_and_element_options_preserve_address_modes() {
    for (ty, member, expected) in [
        ("Int32[]", "IsArray", true),
        ("Int32&", "IsByRef", true),
        ("Int32*", "IsPointer", true),
        ("Int32", "IsByRef", false),
    ] {
        let body = format!(
            "ldtoken {ty}\ncall System.Type::GetTypeFromHandle(System.RuntimeTypeHandle)\ncall instance System.Type::get_{member}()"
        );
        assert_eq!(
            program(&body, "Boolean")
                .run(Limits::default())
                .unwrap()
                .value,
            Value::Boolean(expected)
        );
    }
    for (ty, some) in [("Int32[]", true), ("Int32&", true), ("Int32", false)] {
        let body = format!(
            "ldtoken {ty}\ncall System.Type::GetTypeFromHandle(System.RuntimeTypeHandle)\ncall instance System.Type::GetElementType()\ncall instance System.Option<System.Type>::get_IsSome()"
        );
        assert_eq!(
            program(&body, "Boolean")
                .run(Limits::default())
                .unwrap()
                .value,
            Value::Boolean(some)
        );
    }
}

#[test]
fn transitive_private_signature_metadata_does_not_grant_call_access() {
    let dependencies = ".module Dependencies\n.type Payload\n.end";
    let library = ".module Models\n.references (Dependencies)\n.type Model\n.field private Data [Dependencies]Payload\n.method private static Secret() -> Void\nldvoid\nret\n.end\n.end";
    let app = ".module App\n.references (Models)\n.entry Main\n.function Main() -> String\n.local System.Reflection.FieldInfo descriptor\nldtoken [Models]Model\ncall System.Type::GetTypeFromHandle(System.RuntimeTypeHandle)\nldc.i4 36\ncall System.Reflection.BindingFlags::FromValue(Int32)\ncall instance System.Type::GetFields(System.Reflection.BindingFlags)\nldc.i4 0\nldelem System.Reflection.FieldInfo\nstloc descriptor\nldloca descriptor\ncall instance System.Reflection.FieldInfo::get_FieldType()\ncall instance System.Type::get_Name()\nret\n.end";
    let module = neoclr::assembler::assemble_modules(&[app, library, dependencies]).unwrap();
    let program =
        LoadedProgram::with_modules(&module[0], neoclr::library::system().unwrap(), &module[1..])
            .unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::String("Payload".into())
    );
    let forbidden = app.replace(
        "ldtoken [Models]Model",
        "call [Models]Model::Secret()\npop\nldtoken [Models]Model",
    );
    assert!(neoclr::assembler::assemble_modules(&[&forbidden, library, dependencies]).is_err());
}

#[test]
fn returned_arrays_copy_and_survive_heap_collection_without_recursive_snapshots() {
    let source = r#"
record Node(Next: Node&) {
    func Read() -> int { return 1 }
}
func Main() -> string {
    var fields = typeof(Node).GetFields()
    let copy = fields
    let other = typeof(System.Type).GetFields(System.Reflection.BindingFlags.FromValue(36))
    fields[0] = other[0]
    let holder = new array(1, copy)
    for i in 0..<12 {
        let temporary = new array(1, typeof(Node).GetMethods())
    }
    return holder[0][0].FieldType.Name
}
"#;
    let module = frontend::compile(source).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    let result = program
        .run(Limits {
            heap_objects: 3,
            ..Limits::default()
        })
        .unwrap();
    assert_eq!(result.value, Value::String("Node&".into()));
    assert!(result.heap.statistics().collections > 0);
}

#[test]
fn indexed_properties_report_explicit_parameters_and_static_accessors() {
    let source = r#"
.module App
.type Slots
.property static Item(Int32) -> String
.get Slots::Read(Int32)
.end
.method static Read(Int32 index) -> String
fault "metadata query executed an accessor"
.end
.end
.entry Main
.function Main() -> System.Reflection.PropertyInfo[]
ldtoken Slots
call System.Type::GetTypeFromHandle(System.RuntimeTypeHandle)
call instance System.Type::GetProperties()
ret
.end
"#;
    let module = assemble(source).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    let result = program.run(Limits::default()).unwrap();
    let property = fields(&array(&result.value)[0]);
    assert_eq!(property[3], Value::Boolean(true));
    assert_eq!(property[5], Value::Boolean(false));
    let parameter = fields(&array(&property[7])[0]);
    assert_eq!(parameter[0], Value::String(String::new()));
    assert_eq!(type_name(&parameter[2]), "System.Int32");
}

#[test]
fn library_generic_interfaces_and_conditional_outputs_are_inspectable() {
    let source = r#"
func HasConditionalOutput(method: System.Reflection.MethodInfo) -> bool {
    let parameters = method.GetParameters()
    for i in 0..<parameters.Length {
        if parameters[i].IsOutWhenTrue {
            return !parameters[i].IsOut
        }
    }
    return false
}
func Main() -> string {
    let interfaces = typeof(System.Collections.ArrayList<int&>).GetInterfaces()
    if !interfaces[0].IsInterface { return "not interface" }
    let args = interfaces[0].GetGenericArguments()
    if !args[0].IsByRef { return "lost reference" }
    let methods = typeof(Option<int>).GetMethods()
    for i in 0..<methods.Length {
        if HasConditionalOutput(methods[i]) {
            return interfaces[0].FullName
        }
    }
    return "missing conditional output"
}
"#;
    let module = frontend::compile(source).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::String("System.Collections.List<System.Int32&>".into())
    );
}
