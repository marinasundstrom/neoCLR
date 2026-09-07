use neoclr::{
    Limits, LoadedProgram, Value, assemble,
    assembler::{assemble_modules, parse_function_ref},
    metadata::Type,
};

const SAMPLE: &str = include_str!("../examples/properties.neoil");

#[test]
fn generic_property_round_trips_and_accessors_execute_as_ordinary_calls() {
    let module = assemble(SAMPLE).unwrap();
    let property = &module.types[0].properties[0];
    assert_eq!(property.name, "Value");
    assert_eq!(property.ty, Type::TypeParameter(0));
    assert_eq!(property.getter.as_ref().unwrap().name, "Box.Read");
    let json = serde_json::to_string(&module).unwrap();
    let loaded = neoclr::load(&json).unwrap();
    let before = serde_json::to_value(&loaded).unwrap();
    let program = LoadedProgram::new(&loaded).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().output,
        ["42", "Properties describe ordinary methods"]
    );
    assert_eq!(serde_json::to_value(&loaded).unwrap(), before);
    let graph = program
        .analyze_reachability(
            &[parse_function_ref("instance Box<Int32>::Read()").unwrap()],
            1,
        )
        .unwrap();
    assert_eq!(graph.functions.len(), 1);
    assert!(graph.required_services().is_empty());
}

const INDEXED: &str = ".module Indexed
.type Slots
.property static Item(Int32*) -> Int32
.get Slots::Read(Int32*)
.set Slots::Write(Int32*,Int32)
.end
.method static Read(Int32* slot) -> Int32
ldarg slot
ldobj Int32
ret
.end
.method static Read(String unused) -> Int32
ldc.i4 -1
ret
.end
.method static Write(Int32* slot, Int32 value) -> Void
ldarg slot
ldarg value
stobj Int32
ldvoid
ret
.end
.end
.entry Main
.function Main() -> Int32
.local Int32* slot
ldc.i4 1
heap.alloc Int32
stloc slot
ldloc slot
ldc.i4 42
call Slots::Write(Int32*,Int32)
pop
ldloc slot
call Slots::Read(Int32*)
ldloc slot
heap.free
pop
ret
.end";

#[test]
fn indexed_property_selects_overloads_and_setter_uses_explicit_pointer_storage() {
    let module = assemble(INDEXED).unwrap();
    let property = &module.types[0].properties[0];
    assert!(!property.instance);
    assert_eq!(property.parameters, [Type::Ptr(Box::new(Type::Int32))]);
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn malformed_property_declarations_and_accessor_contracts_are_rejected() {
    for source in [
        SAMPLE.replace(".get instance Box<T>::Read()", ""),
        SAMPLE.replace(
            ".get instance Box<T>::Read()",
            ".get instance Box<T>::Read()\n.get instance Box<T>::Read()",
        ),
        SAMPLE.replace(
            ".get instance Box<T>::Read()",
            ".set instance Box<T>::Read()",
        ),
        SAMPLE.replace(".get instance Box<T>::Read()", ".get Box<T>::Read()"),
        SAMPLE.replace(
            ".get instance Box<T>::Read()",
            ".get instance Box<Int32>::Read()",
        ),
        SAMPLE.replace(
            ".property instance Value() -> T",
            ".property instance Value() -> String",
        ),
        SAMPLE.replace(
            ".property instance Value() -> T",
            ".property instance Box.Value() -> T",
        ),
        SAMPLE.replace(
            ".get instance Box<T>::Read()",
            ".get instance Box<T>::Missing()",
        ),
        SAMPLE.replace(
            ".get instance Box<T>::Read()",
            ".method instance NotAllowed() -> T",
        ),
        INDEXED.replace(
            ".set Slots::Write(Int32*,Int32)",
            ".set Slots::Write(Int32*)",
        ),
        INDEXED.replace(
            ".method static Write(Int32* slot, Int32 value) -> Void",
            ".method static Write(Int32* slot, Int32 value) -> Int32",
        ),
    ] {
        assert!(assemble(&source).is_err(), "accepted {source}");
    }
}

#[test]
fn json_cannot_forge_property_types_duplicate_signatures_or_accessor_tokens() {
    let module = assemble(INDEXED).unwrap();
    let mut invalid = Vec::new();
    let mut duplicate = module.clone();
    duplicate.types[0]
        .properties
        .push(module.types[0].properties[0].clone());
    invalid.push(duplicate);
    let mut ty = module.clone();
    ty.types[0].properties[0].ty = Type::TypeParameter(0);
    invalid.push(ty);
    let mut token = module.clone();
    token.types[0].properties[0]
        .getter
        .as_mut()
        .unwrap()
        .definition = token.functions[1].definition.clone();
    invalid.push(token);
    let mut owner = module.clone();
    owner.types[0].properties[0].getter.as_mut().unwrap().owner = None;
    invalid.push(owner);
    for value in invalid {
        assert!(neoclr::load(&serde_json::to_string(&value).unwrap()).is_err());
    }
}

#[test]
fn property_scopes_and_direct_module_references_are_checked() {
    let models = ".module Models\n.type Data\n.end";
    let app = ".module App\n.references (Models)\n.type Reader\n.property static Current() -> [Models]Data\n.get Reader::Read()\n.end\n.method static Read() -> [Models]Data\nnewobj [Models]Data\nret\n.end\n.end";
    let modules = assemble_modules(&[app, models]).unwrap();
    let program = LoadedProgram::with_modules(
        &modules[0],
        neoclr::library::system().unwrap(),
        &modules[1..],
    )
    .unwrap();
    program.verify().unwrap();
    assert!(assemble_modules(&[&app.replace("[Models]Data", "[Wrong]Data"), models]).is_err());
    assert!(
        assemble_modules(&[
            &app.replace(".references (Models)", ".references ()"),
            models
        ])
        .is_err()
    );
}

#[test]
fn property_metadata_is_optional_and_system_accessors_are_explicit() {
    let module = assemble(".module Legacy\n.type Empty\n.end").unwrap();
    let json = serde_json::to_string(&module).unwrap();
    assert!(!json.contains("properties"));
    assert!(neoclr::load(&json).is_ok());
    let system = neoclr::library::system().unwrap();
    for (name, property) in [("System.Error", "Message"), ("System.Array", "Length")] {
        let def = system.types.iter().find(|ty| ty.name == name).unwrap();
        assert_eq!(def.properties[0].name, property);
        assert!(def.properties[0].setter.is_none());
    }
}
