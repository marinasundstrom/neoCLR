use neoclr::{Limits, LoadedProgram, Value, assemble, load};
fn program(extra: &str, body: &str, returns: &str) -> Result<LoadedProgram, neoclr::Fault> {
    let module = assemble(&format!(
        ".module App\n.entry Main\n{extra}\n.function Main() -> {returns}\n{body}\nret\n.end"
    ))?;
    LoadedProgram::new(&load(&serde_json::to_string(&module).unwrap())?)
}
#[test]
fn sample_passes_and_forwards_an_explicit_reference() {
    let p = LoadedProgram::new(
        &assemble(include_str!("../examples/reference_parameters.neoil")).unwrap(),
    )
    .unwrap();
    p.verify().unwrap();
    let result = p.run(Limits::default()).unwrap();
    assert_eq!(result.output, ["42"]);
    assert_eq!(result.memory.live_allocations(), 0);
}
#[test]
fn string_and_generic_record_slots_do_not_require_native_layout() {
    let extra = ".type Box<T>\n.field Value T\n.end\n.type Assign<T>\n.method static Set(T& destination,T value) -> Void\nldarg destination\nldarg value\nstobj T\nldvoid\nret\n.end\n.end";
    for (ty, before, after, result) in [
        (
            "String",
            "ldstr \"before\"",
            "ldstr \"after\"",
            Value::String("after".into()),
        ),
        (
            "Box<String>",
            "ldstr \"before\"\nnewobj Box<String>",
            "ldstr \"after\"\nnewobj Box<String>",
            Value::Object {
                ty: neoclr::assembler::parse_type("Box<String>").unwrap(),
                fields: vec![Value::String("after".into())],
            },
        ),
    ] {
        let body = format!(
            ".local {ty} value\n{before}\nstloc value\nldloca value\n{after}\ncall Assign<{ty}>::Set({ty}&,{ty})\npop\nldloc value"
        );
        let p = program(extra, &body, ty).unwrap();
        p.verify().unwrap();
        assert_eq!(p.run(Limits::default()).unwrap().value, result);
    }
}
#[test]
fn indirect_store_initializes_a_local_and_normalizes_small_values() {
    let p = program(
        "",
        ".local Byte value\nldloca value\nldc.i4 257\nstobj Byte\nldloc value",
        "Byte",
    )
    .unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Byte(1));
}
#[test]
fn writable_aliases_observe_writes_immediately() {
    let extra = ".function Change(Int32& left,Int32& right) -> Int32\nldarg left\nldc.i4 42\nstobj Int32\nldarg right\nldobj Int32\nret\n.end";
    let p = program(
        extra,
        ".local Int32 value\nldc.i4 0\nstloc value\nldloca value\ndup\ncall Change(Int32&,Int32&)",
        "Int32",
    )
    .unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}
#[test]
fn argument_address_targets_the_callees_value_copy() {
    let extra = ".function Change(Int32 value) -> Int32\nldarga value\nldc.i4 42\nstobj Int32\nldarg value\nret\n.end";
    let p = program(extra, "ldc.i4 7\ncall Change(Int32)", "Int32").unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}
#[test]
fn uninitialized_reference_arguments_fault_even_without_verification() {
    let extra = ".function Read(Int32& value) -> Int32\nldarg value\nldobj Int32\nret\n.end";
    let p = program(
        extra,
        ".local Int32 value\nldloca value\ncall Read(Int32&)",
        "Int32",
    )
    .unwrap();
    assert!(
        p.run(Limits::default())
            .unwrap_err()
            .message
            .contains("uninitialized")
    );
}
#[test]
fn unsupported_reference_storage_and_parameter_rebinding_are_rejected() {
    for (extra, body, returns) in [
        ("", ".local Int32 value\nldloca value", "Int32&"),
        ("", ".local Int32& value\nldloca value\npop\nldvoid", "Void"),
        (
            ".function Bad() -> Int32&\n.pinvoke \"missing\" \"bad\" cdecl\n.end",
            "ldvoid",
            "Void",
        ),
        (
            ".function Bad() -> Int32&\n.methodimpl InternalCall\n.end",
            "ldvoid",
            "Void",
        ),
        (
            "",
            ".local Int32 value\nldloca value\nvalue.pack Int32&",
            "System.Value",
        ),
        (
            ".function Bad(Int32& value) -> Void\nldarg value\nstarg value\nldvoid\nret\n.end",
            "ldvoid",
            "Void",
        ),
        (
            ".function Bad(Int32& value) -> Void\nldarga value\npop\nldvoid\nret\n.end",
            "ldvoid",
            "Void",
        ),
        (
            "",
            ".local Int32 value\nldloca value\nheap.new\npop\nldvoid",
            "Void",
        ),
    ] {
        assert!(
            program(extra, body, returns)
                .and_then(|p| p.run(Limits::default()))
                .is_err(),
            "accepted {body}"
        );
    }
}

