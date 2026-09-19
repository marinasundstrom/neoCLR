use neoclr::{Limits, LoadedProgram, Module, Value, assemble};
use std::{process::Command, sync::OnceLock};

fn library() -> &'static Module {
    static LIBRARY: OnceLock<Module> = OnceLock::new();
    LIBRARY.get_or_init(|| {
        let output = Command::new("python3")
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .args(["-c", "import runpy; m=runpy.run_path('docs/experiments/raven-target/collection_library.py'); print(m['build'](m['ROOT'] / 'runtime/System.neoil'))"])
            .output().unwrap();
        assert!(output.status.success());
        assemble(std::str::from_utf8(&output.stdout).unwrap()).unwrap()
    })
}

const PROBE: &str = r#"
.type class Probe
.implements System.Collections.Iterable<Int32>
.implements System.Collections.Iterator<Int32>
.field Limit Int32
.field Moves Int32
.field Reads Int32
.field Disposals Int32
.field FailMove Int32
.field FailDispose Boolean
.method instance GetIterator() -> System.Collections.Iterator<Int32>
ldarg this
castclass System.Collections.Iterator<Int32>
ret
.end
.method instance MoveNext() -> Boolean
ldarg this
ldarg this
ldfld Probe::Moves
ldc.i4 1
add
stfld Probe::Moves
ldarg this
ldfld Probe::Moves
ldarg this
ldfld Probe::FailMove
beq Fail
ldarg this
ldfld Probe::Moves
ldarg this
ldfld Probe::Limit
cgt
ldc.bool false
ceq
ret
Fail:
fault "iterator failure"
.end
.method instance get_Current() -> Int32
ldarg this
ldarg this
ldfld Probe::Reads
ldc.i4 1
add
stfld Probe::Reads
ldarg this
ldfld Probe::Moves
ldc.i4 1
sub
ret
.end
.method instance Dispose() -> void
ldarg this
ldfld Probe::FailDispose
brtrue Fail
ldarg this
ldarg this
ldfld Probe::Disposals
ldc.i4 1
add
stfld Probe::Disposals
ret
Fail:
fault "dispose failure"
.end
.end
.type class Predicate
.field Mode Int32
.field Calls Int32
.field Trace Int32
.method instance Check(Int32 value) -> Boolean
ldarg this
ldarg this
ldfld Predicate::Calls
ldc.i4 1
add
stfld Predicate::Calls
ldarg this
ldarg this
ldfld Predicate::Trace
ldc.i4 10
mul
ldarg value
add
stfld Predicate::Trace
ldarg this
ldfld Predicate::Mode
ldc.i4 3
beq Fail
ldarg this
ldfld Predicate::Mode
ldc.i4 2
beq Missing
ldarg this
ldfld Predicate::Mode
ldc.i4 1
beq Zero
ldarg value
ldc.i4 2
rem
ldc.i4 1
ceq
ret
Zero:
ldarg value
ldc.i4 0
ceq
ret
Missing:
ldc.bool false
ret
Fail:
fault "predicate failure"
.end
.end
"#;
fn program(body: &str) -> LoadedProgram {
    let source =
        format!(".module Test\n.entry Main\n{PROBE}\n.function Main() -> Int32\n{body}\nret\n.end");
    let app = neoclr::assembler::read_modules(
        &[neoclr::assembler::ModuleInput::Source(&source)],
        library(),
    )
    .unwrap()
    .remove(0);
    let program = LoadedProgram::with_library(&app, library()).unwrap();
    program.verify().unwrap();
    program
}
fn start(count: i32, fail_move: i32, fail_dispose: bool) -> String {
    format!(
        ".local Probe probe\nldc.i4 {count}\nldc.i4 0\nldc.i4 0\nldc.i4 0\nldc.i4 {fail_move}\nldc.bool {fail_dispose}\nnewobj Probe\nstloc probe\n"
    )
}
fn check(actual: &str, expected: i32, label: &str) -> String {
    format!("{actual}\nldc.i4 {expected}\nbeq {label}\nfault \"{label}\"\n{label}:\n")
}

