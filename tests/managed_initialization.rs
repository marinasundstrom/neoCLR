use neoclr::{Limits, LoadedProgram, Value, assemble, metadata::Type};

fn program(extra: &str, body: &str, returns: &str) -> Result<LoadedProgram, neoclr::Fault> {
    let module = assemble(&format!(
        ".module App\n.entry Main\n{extra}\n.function Main() -> {returns}\n{body}\nret\n.end"
    ))?;
    let module = neoclr::load(&serde_json::to_string(&module).unwrap())?;
    LoadedProgram::new(&module)
}

#[test]
fn managed_defaults_have_exact_storage_types() {
    for (ty, expected) in [
        ("Void", Value::Void),
        ("Boolean", Value::Boolean(false)),
        ("SByte", Value::SByte(0)),
        ("Byte", Value::Byte(0)),
        ("Int16", Value::Int16(0)),
        ("UInt16", Value::UInt16(0)),
        ("Char", Value::Char(0)),
        ("Int32", Value::Int32(0)),
        ("UInt32", Value::UInt32(0)),
        ("Int64", Value::Int64(0)),
        ("UInt64", Value::UInt64(0)),
        ("IntPtr", Value::IntPtr(0)),
        ("UIntPtr", Value::UIntPtr(0)),
        ("Single", Value::Single(0.0)),
        ("Double", Value::Double(0.0)),
        (
            "Int32*",
            Value::Pointer(neoclr::memory::Pointer::null(Type::Int32)),
        ),
    ] {
        let p = program(
            "",
            &format!(".local {ty} value\nldloca value\ninitobj {ty}\nldloc value"),
            ty,
        )
        .unwrap();
        p.verify().unwrap();
        let result = p.run(Limits::default()).unwrap();
        assert_eq!(result.value, expected, "{ty}");
        assert_eq!(result.memory.live_allocations(), 0);
        assert!(result.heap.is_empty());
    }
}

#[test]
fn constructor_can_default_its_receiver_then_initialize_fields() {
    let p = LoadedProgram::new(
        &assemble(include_str!("../examples/value_initialization.neoil")).unwrap(),
    )
    .unwrap();
    p.verify().unwrap();
    let result = p.run(Limits::default()).unwrap();
    assert_eq!(result.output, ["0"]);
    assert_eq!(result.value, Value::Int32(42));
}

#[test]
fn initobj_does_not_invoke_constructors_or_require_field_visibility() {
    let extra = ".type Counter\n.field private Age Int32\n.method instance .ctor() -> Void\nfault \"constructor invoked\"\n.end\n.method instance Get() -> Int32\nldarg this\nldfld Counter::Age\nret\n.end\n.end";
    let p = program(extra, ".local Counter counter\nldloca counter\ninitobj Counter\nldloc counter\ncall instance Counter::Get()", "Int32").unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(0));
}

#[test]
fn nested_generic_defaults_and_recursive_pointer_fields_are_supported() {
    let extra = ".type Box<T>\n.field Value T\n.end\n.type Node\n.field Next Node*\n.field Count Int32\n.end";
    let p = program(extra, ".local Box<Box<Node>> value\nldloca value\ninitobj Box<Box<Node>>\nldloc value\nldfld Box<Box<Node>>::Value\nldfld Box<Node>::Value", "Node").unwrap();
    p.verify().unwrap();
    let Value::Object { fields, .. } = p.run(Limits::default()).unwrap().value else {
        panic!("expected record")
    };
    assert_eq!(fields[1], Value::Int32(0));
    let Value::Pointer(pointer) = &fields[0] else {
        panic!("expected pointer")
    };
    assert_eq!(pointer.address, 0);
    assert_eq!(pointer.allocation, None);
}

#[test]
fn resetting_an_owner_preserves_an_existing_field_alias() {
    let extra = ".type Counter\n.field Age Int32\n.end";
    let p = program(extra, ".local Counter counter\n.local Int32& age\nldc.i4 42\nnewobj Counter\nstloc counter\nldloca counter\nldflda Counter::Age\nstloc age\nldloca counter\ninitobj Counter\nldloc age\nldobj Int32", "Int32").unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(0));
}

#[test]
fn initobj_fulfills_outputs_without_reading_uninitialized_destinations() {
    let extra = ".type Defaults<T>\n.method static Reset(out T& value) -> Void\nldarg value\ninitobj T\nldvoid\nret\n.end\n.end";
    let p = program(
        extra,
        ".local Int32 value\nldloca value\ncall Defaults<Int32>::Reset(Int32&)\npop\nldloc value",
        "Int32",
    )
    .unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(0));
    // The open method verifies, but unsupported defaults must still fault when
    // its body is specialized and executed without running the verifier.
    let bad = program(extra, ".local String value\nldloca value\ncall Defaults<String>::Reset(String&)\npop\nldloc value", "String").unwrap();
    assert!(
        bad.run(Limits::default())
            .unwrap_err()
            .message
            .contains("default initialization")
    );
}

#[test]
fn field_initialization_respects_output_paths() {
    let extra = ".type Pair\n.field Left Int32\n.field Right Int32\n.end\n.function Reset(out Int32& left,Int32& right) -> Void\nldarg right\ninitobj Int32\nldvoid\nret\n.end";
    for field in ["Left", "Right"] {
        let body = format!(
            ".local Pair pair\nldc.i4 7\nldc.i4 8\nnewobj Pair\nstloc pair\nldloca pair\nldflda Pair::Left\nldloca pair\nldflda Pair::{field}\ncall Reset(Int32&,Int32&)\npop\nldloc pair\nldfld Pair::Left"
        );
        let p = program(extra, &body, "Int32").unwrap();
        p.verify().unwrap();
        let result = p.run(Limits::default());
        if field == "Left" {
            assert_eq!(result.unwrap().value, Value::Int32(0));
        } else {
            assert!(result.unwrap_err().message.contains("out parameter"));
        }
    }
}

#[test]
fn invalid_defaults_and_mismatched_destinations_are_rejected() {
    for ty in [
        "String",
        "Error",
        "System.Value",
        "RuntimeTypeHandle",
        "Int32&",
        "Int32&",
    ] {
        assert!(
            program(
                "",
                &format!(".local {ty} value\nldloca value\ninitobj {ty}\nldvoid"),
                "Void"
            )
            .is_err(),
            "{ty}"
        );
    }
    let p = program(
        "",
        ".local Int32 value\nldloca value\ninitobj Int64\nldvoid",
        "Void",
    )
    .unwrap();
    assert!(p.verify().is_err());
    assert!(
        p.run(Limits::default())
            .unwrap_err()
            .message
            .contains("type mismatch")
    );
    assert!(
        program(
            ".type Bad\n.field Text String\n.end",
            ".local Bad value\nldloca value\ninitobj Bad\nldvoid",
            "Void"
        )
        .is_err()
    );
}

#[test]
fn recursive_and_excessive_default_shapes_are_rejected() {
    assert!(
        program(
            ".type Loop\n.field Next Loop\n.end",
            ".local Loop value\nldloca value\ninitobj Loop\nldvoid",
            "Void"
        )
        .is_err()
    );
    let mut extra = String::from(".type N0\n.end\n");
    for i in 1..70 {
        extra.push_str(&format!(".type N{i}\n.field Value N{}\n.end\n", i - 1));
    }
    assert!(
        program(
            &extra,
            ".local N69 value\nldloca value\ninitobj N69\nldvoid",
            "Void"
        )
        .is_err()
    );
}