#[test]
fn out_assigns_uninitialized_string_slots_and_forwards_obligations() {
    let extra = ".function Write(out String& destination) -> Void\nldarg destination\nldstr \"assigned\"\nstobj String\nldvoid\nret\n.end\n.function Forward(out String& destination) -> Void\nldarg destination\ncall Write(String&)\nret\n.end";
    let p = program(
        extra,
        ".local String value\nldloca value\ncall Forward(String&)\npop\nldloc value",
        "String",
    )
    .unwrap();
    p.verify().unwrap();
    assert_eq!(
        p.run(Limits::default()).unwrap().value,
        Value::String("assigned".into())
    );
}

#[test]
fn out_requires_a_write_this_invocation_even_when_the_slot_was_initialized() {
    for body in ["ldvoid", "ldarg destination\nldobj Int32\npop\nldvoid"] {
        let extra = format!(".function Bad(out Int32& destination) -> Void\n{body}\nret\n.end");
        let p = program(
            &extra,
            ".local Int32 value\nldc.i4 7\nstloc value\nldloca value\ncall Bad(Int32&)",
            "Void",
        )
        .unwrap();
        assert!(
            p.run(Limits::default())
                .unwrap_err()
                .message
                .contains("out parameter")
        );
    }
}

#[test]
fn aliases_can_fulfill_multiple_out_obligations() {
    let extra = ".function Assign(out Int32& left,out Int32& right) -> Void\nldarg left\nldc.i4 42\nstobj Int32\nldvoid\nret\n.end";
    let p = program(
        extra,
        ".local Int32 value\nldloca value\ndup\ncall Assign(Int32&,Int32&)\npop\nldloc value",
        "Int32",
    )
    .unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    let distinct = program(extra, ".local Int32 left\n.local Int32 right\nldloca left\nldloca right\ncall Assign(Int32&,Int32&)", "Void").unwrap();
    assert!(distinct.run(Limits::default()).is_err());
}

#[test]
fn an_out_argument_does_not_initialize_a_readwrite_alias_at_call_entry() {
    let extra = ".function Assign(out Int32& left,Int32& right) -> Void\nldarg left\nldc.i4 42\nstobj Int32\nldvoid\nret\n.end";
    let p = program(
        extra,
        ".local Int32 value\nldloca value\ndup\ncall Assign(Int32&,Int32&)",
        "Void",
    )
    .unwrap();
    assert!(p.verify().is_err());
    assert!(p.run(Limits::default()).is_err());
}

#[test]
fn output_contract_metadata_rejects_invalid_indices_and_nonreferences() {
    assert!(
        program(
            ".function Bad(out Int32 value) -> Void\nldvoid\nret\n.end",
            "ldvoid",
            "Void"
        )
        .is_err()
    );
    let mut module =
        assemble(".module App\n.function Assign(Int32& value) -> Void\nldvoid\nret\n.end").unwrap();
    for indices in [vec![1], vec![0, 0]] {
        module.functions[0].out_parameters = indices;
        assert!(LoadedProgram::new(&module).is_err());
    }
}

