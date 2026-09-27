use neoclr::{Limits, Value, assemble, run};
fn services() -> String {
    let mut s=".type class abstract System.Object\n.method instance .ctor() -> noresult\nret\n.end\n.end\n".to_string();
    for name in ["ConstructArgs", "Invoke", "FieldGet", "FieldSet"] {
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
