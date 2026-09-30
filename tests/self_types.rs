use neoclr::{Limits, LoadedProgram, Value, assemble, metadata::Type};

const PROGRAM: &str = r#"
.module NativeSelf
.entry Main
.interface Number
.method static Add(Self left, Self right) -> Self
.end
.end
.type Count
.implements Number
.field Value Int32
.method static Add(Count left, Count right) -> Count
ldarg left
ldfld Count::Value
ldarg right
ldfld Count::Value
add
newobj Count
ret
.end
.end
.function Sum<T>(T left, T right) -> T
.constraint T Number
ldarg left
ldarg right
callself T = Number::Add(Self,Self)
ret
.end
.function Main() -> Int32
ldc.i4 20
newobj Count
ldc.i4 22
newobj Count
call Sum<Count>(Count,Count)
ldfld Count::Value
ret
.end
"#;

fn prepare(source: &str) -> Result<LoadedProgram, neoclr::Fault> {
    let module = assemble(source)?;
    let module = neoclr::load(&serde_json::to_string(&module).unwrap())?;
    let program = LoadedProgram::new(&module)?;
    program.verify()?;
    Ok(program)
}

#[test]
fn native_self_static_contract_executes_through_generic_bound() {
    let program = prepare(PROGRAM).unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
    let graph = program
        .analyze_reachability(
            &[neoclr::assembler::parse_function_ref("Main()").unwrap()],
            30,
        )
        .unwrap();
    assert!(graph.functions.iter().any(|f| f.target.name == "Count.Add"));
}

#[test]
fn self_is_not_an_ordinary_generic_parameter_or_lowercase_alias() {
    assert_eq!(
        neoclr::assembler::parse_type("Self").unwrap(),
        Type::SelfType
    );
    assert_eq!(
        neoclr::assembler::parse_type("self").unwrap(),
        Type::Named("self".into())
    );
    let nested = neoclr::assembler::parse_type("Box<Self[]>").unwrap();
    assert!(nested.contains_self());
    assert_eq!(
        nested.substitute_self(&Type::Int32).unwrap(),
        neoclr::assembler::parse_type("Box<Int32[]>").unwrap()
    );
}

#[test]
fn native_self_rejects_missing_bound_mismatched_implementation_and_storage() {
    for source in [
        PROGRAM.replace(".constraint T Number", ""),
        PROGRAM.replace(
            "Add(Count left, Count right) -> Count",
            "Add(Count left, Count right) -> Int32",
        ),
        PROGRAM.replace(
            "callself T = Number::Add(Self,Self)",
            "callvirt Number::Add(Self,Self)",
        ),
        PROGRAM.replace(".field Value Int32", ".field Value Self"),
        PROGRAM.replace(".function Main() -> Int32", ".function Main() -> Self"),
    ] {
        assert!(
            prepare(&source).is_err(),
            "admitted invalid Self contract: {source}"
        );
    }
}

#[test]
fn self_instance_clone_uses_concrete_receiver_and_result() {
    let source = r#"
.module CloneSelf
.entry Main
.interface Cloneable
.method instance Clone() -> Self
.end
.end
.type Cell
.implements Cloneable
.field Value Int32
.method instance Clone() -> Cell
ldarg this
ret
.end
.end
.function Main() -> Int32
ldc.i4 42
newobj Cell
callself Cell = instance Cloneable::Clone()
ldfld Cell::Value
ret
.end
"#;
    let program = prepare(source).unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
    assert!(prepare(&source.replace("callself Cell =", "callvirt")).is_err());
}

#[test]
fn native_self_substitutes_inside_array_and_constructed_signatures() {
    let source = r#"
.module NestedSelf
.entry Main
.interface Wrap
.method static Wrap(Self value) -> Box<Self>
.end
.method static Copy(arrayref<Self> value) -> arrayref<Self>
.end
.end
.type Box<T>
.field Value T
.end
.type Cell
.implements Wrap
.field Value Int32
.method static Wrap(Cell value) -> Box<Cell>
ldarg value
newobj Box<Cell>
ret
.end
.method static Copy(arrayref<Cell> value) -> arrayref<Cell>
ldarg value
ret
.end
.end
.function Main() -> Int32
ldc.i4 42
newobj Cell
callself Cell = Wrap::Wrap(Self)
ldfld Box<Cell>::Value
ldfld Cell::Value
ret
.end
"#;
    assert_eq!(
        prepare(source)
            .unwrap()
            .run(Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
    assert!(
        prepare(&source.replace("Copy(arrayref<Cell> value)", "Copy(arrayref<Int32> value)"))
            .is_err()
    );
}

#[test]
fn self_in_an_inherited_generic_contract_is_bound_to_implementer() {
    let source = PROGRAM.replace(".interface Number", ".interface Comparable<T>\n.method static Compare(T a, T b) -> Int32\n.end\n.end\n.interface Number\n.implements Comparable<Self>")
        .replace(".field Value Int32", ".field Value Int32\n.method static Compare(Count a, Count b) -> Int32\nldc.i4 0\nret\n.end");
    let source = source.replace(
        "callself T = Number::Add(Self,Self)",
        "callself T = Comparable<T>::Compare(T,T)\npop\nldarg left\nldarg right\ncallself T = Number::Add(Self,Self)",
    );
    assert_eq!(
        prepare(&source)
            .unwrap()
            .run(Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
}

#[test]
fn system_number_has_native_self_and_runs_a_constrained_numeric_algorithm() {
    let source = r#"
.module SystemNumberSelf
.entry Main
.function Add<T>(T left, T right) -> T
.constraint T System.Number
ldarg left
ldarg right
callself T = System.Number::op_Addition(Self,Self)
callself T = System.Number::get_Zero()
callself T = System.Number::op_Addition(Self,Self)
ret
.end
.function Main() -> Int32
ldc.i4 20
ldc.i4 22
call Add<Int32>(Int32,Int32)
ret
.end
"#;
    let mut library = assemble(&format!(
        "{}\n{}",
        neoclr::library::system_source(),
        include_str!("../runtime/raven/generated/Number.methods.neoil")
    ))
    .unwrap();
    library
        .types
        .iter_mut()
        .find(|t| t.name == "System.Int32")
        .unwrap()
        .implements
        .push(Type::Named("System.Number".into()));
    // Use the actual generated numeric member bodies, not copies of their arithmetic.
    let methods = include_str!("../runtime/raven/generated/Int32.methods.neoil")
        .split(".method ")
        .filter(|part| {
            part.starts_with("static get_Zero(")
                || part.starts_with("static get_One(")
                || part.starts_with("static op_")
        })
        .map(|part| format!(".method {}\n.end\n", part.split("\n.end").next().unwrap()))
        .collect::<String>();
    let implementations = assemble(&format!(
        ".module System\n.type System.Int32\n{methods}.end\n"
    ))
    .unwrap();
    for mut method in implementations.functions {
        if library.functions.iter().any(|existing| {
            existing.name == method.name && existing.parameters == method.parameters
        }) {
            continue;
        }
        method.definition = None;
        library.functions.push(method);
    }
    let modules = neoclr::assembler::read_modules(
        &[neoclr::assembler::ModuleInput::Source(source)],
        &library,
    )
    .unwrap();
    let program = LoadedProgram::with_library(&modules[0], &library).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}
