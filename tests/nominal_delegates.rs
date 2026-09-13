use neoclr::{Limits, Value, assemble, run, verify};

const TYPES: &str = r#"
.delegate Transform
.method instance Invoke(Int32) -> Int32
.end
.end
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
    let m = assemble(&format!(".module Test\n.entry Main\n{TYPES}\n.function Make() -> Transform\nldc.i4 40\nnewobj Counter\ndelegate.bind Transform = instance Counter::Add(Int32)\nret\n.end\n.function Main() -> Int32\n.local Transform callback\n.local Transform alias\ncall Make()\nstloc callback\nldloc callback\nstloc alias\nldc.i4 100\nnewobj Counter\npop\nldc.i4 200\nnewobj Counter\npop\nldloc alias\nldc.i4 1\ncallvirt instance Transform::Invoke(Int32)\npop\nldloc callback\nldc.i4 1\ncallvirt instance Transform::Invoke(Int32)\nret\n.end")).unwrap();
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
fn null_nominal_delegate_receiver_is_rejected_at_binding() {
    let m = assemble(&format!(".module Test\n.entry Main\n{TYPES}\n.function Main() -> Transform\n.local Counter empty\nldloca empty\ninitobj Counter\nldloc empty\ndelegate.bind Transform = instance Counter::Add(Int32)\nret\n.end")).unwrap();
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
.delegate Getter
.method instance Invoke() -> Int32
.end
.end
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
delegate.bind Getter = instance {target}::Get()
callvirt instance Getter::Invoke()
ret
.end
"#
        );
        let m = assemble(&source).unwrap();
        verify(&m).unwrap();
        assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
    }
}
