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

const GENERIC_CLONE: &str = r#"
.module GenericClone
.entry Main
.interface Clonable
.method instance Clone() -> Self
.end
.end
.type Cell
.implements Clonable
.field Value Int32
.method instance byref Clone() -> Cell
ldarg this
ldobj Cell
ret
.end
.end
.type class Box
.implements Clonable
.field Value Int32
.method instance .ctor(Int32 value) -> noresult
ldarg this
ldarg value
stfld Box::Value
ret
.end
.method instance Clone() -> Box
ldarg this
ldfld Box::Value
newobj instance Box::.ctor(Int32)
ret
.end
.end
.function Clone<T>(T value) -> T
.constraint T Clonable
ldarga value
callself borrow T = instance Clonable::Clone()
ret
.end
.function Main() -> Int32
.local Cell cell
.local Box original
.local Box copy
ldc.i4 20
newobj Cell
call Clone<Cell>(Cell)
stloc cell
ldc.i4 22
newobj instance Box::.ctor(Int32)
stloc original
ldloc original
call Clone<Box>(Box)
stloc copy
ldloc original
ldc.i4 99
stfld Box::Value
ldloc copy
ldfld Box::Value
ldloc cell
ldfld Cell::Value
add
ret
.end
"#;

#[test]
fn generic_clone_adapts_value_and_reference_receivers_without_boxing() {
    let program = prepare(GENERIC_CLONE).unwrap();
    let execution = program.run(Limits::default()).unwrap();
    assert_eq!(execution.value, Value::Int32(42));
    assert_eq!(execution.heap.statistics().allocated_objects, 2);
}

#[test]
fn generic_clone_rejects_unproven_copying_and_readonly_receivers() {
    for source in [
        GENERIC_CLONE.replace(".constraint T Clonable", ""),
        GENERIC_CLONE.replace("callself borrow T", "callself T"),
        GENERIC_CLONE.replace("ldarga value", "ldarg value"),
        GENERIC_CLONE.replace(
            "instance byref Clone() -> Cell\nldarg this\nldobj Cell",
            "instance Clone() -> Cell\nldarg this",
        ),
    ] {
        assert!(
            prepare(&source)
                .and_then(|program| program.run(Limits::default()))
                .is_err(),
            "invalid borrowed Self call accepted: {source}"
        );
    }
    let readonly = GENERIC_CLONE
        .split(".function Main()")
        .next()
        .unwrap()
        .replace("Clone<T>(T value)", "Clone<T>(readonly T& value)")
        .replace("ldarga value", "ldarg value")
        + ".function Main() -> Int32\nldc.i4 0\nret\n.end\n";
    assert!(
        prepare(&readonly)
            .err()
            .unwrap()
            .message
            .contains("readonly")
    );
    assert!(prepare(&PROGRAM.replace("callself T", "callself borrow T")).is_err());
}

#[test]
fn generic_clone_checks_null_even_when_implementation_ignores_receiver() {
    let source = GENERIC_CLONE
        .replace("ldarg this\nldfld Box::Value\nnewobj", "ldc.i4 42\nnewobj")
        .replace(
            "ldc.i4 22\nnewobj instance Box::.ctor(Int32)\nstloc original",
            "ldloca original\ninitobj Box",
        );
    assert!(
        prepare(&source)
            .unwrap()
            .run(Limits::default())
            .unwrap_err()
            .message
            .contains("null Self receiver")
    );
}

