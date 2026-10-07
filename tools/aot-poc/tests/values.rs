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
    compile_mode(bytes, temp, root, false)
}
fn compile_mode(bytes: &[u8], temp: &Temp, root: &str, closed: bool) -> std::process::Output {
    let input = temp.0.join("input.bin");
    fs::write(&input, bytes).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"));
    if closed {
        command.arg("--closed-world");
    }
    command
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
    native_mode(bytes, input, expected_status, expected_value, root, false);
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_mode(
    bytes: &[u8],
    input: i32,
    expected_status: i32,
    expected_value: i32,
    root: &str,
    closed: bool,
) {
    let temp = Temp::new();
    let result = compile_mode(bytes, &temp, root, closed);
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

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn raven_nested_values_preserve_constructor_and_copy_semantics() {
    let bytes = include_bytes!("../../../docs/experiments/aot-values/Nested.pe");
    let m = module(bytes);
    assert_eq!(
        neoclr::run(&m, neoclr::Limits::default()).unwrap().value,
        neoclr::Value::Int32(0)
    );
    native(bytes, 0, 0);
    native(&neoclr::metadata_container::write_module(&m).unwrap(), 0, 0);
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn nested_field_addresses_and_aggregate_updates_preserve_neighbors() {
    let source = r#"
.module Nested
.entry Main
.type Empty
.end
.type Pair
.field A Int32
.field B Int32
.end
.type Middle
.field Empty Empty
.field Pair Pair
.field Tail Int32
.end
.type Outer
.field Prefix Int32
.field Middle Middle
.field Suffix Int32
.end
.function Echo(Outer value) -> Outer
ldarg value
ret
.end
.function Main() -> Int32
.local Outer original
.local Outer copy
.local Int32 observed
ldc.i4 1000
newobj Empty
ldc.i4 3
ldc.i4 4
newobj Pair
ldc.i4 100
newobj Middle
ldc.i4 10
newobj Outer
stloc original
; Whole aggregate snapshots and nested value stfld.
ldloc original
ldloc original
ldfld Outer::Middle
ldc.i4 5
ldc.i4 6
newobj Pair
stfld Middle::Pair
stfld Outer::Middle
call Echo(Outer)
stloc copy
; Keep a deep interior address across replacement of its containing value.
ldloca original
ldflda Outer::Middle
ldflda Middle::Pair
ldflda Pair::B
ldloca original
ldloc copy
stobj Outer
ldc.i4 7
stobj Int32
ldloc original
ldfld Outer::Middle
ldfld Middle::Pair
ldfld Pair::B
stloc observed
; Aggregate field store through borrowed storage.
ldloca original
ldflda Outer::Middle
ldc.i4 8
ldc.i4 9
newobj Pair
stfld Middle::Pair
pop
; Clearing nested storage must preserve adjacent fields.
ldloca copy
ldflda Outer::Middle
ldflda Middle::Pair
initobj Pair
ldloc original
ldfld Outer::Prefix
ldloc original
ldfld Outer::Middle
ldfld Middle::Tail
add
ldloc original
ldfld Outer::Suffix
add
ldloc original
ldfld Outer::Middle
ldfld Middle::Pair
ldfld Pair::A
add
ldloc original
ldfld Outer::Middle
ldfld Middle::Pair
ldfld Pair::B
add
ldloc copy
ldfld Outer::Middle
ldfld Middle::Pair
ldfld Pair::A
add
ldloc observed
add
ret
.end
"#;
    let m = neoclr::assemble(source).unwrap();
    assert_eq!(
        neoclr::run(&m, neoclr::Limits::default()).unwrap().value,
        neoclr::Value::Int32(1134)
    );
    native(
        &neoclr::metadata_container::write_module(&m).unwrap(),
        0,
        1134,
    );
}

#[test]
fn cyclic_unknown_and_oversized_inline_layouts_never_emit() {
    for (fields, diagnostic) in [
        (
            ".type A\n.field Child B\n.end\n.type B\n.field Child A\n.end",
            "recursive inline",
        ),
        (".type A\n.field Child Int32\n.end", "local named record"),
        (
            ".type A\n.field X Int32\n.field Y Int32\n.field Z Int32\n.end\n.type B\n.field X A\n.field Y A\n.field Z A\n.end",
            "eight flattened lanes",
        ),
    ] {
        let source = format!(
            ".module Invalid\n.entry Main\n{fields}\n.function Main() -> Int32\nldc.i4 0\nret\n.end\n"
        );
        let mut m = neoclr::assemble(&source).unwrap();
        if diagnostic == "local named record" {
            m.types[0].fields[0].ty = neoclr::metadata::Type::Named("Missing".into());
        }
        let temp = Temp::new();
        let result = compile(
            &neoclr::metadata_container::write_module(&m).unwrap(),
            &temp,
        );
        assert!(!result.status.success());
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(diagnostic),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(!temp.0.join("value.o").exists());
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn raven_output_members_forward_record_storage() {
    let bytes = include_bytes!("../../../docs/experiments/aot-values/Outputs.pe");
    let m = module(bytes);
    assert_eq!(
        neoclr::run(&m, neoclr::Limits::default()).unwrap().value,
        neoclr::Value::Int32(0)
    );
    native(bytes, 0, 0);
    native(&neoclr::metadata_container::write_module(&m).unwrap(), 0, 0);
}

const OUTPUTS: &str = r#"
.module Outputs
.entry Main
.type Payload
.field Value Int32
.field Ready Boolean
.end
.type Envelope
.field Before Int32
.field Data Payload
.field After Int32
.end
.function Try(out Payload& value, Boolean success) -> Boolean
ldarg value
initobj Payload
ldarg success
brfalse Miss
ldarg value
ldc.i4 42
ldc.bool true
newobj Payload
stobj Payload
ldc.bool true
ret
Miss:
ldc.bool false
ret
.end
.function Forward(out Payload& value, Boolean success) -> Boolean
ldarg value
ldarg success
call Try(Payload&,Boolean)
ret
.end
"#;

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn ordinary_outputs_initialize_on_both_success_and_miss() {
    for initialized in [false, true] {
        for success in [false, true] {
            let init = if initialized {
                "ldc.i4 7\nldc.bool false\nnewobj Payload\nstloc value"
            } else {
                ""
            };
            let miss = if initialized {
                "ldloc value\nldfld Payload::Value"
            } else {
                "ldc.i4 -1"
            };
            let source = format!(
                "{OUTPUTS}\n.function Main() -> Int32\n.local Payload value\n{init}\nldloca value\nldc.bool {success}\ncall Forward(Payload&,Boolean)\nbrfalse Miss\nldloc value\nldfld Payload::Value\nret\nMiss:\n{miss}\nret\n.end"
            );
            let m = neoclr::assemble(&source).unwrap();
            let expected = if success {
                42
            } else if initialized {
                0
            } else {
                -1
            };
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
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn output_aliases_can_target_nested_storage_and_faults_preserve_export_result() {
    let body = r#"
.function Alias(out Payload& first, out Payload& second) -> Void
ldarg first
ldc.i4 3
ldc.bool false
newobj Payload
stobj Payload
ldarg second
ldc.i4 9
ldc.bool true
newobj Payload
stobj Payload
ldvoid
ret
.end
.function Main() -> Int32
.local Envelope item
ldloca item
initobj Envelope
ldloca item
ldflda Envelope::Data
ldloca item
ldflda Envelope::Data
call Alias(Payload&,Payload&)
pop
ldloc item
ldfld Envelope::Data
ldfld Payload::Value
ret
.end
"#;
    let source = format!("{OUTPUTS}{body}");
    let m = neoclr::assemble(&source).unwrap();
    assert_eq!(
        neoclr::run(&m, neoclr::Limits::default()).unwrap().value,
        neoclr::Value::Int32(9)
    );
    native(&neoclr::metadata_container::write_module(&m).unwrap(), 0, 9);
    let faulty = source.replace("ldvoid\nret", "ldc.i4 1\nldc.i4 0\ndiv\npop\nldvoid\nret");
    let m = neoclr::assemble(&faulty).unwrap();
    assert_eq!(
        neoclr::run(&m, neoclr::Limits::default()).unwrap_err().code,
        neoclr::FaultCode::DivideByZero
    );
    native(
        &neoclr::metadata_container::write_module(&m).unwrap(),
        1,
        12345,
    );
}

#[test]
fn invalid_output_initialization_and_borrow_contracts_never_emit() {
    let caller = "\n.function Main() -> Int32\n.local Payload value\nldloca value\nldc.bool true\ncall Forward(Payload&,Boolean)\nbrfalse Miss\nldloc value\nldfld Payload::Value\nret\nMiss:\nldc.i4 -1\nret\n.end";
    let valid = format!("{OUTPUTS}{caller}");
    let cases = [
        // Returning normally without assigning output is rejected.
        valid.replace("ldarg value\ninitobj Payload\n", ""),
        // Reading output before its first write is rejected.
        valid.replace(
            "ldarg value\ninitobj Payload",
            "ldarg value\nldobj Payload\npop\nldarg value\ninitobj Payload",
        ),
        // Conditional contracts and general ref remain outside this slice.
        valid.replace("out ", "out(true) "),
        valid.replace("out ", ""),
        valid.clone(),
        valid.replace("ldc.bool true\nret", "ldarg value\nret"),
    ];
    for (i, source) in cases.iter().enumerate() {
        let mut m = neoclr::assemble(source).unwrap();
        if i == 4 {
            m.functions[0].out_parameters = vec![99];
        }
        let temp = Temp::new();
        let result = compile(
            &neoclr::metadata_container::write_module(&m).unwrap(),
            &temp,
        );
        assert!(!result.status.success(), "case {i}");
        assert!(!String::from_utf8_lossy(&result.stderr).contains("panicked"));
        assert!(!temp.0.join("value.o").exists());
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn raven_byte_tag_members_branch_without_runtime_support() {
    let bytes = include_bytes!("../../../docs/experiments/aot-values/Tags.pe");
    let m = module(bytes);
    assert_eq!(
        neoclr::run(&m, neoclr::Limits::default()).unwrap().value,
        neoclr::Value::Int32(0)
    );
    native(bytes, 0, 0);
    native(&neoclr::metadata_container::write_module(&m).unwrap(), 0, 0);
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn byte_storage_truncates_at_every_native_boundary() {
    let source = r#"
.module Bytes
.type Tag
.field Code Byte
.end
.function Echo(Byte value) -> Byte
ldarg value
ret
.end
.function Update(Byte slot, Int32 input) -> Byte
ldarg input
starg slot
ldarg slot
ret
.end
.function Result(Int32 value) -> Byte
ldarg value
ret
.end
.function Assign(out Byte& value, Int32 input) -> Void
ldarg value
ldarg input
conv.u1
stobj Byte
ldvoid
ret
.end
.function Main(Int32 input) -> Int32
.local Byte value
.local Tag tag
; Local storage.
ldarg input
stloc value
ldloc value
; Indirect storage, with no explicit conversion.
ldloca value
ldarg input
stobj Byte
ldloc value
add
; Raw record construction.
ldarg input
newobj Tag
stloc tag
ldloc tag
ldfld Tag::Code
add
; Value field store.
ldloc tag
ldarg input
stfld Tag::Code
ldfld Tag::Code
add
; Borrowed field store.
ldloca tag
ldarg input
stfld Tag::Code
pop
ldloc tag
ldfld Tag::Code
add
; Parameter and return storage.
ldarg input
call Echo(Byte)
add
ldarg input
call Result(Int32)
add
ldc.i4 0
ldarg input
call Update(Byte,Int32)
add
; Output storage and conv.u1 preserve Int32 evaluation-stack semantics.
ldloca value
ldarg input
call Assign(Byte&,Int32)
pop
ldloc value
conv.i4
add
; Conversion result remains Int32 until stored.
ldarg input
conv.u1
ldc.i4 256
add
add
ret
.end
"#;
    let m = neoclr::assemble(source).unwrap();
    let program = neoclr::LoadedProgram::new(&m).unwrap();
    let entry = program
        .resolve_function(&neoclr::assembler::parse_function_ref("Main(Int32)").unwrap())
        .unwrap();
    for input in [-1, 0, 128, 255, 256, 511, i32::MIN, i32::MAX] {
        let expected = (input as u8 as i32) * 10 + 256;
        assert_eq!(
            entry
                .invoke(vec![neoclr::Value::Int32(input)], neoclr::Limits::default())
                .unwrap()
                .value,
            neoclr::Value::Int32(expected)
        );
        native_input(
            &neoclr::metadata_container::write_module(&m).unwrap(),
            input,
            0,
            expected,
            "Main",
        );
    }
}

#[test]
fn byte_profile_rejects_invalid_conversions_and_distinct_borrows() {
    for body in [
        "ldc.bool true\nconv.u1\nret",
        "ldc.i4 256\nconv.ovf.u1\nret",
        ".local Byte value\nldloca value\ninitobj Int32\nldc.i4 0\nret",
    ] {
        let source = format!(
            ".module Invalid\n.entry Main\n.type Tag\n.field Code Byte\n.end\n.function Main() -> Int32\n{body}\n.end"
        );
        let m = neoclr::assemble(&source).unwrap();
        let temp = Temp::new();
        let result = compile(
            &neoclr::metadata_container::write_module(&m).unwrap(),
            &temp,
        );
        assert!(!result.status.success());
        assert!(!String::from_utf8_lossy(&result.stderr).contains("panicked"));
        assert!(!temp.0.join("value.o").exists());
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn raven_overloaded_members_resolve_by_signature() {
    let bytes = include_bytes!("../../../docs/experiments/aot-values/Overloads.pe");
    let m = module(bytes);
    assert_eq!(
        neoclr::run(&m, neoclr::Limits::default()).unwrap().value,
        neoclr::Value::Int32(0)
    );
    native(bytes, 0, 0);
    native(&neoclr::metadata_container::write_module(&m).unwrap(), 0, 0);
    let mut symbolic = m.clone();
    for f in &mut symbolic.functions {
        for op in &mut f.body {
            if let neoclr::metadata::Instruction::Call(target)
            | neoclr::metadata::Instruction::Construct(target) = op
            {
                target.definition = None;
            }
        }
    }
    native(
        &neoclr::metadata_container::write_module(&symbolic).unwrap(),
        0,
        0,
    );
}

const OVERLOADED_CONSTRUCTORS: &str = r#"
.module Overloads
.entry Main
.type Cell
.field Value Int32
.method instance byref .ctor(Int32 value) -> noresult
ldarg 0
ldarg value
stfld Cell::Value
pop
ret
.end
.method instance byref .ctor(Boolean value) -> noresult
ldarg 0
ldc.i4 7
stfld Cell::Value
pop
ret
.end
.method instance byref Read() -> Int32
ldarg 0
ldfld Cell::Value
ret
.end
.end
.function Main() -> Int32
.local Cell first
.local Cell second
ldc.i4 42
newobj instance Cell::.ctor(Int32)
stloc first
ldc.bool true
newobj instance Cell::.ctor(Boolean)
stloc second
ldloca first
call instance Cell::Read()
ldloca second
call instance Cell::Read()
add
ret
.end
"#;

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn overloaded_constructors_preserve_member_identity() {
    let m = neoclr::assemble(OVERLOADED_CONSTRUCTORS).unwrap();
    assert_eq!(
        neoclr::run(&m, neoclr::Limits::default()).unwrap().value,
        neoclr::Value::Int32(49)
    );
    native(
        &neoclr::metadata_container::write_module(&m).unwrap(),
        0,
        49,
    );
}

#[test]
fn overloaded_calls_reject_identity_mismatches_and_ambiguous_roots() {
    use neoclr::metadata::{Instruction as Op, Type};
    let original = neoclr::assemble(OVERLOADED_CONSTRUCTORS).unwrap();
    for case in 0..5 {
        let mut m = original.clone();
        let wrong_id = m.functions[1].definition.clone();
        let main = m.functions.iter_mut().find(|f| f.name == "Main").unwrap();
        let Op::Construct(target) = &mut main.body[1] else {
            panic!("fixture changed")
        };
        match case {
            0 => target.definition = wrong_id,
            1 => target.owner = None,
            2 => target.parameters = vec![Type::Byte],
            3 => target.instance = false,
            _ => {
                let mut duplicate = main.clone();
                duplicate.definition = None;
                duplicate.parameters = vec![Type::Int32];
                duplicate.parameter_names = vec![Some("input".into())];
                m.functions.push(duplicate);
            }
        }
        let temp = Temp::new();
        let result = compile(
            &neoclr::metadata_container::write_module(&m).unwrap(),
            &temp,
        );
        assert!(!result.status.success(), "case {case}");
        assert!(!String::from_utf8_lossy(&result.stderr).contains("panicked"));
        assert!(!temp.0.join("value.o").exists());
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn overloaded_outputs_keep_identically_shaped_payloads_nominal() {
    let source = r#"
.module Payloads
.entry Main
.type Left
.field Value Int32
.end
.type Right
.field Value Int32
.end
.function Extract(out Left& value) -> Void
ldarg value
ldc.i4 42
newobj Left
stobj Left
ldvoid
ret
.end
.function Extract(out Right& value) -> Void
ldarg value
ldc.i4 7
newobj Right
stobj Right
ldvoid
ret
.end
.function Main() -> Int32
.local Left left
.local Right right
ldloca left
call Extract(Left&)
pop
ldloca right
call Extract(Right&)
pop
ldloc left
ldfld Left::Value
ldloc right
ldfld Right::Value
sub
ret
.end
"#;
    let m = neoclr::assemble(source).unwrap();
    assert_eq!(
        neoclr::run(&m, neoclr::Limits::default()).unwrap().value,
        neoclr::Value::Int32(35)
    );
    native(
        &neoclr::metadata_container::write_module(&m).unwrap(),
        0,
        35,
    );
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn closed_world_compiles_original_raven_choice_without_runtime_imports() {
    let bytes = include_bytes!("../../../docs/experiments/aot-values/Choice.pe");
    native_mode(bytes, 0, 0, 0, "@entry", true);
    native_mode(
        &neoclr::metadata_container::write_module(&module(bytes)).unwrap(),
        0,
        0,
        0,
        "@entry",
        true,
    );
}

#[test]
fn closed_world_is_explicit_and_never_omits_called_unsupported_code() {
    use neoclr::metadata::Instruction as Op;
    let source = ".module Selection\n.entry Main\n.type Cell\n.field Value Int32\n.end\n.function Main() -> Int32\nldc.i4 42\nret\n.end\n.function Unused() -> Int32\nldc.i8 1\npop\nldc.i4 0\nret\n.end";
    let m = neoclr::assemble(source).unwrap();
    let bytes = neoclr::metadata_container::write_module(&m).unwrap();
    let temp = Temp::new();
    assert!(!compile(&bytes, &temp).status.success());
    let selected = compile_mode(&bytes, &temp, "@entry", true);
    assert!(
        selected.status.success(),
        "{}",
        String::from_utf8_lossy(&selected.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&selected.stdout).unwrap();
    assert_eq!(report["excludedFunctions"][0]["name"], "Unused");
    assert_eq!(report["excludedTypes"][0]["name"], "Cell");
    for kind in 0..3 {
        let mut called = m.clone();
        called.functions[0].body = vec![
            Op::Call(neoclr::assembler::parse_function_ref("Unused()").unwrap()),
            Op::Return,
        ];
        if kind == 1 {
            // Calls in dead instructions are retained conservatively too.
            called.functions[0]
                .body
                .splice(0..0, [Op::Int(42), Op::Return]);
        } else if kind == 2 {
            if let Op::Call(target) = &mut called.functions[0].body[0] {
                target.definition = m.functions[1].definition.clone().map(|mut id| {
                    id.index = 999;
                    id
                });
            }
        }
        let temp = Temp::new();
        let result = compile_mode(
            &neoclr::metadata_container::write_module(&called).unwrap(),
            &temp,
            "@entry",
            true,
        );
        assert!(!result.status.success());
        assert!(!temp.0.join("value.o").exists());
        assert!(!String::from_utf8_lossy(&result.stderr).contains("panicked"));
    }
}

#[test]
fn closed_world_keeps_private_field_access_checks() {
    use neoclr::metadata::Instruction as Op;
    let mut m = module(COUNTER);
    let root = m.functions.iter_mut().find(|f| f.name == m.entry).unwrap();
    let constructor = root
        .body
        .iter()
        .find(|op| matches!(op, Op::Construct(_)))
        .unwrap()
        .clone();
    root.body = vec![constructor, Op::Field(0), Op::Return];
    let temp = Temp::new();
    let result = compile_mode(
        &neoclr::metadata_container::write_module(&m).unwrap(),
        &temp,
        "@entry",
        true,
    );
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("access denied"),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(!temp.0.join("value.o").exists());
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn raven_union_app_exercises_both_cases_patterns_and_copies() {
    let bytes = include_bytes!("../../../docs/experiments/aot-values/UnionApp.pe");
    native_mode(bytes, 0, 0, 0, "@entry", true);
    native_mode(
        &neoclr::metadata_container::write_module(&module(bytes)).unwrap(),
        0,
        0,
        0,
        "@entry",
        true,
    );
    let temp = Temp::new();
    assert!(
        !compile(bytes, &temp).status.success(),
        "whole-module admission must remain strict"
    );
    assert!(!temp.0.join("value.o").exists());
    let result = compile_mode(bytes, &temp, "@entry", true);
    assert!(result.status.success());
    let selection: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(selection["types"].as_array().unwrap().len(), 3);
    assert!(selection["excludedTypes"].as_array().unwrap().len() >= 3);
    assert!(
        selection["excludedFunctions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["name"].as_str().unwrap().ends_with(".ToString"))
    );
}

#[test]
fn closed_world_preserves_readonly_origin_facts_and_assembly_scope() {
    let original = module(COUNTER);
    for case in 0..2 {
        let mut m = original.clone();
        if case == 0 {
            m.types[0].origin.as_mut().unwrap().field_readonly = vec![true];
        } else {
            m.functions[0].origin.as_mut().unwrap().assembly = "Foreign".into();
        }
        let temp = Temp::new();
        let result = compile_mode(
            &neoclr::metadata_container::write_module(&m).unwrap(),
            &temp,
            "@entry",
            true,
        );
        assert!(!result.status.success());
        let error = String::from_utf8_lossy(&result.stderr);
        assert!(!error.contains("panicked"));
        assert!(
            if case == 0 {
                error.contains("readonly")
            } else {
                error.contains("source assembly")
            },
            "{error}"
        );
        assert!(!temp.0.join("value.o").exists());
    }
}
