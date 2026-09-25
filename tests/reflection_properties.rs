use neoclr::{FaultCode, Limits, Value, assemble, run};

const SERVICES: &str = r#"
.type class abstract System.Object
.end
.function neoCLR.Runtime.ReflectionPropertyGetCheck(System.RuntimeTypeHandle,Int32,System.Object) -> Int32
.methodimpl InternalCall
.end
.function neoCLR.Runtime.ReflectionPropertySetCheck(System.RuntimeTypeHandle,Int32,System.Object,System.Object) -> Int32
.methodimpl InternalCall
.end
.function neoCLR.Runtime.ReflectionPropertyGet(System.RuntimeTypeHandle,Int32,System.Object) -> System.Object
.methodimpl InternalCall
.end
.function neoCLR.Runtime.ReflectionPropertySet(System.RuntimeTypeHandle,Int32,System.Object,System.Object) -> Void
.methodimpl InternalCall
.end
"#;
const MODEL: &str = r#"
.type class Model
.extends System.Object
.field N Int32
.property instance Number() -> Int32
.get instance Model::get_Number()
.set instance Model::set_Number(Int32)
.end
.method instance virtual get_Number() -> Int32
ldarg this
ldfld Model::N
ldc.i4 2
mul
ret
.end
.method instance virtual set_Number(Int32 value) -> noresult
ldarg this
ldarg value
ldc.i4 1
add
stfld Model::N
ret
.end
.end
"#;
const RECEIVER: &str = "ldc.i4 0\nnewobj Model\ncastclass System.Object\nstloc item\n";
const GET_ARGS: &str = "ldtoken Model\nldc.i4 0\nldloc item\n";
const GET: &str = "call neoCLR.Runtime.ReflectionPropertyGet(System.RuntimeTypeHandle,Int32,System.Object)\nunbox.any Int32";
const SET: &str = "call neoCLR.Runtime.ReflectionPropertySet(System.RuntimeTypeHandle,Int32,System.Object,System.Object)\npop";
fn app(types: &str, body: &str) -> neoclr::Module {
    app_returning(types, body, "Int32")
}
fn app_returning(types: &str, body: &str, returns: &str) -> neoclr::Module {
    assemble(&format!(".module ReflectedProperties\n.entry Main\n{SERVICES}{types}\n.function Main() -> {returns}\n.local System.Object item\n.local System.Object empty\n{body}\nret\n.end")).unwrap()
}

#[test]
fn getter_and_setter_execute_code_and_box_exact_scalar_values() {
    let module = app(
        MODEL,
        &format!("{RECEIVER}{GET_ARGS}ldc.i4 20\nbox Int32\n{SET}\n{GET_ARGS}{GET}"),
    );
    let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let result = run(&module, Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.heap.len(), 0);
}

