use neoclr::{Limits, LoadedProgram, Module, Value, assemble};
use std::{process::Command, sync::OnceLock};

fn library() -> &'static Module {
    static LIBRARY: OnceLock<Module> = OnceLock::new();
    LIBRARY.get_or_init(|| {
        let output = Command::new("python3").current_dir(env!("CARGO_MANIFEST_DIR"))
            .args(["-c", "import runpy; m=runpy.run_path('docs/experiments/raven-target/collection_library.py'); print(m['build'](m['ROOT'] / 'runtime/System.neoil'))"])
            .output().unwrap();
        assert!(output.status.success());
        assemble(std::str::from_utf8(&output.stdout).unwrap()).unwrap()
    })
}
const CALLBACK: &str = r#"
.type class Callback
.field Calls Int32
.field Trace Int32
.field Target Int32
.method instance Check(Int32 value) -> Boolean
ldarg this
ldarg this
ldfld Callback::Calls
ldc.i4 1
add
stfld Callback::Calls
ldarg this
ldarg this
ldfld Callback::Trace
ldc.i4 100
mul
ldarg value
add
stfld Callback::Trace
ldarg value
ldarg this
ldfld Callback::Target
ceq
ret
.end
.end
.function Yes(Int32 value) -> Boolean
ldc.bool true
ret
.end
.function Fail(Int32 value) -> Boolean
fault "predicate failure"
.end
"#;
fn program(body: &str) -> LoadedProgram {
    program_with(body, "")
}
fn program_with(body: &str, extra: &str) -> LoadedProgram {
    let source = format!(
        ".module Test\n.entry Main\n{CALLBACK}\n{extra}\n.function Main() -> Int32\n{body}\nret\n.end"
    );
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
fn start(values: &[i32], target: i32) -> String {
    let mut body = format!(
        ".local System.Collections.ArrayList<Int32> list\n.local Callback callback\nnewobj instance System.Collections.ArrayList<Int32>::.ctor()\nstloc list\nldc.i4 0\nldc.i4 0\nldc.i4 {target}\nnewobj Callback\nstloc callback\n"
    );
    for value in values {
        body += &format!(
            "ldloc list\nldc.i4 {value}\ncall instance System.Collections.ArrayList<Int32>::Add(Int32)\n"
        );
    }
    body
}
fn invoke(name: &str) -> String {
    format!(
        "ldloc list\nldloc callback\ndelegate.bind System.Func<Int32,Boolean> = instance Callback::Check(Int32)\ncall instance System.Collections.ArrayList<Int32>::{name}(System.Func<Int32,Boolean>)\n"
    )
}
fn assert_integer(actual: &str, expected: i32, label: &str) -> String {
    format!("{actual}\nldc.i4 {expected}\nbeq {label}\nfault \"{label}\"\n{label}:\n")
}
#[test]
fn scans_have_expected_direction_short_circuiting_and_outcomes() {
    for (method, calls, trace, expected) in [
        ("Find", 2, 7, 7),
        ("FindLast", 1, 7, 7),
        ("FindIndex", 2, 7, 1),
        ("FindLastIndex", 1, 7, 3),
        ("Exists", 2, 7, 1),
        ("TrueForAll", 1, 0, 0),
        ("FindAll", 4, 74207, 2),
    ] {
        let mut body = start(&[0, 7, 42, 7], 7) + &invoke(method);
        if method == "Exists" || method == "TrueForAll" {
            body += if expected == 1 {
                "brtrue Outcome\n"
            } else {
                "brfalse Outcome\n"
            };
            body += "fault \"Boolean outcome\"\nOutcome:\n";
        } else {
            let extract = if method == "FindAll" {
                "call instance System.Collections.ArrayList<Int32>::get_Count()"
            } else {
                "call instance System.Option<Int32>::GetSomeCase()\ncall instance System.Option.Some<Int32>::get_Value()"
            };
            body += &assert_integer(extract, expected, "Outcome");
        }
        body += &assert_integer("ldloc callback\nldfld Callback::Calls", calls, "Calls");
        body += &assert_integer("ldloc callback\nldfld Callback::Trace", trace, "Trace");
        body += "ldc.i4 42";
        assert_eq!(
            program(&body).run(Limits::default()).unwrap().value,
            Value::Int32(42),
            "{method}"
        );
    }
}
#[test]
fn missing_and_empty_sequences_have_total_outcomes_without_default_elements() {
    for values in [&[][..], &[0, 7, 42, 7][..]] {
        for method in [
            "Find",
            "FindLast",
            "FindIndex",
            "FindLastIndex",
            "Exists",
            "FindAll",
            "TrueForAll",
        ] {
            let mut body = start(values, 5) + &invoke(method);
            match method {
                "Exists" => body += "brfalse Good\nfault \"Exists\"\nGood:\n",
                "TrueForAll" => {
                    body += if values.is_empty() {
                        "brtrue Good\n"
                    } else {
                        "brfalse Good\n"
                    };
                    body += "fault \"TrueForAll\"\nGood:\n";
                }
                "FindAll" => {
                    body += &assert_integer(
                        "call instance System.Collections.ArrayList<Int32>::get_Count()",
                        0,
                        "Empty",
                    )
                }
                _ => {
                    body += "call instance System.Option<Int32>::get_IsNone()\nbrtrue Missing\nfault \"missing case\"\nMissing:\n"
                }
            }
            let calls = if method == "TrueForAll" && !values.is_empty() {
                1
            } else {
                values.len() as i32
            };
            body += &assert_integer("ldloc callback\nldfld Callback::Calls", calls, "Calls");
            if method == "FindLastIndex" && !values.is_empty() {
                body += &assert_integer(
                    "ldloc callback\nldfld Callback::Trace",
                    7420700,
                    "ReverseOrder",
                );
            }
            body += "ldc.i4 42";
            assert_eq!(
                program(&body).run(Limits::default()).unwrap().value,
                Value::Int32(42)
            );
        }
    }
}
#[test]
fn scalar_searches_add_no_managed_allocations_and_predicate_faults_stay_faults() {
    let setup = start(&[42], 42);
    let baseline = program(&(setup.clone() + "ldc.i4 0"))
        .run(Limits::default())
        .unwrap()
        .heap
        .statistics()
        .allocated_objects;
    for method in [
        "Find",
        "FindLast",
        "FindIndex",
        "FindLastIndex",
        "Exists",
        "TrueForAll",
    ] {
        let body = setup.clone()
            + &format!(
                "ldloc list\ndelegate.bind System.Func<Int32,Boolean> = Yes(Int32)\ncall instance System.Collections.ArrayList<Int32>::{method}(System.Func<Int32,Boolean>)\npop\nldc.i4 0"
            );
        let result = program(&body).run(Limits::default()).unwrap();
        assert_eq!(
            result.heap.statistics().allocated_objects,
            baseline,
            "{method}"
        );
    }
    for method in [
        "Find",
        "FindLast",
        "FindIndex",
        "FindLastIndex",
        "Exists",
        "TrueForAll",
        "FindAll",
    ] {
        let body = setup.clone()
            + &format!(
                "ldloc list\ndelegate.bind System.Func<Int32,Boolean> = Fail(Int32)\ncall instance System.Collections.ArrayList<Int32>::{method}(System.Func<Int32,Boolean>)\npop\nldc.i4 0"
            );
        assert!(
            program(&body)
                .run(Limits::default())
                .unwrap_err()
                .message
                .contains("predicate failure")
        );
    }
}

#[test]
fn filtering_retains_the_initial_buffer_and_values_while_callbacks_grow_and_collect() {
    let extra = r#"
.type class Noise
.field Value Int32
.end
.type class Mutator
.field List System.Collections.ArrayList<Int32>
.method instance Check(Int32 value) -> Boolean
.local Int32 remaining
ldarg this
ldfld Mutator::List
ldc.i4 123
call instance System.Collections.ArrayList<Int32>::Add(Int32)
ldc.i4 50
stloc remaining
Loop:
ldloc remaining
newobj Noise
pop
ldloc remaining
ldc.i4 1
sub
stloc remaining
ldloc remaining
brtrue Loop
ldc.bool true
ret
.end
.end
"#;
    let body = r#"
.local System.Collections.ArrayList<Int32> list
.local System.Collections.ArrayList<Int32> filtered
.local Mutator callback
ldc.i4 3
newobj instance System.Collections.ArrayList<Int32>::.ctor(Int32)
stloc list
ldloc list
ldc.i4 7
call instance System.Collections.ArrayList<Int32>::Add(Int32)
ldloc list
ldc.i4 42
call instance System.Collections.ArrayList<Int32>::Add(Int32)
ldloc list
ldc.i4 99
call instance System.Collections.ArrayList<Int32>::Add(Int32)
ldloc list
newobj Mutator
stloc callback
ldloc list
ldloc callback
delegate.bind System.Func<Int32,Boolean> = instance Mutator::Check(Int32)
call instance System.Collections.ArrayList<Int32>::FindAll(System.Func<Int32,Boolean>)
stloc filtered
ldloc filtered
call instance System.Collections.ArrayList<Int32>::get_Count()
ldc.i4 3
beq Extent
fault "filter followed growth"
Extent:
ldloc list
call instance System.Collections.ArrayList<Int32>::get_Count()
ldc.i4 6
beq Source
fault "source did not grow"
Source:
ldloc filtered
ldc.i4 2
call instance System.Collections.ArrayList<Int32>::get_Item(Int32)
"#;
    let result = program_with(body, extra)
        .run(Limits {
            heap_objects: 20,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(result.value, Value::Int32(99));
    assert!(result.heap.statistics().collections > 0);
}
