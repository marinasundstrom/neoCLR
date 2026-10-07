use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};
const COUNTER: &[u8] = include_bytes!("../../../docs/experiments/aot-values/Counter.pe");
const COPIES: &[u8] = include_bytes!("../../../docs/experiments/aot-values/Copies.pe");
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temp(std::path::PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "neoclr-aot-values-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn compile(bytes: &[u8], temp: &Temp) -> std::process::Output {
    compile_root(bytes, temp, "@entry")
}
fn compile_root(bytes: &[u8], temp: &Temp, root: &str) -> std::process::Output {
    let input = temp.0.join("input.bin");
    fs::write(&input, bytes).unwrap();
    Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"))
        .arg(input)
        .arg(root)
        .arg(temp.0.join("value.o"))
        .output()
        .unwrap()
}
fn module(bytes: &[u8]) -> neoclr::Module {
    neoclr::metadata_container::decode(bytes).unwrap()
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native(bytes: &[u8], expected_status: i32, expected_value: i32) {
    native_input(bytes, 0, expected_status, expected_value, "@entry");
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_input(bytes: &[u8], input: i32, expected_status: i32, expected_value: i32, root: &str) {
    let temp = Temp::new();
    let result = compile_root(bytes, &temp, root);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let host = temp.0.join("host.c");
    fs::write(
        &host,
        format!(
            r#"
#include <stdint.h>
#include <stdio.h>
extern int32_t neoclr_entry_v2(int32_t, int32_t *);
int main(void) {{
    int32_t value = 12345;
    int32_t status = neoclr_entry_v2({input}, &value);
    if (status != {expected_status} || value != {expected_value}) {{
        fprintf(stderr, "status=%d value=%d\n", status, value);
        return 1;
    }}
    return 0;
}}
"#
        ),
    )
    .unwrap();
    let binary = temp.0.join("values");
    let result = Command::new("clang")
        .args(["-arch", "arm64", "-Wall", "-Wextra", "-Werror"])
        .arg(host)
        .arg(temp.0.join("value.o"))
        .arg("-o")
        .arg(&binary)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let imports = Command::new("nm")
        .arg("-u")
        .arg(temp.0.join("value.o"))
        .output()
        .unwrap();
    assert!(imports.status.success());
    assert!(
        imports.stdout.is_empty(),
        "no runtime/interpreter imports allowed"
    );
    let listing = Command::new("otool")
        .arg("-L")
        .arg(&binary)
        .output()
        .unwrap();
    assert!(listing.status.success());
    let text = String::from_utf8(listing.stdout).unwrap();
    assert_eq!(
        text.lines()
            .skip(1)
            .map(|l| l.split_whitespace().next().unwrap())
            .collect::<Vec<_>>(),
        ["/usr/lib/libSystem.B.dylib"]
    );
    let isolated = Temp::new();
    let executable = isolated.0.join("values");
    fs::copy(binary, &executable).unwrap();
    let result = Command::new(executable)
        .env_clear()
        .current_dir(&isolated.0)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(result.stdout.is_empty());
    assert!(result.stderr.is_empty());
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn raven_records_preserve_copy_constructor_member_and_branch_semantics() {
    for bytes in [COUNTER, COPIES] {
        let m = module(bytes);
        let execution = neoclr::LoadedProgram::new(&m)
            .unwrap()
            .run(neoclr::Limits::default())
            .unwrap();
        assert_eq!(execution.value, neoclr::Value::Int32(0));
        native(bytes, 0, 0);
        native(&neoclr::metadata_container::write_module(&m).unwrap(), 0, 0);
    }
}

const SNAPSHOTS: &str = r#"
.module Snapshot
.entry Main
.type Pair
.field Left Int32
.field Ready Boolean
.method instance byref Set(Int32 value) -> Void
ldarg this
ldarg value
stfld Pair::Left
ret
.end
.end
.function Change(Pair value) -> Pair
ldarga value
ldc.i4 99
call instance Pair::Set(Int32)
pop
ldarg value
ret
.end
.function Main() -> Int32
.local Pair original
.local Pair copy
.local Int32 saved
ldc.i4 7
ldc.bool true
newobj Pair
stloc original
; Keep a value snapshot on the evaluation stack across a mutating call.
ldloc original
ldloca original
ldc.i4 9
call instance Pair::Set(Int32)
pop
stloc copy
; Both sides of this join carry a whole value, not an address to its local.
ldc.bool true
brtrue ChooseCopy
ldloc original
br Chosen
ChooseCopy:
ldloc copy
Chosen:
ldfld Pair::Left
stloc saved
; Passing by value must not change the caller, even through ldarga in Change.
ldloc original
call Change(Pair)
ldfld Pair::Left
ldloc original
ldfld Pair::Left
ldc.i4 100
mul
add
ldloc saved
ldc.i4 10000
mul
add
ret
.end
"#;

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn aggregate_stack_snapshots_joins_and_argument_copies_match_interpreter() {
    for truth in ["true", "false"] {
        let source = SNAPSHOTS.replace(
            "ldc.bool true\nbrtrue",
            &format!("ldc.bool {truth}\nbrtrue"),
        );
        let m = neoclr::assemble(&source).unwrap();
        let expected = if truth == "true" { 70999 } else { 90999 };
        assert_eq!(
            neoclr::run(&m, neoclr::Limits::default()).unwrap().value,
            neoclr::Value::Int32(expected)
        );
        native(
            &neoclr::metadata_container::write_module(&m).unwrap(),
            0,
            expected,
        );
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn constructor_and_member_faults_do_not_publish_results() {
    use neoclr::metadata::Instruction as Op;
    for constructor in [true, false] {
        let mut m = module(COUNTER);
        let f = m
            .functions
            .iter_mut()
            .find(|f| f.origin.as_ref().unwrap().name == if constructor { ".ctor" } else { "Bump" })
            .unwrap();
        // Keep the producer's receiver/signature contract; both are no-result or
        // Int32 as originally emitted. The fault happens before a normal return.
        if constructor {
            let end = f.body.len() - 1;
            f.body
                .splice(end..end, [Op::Int(1), Op::Int(0), Op::Divide, Op::Pop]);
        } else {
            f.body = vec![Op::Int(i32::MAX), Op::Int(1), Op::AddChecked, Op::Return];
        }
        neoclr::LoadedProgram::new(&m).unwrap().verify().unwrap();
        let fault = neoclr::run(&m, neoclr::Limits::default()).unwrap_err();
        assert_eq!(
            fault.code,
            if constructor {
                neoclr::FaultCode::DivideByZero
            } else {
                neoclr::FaultCode::ArithmeticOverflow
            }
        );
        native(
            &neoclr::metadata_container::write_module(&m).unwrap(),
            if constructor { 1 } else { 2 },
            12345,
        );
    }
}

#[test]
fn unsupported_value_shapes_and_invalid_borrows_never_emit() {
    use neoclr::metadata::{Instruction as Op, Type};
    let original = module(COUNTER);
    let mut cases = vec![];
    let mut m = original.clone();
    m.types[0].is_reference_type = true;
    cases.push(m);
    let mut m = original.clone();
    m.types[0].fields[0].ty = Type::String;
    cases.push(m);
    let mut m = original.clone();
    m.types[0].fields[0].ty = Type::Named(m.types[0].name.clone());
    cases.push(m);
    let mut m = original.clone();
    m.types[0].packing = Some(1);
    cases.push(m);
    let mut m = original.clone();
    m.functions[1].receiver_byref = false;
    cases.push(m);
    let mut m = original.clone();
    m.functions[1].body = vec![Op::Arg(0), Op::Field(99), Op::Return];
    cases.push(m);
    let mut m = original.clone();
    m.functions[1].body = vec![Op::Int(0), Op::Return, Op::String("unused".into())];
    cases.push(m);
    let mut m = original.clone();
    m.functions[1].body = vec![Op::Arg(0), Op::Return];
    cases.push(m);
    let mut m = original.clone();
    if let Op::Call(target) = &mut m.functions[0].body[3] {
        target.instance = false;
    } else {
        panic!("fixture changed");
    }
    cases.push(m);
    let mut m = original.clone();
    m.functions[0].body = vec![Op::Load(0), Op::Field(0), Op::Return];
    cases.push(m);
    for (i, m) in cases.iter().enumerate() {
        let temp = Temp::new();
        let bytes = neoclr::metadata_container::write_module(m).unwrap();
        let result = compile(&bytes, &temp);
        assert!(!result.status.success(), "invalid case {i} accepted");
        assert!(!temp.0.join("value.o").exists());
        assert!(
            !String::from_utf8_lossy(&result.stderr).contains("panicked"),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn field_borrows_object_copies_and_value_field_stores_preserve_aliases() {
    let source = SNAPSHOTS.replace(".entry Main", ".entry Inspect")
        + r#"
.function Inspect() -> Int32
.local Pair original
.local Pair copy
ldc.i4 7
ldc.bool true
newobj Pair
stloc original
; Value stfld returns an updated copy; original must stay 7.
ldloc original
ldc.i4 41
stfld Pair::Left
stloc copy
; Copy through addresses, then retain an interior borrow across self-copy.
ldloca original
ldloc copy
stobj Pair
ldloca original
ldflda Pair::Left
ldloca original
ldloca original
ldobj Pair
stobj Pair
ldc.i4 42
stobj Int32
ldloca original
ldobj Pair
ldfld Pair::Left
ldloc copy
ldfld Pair::Left
add
ret
.end
"#;
    let m = neoclr::assemble(&source).unwrap();
    assert_eq!(
        neoclr::run(&m, neoclr::Limits::default()).unwrap().value,
        neoclr::Value::Int32(83)
    );
    native(
        &neoclr::metadata_container::write_module(&m).unwrap(),
        0,
        83,
    );
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn empty_and_eight_lane_records_cross_private_call_boundary() {
    for count in [0, 8] {
        let fields = (0..count)
            .map(|i| format!(".field F{i} Int32\n"))
            .collect::<String>();
        let constants = (0..count)
            .map(|i| format!("ldc.i4 {i}\n"))
            .collect::<String>();
        let read = if count == 0 {
            "pop\nldc.i4 0".to_owned()
        } else {
            format!("ldfld {}", count - 1)
        };
        let source = format!(
            ".module Lanes\n.entry Main\n.type Record\n{fields}.end\n.function Echo(Record value) -> Record\nldarg value\nret\n.end\n.function Main() -> Int32\n{constants}newobj Record\ncall Echo(Record)\n{read}\nret\n.end\n"
        );
        let m = neoclr::assemble(&source).unwrap();
        let expected = if count == 0 { 0 } else { count - 1 };
        assert_eq!(
            neoclr::run(&m, neoclr::Limits::default()).unwrap().value,
            neoclr::Value::Int32(expected)
        );
        native(
            &neoclr::metadata_container::write_module(&m).unwrap(),
            0,
            expected,
        );
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn scalar_host_input_reaches_value_functions() {
    let source = ".module Input\n.type Cell\n.field Value Int32\n.end\n.function Echo(Cell item) -> Cell\nldarg item\nret\n.end\n.function Main(Int32 value) -> Int32\nldarg value\nnewobj Cell\ncall Echo(Cell)\nldfld 0\nret\n.end\n";
    let m = neoclr::assemble(source).unwrap();
    let program = neoclr::LoadedProgram::new(&m).unwrap();
    let entry = program
        .resolve_function(&neoclr::assembler::parse_function_ref("Main(Int32)").unwrap())
        .unwrap();
    for input in [42, i32::MIN, i32::MAX] {
        assert_eq!(
            entry
                .invoke(vec![neoclr::Value::Int32(input)], neoclr::Limits::default())
                .unwrap()
                .value,
            neoclr::Value::Int32(input)
        );
        native_input(
            &neoclr::metadata_container::write_module(&m).unwrap(),
            input,
            0,
            input,
            "Main",
        );
    }
}