#[test]
fn reference_receiver_mutates_original_while_value_receiver_reads_a_copy() {
    let p = LoadedProgram::new(
        &assemble(include_str!("../examples/reference_receivers.neoil")).unwrap(),
    )
    .unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().output, ["42"]);
    let graph = p
        .analyze_reachability(
            &[neoclr::assembler::parse_function_ref("Main()").unwrap()],
            10,
        )
        .unwrap();
    assert!(
        graph
            .functions
            .iter()
            .find(|f| f.target.name == "Counter.Set")
            .unwrap()
            .receiver_byref
    );
    let wrong = include_str!("../examples/reference_receivers.neoil")
        .replace("ldloca counter", "ldloc counter");
    let p = LoadedProgram::new(&assemble(&wrong).unwrap()).unwrap();
    assert!(p.verify().is_err());
    assert!(p.run(Limits::default()).is_err());
}

#[test]
fn generic_reference_receivers_can_replace_non_native_payloads() {
    let extra = ".type Box<T>\n.field Value T\n.method instance byref Set(T value) -> Void\nldarg this\nldarg value\nnewobj Box<T>\nstobj Box<T>\nldvoid\nret\n.end\n.end";
    let p = program(extra, ".local Box<String> box\nldstr \"before\"\nnewobj Box<String>\nstloc box\nldloca box\nldstr \"after\"\ncall instance Box<String>::Set(String)\npop\nldloc box\nldfld Box<String>::Value", "String").unwrap();
    p.verify().unwrap();
    assert_eq!(
        p.run(Limits::default()).unwrap().value,
        Value::String("after".into())
    );
}

#[test]
fn receiver_modes_cannot_be_used_for_static_methods_or_constructors_or_overload_identity() {
    for source in [
        ".type C\n.method static byref M() -> Void\nldvoid\nret\n.end\n.end",
        ".type C\n.method instance byref .ctor() -> Void\nldvoid\nret\n.end\n.end",
        ".type C\n.method instance M() -> Void\nldvoid\nret\n.end\n.method instance byref M() -> Void\nldvoid\nret\n.end\n.end",
    ] {
        assert!(program(source, "ldvoid", "Void").is_err());
    }
}

#[test]
fn conditional_out_initializes_only_the_success_branch() {
    let extra = ".function Try(out(true) Int32& destination,Boolean success) -> Boolean\nldarg success\nbrfalse Miss\nldarg destination\nldc.i4 42\nstobj Int32\nldc.bool true\nret\nMiss:\nldc.bool false\nret\n.end";
    for branch in [
        "brfalse Miss\nldloc value\nret\nMiss:\nldc.i4 -1",
        "brtrue Hit\nldc.i4 -1\nret\nHit:\nldloc value",
    ] {
        for success in [true, false] {
            let p = program(extra, &format!(".local Int32 value\nldloca value\nldc.bool {success}\ncall Try(Int32&,Boolean)\n{branch}"), "Int32").unwrap();
            p.verify().unwrap();
            assert_eq!(
                p.run(Limits::default()).unwrap().value,
                Value::Int32(if success { 42 } else { -1 })
            );
        }
    }
    let p = program(extra, ".local Int32 value\nldloca value\nldc.bool false\ncall Try(Int32&,Boolean)\npop\nldloc value", "Int32").unwrap();
    assert!(p.verify().is_err());
    assert!(p.run(Limits::default()).is_err());
    let same_edge = program(extra, ".local Int32 value\nldloca value\nldc.bool false\ncall Try(Int32&,Boolean)\nbrtrue Join\nJoin:\nldloc value", "Int32").unwrap();
    assert!(same_edge.verify().is_err());
}

#[test]
fn conditional_out_requires_assignment_on_true_but_not_false() {
    for success in [true, false] {
        let extra = format!(
            ".function Try(out(true) Int32& destination) -> Boolean\nldc.bool {success}\nret\n.end"
        );
        let p = program(
            &extra,
            ".local Int32 value\nldloca value\ncall Try(Int32&)",
            "Boolean",
        )
        .unwrap();
        p.verify().unwrap();
        let result = p.run(Limits::default());
        if success {
            assert!(result.unwrap_err().message.contains("out parameter"));
        } else {
            assert_eq!(result.unwrap().value, Value::Boolean(false));
        }
    }
    assert!(
        program(
            ".function Bad(out(true) Int32& value) -> Void\nldvoid\nret\n.end",
            "ldvoid",
            "Void"
        )
        .is_err()
    );
}

