use neoclr::{Limits, assemble, run, verify};

fn program(body: &str, returns: &str) -> neoclr::Module {
    assemble(&format!(
        ".module Test\n.entry Main\n.function Main() -> {returns}\n{body}\n.end"
    ))
    .unwrap()
}

#[test]
fn rejects_wrong_calls_receivers_returns_and_local_stores() {
    for body in [
        "ldstr \"bad\"\ncall System.Console::WriteLine(Int32)\nret",
        "ldstr \"bad\"\nret",
        ".local Int32 value\nldstr \"bad\"\nstloc value\nldloc value\nret",
    ] {
        assert!(verify(&program(body, "Int32")).is_err(), "{body}");
    }
    let source = ".module Test\n.type A\n.method instance Get() -> Int32\nldc.i4 42\nret\n.end\n.end\n.type B\n.end\n.function Main() -> Int32\nnewobj B\ncall instance A::Get()\nret\n.end";
    assert!(verify(&assemble(source).unwrap()).is_err());
}

#[test]
fn equal_height_joins_require_equal_stack_types() {
    let body = "ldc.bool true\nbrtrue Left\nldc.i4 42\nbr Join\nLeft:\nldstr \"bad\"\nJoin:\npop\nldvoid\nret";
    assert!(
        verify(&program(body, "Void"))
            .unwrap_err()
            .message
            .contains("stack types")
    );
    let source = ".module Test\n.type Box<T>\n.field Value T\n.end\n.function Main() -> Void\nldc.bool true\nbrtrue Left\nldc.i4 42\nnewobj Box<Int32>\nbr Join\nLeft:\nldstr \"value\"\nnewobj Box<String>\nJoin:\npop\nldvoid\nret\n.end";
    assert!(
        verify(&assemble(source).unwrap())
            .unwrap_err()
            .message
            .contains("stack types")
    );
}

#[test]
fn storage_normalization_accepts_small_integers_and_single_precision() {
    for (body, returns) in [
        (
            ".local Byte value\nldc.i4 257\nstloc value\nldloc value\nret",
            "Int32",
        ),
        (
            ".local Single value\nldc.r8 1.25\nstloc value\nldloc value\nret",
            "Double",
        ),
        ("ldc.i8 42\nret", "UInt64"),
    ] {
        let module = program(body, returns);
        assert!(verify(&module).is_ok(), "{body}");
        assert!(run(&module, Limits::default()).is_ok());
    }
    let body = ".local Byte value\nldc.bool true\nbrtrue Left\nldc.i4 42\nbr Join\nLeft:\nldc.i4 2\nstloc value\nldloc value\nJoin:\nret";
    assert!(verify(&program(body, "Int32")).is_ok());
}

#[test]
fn record_construction_and_fields_use_substituted_signatures() {
    let base = ".module Test\n.type Box<T>\n.field Value T\n.end\n.function Main() -> Int32\nldc.i4 42\nnewobj Box<Int32>\nldfld 0\nret\n.end";
    assert!(verify(&assemble(base).unwrap()).is_ok());
    for source in [
        base.replace("newobj Box<Int32>", "newobj Box<String>"),
        base.replace("ldfld 0", "ldfld 1"),
        base.replace("ldfld 0", "ldstr \"bad\"\nstfld 0\nldfld 0"),
    ] {
        assert!(verify(&assemble(&source).unwrap()).is_err());
    }
}

#[test]
fn arithmetic_conversion_and_branch_categories_match_the_runtime() {
    for body in [
        "ldc.i4 1\nldc.i8 2\nadd\nret",
        "ldc.bool true\nneg\nret",
        "ldc.r8 1\nldc.r8 2\nadd.ovf\nret",
        "ldc.r8 1\nconv.r.un\nret",
        "ldc.i4 1\nldc.r8 2\nshl\nret",
        "ldstr \"bad\"\nbrtrue Done\nDone:\nldc.i4 42\nret",
        "ldc.i8 0\nswitch (Done)\nDone:\nldc.i4 42\nret",
        "ptr.null Int32\nconv.ovf.i\nret",
    ] {
        assert!(verify(&program(body, "Int32")).is_err(), "{body}");
    }
    assert!(verify(&program("ptr.null Int32\nconv.i\nret", "IntPtr")).is_ok());
}

#[test]
fn pointer_type_checks_do_not_claim_pointer_validity() {
    for body in [
        "ldc.i4 0\nldobj Int32\nret",
        "ptr.null Byte\nldind.i4\nret",
        "ptr.null Int32\nldc.i8 42\nstobj Int32\nldc.i4 0\nret",
        "ptr.null Int32\nptr.null Byte\ncpobj Int32\nldc.i4 0\nret",
        "ldc.i4 99\nldc.i4 4\nlocalloc\npop\nret",
    ] {
        assert!(verify(&program(body, "Int32")).is_err(), "{body}");
    }
    let module = program("ptr.null Int32\nldind.i4\nret", "Int32");
    assert!(verify(&module).is_ok());
    assert!(run(&module, Limits::default()).is_err());
}

#[test]
fn union_payloads_and_heap_values_preserve_raw_storage_types() {
    let source = ".module Test\n.function F(Option<Byte> value) -> Int32\nldarg value\nldcase Some\nldc.i4 1\nadd\nret\n.end";
    // ldcase currently returns the stored Byte without stack normalization.
    assert!(verify(&assemble(source).unwrap()).is_err());
    assert!(
        verify(
            &assemble(
                &source
                    .replace("ldc.i4 1\nadd", "stloc copy\nldloc copy")
                    .replace("ldarg value", ".local Byte copy\nldarg value")
            )
            .unwrap()
        )
        .is_ok()
    );
    assert!(verify(&program("none Int32\nldcase Ok\nret", "Int32")).is_err());
    let source = ".module Test\n.function F(Ref<Byte> value) -> Void\nldarg value\nldc.i4 1\nheap.store\nret\n.end";
    assert!(verify(&assemble(source).unwrap()).is_err());
}

#[test]
fn generic_parameters_are_symbolic_and_do_not_erase_normalization() {
    let source = ".module Test\n.type Box<T>\n.field Value T\n.method instance Get() -> T\nldarg this\nldfld Box<T>::Value\nret\n.end\n.end";
    assert!(verify(&assemble(source).unwrap()).is_ok());
    let source = ".module Test\n.type Wrap<T>\n.method static Make(T value) -> Option<T>\nldarg value\nsome\nret\n.end\n.end";
    assert!(
        verify(&assemble(source).unwrap())
            .unwrap_err()
            .message
            .contains("normalization")
    );
    let source = ".module Test\n.type Identity<T>\n.method static F(T value) -> T\n.local T copy\nldarg value\nstloc copy\nldloc copy\nret\n.end\n.end";
    assert!(verify(&assemble(source).unwrap()).is_ok());
}
