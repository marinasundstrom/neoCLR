use neoclr::{Limits, Value, assemble, run, verify};
use serde_json::json;

fn module(body: &str) -> neoclr::Module {
    let module = assemble(&format!(
        ".module ReadOnly\n.entry Main\n.type class Counter\n.field Number Int32\n.method instance .ctor() -> noresult\nldarg 0\nldc.i4 42\nstfld Counter::Number\nret\n.end\n.method instance Read() -> Int32\n{body}\n.end\n.end\n.function Main() -> Int32\nnewobj instance Counter::.ctor()\ncall instance Counter::Read()\nret\n.end"
    )).unwrap();
    let mut value = serde_json::to_value(module).unwrap();
    value["assemblies"] = json!([{"name":"ReadOnly", "full_name":"ReadOnly", "modules":["ReadOnly.dll"], "references":[]}]);
    value["types"][0]["origin"] = json!({"assembly":"ReadOnly", "module":"ReadOnly.dll", "name":"Counter", "token":0x02000001,
        "field_tokens":[0x04000001], "field_readonly":[true]});
    serde_json::from_value(value).unwrap()
}

#[test]
fn declaring_constructor_can_initialize_readonly_field() {
    let module = module("ldarg 0\nldfld Counter::Number\nret");
    verify(&module).unwrap();
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn ordinary_method_cannot_store_directly_or_through_managed_address() {
    for body in [
        "ldarg 0\nldc.i4 1\nstfld Counter::Number\nldc.i4 42\nret",
        "ldarg 0\nldflda Counter::Number\nldc.i4 1\nstobj Int32\nldc.i4 42\nret",
    ] {
        let module = module(body);
        assert!(verify(&module).unwrap_err().message.contains("readonly"));
        assert!(
            run(&module, Limits::default())
                .unwrap_err()
                .message
                .contains("readonly")
        );
    }
}

#[test]
fn readonly_managed_field_address_can_still_be_read() {
    let module = module("ldarg 0\nldflda Counter::Number\nldobj Int32\nret");
    verify(&module).unwrap();
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn declaring_constructor_can_initialize_through_managed_address() {
    use neoclr::metadata::{Instruction as Op, Type};
    let mut module = module("ldarg 0\nldfld Counter::Number\nret");
    module
        .functions
        .iter_mut()
        .find(|f| f.name.ends_with("..ctor"))
        .unwrap()
        .body = vec![
        Op::Arg(0),
        Op::FieldAddress(0),
        Op::Int(42),
        Op::StoreObject(Type::Int32),
        Op::Return,
    ];
    verify(&module).unwrap();
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn associated_init_accessor_can_write_its_declaring_readonly_storage() {
    let text = ".module ReadOnly
.entry Main
.type class Counter
.field Number Int32
.property instance Value() -> Int32
.get instance Counter::Read()
.set instance Counter::Write(Int32)
.end
.method instance .ctor() -> noresult
ldarg 0
ldc.i4 42
stfld Counter::Number
ret
.end
.method instance Read() -> Int32
ldarg 0
ldfld Counter::Number
ret
.end
.method instance Write(Int32 value) -> Void
ldarg 0
ldarg value
stfld Counter::Number
ldvoid
ret
.end
.end
.function Main() -> Int32
newobj instance Counter::.ctor()
dup
ldc.i4 7
call instance Counter::Write(Int32)
pop
call instance Counter::Read()
ret
.end";
    let mut value = serde_json::to_value(assemble(text).unwrap()).unwrap();
    value["assemblies"] = json!([{"name":"ReadOnly", "full_name":"ReadOnly", "modules":["ReadOnly.dll"], "references":[]}]);
    value["types"][0]["origin"] = json!({"assembly":"ReadOnly", "module":"ReadOnly.dll", "name":"Counter", "token":0x02000001,
        "field_tokens":[0x04000001], "field_readonly":[true], "property_tokens":[0x17000001]});
    value["types"][0]["properties"][0]["init_only"] = json!(true);
    let module = serde_json::from_value(value.clone()).unwrap();
    verify(&module).unwrap();
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(7)
    );
    value["types"][0]["properties"][0]["init_only"] = json!(false);
    let ordinary = serde_json::from_value(value).unwrap();
    assert!(verify(&ordinary).unwrap_err().message.contains("readonly"));
    assert!(
        run(&ordinary, Limits::default())
            .unwrap_err()
            .message
            .contains("readonly")
    );
}