#[test]
fn conditional_output_forwarding_preserves_each_invocations_obligation() {
    let extra = r#"
.function Try(out(true) Int32& destination, Boolean success) -> Boolean
    ldarg success
    brfalse Miss
    ldarg destination
    ldc.i4 42
    stobj Int32
    ldc.bool true
    ret
Miss:
    ldc.bool false
    ret
.end
.function Forward(out(true) Int32& destination, Boolean success) -> Boolean
    ldarg destination
    ldarg success
    call Try(Int32&,Boolean)
    ret
.end
"#;
    for success in [true, false] {
        let body = format!(
            ".local Int32 value\nldloca value\nldc.bool {success}\ncall Forward(Int32&,Boolean)\nbrfalse Miss\nldloc value\nret\nMiss:\nldc.i4 -1"
        );
        let p = program(extra, &body, "Int32").unwrap();
        p.verify().unwrap();
        assert_eq!(
            p.run(Limits::default()).unwrap().value,
            Value::Int32(if success { 42 } else { -1 })
        );
    }
    // Forwarding a miss cannot satisfy an unconditional output promise, even
    // when the caller supplied a previously initialized slot.
    let unconditional = extra.replace("Forward(out(true)", "Forward(out");
    let p = program(&unconditional, ".local Int32 value\nldc.i4 7\nstloc value\nldloca value\nldc.bool false\ncall Forward(Int32&,Boolean)", "Boolean").unwrap();
    p.verify().unwrap();
    let fault = p.run(Limits::default()).unwrap_err();
    assert!(fault.message.contains("out parameter"), "{fault}");
    assert_eq!(fault.function.as_deref(), Some("Forward"));
}

#[test]
fn conditional_output_alias_writes_satisfy_only_the_slots_actually_written() {
    let extra = r#"
.function Try(out(true) Int32& left, out(true) Int32& right) -> Boolean
    ldarg left
    ldc.i4 42
    stobj Int32
    ldc.bool true
    ret
.end
"#;
    for alias in [true, false] {
        let right = if alias { "left" } else { "right" };
        let body = format!(
            ".local Int32 left\n.local Int32 right\nldloca left\nldloca {right}\ncall Try(Int32&,Int32&)\nbrfalse Miss\nldloc left\nret\nMiss:\nldc.i4 -1"
        );
        let p = program(extra, &body, "Int32").unwrap();
        p.verify().unwrap();
        let result = p.run(Limits::default());
        if alias {
            assert_eq!(result.unwrap().value, Value::Int32(42));
        } else {
            assert!(result.unwrap_err().message.contains("out parameter"));
        }
    }
}

#[test]
fn reference_returns_sample_preserves_the_callers_counter() {
    let p =
        LoadedProgram::new(&assemble(include_str!("../examples/reference_returns.neoil")).unwrap())
            .unwrap();
    p.verify().unwrap();
    let result = p.run(Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.is_empty());
    assert_eq!(result.memory.live_allocations(), 0);
}

#[test]
fn returned_reference_keeps_the_callers_slot_and_sees_later_assignment() {
    let extra = ".function Forward(Int32& value) -> Int32&\nldarg value\nret\n.end";
    let p = program(extra, ".local Int32 value\n.local Int32& alias\nldc.i4 7\nstloc value\nldloca value\ncall Forward(Int32&)\ncall Forward(Int32&)\nstloc alias\nldc.i4 42\nstloc value\nldloc alias\nldobj Int32", "Int32").unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}

#[test]
fn current_frame_references_cannot_escape_directly_or_through_aliases() {
    let forward = ".function Forward(String& value) -> String&\nldarg value\nret\n.end";
    for body in [
        ".local String value\nldstr \"local\"\nstloc value\nldloca value",
        "ldarga value",
        ".local String& alias\nldarga value\nstloc alias\nldloc alias",
        "ldarga value\ncall Forward(String&)",
    ] {
        let extra = format!("{forward}\n.function Bad(String value) -> String&\n{body}\nret\n.end");
        let p = program(
            &extra,
            "ldstr \"argument\"\ncall Bad(String)\nldobj String",
            "String",
        )
        .unwrap();
        // Origin tracking through aliases/calls is optional; execution always checks.
        let fault = p.run(Limits::default()).unwrap_err();
        assert!(fault.message.contains("current frame"), "{fault}");
        assert_eq!(fault.function.as_deref(), Some("Bad"));
        if !body.contains("alias") && !body.contains("Forward") {
            assert!(p.verify().is_err());
        }
    }
}

