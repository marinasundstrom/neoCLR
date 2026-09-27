use neoclr::{Limits, Value, assemble, run};
fn services() -> String {
    let mut s=".type class abstract System.Object\n.method instance .ctor() -> noresult\nret\n.end\n.end\n".to_string();
    for name in [
        "ConstructArgs",
        "Invoke",
        "FieldGet",
        "FieldSet",
        "ConstructorInvoke",
    ] {
        for check in [false, true] {
            s += &format!(
                ".function neoCLR.Runtime.Reflection{name}{}(System.RuntimeTypeHandle,Int32,System.Object,arrayref<System.Object>) -> {}\n.methodimpl InternalCall\n.end\n",
                if check { "Check" } else { "" },
                if check { "Int32" } else { "System.Object" }
            );
        }
    }
    s
}
const MODEL: &str = r#"
.type class Model
.extends System.Object
.field N Int32
.method instance .ctor(Int32 value) -> noresult
ldarg this
call instance System.Object::.ctor()
ldarg this
ldarg value
stfld Model::N
ret
.end
.method instance virtual Add(Int32 value) -> Int32
ldarg this
ldfld Model::N
ldarg value
add
ret
.end
.end
"#;
fn invoke(types: &str, body: &str) -> neoclr::Module {
    assemble(&format!(".module Members\n.entry Main\n{}{types}\n.function Main() -> Int32\n.local System.Object item\n.local System.Object empty\n.local arrayref<System.Object> arguments\nldloca empty\ninitobj System.Object\n{body}\nret\n.end\n",services())).unwrap()
}
fn args(value: i32) -> String {
    format!(
        "ldc.i4 1\nnewarr System.Object\nstloc arguments\nldloc arguments\nldc.i4 0\nldc.i4 {value}\nbox Int32\nstelem System.Object\n"
    )
}
fn call(name: &str, index: i32, receiver: &str) -> String {
    format!(
        "ldtoken Model\nldc.i4 {index}\nldloc {receiver}\nldloc arguments\ncall neoCLR.Runtime.Reflection{name}(System.RuntimeTypeHandle,Int32,System.Object,arrayref<System.Object>)\n"
    )
}
#[test]
fn constructor_arguments_method_call_and_field_round_trip() {
    // DefinitionIndex uses canonical module function indices; services precede Model.
    let m = invoke(
        MODEL,
        &format!(
            "{}{}stloc item\n{}{}pop\nldc.i4 0\nnewarr System.Object\nstloc arguments\n{}unbox.any Int32",
            args(17),
            call("ConstructArgs", 0, "empty"),
            args(42),
            call("FieldSet", 0, "item"),
            call("FieldGet", 0, "item")
        ),
    );
    let result = run(&m, Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.is_empty());
    let mut m = invoke(
        MODEL,
        &format!(
            "{}{}stloc item\n{}{}unbox.any Int32",
            args(17),
            call("ConstructArgs", 0, "empty"),
            args(25),
            call("Invoke", 9, "item")
        ),
    );
    let index = m
        .functions
        .iter()
        .position(|f| f.name == "Model.Add")
        .unwrap() as i32;
    for f in &mut m.functions {
        if f.name == "Main" {
            for op in &mut f.body {
                if let neoclr::metadata::Instruction::Int(9) = op {
                    *op = neoclr::metadata::Instruction::Int(index);
                }
            }
        }
    }
    let result = run(&m, Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(42));
}
#[test]
fn validation_rejects_receiver_arity_and_private_fields() {
    let m = invoke(
        MODEL,
        &format!("{}{}", args(1), call("FieldSetCheck", 0, "empty")),
    );
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(6));
    let m = invoke(
        MODEL,
        &format!(
            "{}{}stloc item\nldc.i4 0\nnewarr System.Object\nstloc arguments\n{}",
            args(1),
            call("ConstructArgs", 0, "empty"),
            call("FieldSetCheck", 0, "item")
        ),
    );
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(8));
    let m = invoke(
        &MODEL.replace(".field N Int32", ".field private N Int32"),
        &format!(
            "{}{}stloc item\n{}",
            args(1),
            call("ConstructArgs", 0, "empty"),
            call("FieldSetCheck", 0, "item")
        ),
    );
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(3));
}

#[test]
fn constructor_matching_rejects_ambiguous_reference_arguments() {
    let model = r#"
.type class Model
.extends System.Object
.method instance .ctor(System.Object value) -> noresult
ldarg this
call instance System.Object::.ctor()
ret
.end
.method instance .ctor(Model value) -> noresult
ldarg this
call instance System.Object::.ctor()
ret
.end
.end
"#;
    let m = invoke(
        model,
        &format!(
            "ldc.i4 1\nnewarr System.Object\nstloc arguments\nldloc arguments\nldc.i4 0\nldloc empty\nstelem System.Object\n{}",
            call("ConstructArgsCheck", 0, "empty")
        ),
    );
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(9));
}

