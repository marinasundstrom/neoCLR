use neoclr::{Limits, LoadedProgram, Value, frontend};

fn run(source: &str) -> neoclr::Execution {
    let module = frontend::compile(source).unwrap();
    let json = serde_json::to_string(&module).unwrap();
    let module = neoclr::load(&json).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    program.run(Limits::default()).unwrap()
}

#[test]
fn flags_preserve_identity_and_integer_domain() {
    assert_eq!(
        run(r#"
flags enum Access { None = 0, Read = 1, Write = 2, Both = 3 }
enum Status { Start = -1, Ready, Done }
func Echo<T>(value: T) -> T { return value }
func Main() -> int {
    let flags = Echo(Access.Read | Access.Write)
    if flags != Access.Both { return -1 }
    if !flags.HasFlag(Access.Read) { return -2 }
    if !flags.HasFlag(Access.None) { return -3 }
    if (flags & Access.Read) != Access.Read { return -4 }
    if (flags ^ Access.Write) != Access.Read { return -5 }
    if (~Access.None).Value != -1 { return -6 }
    if default(Access) != Access.None { return -7 }
    if Status.Ready.Value != 0 { return -8 }
    if Access.FromValue(42).Value != 42 { return -9 }
    var copy = flags
    copy = Access.None
    if flags != Access.Both { return -10 }
    return Access.FromValue(42).Value
}
"#)
        .value,
        Value::Int32(42)
    );
}

#[test]
fn binding_flags_support_constants_operators_and_existing_factories() {
    assert_eq!(
        run(r#"
record Cell(Value: int)
func Main() -> int {
    let flags = System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Instance
    let fields = typeof(Cell).GetFields(flags)
    if fields.Length != 1 { return -1 }
    let old = System.Reflection.BindingFlags.Public().Or(System.Reflection.BindingFlags.Instance())
    if old != flags { return -2 }
    if !flags.HasFlag(System.Reflection.BindingFlags.Public) { return -3 }
    if flags.Value != 20 { return -4 }
    return 42
}
"#)
        .value,
        Value::Int32(42)
    );
    let system = neoclr::library::system().unwrap();
    let def = system
        .types
        .iter()
        .find(|d| d.name == "System.Reflection.BindingFlags")
        .unwrap();
    assert!(def.enum_info.as_ref().unwrap().flags);
    assert_eq!(def.enum_info.as_ref().unwrap().members.len(), 6);
}

#[test]
fn enum_source_rejects_invalid_contracts() {
    for source in [
        "enum E { A = 2147483648 }",
        "enum E { A = 2147483647, B }",
        "enum E { A, A }",
        "enum E { Or }",
        "enum E : string { A }",
    ] {
        assert!(
            frontend::compile(&format!("{source}\nfunc Main() -> int {{ return 0 }}")).is_err(),
            "{source}"
        );
    }
    for body in [
        "let value: E = 1\nreturn 0",
        "let value = E.A | F.A\nreturn 0",
        "let value = E.A + E.A\nreturn 0",
        "return E.A",
        "let value = E.FromValue(F.A)\nreturn 0",
    ] {
        assert!(
            frontend::compile(&format!(
                "enum E {{ A }}\nenum F {{ A }}\nfunc Main() -> int {{ {body} }}"
            ))
            .is_err(),
            "{body}"
        );
    }
}

#[test]
fn raw_metadata_enforces_enum_layout_and_nominal_calls() {
    let source = "enum E { A }\nfunc Main() -> int { return E.A.Value }";
    let good = frontend::compile(source).unwrap();
    for mutation in 0..5 {
        let mut bad = good.clone();
        let def = bad.types.iter_mut().find(|d| d.name == "E").unwrap();
        match mutation {
            0 => def.fields.clear(),
            1 => def.enum_info.as_mut().unwrap().underlying = neoclr::metadata::Type::Int64,
            2 => def.fields[0].visibility = neoclr::metadata::Visibility::Public,
            3 => def
                .enum_info
                .as_mut()
                .unwrap()
                .members
                .push(neoclr::metadata::EnumMember {
                    name: "A".into(),
                    value: 9,
                }),
            _ => def.is_abstract = true,
        }
        assert!(LoadedProgram::new(&bad).is_err(), "mutation {mutation}");
    }
    let il = ".module Bad\n.entry Main\n.type E\n.enum Int32\n.field private Bits Int32\n.end\n.type F\n.enum Int32\n.field private Bits Int32\n.end\n.function Read(E value) -> Int32\nldc.i4 0\nret\n.end\n.function Main() -> Int32\nldc.i4 0\nnewobj F\ncall Read(E)\nret\n.end";
    let module = neoclr::assemble(il).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    assert!(program.verify().is_err());
    assert!(program.run(Limits::default()).is_err());
}

#[test]
fn integer_bitwise_precedence_and_enum_reference_access_work() {
    assert_eq!(
        run(r#"
flags enum E { Low = 1, High = 2 }
func Main() -> int {
    if (1 | 2 & 4) != 1 { return -1 }
    if (7 ^ 3) != 4 { return -2 }
    var value = E.Low
    let reference = &value
    if reference.Value != 1 { return -3 }
    if reference != E.Low { return -4 }
    return 42
}
"#)
        .value,
        Value::Int32(42)
    );
}

#[test]
fn reflection_reports_enum_names_and_underlying_type() {
    assert_eq!(
        run(r#"
enum Order { Negative = -1, One = 1, Zero = 0, Alias = 1 }
func Main() -> int {
    if !typeof(Order).IsEnum { return -1 }
    if typeof(int).IsEnum { return -2 }
    if typeof(Order&).IsEnum { return -3 }
    if !typeof(Order).GetEnumUnderlyingType().Equals(typeof(int)) { return -4 }
    let names = typeof(Order).GetEnumNames()
    if names.Length != 4 { return -5 }
    if !names[0].Equals("Zero") { return -6 }
    if !names[1].Equals("One") { return -7 }
    if !names[2].Equals("Alias") { return -8 }
    if !names[3].Equals("Negative") { return -9 }
    return 42
}
"#)
        .value,
        Value::Int32(42)
    );
    let module = frontend::compile(
        "func Main() -> int { let names = typeof(int).GetEnumNames()\nreturn 0 }",
    )
    .unwrap();
    assert!(
        LoadedProgram::new(&module)
            .unwrap()
            .run(Limits::default())
            .is_err()
    );
}

#[test]
fn enum_example_runs() {
    assert_eq!(
        run(include_str!("../examples/source/enums.neo")).value,
        Value::Int32(0)
    );
}

#[test]
fn constants_lower_from_metadata_and_private_payload_stays_private() {
    let il = frontend::lower_to_il("enum E { A = 42 }\nfunc Main() -> int { return E.A.Value }")
        .unwrap();
    let main = il.split(".function Main").nth(1).unwrap();
    assert!(main.contains("ldc.i4 42\nnewobj E"));
    assert!(!main.contains("call E::A()"));
    let raw = ".module Bad\n.entry Main\n.type E\n.enum Int32\n.literal A 42\n.field private Bits Int32\n.end\n.function Main() -> Int32\nldc.i4 42\nnewobj E\nldfld E::Bits\nret\n.end";
    let module = neoclr::assemble(raw).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    assert!(program.verify().is_err());
    assert!(program.run(Limits::default()).is_err());
}