#[test]
fn reference_local_rebinding_and_self_assignment_preserve_other_aliases() {
    let extra = ".function Forward(String& value) -> String&\nldarg value\nret\n.end";
    let p = program(extra, ".local String one\n.local String two\n.local String& first\n.local String& alias\nldstr \"first\"\nstloc one\nldstr \"second\"\nstloc two\nldloca one\ncall Forward(String&)\nstloc first\nldloc first\nstloc alias\nldloc first\nstloc first\nldloca two\ncall Forward(String&)\nstloc first\nldloc alias\nldstr \"still alive\"\nstobj String\nldloc first\nldobj String", "String").unwrap();
    p.verify().unwrap();
    assert_eq!(
        p.run(Limits::default()).unwrap().value,
        Value::String("second".into())
    );
}

#[test]
fn uninitialized_references_cannot_be_stored_or_returned() {
    for extra in [
        ".function Bad() -> Int32&\n.local Int32 value\nldloca value\nret\n.end",
        ".function Bad() -> Int32&\n.local Int32 value\n.local Int32& alias\nldloca value\nstloc alias\nldloc alias\nret\n.end",
    ] {
        let p = program(extra, "call Bad()\nldobj Int32", "Int32").unwrap();
        assert!(p.verify().is_err());
        assert!(
            p.run(Limits::default())
                .unwrap_err()
                .message
                .contains("uninitialized")
        );
    }
}

#[test]
fn returning_an_out_reference_requires_assignment_during_that_call() {
    for assign in ["", "ldarg value\nldc.i4 42\nstobj Int32"] {
        let extra =
            format!(".function Set(out Int32& value) -> Int32&\n{assign}\nldarg value\nret\n.end");
        let p = program(&extra, ".local Int32 value\nldc.i4 7\nstloc value\nldloca value\ncall Set(Int32&)\nldobj Int32", "Int32").unwrap();
        p.verify().unwrap();
        let result = p.run(Limits::default());
        if assign.is_empty() {
            assert!(result.unwrap_err().message.contains("out parameter"));
        } else {
            assert_eq!(result.unwrap().value, Value::Int32(42));
        }
    }
}

#[test]
fn host_frame_reference_results_are_rejected_by_return_provenance() {
    let p = program(
        "",
        ".local Int32 value\nldc.i4 42\nstloc value\nldloca value",
        "Int32&",
    )
    .unwrap();
    assert!(p.verify().is_err());
    assert!(
        p.run(Limits::default())
            .unwrap_err()
            .message
            .contains("current frame")
    );
    let target = neoclr::assembler::parse_function_ref("Main()").unwrap();
    assert!(
        p.resolve_function(&target)
            .unwrap()
            .invoke(vec![], Limits::default())
            .unwrap_err()
            .message
            .contains("current frame")
    );
}

#[test]
fn returned_interface_view_must_refer_to_caller_storage() {
    let extra = r#"
.interface Readable
    .method instance Read() -> String
    .end
.end
.type Text
    .implements Readable
    .field Value String
    .method instance Read() -> String
        ldarg this
        ldfld Text::Value
        ret
    .end
.end
.function View(Text& text) -> Readable&
    ldarg text
    interface.borrow Readable
    ret
.end
.function Bad() -> Readable&
    .local Text text
    ldstr "local"
    newobj Text
    stloc text
    ldloca text
    call View(Text&)
    ret
.end
"#;
    let p = program(extra, ".local Text text\n.local Readable& value\nldstr \"retained\"\nnewobj Text\nstloc text\nldloca text\ncall View(Text&)\nstloc value\nldloc value\ncallvirt instance Readable::Read()", "String").unwrap();
    p.verify().unwrap();
    assert_eq!(
        p.run(Limits::default()).unwrap().value,
        Value::String("retained".into())
    );
    let bad = program(
        extra,
        "call Bad()\ncallvirt instance Readable::Read()",
        "String",
    )
    .unwrap();
    assert!(
        bad.run(Limits::default())
            .unwrap_err()
            .message
            .contains("current frame")
    );
}