#[test]
fn base_property_dispatches_both_accessors_to_derived_overrides() {
    let child = r#"
.type class Child
.extends Model
.method instance override get_Number() -> Int32
ldarg this
ldfld Model::N
ldc.i4 2
add
ret
.end
.method instance override set_Number(Int32 value) -> noresult
ldarg this
ldarg value
ldc.i4 2
mul
stfld Model::N
ret
.end
.end
"#;
    let source = format!("{MODEL}{child}");
    let body = format!(
        "{}{GET_ARGS}ldc.i4 20\nbox Int32\n{SET}\n{GET_ARGS}{GET}",
        RECEIVER.replace("newobj Model", "newobj Child")
    );
    assert_eq!(
        run(&app(&source, &body), Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn rejects_missing_private_indexed_and_static_accessors_without_execution() {
    let mut cases = vec![
        (MODEL.replace(".get instance Model::get_Number()\n", ""), 5),
        (
            MODEL.replace("instance virtual get_Number", "private instance get_Number"),
            3,
        ),
    ];
    cases.push((
        r#"
.type class Model
.extends System.Object
.field N Int32
.property instance Number(Int32) -> Int32
.get instance Model::get_Number(Int32)
.end
.method instance get_Number(Int32 index) -> Int32
fault "must not run"
.end
.end
"#
        .into(),
        2,
    ));
    cases.push((
        r#"
.type class Model
.extends System.Object
.field N Int32
.property static Number() -> Int32
.get Model::get_Number()
.end
.method static get_Number() -> Int32
fault "must not run"
.end
.end
"#
        .into(),
        2,
    ));
    for (types, status) in cases {
        let body = format!(
            "{RECEIVER}{GET_ARGS}call neoCLR.Runtime.ReflectionPropertyGetCheck(System.RuntimeTypeHandle,Int32,System.Object)"
        );
        assert_eq!(
            run(&app(&types, &body), Limits::default()).unwrap().value,
            Value::Int32(status)
        );
        let direct = app(&types, &format!("{RECEIVER}{GET_ARGS}{GET}"));
        assert!(
            run(&direct, Limits::default())
                .unwrap_err()
                .message
                .contains("reflection property access rejected")
        );
    }
}

#[test]
fn rejects_wrong_receivers_indices_and_values_and_preserves_readonly_property() {
    let extra = ".type class Other\n.extends System.Object\n.end\n";
    for receiver in [
        "ldloca empty\ninitobj System.Object\nldloc empty\nstloc item\n",
        "newobj Other\ncastclass System.Object\nstloc item\n",
    ] {
        let body = format!(
            "{receiver}{GET_ARGS}call neoCLR.Runtime.ReflectionPropertyGetCheck(System.RuntimeTypeHandle,Int32,System.Object)"
        );
        assert_eq!(
            run(&app(&format!("{MODEL}{extra}"), &body), Limits::default())
                .unwrap()
                .value,
            Value::Int32(6)
        );
    }
    for index in [-1, 1] {
        let args = GET_ARGS.replace("ldc.i4 0", &format!("ldc.i4 {index}"));
        let body = format!(
            "{RECEIVER}{args}call neoCLR.Runtime.ReflectionPropertyGetCheck(System.RuntimeTypeHandle,Int32,System.Object)"
        );
        assert_eq!(
            run(&app(MODEL, &body), Limits::default()).unwrap().value,
            Value::Int32(4)
        );
    }
    for value in [
        "ldloca empty\ninitobj System.Object\nldloc empty",
        "ldc.i8 20\nbox Int64",
    ] {
        let body = format!(
            "{RECEIVER}{GET_ARGS}{value}\ncall neoCLR.Runtime.ReflectionPropertySetCheck(System.RuntimeTypeHandle,Int32,System.Object,System.Object)"
        );
        assert_eq!(
            run(&app(MODEL, &body), Limits::default()).unwrap().value,
            Value::Int32(7)
        );
        let direct = app(
            MODEL,
            &format!("{RECEIVER}{GET_ARGS}{value}\n{SET}\nldc.i4 0"),
        );
        assert!(
            run(&direct, Limits::default())
                .unwrap_err()
                .message
                .contains("InvalidValue")
        );
    }
    let readonly = MODEL.replace(".set instance Model::set_Number(Int32)\n", "");
    let body = format!(
        "{RECEIVER}{GET_ARGS}ldc.i4 20\nbox Int32\ncall neoCLR.Runtime.ReflectionPropertySetCheck(System.RuntimeTypeHandle,Int32,System.Object,System.Object)"
    );
    assert_eq!(
        run(&app(&readonly, &body), Limits::default())
            .unwrap()
            .value,
        Value::Int32(5)
    );
}

#[test]
fn arguments_and_receiver_survive_collection_during_setter() {
    let pressure = "ldc.i4 0\nnewobj Garbage\npop\n".repeat(32);
    let types = format!(
        "{}\n.type class Garbage\n.field N Int32\n.end\n",
        MODEL.replace(
            "ldarg value\nldc.i4 1",
            &format!("{pressure}ldarg value\nldc.i4 1")
        )
    );
    let body = format!("{RECEIVER}{GET_ARGS}ldc.i4 20\nbox Int32\n{SET}\n{GET_ARGS}{GET}");
    let result = run(
        &app(&types, &body),
        Limits {
            heap_objects: 5,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.collections() > 1);
    assert_eq!(result.heap.len(), 0);
}

#[test]
fn accessor_faults_and_frame_limit_remain_terminal() {
    let body = format!("{RECEIVER}{GET_ARGS}{GET}");
    let faulting = MODEL.replace("ldc.i4 2\nmul", "ldc.i4 0\ndiv");
    assert!(
        run(&app(&faulting, &body), Limits::default())
            .unwrap_err()
            .message
            .contains("zero")
    );
    assert_eq!(
        run(
            &app(MODEL, &body),
            Limits {
                frames: 2,
                ..Limits::default()
            }
        )
        .unwrap_err()
        .code,
        FaultCode::StackOverflow
    );
}

#[test]
fn signature_checks_reject_wrong_types_and_noresult_setter_service() {
    for signature in [
        "ReflectionPropertyGet(System.RuntimeTypeHandle,Int32,System.Object) -> Int32",
        "ReflectionPropertySet(System.RuntimeTypeHandle,Int32,System.Object,System.Object) -> noresult",
        "ReflectionPropertySetCheck(System.RuntimeTypeHandle,Int32,System.Object,Int32) -> Int32",
    ] {
        assert!(assemble(&format!(".module Invalid\n.type class System.Object\n.end\n.function neoCLR.Runtime.{signature}\n.methodimpl InternalCall\n.end")).is_err());
    }
}

#[test]
fn string_and_null_reference_properties_round_trip_through_object() {
    let model = MODEL
        .replace("N Int32", "N String")
        .replace("Number() -> Int32", "Number() -> String")
        .replace("set_Number(Int32", "set_Number(String")
        .replace("ldc.i4 2\nmul\n", "")
        .replace("ldc.i4 1\nadd\n", "");
    let receiver = RECEIVER.replace("ldc.i4 0", "ldstr \"\"");
    let body = format!(
        "{receiver}{GET_ARGS}ldstr \"Café\"\ncastclass System.Object\n{SET}\n{GET_ARGS}call neoCLR.Runtime.ReflectionPropertyGet(System.RuntimeTypeHandle,Int32,System.Object)\ncastclass String"
    );
    assert_eq!(
        run(&app_returning(&model, &body, "String"), Limits::default())
            .unwrap()
            .value,
        Value::String("Café".into())
    );
    let body = format!(
        "{receiver}{GET_ARGS}ldloca empty\ninitobj System.Object\nldloc empty\n{SET}\n{GET_ARGS}call neoCLR.Runtime.ReflectionPropertyGet(System.RuntimeTypeHandle,Int32,System.Object)\nref.isnull\nbrtrue accepted\nfault \"null reference was lost\"\naccepted:\nldc.i4 0"
    );
    assert_eq!(
        run(&app(&model, &body), Limits::default()).unwrap().value,
        Value::Int32(0)
    );
}