#[test]
fn terminals_report_cardinality_and_dispose_at_the_expected_boundary() {
    for (operator, count, moves, reads, outcome, expected) in [
        ("First", 0, 1, 0, "None", 1),
        ("First", 3, 1, 1, "Some", 0),
        ("Last", 0, 1, 0, "None", 1),
        ("Last", 3, 4, 3, "Some", 2),
        ("Single", 0, 1, 0, "Empty", 1),
        ("Single", 1, 2, 1, "Ok", 0),
        ("Single", 3, 2, 1, "Multiple", 1),
    ] {
        let mut body = start(count, -1, false);
        body += &format!(
            "ldloc probe\ncall System.Linq.Operators::{operator}<Int32>(System.Collections.Iterable<Int32>)\n"
        );
        let result = match outcome {
            "Some" => "call instance System.Option<Int32>::GetSomeCase()\ncall instance System.Option.Some<Int32>::get_Value()".into(),
            "None" => "call instance System.Option<Int32>::get_IsNone()".into(),
            "Ok" => "call instance System.Result<Int32,System.Linq.SingleError>::GetOkCase()\ncall instance System.Result.Ok<Int32>::get_Value()".into(),
            case => format!("call instance System.Result<Int32,System.Linq.SingleError>::GetErrorCase()\ncall instance System.Result.Error<System.Linq.SingleError>::get_Value()\ncall instance System.Linq.SingleError::get_Is{case}()"),
        };
        if matches!(outcome, "Some" | "Ok") {
            body += &check(&result, expected, "Outcome");
        } else {
            body += &(result + "\nbrtrue Outcome\nfault \"wrong union case\"\nOutcome:\n");
        }
        for (field, expected) in [("Moves", moves), ("Reads", reads), ("Disposals", 1)] {
            body += &check(
                &format!("ldloc probe\nldfld Probe::{field}"),
                expected,
                field,
            );
        }
        body += "ldc.i4 42";
        assert_eq!(
            program(&body).run(Limits::default()).unwrap().value,
            Value::Int32(42),
            "{operator} {count}"
        );
    }
}

#[test]
fn iterator_and_disposal_faults_are_not_turned_into_union_outcomes() {
    for operator in ["First", "Last", "Single"] {
        for (fail_move, fail_dispose, message) in [
            (1, false, "iterator failure"),
            (-1, true, "dispose failure"),
        ] {
            let body = start(1, fail_move, fail_dispose)
                + &format!(
                    "ldloc probe\ncall System.Linq.Operators::{operator}<Int32>(System.Collections.Iterable<Int32>)\npop\nldc.i4 0"
                );
            let fault = program(&body).run(Limits::default()).unwrap_err();
            assert!(fault.message.contains(message), "{fault:?}");
        }
    }
}

#[test]
fn concrete_find_avoids_query_chain_allocations_for_the_same_result() {
    let mut allocations = Vec::new();
    for query in [false, true] {
        let operation = if query {
            "call System.Linq.Operators::Filter<Int32>(System.Collections.Iterable<Int32>,System.Func<Int32,Boolean>)\ncall System.Linq.Operators::First<Int32>(System.Collections.Iterable<Int32>)"
        } else {
            "call instance System.Collections.ArrayList<Int32>::Find(System.Func<Int32,Boolean>)"
        };
        let source = format!(
            r#"
.module Test
.entry Main
.function Match(Int32 value) -> Boolean
ldarg value
ldc.i4 42
ceq
ret
.end
.function Main() -> Int32
.local System.Collections.ArrayList<Int32> values
newobj instance System.Collections.ArrayList<Int32>::.ctor()
stloc values
ldloc values
ldc.i4 42
call instance System.Collections.ArrayList<Int32>::Add(Int32)
ldloc values
delegate.bind System.Func<Int32,Boolean> = Match(Int32)
{operation}
call instance System.Option<Int32>::GetSomeCase()
call instance System.Option.Some<Int32>::get_Value()
ret
.end
"#
        );
        let app = neoclr::assembler::read_modules(
            &[neoclr::assembler::ModuleInput::Source(&source)],
            library(),
        )
        .unwrap()
        .remove(0);
        let p = LoadedProgram::with_library(&app, library()).unwrap();
        p.verify().unwrap();
        let result = p.run(Limits::default()).unwrap();
        assert_eq!(result.value, Value::Int32(42));
        allocations.push(result.heap.statistics().allocated_objects);
    }
    println!(
        "Managed allocations: ArrayList.Find={}, Filter.First={}",
        allocations[0], allocations[1]
    );
    assert!(allocations[0] < allocations[1]);
}