// Bind the retained descriptor index, rather than invoking overload selection.
fn exact(types: &str, owner: &str, member: &str, body: &str, check: bool) -> neoclr::Module {
    let mut m = invoke(
        types,
        &format!(
            "{body}ldtoken {owner}\nldc.i4 123456\nldloc empty\nldloc arguments\ncall neoCLR.Runtime.ReflectionConstructorInvoke{}(System.RuntimeTypeHandle,Int32,System.Object,arrayref<System.Object>)\n{}",
            if check { "Check" } else { "" },
            if check {
                ""
            } else {
                "unbox.any Model\nldfld Model::N"
            }
        ),
    );
    let index = m.functions.iter().position(|f| f.name == member).unwrap() as i32;
    for f in &mut m.functions {
        for op in &mut f.body {
            if let neoclr::metadata::Instruction::Int(123456) = op {
                *op = neoclr::metadata::Instruction::Int(index);
            }
        }
    }
    m
}

#[test]
fn retained_constructor_builds_boxed_value_and_survives_artifact_round_trip() {
    let record = MODEL
        .replace(".type class Model\n.extends System.Object", ".type Model")
        .replace("ldarg this\ncall instance System.Object::.ctor()\n", "")
        .replace(".method instance .ctor", ".method instance byref .ctor")
        .replace(".method instance virtual Add", ".method instance byref Add")
        .replace(
            "ldarg this\nldarg value\nstfld Model::N",
            "ldarg this\nldflda Model::N\nldarg value\nstobj Int32",
        );
    let m = exact(&record, "Model", "Model..ctor", &args(42), false);
    let restored = neoclr::load(&serde_json::to_string(&m).unwrap()).unwrap();
    let result = run(&restored, Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.is_empty());
}

#[test]
fn retained_constructor_checks_identity_access_and_exact_arguments() {
    for (model, owner, member, arguments, expected) in [
        (MODEL.to_string(), "Model", "Model..ctor", args(3), 0),
        (MODEL.to_string(), "Model", "Model.Add", args(3), 4),
        (
            MODEL.to_string(),
            "System.Object",
            "Model..ctor",
            args(3),
            2,
        ),
        (
            format!("{MODEL}\n.type class Other\n.extends System.Object\n.end\n"),
            "Other",
            "Model..ctor",
            args(3),
            4,
        ),
        (
            MODEL.replace(
                ".method instance .ctor(Int32",
                ".method private instance .ctor(Int32",
            ),
            "Model",
            "Model..ctor",
            args(3),
            3,
        ),
        (
            MODEL.replace(".type class Model", ".type class abstract Model"),
            "Model",
            "Model..ctor",
            args(3),
            2,
        ),
        (
            MODEL.to_string(),
            "Model",
            "Model..ctor",
            "ldc.i4 0\nnewarr System.Object\nstloc arguments\n".to_string(),
            8,
        ),
        (
            MODEL.to_string(),
            "Model",
            "Model..ctor",
            args(3).replace("ldc.i4 3\nbox Int32", "ldstr \"3\"\nbox String"),
            8,
        ),
        (
            MODEL.to_string(),
            "Model",
            "Model..ctor",
            args(3).replace("ldc.i4 3\nbox Int32", "ldloc empty"),
            8,
        ),
    ] {
        let m = exact(&model, owner, member, &arguments, true);
        assert_eq!(
            run(&m, Limits::default()).unwrap().value,
            Value::Int32(expected),
            "{model} {owner} {member}"
        );
    }
}

#[test]
fn retained_constructor_does_not_execute_on_invalid_arguments_and_keeps_faults_terminal() {
    let faulty = MODEL.replace(
        "ldarg this\nldarg value\nstfld Model::N\nret",
        "fault \"constructor executed\"",
    );
    let rejected = exact(
        &faulty,
        "Model",
        "Model..ctor",
        "ldc.i4 0\nnewarr System.Object\nstloc arguments\n",
        true,
    );
    assert_eq!(
        run(&rejected, Limits::default()).unwrap().value,
        Value::Int32(8)
    );
    let mut invoked = exact(&faulty, "Model", "Model..ctor", &args(1), false);
    // Reference result access is never reached: the original constructor faults.
    let main = invoked
        .functions
        .iter_mut()
        .find(|f| f.name == "Main")
        .unwrap();
    for op in &mut main.body {
        if let neoclr::metadata::Instruction::UnboxAny(t) = op {
            *op = neoclr::metadata::Instruction::CastClass(t.clone());
        }
    }
    let fault = run(&invoked, Limits::default()).unwrap_err();
    assert_eq!(fault.code, neoclr::FaultCode::UserFault);
    assert!(fault.message.contains("constructor executed"));
}
