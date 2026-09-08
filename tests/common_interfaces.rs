use neoclr::{Limits, LoadedProgram, Value, assemble, frontend};

fn neo(source: &str) -> LoadedProgram {
    LoadedProgram::new(&frontend::compile(source).unwrap()).unwrap()
}

#[test]
fn sample_uses_readonly_iterable_and_comparable_views() {
    let result = neo(include_str!("../examples/source/common-interfaces.neo"))
        .run(Limits::default())
        .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.output, ["0", "42"]);
}

#[test]
fn integer_ordering_is_overflow_safe_and_uses_typed_interfaces() {
    for (ty, low, high) in [
        ("Boolean", "ldc.bool false", "ldc.bool true"),
        ("Char", "ldc.i4 0", "ldc.i4 65535"),
        ("Int32", "ldc.i4 -2147483648", "ldc.i4 2147483647"),
        (
            "Int64",
            "ldc.i8 -9223372036854775808",
            "ldc.i8 9223372036854775807",
        ),
        ("UInt64", "ldc.i4 0\nconv.u8", "ldc.i8 -1\nconv.u8"),
        ("UInt32", "ldc.i4 0\nconv.u4", "ldc.i4 -1\nconv.u4"),
        ("UInt16", "ldc.i4 0\nconv.u2", "ldc.i4 65535\nconv.u2"),
        ("Int16", "ldc.i4 -32768\nconv.i2", "ldc.i4 32767\nconv.i2"),
        ("Byte", "ldc.i4 0\nconv.u1", "ldc.i4 255\nconv.u1"),
        ("SByte", "ldc.i4 -128\nconv.i1", "ldc.i4 127\nconv.i1"),
        ("IntPtr", "ldc.i4 -1\nconv.i", "ldc.i4 1\nconv.i"),
        ("UIntPtr", "ldc.i4 0\nconv.u", "ldc.i8 -1\nconv.u"),
    ] {
        for (left, right, expected) in [(low, high, -1), (high, low, 1), (high, high, 0)] {
            let source = format!(
                ".module Test\n.entry Main\n.function Main() -> Int32\n.local {ty} value\n{left}\nstloc value\nldloca value\ninterface.borrow System.Comparable<{ty}>\n{right}\ncallvirt instance System.Comparable<{ty}>::CompareTo({ty})\nret\n.end"
            );
            let module = assemble(&source).unwrap();
            assert_eq!(
                LoadedProgram::new(&module)
                    .unwrap()
                    .run(Limits::default())
                    .unwrap()
                    .value,
                Value::Int32(expected),
                "{ty}"
            );
        }
    }
}

#[test]
fn iterator_outlives_stack_descriptor_and_survives_gc() {
    let p = neo(r#"
func Make() -> System.Collections.Iterator<int>& {
    var list = System.Collections.ArrayList<int>.Allocate(0)
    list.Add(20)
    list.Add(22)
    return list.GetIterator()
}
func Main() -> int {
    let iterator = Make()
    for i in 0..<30 { let discarded = Make() }
    var total = 0
    while iterator.MoveNext() { total = total + iterator.Current }
    if iterator.MoveNext() { return -1 }
    iterator.Dispose()
    iterator.Dispose()
    if iterator.MoveNext() { return -2 }
    return total
}
"#);
    let r = p
        .run(Limits {
            heap_objects: 10,
            ..Limits::default()
        })
        .unwrap();
    assert_eq!(r.value, Value::Int32(42));
    assert!(r.heap.statistics().collections > 0);
}

#[test]
fn each_iterator_is_independent_and_reference_elements_remain_references() {
    assert_eq!(
        neo(r#"
record Counter(Age: int)
func Main() -> int {
    var list = System.Collections.ArrayList<Counter&>.Allocate(0)
    let counter = new Counter(40)
    list.Add(counter)
    let view: System.Collections.List<Counter&>& = &list
    let first = view.GetIterator()
    let second = view.GetIterator()
    first.MoveNext()
    second.MoveNext()
    let value = first.Current
    value.Age = value.Age + 2
    first.Dispose()
    return second.Current.Age
}
"#)
        .run(Limits::default())
        .unwrap()
        .value,
        Value::Int32(42)
    );
}

#[test]
fn current_faults_outside_active_iteration() {
    for actions in ["", "iterator.MoveNext()", "list.Add(1); iterator.Dispose()"] {
        let s = format!(
            "func Main() -> int {{ var list = System.Collections.ArrayList<int>.Allocate(0); let iterator = list.GetIterator(); {actions}; return iterator.Current }}"
        );
        assert!(
            neo(&s)
                .run(Limits::default())
                .unwrap_err()
                .message
                .contains("no current element")
        );
    }
}

#[test]
fn iterator_keeps_initial_extent_and_backing_buffer() {
    assert_eq!(
        neo(r#"
func Main() -> int {
    var list = System.Collections.ArrayList<int>.Allocate(1)
    list.Add(1)
    let iterator = list.GetIterator()
    list[0] = 42
    list.Add(2)
    list[0] = 99
    if !iterator.MoveNext() { return -1 }
    let value = iterator.Current
    if iterator.MoveNext() { return -2 }
    return value
}
"#)
        .run(Limits::default())
        .unwrap()
        .value,
        Value::Int32(42)
    );
}

#[test]
fn floating_ordering_matches_dotnet_nan_and_zero_rules() {
    for ty in ["Single", "Double"] {
        let op = if ty == "Single" { "ldc.r4" } else { "ldc.r8" };
        for (a, b, expected) in [
            ("NaN", "0", -1),
            ("0", "NaN", 1),
            ("NaN", "NaN", 0),
            ("-0", "0", 0),
            ("-Infinity", "Infinity", -1),
        ] {
            let source = format!(
                ".module Test\n.entry Main\n.function Main() -> Int32\n.local {ty} value\n{op} {a}\nstloc value\nldloca value\n{op} {b}\ncall instance System.{ty}::CompareTo({ty})\nret\n.end"
            );
            let m = assemble(&source).unwrap();
            assert_eq!(
                LoadedProgram::new(&m)
                    .unwrap()
                    .run(Limits::default())
                    .unwrap()
                    .value,
                Value::Int32(expected)
            );
        }
    }
}

#[test]
fn readonly_iterator_can_read_but_cannot_advance() {
    let source = "func Main() -> bool { var list = System.Collections.ArrayList<int>.Allocate(0); let iterator: readonly System.Collections.Iterator<int>& = list.GetIterator(); return iterator.MoveNext() }";
    assert!(frontend::compile(source).is_err());
}

#[test]
fn custom_comparable_and_value_elements_use_value_semantics() {
    let p = neo(r#"
record Score(Value: int): System.Comparable<Score> {
    readonly func CompareTo(other: Score) -> int { return this.Value.CompareTo(other.Value) }
}
func Main() -> int {
    var list = System.Collections.ArrayList<Score>.Allocate(0)
    list.Add(Score(42))
    let iterator = list.GetIterator()
    iterator.MoveNext()
    var copy = iterator.Current
    copy.Value = 7
    let ordering: System.Comparable<Score>& = &copy
    if ordering.CompareTo(Score(42)) >= 0 { return -1 }
    let original = iterator.Current
    iterator.Dispose()
    return original.Value
}
"#);
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}