fn predicate_start(count: i32, mode: i32, fail_move: i32, fail_dispose: bool) -> String {
    ".local Predicate callback\n".to_string()
        + &start(count, fail_move, fail_dispose)
        + &format!(
            "ldc.i4 {mode}\nldc.i4 0\nldc.i4 0\nnewobj Predicate\nstloc callback\nldloc probe\nldloc callback\ndelegate.bind System.Func<Int32,Boolean> = instance Predicate::Check(Int32)\n"
        )
}
fn predicate_call(operator: &str, via_where: bool) -> String {
    if via_where {
        format!(
            "call System.Linq.Operators::Filter<Int32>(System.Collections.Iterable<Int32>,System.Func<Int32,Boolean>)\ncall System.Linq.Operators::{operator}<Int32>(System.Collections.Iterable<Int32>)\n"
        )
    } else {
        format!(
            "call System.Linq.Operators::{operator}<Int32>(System.Collections.Iterable<Int32>,System.Func<Int32,Boolean>)\n"
        )
    }
}

#[test]
fn predicate_terminals_preserve_matching_order_outcomes_and_cleanup_without_query_wrappers() {
    for (operator, count, mode, moves, reads, trace, outcome, expected) in [
        ("First", 6, 0, 2, 2, 1, "Some", 1),
        ("Last", 6, 0, 7, 6, 12345, "Some", 5),
        ("Single", 6, 0, 4, 4, 123, "Multiple", 0),
        ("Single", 6, 1, 7, 6, 12345, "Ok", 0),
        ("Last", 6, 1, 7, 6, 12345, "Some", 0),
        ("First", 6, 2, 7, 6, 12345, "None", 0),
        ("Last", 6, 2, 7, 6, 12345, "None", 0),
        ("Single", 6, 2, 7, 6, 12345, "Empty", 0),
        ("First", 0, 0, 1, 0, 0, "None", 0),
        ("Last", 0, 0, 1, 0, 0, "None", 0),
        ("Single", 0, 0, 1, 0, 0, "Empty", 0),
    ] {
        let mut allocations = Vec::new();
        for via_where in [false, true] {
            let mut body = predicate_start(count, mode, -1, false);
            body += &predicate_call(operator, via_where);
            let result: String = match outcome {
                "Some" => "call instance System.Option<Int32>::GetSomeCase()\ncall instance System.Option.Some<Int32>::get_Value()".into(),
                "None" => "call instance System.Option<Int32>::get_IsNone()".into(),
                "Ok" => "call instance System.Result<Int32,System.Linq.SingleError>::GetOkCase()\ncall instance System.Result.Ok<Int32>::get_Value()".into(),
                case => format!("call instance System.Result<Int32,System.Linq.SingleError>::GetErrorCase()\ncall instance System.Result.Error<System.Linq.SingleError>::get_Value()\ncall instance System.Linq.SingleError::get_Is{case}()"),
            };
            if matches!(outcome, "Some" | "Ok") {
                body += &check(&result, expected, "Outcome");
            } else {
                body += &(result + "\nbrtrue Outcome\nfault \"wrong union case\"\nOutcome:\n");
            }
            for (field, expected) in [("Moves", moves), ("Reads", reads), ("Disposals", 1)] {
                body += &check(
                    &format!("ldloc probe\nldfld Probe::{field}"),
                    expected,
                    field,
                );
            }
            for (field, expected) in [("Calls", reads), ("Trace", trace)] {
                body += &check(
                    &format!("ldloc callback\nldfld Predicate::{field}"),
                    expected,
                    field,
                );
            }
            body += "ldc.i4 42";
            let result = program(&body).run(Limits::default()).unwrap();
            assert_eq!(result.value, Value::Int32(42));
            allocations.push(result.heap.statistics().allocated_objects);
        }
        assert!(
            allocations[0] < allocations[1],
            "{operator}: {allocations:?}"
        );
    }
}

#[test]
fn predicate_terminals_preserve_fault_boundaries() {
    for operator in ["First", "Last", "Single"] {
        for (mode, fail_move, fail_dispose, message) in [
            (3, -1, true, "predicate failure"),
            (0, 1, true, "iterator failure"),
            (0, -1, true, "dispose failure"),
        ] {
            let body = predicate_start(3, mode, fail_move, fail_dispose)
                + &predicate_call(operator, false)
                + "pop\nldc.i4 0";
            assert!(
                program(&body)
                    .run(Limits::default())
                    .unwrap_err()
                    .message
                    .contains(message)
            );
        }
    }
}