#[test]
fn borrowed_self_mutation_preserves_the_original_value_slot() {
    let source = GENERIC_CLONE
        .replace("Clone<T>(T value)", "Clone<T>(T& value)")
        .replace("ldarga value", "ldarg value")
        .replace("instance byref Clone() -> Cell\nldarg this", "instance byref Clone() -> Cell\nldarg this\nldc.i4 21\nstfld Cell::Value\npop\nldarg this");
    let source = source.split(".function Main()").next().unwrap().to_owned()
        + r#"
.function Main() -> Int32
.local Cell original
ldc.i4 20
newobj Cell
stloc original
ldloca original
call Clone<Cell>(Cell&)
ldfld Cell::Value
ldloc original
ldfld Cell::Value
add
ret
.end
"#;
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
fn borrowed_self_reference_dispatch_preserves_virtual_targets_in_execution_and_graph() {
    let source = r#"
.module BorrowedVirtual
.entry Main
.interface Reader
.method instance Read() -> Int32
.end
.end
.type class Base
.implements Reader
.method instance .ctor() -> noresult
ret
.end
.method instance virtual Read() -> Int32
ldc.i4 20
ret
.end
.end
.type class Derived
.extends Base
.method instance .ctor() -> noresult
ldarg this
call instance Base::.ctor()
ret
.end
.method instance override Read() -> Int32
ldc.i4 42
ret
.end
.end
.function Read<T>(T value) -> Int32
.constraint T Reader
ldarga value
callself borrow T = instance Reader::Read()
ret
.end
.function Main() -> Int32
newobj instance Derived::.ctor()
call Read<Base>(Base)
ret
.end
"#;
    let program = prepare(source).unwrap();
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
    assert!(
        graph
            .functions
            .iter()
            .any(|f| f.target.name == "Derived.Read")
    );
}

#[test]
fn system_clonable_uses_native_self_for_value_and_reference_clones() {
    let source = GENERIC_CLONE
        .replace(
            ".interface Clonable\n.method instance Clone() -> Self\n.end\n.end",
            include_str!("../runtime/raven/generated/Clonable.methods.neoil"),
        )
        .replace(".implements Clonable", ".implements System.Clonable")
        .replace(".constraint T Clonable", ".constraint T System.Clonable")
        .replace(
            "instance Clonable::Clone()",
            "instance System.Clonable::Clone()",
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

const INHERITED_CLONE: &str = r#"
.module InheritedClone
.entry Main
.interface Clonable
.method instance Clone() -> Self
.end
.end
.type class Base
.implements Clonable
.method instance .ctor() -> noresult
ret
.end
.method instance virtual Clone() -> Base
newobj instance Base::.ctor()
ret
.end
.method instance virtual Read() -> Int32
ldc.i4 20
ret
.end
.end
.type class Derived
.extends Base
.method instance .ctor() -> noresult
ldarg this
call instance Base::.ctor()
ret
.end
.method instance override Read() -> Int32
ldc.i4 42
ret
.end
.end
.function Copy<T>(T value) -> T
.constraint T Clonable
ldarga value
callself borrow T = instance Clonable::Clone()
ret
.end
.function Main() -> Int32
newobj instance Derived::.ctor()
call Copy<Base>(Base)
callvirt instance Base::Read()
ret
.end
"#;

#[test]
fn inherited_self_keeps_base_result_and_virtual_override_contract() {
    assert_eq!(
        prepare(INHERITED_CLONE)
            .unwrap()
            .run(Limits::default())
            .unwrap()
            .value,
        Value::Int32(20)
    );
    let overridden = INHERITED_CLONE.replace(".method instance override Read()", ".method instance override Clone() -> Base\nnewobj instance Derived::.ctor()\nret\n.end\n.method instance override Read()");
    assert_eq!(
        prepare(&overridden)
            .unwrap()
            .run(Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
}

#[test]
fn inherited_self_does_not_satisfy_derived_generic_bound_or_direct_dispatch() {
    let invalid = INHERITED_CLONE.replace("call Copy<Base>(Base)", "call Copy<Derived>(Derived)");
    assert!(
        prepare(&invalid)
            .unwrap_err()
            .to_string()
            .contains("constraint")
    );
    let invalid = INHERITED_CLONE.replace("newobj instance Derived::.ctor()\ncall Copy<Base>(Base)", ".local Derived value\nnewobj instance Derived::.ctor()\nstloc value\nldloca value\ncallself borrow Derived = instance Clonable::Clone()");
    assert!(
        prepare(&invalid)
            .unwrap_err()
            .to_string()
            .contains("conformance declared")
    );
}

#[test]
fn derived_self_requires_redeclaration_and_exact_derived_result() {
    let redeclared =
        INHERITED_CLONE.replace(".extends Base", ".extends Base\n.implements Clonable");
    assert!(prepare(&redeclared).is_err());
    let valid = redeclared.replace(".method instance override Read()", ".method private instance DerivedClone() -> Derived\n.override instance Clonable::Clone()\nnewobj instance Derived::.ctor()\nret\n.end\n.method instance override Read()")
        .replace("call Copy<Base>(Base)", "call Copy<Derived>(Derived)");
    assert_eq!(
        prepare(&valid)
            .unwrap()
            .run(Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
    let base_view = valid.replace("call Copy<Derived>(Derived)", "call Copy<Base>(Base)");
    assert_eq!(
        prepare(&base_view)
            .unwrap()
            .run(Limits::default())
            .unwrap()
            .value,
        Value::Int32(20)
    );
}