#[test]
fn nested_generic_field_alias_tracks_same_type_parent_replacement() {
    let extra = r#"
.type Box<T>
    .field Value T
.end
.function Field(Box<Box<String>>& box) -> String&
    ldarg box
    ldflda Box<Box<String>>::Value
    ldflda Box<String>::Value
    ret
.end
"#;
    let p = program(extra, ".local Box<Box<String>> box\n.local String& field\nldstr \"before\"\nnewobj Box<String>\nnewobj Box<Box<String>>\nstloc box\nldloca box\ncall Field(Box<Box<String>>&)\nstloc field\nldstr \"replacement\"\nnewobj Box<String>\nnewobj Box<Box<String>>\nstloc box\nldloc field\nldobj String", "String").unwrap();
    p.verify().unwrap();
    assert_eq!(
        p.run(Limits::default()).unwrap().value,
        Value::String("replacement".into())
    );
}

#[test]
fn field_references_cannot_escape_their_owning_frame() {
    let prefix = ".type Counter\n.field Age Int32\n.end\n.function Age(Counter& counter) -> Int32&\nldarg counter\nldflda Counter::Age\nret\n.end";
    for body in [
        "ldarga counter\nldflda Counter::Age",
        "ldarga counter\ncall Age(Counter&)",
        ".local Counter local\nldarg counter\nstloc local\nldloca local\ncall Age(Counter&)",
        ".local Int32& alias\nldarga counter\ncall Age(Counter&)\nstloc alias\nldloc alias",
    ] {
        let extra =
            format!("{prefix}\n.function Bad(Counter counter) -> Int32&\n{body}\nret\n.end");
        let p = program(
            &extra,
            "ldc.i4 7\nnewobj Counter\ncall Bad(Counter)\nldobj Int32",
            "Int32",
        )
        .unwrap();
        let fault = p.run(Limits::default()).unwrap_err();
        assert!(fault.message.contains("current frame"), "{fault}");
        assert_eq!(fault.function.as_deref(), Some("Bad"));
    }
}

#[test]
fn sibling_field_writes_do_not_fulfill_an_output_obligation() {
    let extra = ".type Pair\n.field Left Int32\n.field Right Int32\n.end\n.function Assign(out Int32& left,Int32& right) -> Void\nldarg right\nldc.i4 42\nstobj Int32\nldvoid\nret\n.end";
    for field in ["Left", "Right"] {
        let body = format!(
            ".local Pair pair\nldc.i4 0\nldc.i4 0\nnewobj Pair\nstloc pair\nldloca pair\nldflda Pair::Left\nldloca pair\nldflda Pair::{field}\ncall Assign(Int32&,Int32&)\npop\nldloc pair\nldfld Pair::Left"
        );
        let p = program(extra, &body, "Int32").unwrap();
        p.verify().unwrap();
        let result = p.run(Limits::default());
        if field == "Left" {
            assert_eq!(result.unwrap().value, Value::Int32(42));
        } else {
            assert!(result.unwrap_err().message.contains("out parameter"));
        }
    }
}

#[test]
fn replacing_a_parent_fulfills_its_field_output_obligation() {
    let extra = ".type Counter\n.field Age Int32\n.end\n.function Replace(out Int32& age,Counter& counter) -> Void\nldarg counter\nldc.i4 42\nnewobj Counter\nstobj Counter\nldvoid\nret\n.end";
    let p = program(extra, ".local Counter counter\nldc.i4 7\nnewobj Counter\nstloc counter\nldloca counter\nldflda Counter::Age\nldloca counter\ncall Replace(Int32&,Counter&)\npop\nldloc counter\nldfld Counter::Age", "Int32").unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}

#[test]
fn managed_field_addresses_require_initialized_records_and_valid_fields() {
    for body in [
        ".local Counter counter\nldloca counter\nldflda Counter::Age\nldobj Int32",
        ".local Counter counter\nldc.i4 7\nnewobj Counter\nstloc counter\nldloca counter\nldflda 9\nldobj Int32",
    ] {
        let p = program(".type Counter\n.field Age Int32\n.end", body, "Int32").unwrap();
        assert!(p.verify().is_err());
        assert!(p.run(Limits::default()).is_err());
    }
}
