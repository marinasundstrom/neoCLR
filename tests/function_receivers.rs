use neoclr::{Limits, Value, assemble, run, verify};

const TYPES: &str = r#"
.type class Counter
.field Age Int32
.method instance Add(Int32 delta) -> Int32
ldarg 0
ldarg 0
ldfld Counter::Age
ldarg delta
add
stfld Counter::Age
ldarg 0
ldfld Counter::Age
ret
.end
.end
"#;
#[test]
fn bound_nominal_receiver_survives_return_copy_and_gc() {
    let m = assemble(&format!(".module Test\n.entry Main\n{TYPES}\n.function Make() -> fn<Int32,Int32>\nldc.i4 40\nnewobj Counter\nfunction.bind fn<Int32,Int32> = instance Counter::Add(Int32)\nret\n.end\n.function Main() -> Int32\n.local fn<Int32,Int32> callback\n.local fn<Int32,Int32> alias\ncall Make()\nstloc callback\nldloc callback\nstloc alias\nldc.i4 100\nnewobj Counter\npop\nldc.i4 200\nnewobj Counter\npop\nldloc alias\nldc.i4 1\ncallvirt instance fn<Int32,Int32>::Invoke(Int32)\npop\nldloc callback\nldc.i4 1\ncallvirt instance fn<Int32,Int32>::Invoke(Int32)\nret\n.end")).unwrap();
    verify(&m).unwrap();
    let result = run(
        &m,
        Limits {
            heap_objects: 2,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.statistics().collections > 0);
}

#[test]
fn null_nominal_function_receiver_is_rejected_at_binding() {
    let m = assemble(&format!(".module Test\n.entry Main\n{TYPES}\n.function Main() -> fn<Int32,Int32>\n.local Counter empty\nldloca empty\ninitobj Counter\nldloc empty\nfunction.bind fn<Int32,Int32> = instance Counter::Add(Int32)\nret\n.end")).unwrap();
    verify(&m).unwrap();
    assert!(run(&m, Limits::default()).is_err());
}

#[test]
fn interface_and_virtual_bound_targets_select_the_concrete_implementation() {
    for target in ["Base", "Read"] {
        let source = format!(
            r#"
.module Test
.entry Main
.interface Read
.method instance Get() -> Int32
.end
.end
.type class abstract Base
.implements Read
.method instance abstract Get() -> Int32
.end
.end
.type class Derived
.extends Base
.method instance override Get() -> Int32
ldc.i4 42
ret
.end
.end
.function Main() -> Int32
.local {target} view
newobj Derived
stloc view
ldloc view
function.bind fn<Int32> = instance {target}::Get()
callvirt instance fn<Int32>::Invoke()
ret
.end
"#
        );
        let m = assemble(&source).unwrap();
        verify(&m).unwrap();
        assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
    }
}

#[test]
fn equality_compares_method_and_class_receiver_identity() {
    for (second_receiver, expected) in [
        ("ldloc receiver", true),
        ("ldc.i4 40\nnewobj Counter", false),
    ] {
        let source = format!(
            ".module Test\n.entry Main\n{TYPES}\n.function Main() -> Boolean\n.local Counter receiver\nldc.i4 40\nnewobj Counter\nstloc receiver\nldloc receiver\nfunction.bind fn<Int32,Int32> = instance Counter::Add(Int32)\n{second_receiver}\nfunction.bind fn<Int32,Int32> = instance Counter::Add(Int32)\nceq\nret\n.end"
        );
        let m = assemble(&source).unwrap();
        verify(&m).unwrap();
        assert_eq!(
            run(&m, Limits::default()).unwrap().value,
            Value::Boolean(expected)
        );
    }
}
