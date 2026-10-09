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
fn empty_and_sixteen_lane_records_cross_private_call_boundary() {
    for count in [0, 8, 16] {
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
            ".type A\n.field X Int32\n.field Y Int32\n.field Z Int32\n.field U Int32\n.field V Int32\n.field W Int32\n.end\n.type B\n.field X A\n.field Y A\n.field Z A\n.field U A\n.field V A\n.field W A\n.field A0 A\n.field A1 A\n.field A2 A\n.field A3 A\n.field A4 A\n.end",
            "sixty-four flattened lanes",
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
fn narrow_integer_storage_normalizes_every_native_boundary() {
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
    for (ty, conversion) in [("Byte", "conv.u1"), ("SByte", "conv.i1"), ("Int16", "conv.i2"), ("UInt16", "conv.u2")] {
        let source = source.replace("Byte", ty).replace("conv.u1", conversion);
        let m = neoclr::assemble(&source).unwrap();
        let program = neoclr::LoadedProgram::new(&m).unwrap();
        let entry = program
            .resolve_function(&neoclr::assembler::parse_function_ref("Main(Int32)").unwrap())
            .unwrap();
        for input in [-1, 0, 127, 128, 255, 256, 32767, 32768, 65535, 65536, i32::MIN, i32::MAX] {
            let narrowed = match ty {
                "Byte" => input as u8 as i32,
                "SByte" => input as i8 as i32,
                "Int16" => input as i16 as i32,
                "UInt16" => input as u16 as i32,
                _ => unreachable!(),
            };
            let expected = narrowed * 10 + 256;
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
    let source = ".module Selection\n.entry Main\n.type Cell\n.field Value Int32\n.end\n.function Main() -> Int32\nldc.i4 42\nret\n.end\n.function Unused() -> Int32\nldc.i4 1\nnewarr Int32\npop\nldc.i4 0\nret\n.end";
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

const RESULT_APP: &[u8] = include_bytes!("../../../docs/experiments/aot-values/ResultApp.pe");

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn raven_generic_result_and_pattern_bindings_run_in_both_containers() {
    for bytes in [
        RESULT_APP,
        include_bytes!("../../../docs/experiments/aot-values/PatternApp.pe").as_slice(),
    ] {
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
    let temp = Temp::new();
    assert!(!compile(RESULT_APP, &temp).status.success());
    assert!(!temp.0.join("value.o").exists());
    let result = compile_mode(RESULT_APP, &temp, "@entry", true);
    assert!(result.status.success());
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    let types = report["specialization"]["types"].as_array().unwrap();
    assert_eq!(types.len(), 3);
    assert_eq!(types[0]["arguments"], serde_json::json!(["Int32", "Byte"]));
    assert_ne!(
        types[1]["name"], types[2]["name"],
        "equal native lane widths must retain distinct nominal case identities"
    );
}

#[test]
fn specialization_rejects_unsupported_shapes_and_preserves_verification() {
    use neoclr::metadata::{ConstraintKind, GenericConstraint, Instruction as Op, Type};
    for case in 0..10 {
        let mut m = module(RESULT_APP);
        let carrier = m
            .types
            .iter()
            .position(|t| t.generic_parameters.len() == 2)
            .unwrap();
        let companion = m
            .types
            .iter()
            .find(|t| t.is_reference_type && t.is_abstract)
            .unwrap()
            .name
            .clone();
        let root = m.functions.iter_mut().find(|f| f.name == m.entry).unwrap();
        let shape = root
            .locals
            .iter()
            .find(|t| matches!(t, Type::Constructed { .. }))
            .unwrap()
            .clone();
        match case {
            0..=3 => {
                let Type::Constructed {
                    mut arguments,
                    definition,
                } = shape
                else {
                    unreachable!()
                };
                match case {
                    0 => arguments[0] = Type::Boolean,
                    1 => arguments[0] = Type::String,
                    2 => arguments[0] = Type::TypeParameter(0),
                    _ => {
                        arguments.pop();
                    }
                }
                root.locals.push(Type::Constructed {
                    definition,
                    arguments,
                });
            }
            4 => m.types[carrier]
                .generic_constraints
                .push(GenericConstraint {
                    parameter: 0,
                    kind: ConstraintKind::ValueType,
                }),
            5 => root.generic_parameters.push(Some("T".into())),
            6 => root.locals.push(Type::Named(companion)),
            7 => {
                let target = root
                    .body
                    .iter_mut()
                    .find_map(|op| if let Op::Call(t) = op { Some(t) } else { None })
                    .unwrap();
                target.generic_arguments.push(Type::Int32);
            }
            8 => {
                let target = root
                    .body
                    .iter_mut()
                    .find_map(|op| if let Op::Call(t) = op { Some(t) } else { None })
                    .unwrap();
                target.definition = Some(neoclr::metadata::MemberId {
                    module: m.name.clone(),
                    revision: m.revision.clone(),
                    index: 999,
                });
            }
            _ => m.types[carrier].origin.as_mut().unwrap().assembly = "Foreign".into(),
        }
        let temp = Temp::new();
        let result = compile_mode(
            &neoclr::metadata_container::write_module(&m).unwrap(),
            &temp,
            "@entry",
            true,
        );
        let error = String::from_utf8_lossy(&result.stderr);
        if case < 2 {
            // Multi-shape token cloning requires original-scope verification. This
            // old single-artifact fixture omits its declared Raven dependency.
            assert!(error.contains("missing referenced module"), "case {case}: {error}");
        }
        assert!(
            !result.status.success(),
            "case {case} unexpectedly accepted"
        );
        assert!(!temp.0.join("value.o").exists());
        assert!(!error.contains("panicked"), "case {case}: {error}");
        if case == 9 {
            assert!(error.contains("source assembly"), "{error}");
        }
    }
}

#[test]
fn static_owner_methods_work_in_whole_module_and_closed_world_modes() {
    let m = neoclr::assemble(".module StaticOwner\n.entry Main\n.type class abstract Helpers\n.sealed\n.method static Answer() -> Int32\nldc.i4 42\nret\n.end\n.end\n.function Main() -> Int32\ncall Helpers::Answer()\nret\n.end").unwrap();
    let bytes = neoclr::metadata_container::write_module(&m).unwrap();
    for closed in [false, true] {
        let dir = Temp::new();
        let result = compile_mode(&bytes, &dir, "@entry", closed);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        native_mode(&bytes, 0, 0, 42, "@entry", closed);
    }
}

#[test]
fn erased_primitives_preserve_exact_tags_copies_and_call_results() {
    let kinds = [
        ("Void", "ldvoid"),
        ("Int32", "ldc.i4 255"),
        ("Byte", "ldc.i4 511\nconv.u1"),
        ("Boolean", "ldc.bool true"),
    ];
    let mut source = String::from(
        ".module Erased\n.entry Main\n.function Copy(Value item) -> Value\nldarg item\nret\n.end\n.function Main() -> Int32\n.local Value item\n",
    );
    for (kind, value) in kinds {
        source += &format!("{value}\nvalue.pack {kind}\ncall Copy(Value)\nstloc item\n");
        for (target, _) in kinds {
            source += &format!(
                "ldloc item\nvalue.is {target}\n{} fail\n",
                if kind == target { "brfalse" } else { "brtrue" }
            );
        }
        source += &format!("ldloc item\nvalue.unpack {kind}\n");
        source += match kind {
            "Void" => "pop\n",
            "Boolean" => "brfalse fail\n",
            _ => "ldc.i4 255\nceq\nbrfalse fail\n",
        };
    }
    source += "ldc.i4 42\nret\nfail:\nldc.i4 -1\nret\n.end";
    let m = neoclr::assemble(&source).unwrap();
    assert_eq!(
        neoclr::LoadedProgram::new(&m)
            .unwrap()
            .run(neoclr::Limits::default())
            .unwrap()
            .value,
        neoclr::Value::Int32(42)
    );
    let bytes = neoclr::metadata_container::write_module(&m).unwrap();
    for closed in [false, true] {
        let temp = Temp::new();
        let compiled = compile_mode(&bytes, &temp, "@entry", closed);
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        native_mode(&bytes, 0, 0, 42, "@entry", closed);
    }
}

#[test]
fn erased_outputs_addresses_and_branch_joins_preserve_payloads() {
    let m = neoclr::assemble(".module ErasedOutput\n.entry Main\n.function Fill(out Value& result) -> noresult\nldarg result\nldc.i4 255\nvalue.pack Byte\nstobj Value\nret\n.end\n.function Main() -> Int32\n.local Value item\nldloca item\ncall Fill(Value&)\nldc.bool true\nbrfalse other\nldloca item\nldobj Value\nbr join\nother:\nldvoid\nvalue.pack Void\njoin:\nvalue.is Byte\nbrfalse fail\nldloc item\nvalue.unpack Byte\nret\nfail:\nldc.i4 -1\nret\n.end").unwrap();
    assert_eq!(
        neoclr::LoadedProgram::new(&m)
            .unwrap()
            .run(neoclr::Limits::default())
            .unwrap()
            .value,
        neoclr::Value::Int32(255)
    );
    let bytes = neoclr::metadata_container::write_module(&m).unwrap();
    let temp = Temp::new();
    let compiled = compile(&bytes, &temp);
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    native(&bytes, 0, 255);
}

#[test]
fn wrong_erased_unpack_propagates_runtime_fault_without_publishing_result() {
    for (kind, value) in [
        ("Void", "ldvoid"),
        ("Int32", "ldc.i4 255"),
        ("Boolean", "ldc.bool true"),
    ] {
        let m = neoclr::assemble(&format!(".module WrongErased\n.entry Main\n.function Unpack(Value item) -> Int32\nldarg item\nvalue.unpack Byte\nret\n.end\n.function Main() -> Int32\n{value}\nvalue.pack {kind}\ncall Unpack(Value)\nldc.i4 1\nldc.i4 0\ndiv\nadd\nret\n.end")).unwrap();
        let error = neoclr::LoadedProgram::new(&m)
            .unwrap()
            .run(neoclr::Limits::default())
            .unwrap_err();
        assert!(
            error.to_string().contains("erased value contains"),
            "{error}"
        );
        let bytes = neoclr::metadata_container::write_module(&m).unwrap();
        let temp = Temp::new();
        let compiled = compile(&bytes, &temp);
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        native(&bytes, 3, 12345);
    }
}

#[test]
fn erased_profile_rejects_defaults_uninitialized_slots_and_unsupported_payloads() {
    for (locals, body) in [
        (
            ".local Value item\n",
            "ldloca item\ninitobj Value\nldc.i4 0\nret",
        ),
        (".local Value item\n", "ldloc item\nvalue.unpack Int32\nret"),
        ("", "ldc.i4 0\nret\nvalue.pack Char"),
        ("", "ldc.i4 0\nret\nvalue.is Value"),
        ("", "ldc.i4 0\nret\nvalue.unpack Int32&"),
        ("", "ldc.i4 42\nvalue.is Int32\npop\nldc.i4 0\nret"),
    ] {
        // Feed the source directly so AOT must reject malformed or unsupported IL.
        let source = format!(
            ".module InvalidErased\n.entry Main\n.function Main() -> Int32\n{locals}{body}\n.end"
        );
        let temp = Temp::new();
        let input = temp.0.join("invalid.neoil");
        fs::write(&input, source).unwrap();
        let compiled = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"))
            .arg("--closed-world")
            .arg(input)
            .arg("@entry")
            .arg(temp.0.join("value.o"))
            .output()
            .unwrap();
        assert!(!compiled.status.success(), "accepted {body}");
        assert!(!temp.0.join("value.o").exists());
        assert!(!String::from_utf8_lossy(&compiled.stderr).contains("panicked"));
    }
}

#[test]
fn raven_generic_methods_compile_from_pe_and_neox_with_original_tokens_reported() {
    let pe = include_bytes!("../../../docs/experiments/aot-input/GenericMethods.pe");
    let m = module(pe);
    for bytes in [
        pe.to_vec(),
        neoclr::metadata_container::write_module(&m).unwrap(),
    ] {
        let temp = Temp::new();
        let compiled = compile_mode(&bytes, &temp, "@entry", true);
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&compiled.stdout).unwrap();
        let methods = report["specialization"]["methods"].as_array().unwrap();
        assert_eq!(methods.len(), 6);
        for method in methods {
            let source = method["sourceIndex"].as_u64().unwrap() as usize;
            assert_eq!(
                method["sourceOrigin"]["token"],
                m.functions[source].origin.as_ref().unwrap().token
            );
        }
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        native_mode(&bytes, 0, 0, 0, "@entry", true);
    }
    let mut invalid = m;
    invalid.functions[1].origin.as_mut().unwrap().token =
        invalid.functions[0].origin.as_ref().unwrap().token;
    let temp = Temp::new();
    let result = compile_mode(
        &neoclr::metadata_container::write_module(&invalid).unwrap(),
        &temp,
        "@entry",
        true,
    );
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("metadata token"));
    assert!(!temp.0.join("value.o").exists());
}

#[test]
fn generic_method_clones_are_bounded_and_keep_plain_callees_at_the_limit() {
    for count in [256, 257] {
        let mut source =
            String::from(".module MethodLimit\n.entry Main\n.function Main() -> Int32\n");
        for i in 0..count {
            source += &format!("call Helper{i}<Int32>()\npop\n");
        }
        source += "call Tail()\nret\n.end\n.function Tail() -> Int32\nldc.i4 42\nret\n.end\n";
        for i in 0..count {
            source += &format!(".function Helper{i}<T>() -> Int32\nldc.i4 1\nret\n.end\n");
        }
        let m = neoclr::assemble(&source).unwrap();
        let temp = Temp::new();
        let result = compile_mode(
            &neoclr::metadata_container::write_module(&m).unwrap(),
            &temp,
            "@entry",
            true,
        );
        assert_eq!(
            result.status.success(),
            count == 256,
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        if count == 257 {
            assert!(String::from_utf8_lossy(&result.stderr).contains("256 clones"));
            assert!(!temp.0.join("value.o").exists());
        }
    }
}

#[test]
fn user_fault_terminates_value_calls_and_output_assignment_paths() {
    let source = r#".module ValueFault
.function Fill(out Int32& value, Int32 fail) -> Void
    ldarg fail
    brfalse Success
    fault "output rejected"
Success:
    ldarg value
    ldc.i4 42
    stobj Int32
    ldvoid
    ret
.end
.function Forward(Int32 fail) -> Int32
    .local Int32 value
    ldloca value
    ldarg fail
    call Fill(Int32&, Int32)
    pop
    ldloc value
    ret
.end
.function Main(Int32 fail) -> Int32
    ldarg fail
    value.pack Int32
    value.unpack Int32
    call Forward(Int32)
    ldarg fail
    brfalse Done
    ldc.i4 1
    ldc.i4 0
    div
    add
Done:
    ret
.end
"#;
    let m = neoclr::assemble(source).unwrap();
    let program = neoclr::LoadedProgram::new(&m).unwrap();
    program.verify().unwrap();
    let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Main(Int32)").unwrap()).unwrap();
    let fault = method.invoke(vec![neoclr::Value::Int32(1)], neoclr::Limits::default()).unwrap_err();
    assert_eq!(fault.code, neoclr::FaultCode::UserFault);
    assert!(fault.message.contains("output rejected"));
    let bytes = neoclr::metadata_container::write_module(&m).unwrap();
    for closed in [false, true] {
        let temp = Temp::new();
        let result = compile_mode(&bytes, &temp, "Main", closed);
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))] {
            native_mode(&bytes, 1, 4, 12345, "Main", closed);
            native_mode(&bytes, 0, 0, 42, "Main", closed);
        }
    }
    // A normal return still has to assign the output, even beside a fault path.
    let bad = neoclr::assemble(&source.replace("    ldarg value\n    ldc.i4 42\n    stobj Int32\n", "")).unwrap();
    let temp = Temp::new();
    let result = compile_mode(&neoclr::metadata_container::write_module(&bad).unwrap(), &temp, "Main", true);
    assert!(!result.status.success());
    assert!(!temp.0.join("value.o").exists());
    assert!(String::from_utf8_lossy(&result.stderr).contains("definite whole-slot assignment"));
}

#[test]
fn user_fault_does_not_hide_unsupported_value_il() {
    // Both paths are reachable: a fault on one input cannot hide a missing
    // allocation capability on the other input.
    let m = neoclr::assemble(".module ValueFault\n.function Main(Int32 choose) -> Int32\nldarg choose\nbrfalse Allocate\nfault \"stop\"\nAllocate:\nldc.i4 1\nnewarr Int32\nldlen\nconv.i4\nret\n.end").unwrap();
    neoclr::LoadedProgram::new(&m).unwrap().verify().unwrap();
    let bytes = neoclr::metadata_container::write_module(&m).unwrap();
    let temp = Temp::new();
    let result = compile_mode(&bytes, &temp, "Main", true);
    assert!(!result.status.success());
    assert!(!String::from_utf8_lossy(&result.stderr).contains("panicked"));
    assert!(!temp.0.join("value.o").exists());
}

#[test]
fn immutable_utf8_literals_cross_locals_outputs_calls_and_joins() {
    let source = include_str!("../../../docs/experiments/aot-input/literals.neoil");
    let m = neoclr::assemble(source).unwrap();
    let program = neoclr::LoadedProgram::new(&m).unwrap();
    program.verify().unwrap();
    let main = program.resolve_function(&neoclr::assembler::parse_function_ref("Main(Int32)").unwrap()).unwrap();
    for input in [0, 1] {
        assert_eq!(main.invoke(vec![neoclr::Value::Int32(input)], neoclr::Limits::default()).unwrap().value, neoclr::Value::Int32(42));
    }
    let bytes = neoclr::metadata_container::write_module(&m).unwrap();
    for closed in [false, true] {
        let temp = Temp::new();
        let result = compile_mode(&bytes, &temp, "Main", closed);
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        let object = fs::read(temp.0.join("value.o")).unwrap();
        for text in ["Hej, världen 🌍", "embedded\0null", ""] {
            let mut payload = (text.len() as u64).to_le_bytes().to_vec();
            payload.extend_from_slice(text.as_bytes());
            assert!(object.windows(payload.len()).any(|w| w == payload));
        }
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        for input in [0, 1] {
            native_mode(&bytes, input, 0, 42, "Main", closed);
        }
    }
}

#[test]
fn raven_literal_transport_compiles_from_pe_and_neox() {
    let pe = include_bytes!("../../../docs/experiments/aot-input/Literals.pe");
    let m = module(pe);
    assert_eq!(neoclr::run(&m, neoclr::Limits::default()).unwrap().value, neoclr::Value::Int32(42));
    let neox = neoclr::metadata_container::write_module(&m).unwrap();
    for bytes in [pe.as_slice(), neox.as_slice()] {
        let temp = Temp::new();
        let result = compile_mode(bytes, &temp, "@entry", true);
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        native_mode(bytes, 0, 0, 42, "@entry", true);
    }
}

#[test]
fn text_profile_rejects_uninitialized_and_mistyped_copies() {
    for (declarations, body) in [
        (".local String text", "ldloc text\npop\nldc.i4 0\nret"),
        (".local String text", "ldc.i4 0\nstloc text\nldc.i4 0\nret"),
    ] {
        let m = neoclr::assemble(&format!(".module InvalidLiteral\n.function Main() -> Int32\n{declarations}\n{body}\n.end")).unwrap();
        let bytes = neoclr::metadata_container::write_module(&m).unwrap();
        let temp = Temp::new();
        let result = compile_mode(&bytes, &temp, "Main", true);
        assert!(!result.status.success(), "accepted {body}");
        assert!(!temp.0.join("value.o").exists());
        assert!(!String::from_utf8_lossy(&result.stderr).contains("panicked"));
    }

}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn multiple_closed_value_shapes_keep_layouts_members_and_faults_distinct() {
    let source = r#"
.module MultipleShapes
.type Box<T>
.field Value T
.method instance byref Get() -> T
ldarg 0
ldfld Box<T>::Value
ret
.end
.method instance byref Fail() -> Void
fault "shape failure"
.end
.end
.function Main(Int32 input) -> Int32
.local Box<Int32> number
.local Box<Boolean> flag
ldc.i4 42
newobj Box<Int32>
stloc number
ldc.bool true
newobj Box<Boolean>
stloc flag
ldarg input
ldc.i4 1
beq IntFault
ldarg input
ldc.i4 2
beq BoolFault
ldloca flag
call instance Box<Boolean>::Get()
brfalse Wrong
ldloca number
call instance Box<Int32>::Get()
ret
IntFault:
ldloca number
call instance Box<Int32>::Fail()
pop
br Wrong
BoolFault:
ldloca flag
call instance Box<Boolean>::Fail()
pop
Wrong:
ldc.i4 -1
ret
.end
"#;
    let m = neoclr::assemble(source).unwrap();
    let bytes = neoclr::metadata_container::write_module(&m).unwrap();
    let program = neoclr::LoadedProgram::new(&m).unwrap();
    let main = program.resolve_function(&neoclr::assembler::parse_function_ref("Main(Int32)").unwrap()).unwrap();
    for input in [0,1,2] {
        let reference = main.invoke(vec![neoclr::Value::Int32(input)],neoclr::Limits::default());
        if input == 0 {
            assert_eq!(reference.unwrap().value,neoclr::Value::Int32(42));
            native_mode(&bytes,input,0,42,"Main",true);
        } else {
            assert_eq!(reference.unwrap_err().code,neoclr::FaultCode::UserFault);
            native_mode(&bytes,input,4,12345,"Main",true);
        }
    }
    let temp = Temp::new();
    let r = compile_mode(&bytes,&temp,"Main",true);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let report: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    let shapes = report["types"].as_array().unwrap();
    assert_eq!(shapes.len(),2);
    assert_eq!(shapes[0]["definition"],shapes[1]["definition"]);
    assert_ne!(shapes[0]["compiledName"],shapes[1]["compiledName"]);
    assert_ne!(shapes[0]["typeArguments"],shapes[1]["typeArguments"]);
    for name in ["Box.Get","Box.Fail"] {
        let rows: Vec<_> = report["functions"].as_array().unwrap().iter().filter(|r| r["name"] == name).collect();
        assert_eq!(rows.len(),2);
        assert_eq!(rows[0]["definition"],rows[1]["definition"]);
        assert_ne!(rows[0]["compiledIndex"],rows[1]["compiledIndex"]);
        assert_ne!(rows[0]["typeArguments"],rows[1]["typeArguments"]);
    }
}

#[test]
fn multiple_value_shape_discovery_stays_bounded() {
    for count in [128,129] {
    let mut source = String::from(".module ManyShapes\n.type Box<T>\n.field Value T\n.end\n");
    for n in 0..count { source.push_str(&format!(".type Leaf{n}\n.end\n")); }
    source.push_str(".function Main() -> Int32\n");
    for n in 0..count { source.push_str(&format!(".local Box<Leaf{n}> item{n}\n")); }
    source.push_str("ldc.i4 0\nret\n.end\n");
    let m = neoclr::assemble(&source).unwrap();
    let temp = Temp::new();
    let r = compile_mode(&neoclr::metadata_container::write_module(&m).unwrap(),&temp,"Main",true);
    assert_eq!(r.status.success(),count==128,"{}",String::from_utf8_lossy(&r.stderr));
    if count==129 {
    assert!(!temp.0.join("value.o").exists());
    assert!(String::from_utf8_lossy(&r.stderr).contains("specialized type count exceeds 256"));
    }
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn unit_payloads_and_output_borrows_preserve_adjacent_fields() {
    let source=r#"
.module UnitStorage
.entry Main
.type Box<T>
.field Value T
.field Neighbor Int32
.end
.function Fill(out Void& value) -> noresult
ldarg value
ldvoid
stobj Void
ret
.end
.function Echo(Void value) -> Void
ldarg value
ret
.end
.function Main() -> Int32
.local Box<Void> box
.local Void unit
ldloca unit
initobj Void
ldloc unit
call Echo(Void)
ldc.i4 42
newobj Box<Void>
stloc box
ldloca box
ldflda Box<Void>::Value
call Fill(Void&)
ldloc box
ldfld Box<Void>::Value
pop
ldloc box
ldfld Box<Void>::Neighbor
ret
.end
"#;
    let m=neoclr::assemble(source).unwrap();
    assert_eq!(neoclr::run(&m,neoclr::Limits::default()).unwrap().value,neoclr::Value::Int32(42));
    native_mode(&neoclr::metadata_container::write_module(&m).unwrap(),0,0,42,"Main",true);
}

#[test]
fn expanded_value_function_budget_accepts_1024_and_rejects_1025() {
    for count in [1024,1025] {
        let mut source=String::from(".module FunctionBudget\n.function Main() -> Int32\n");
        for n in 1..count { source.push_str(&format!("call F{n}()\npop\n")); }
        source.push_str("ldc.i4 42\nret\n.end\n");
        for n in 1..count { source.push_str(&format!(".function F{n}() -> Int32\nldc.i4 0\nret\n.end\n")); }
        source.push_str(".function Unselected() -> Int32\nldc.i4 0\nret\n.end\n");
        let module=neoclr::assemble(&source).unwrap();
        let temp=Temp::new();
        let r=compile_mode(&neoclr::metadata_container::write_module(&module).unwrap(),&temp,"Main",true);
        assert_eq!(r.status.success(),count==1024,"{}",String::from_utf8_lossy(&r.stderr));
        if count==1025 {
            assert!(!temp.0.join("value.o").exists());
            assert!(String::from_utf8_lossy(&r.stderr).contains("selected functions exceed"));
        }
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn thirty_two_lane_nested_record_crosses_call_boundary() {
    let fields=(0..16).map(|i| format!(".field F{i} Pair\n")).collect::<String>();
    let values=(0..16).map(|i| format!("ldc.i4 {i}\nldc.i4 {}\nnewobj Pair\n",i+1)).collect::<String>();
    let source=format!(".module Wide\n.type Pair\n.field X Int32\n.field Y Int32\n.end\n.type Wide\n{fields}.end\n.function Echo(Wide value) -> Wide\nldarg value\nret\n.end\n.function Main() -> Int32\n{values}newobj Wide\ncall Echo(Wide)\nldfld 15\nldfld 1\nret\n.end\n");
    let m=neoclr::assemble(&source).unwrap();
    native_mode(&neoclr::metadata_container::write_module(&m).unwrap(),0,0,16,"Main",true);
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn sixty_four_lane_nested_record_crosses_call_boundary() {
    let fields=(0..16).map(|i| format!(".field F{i} Pair\n")).collect::<String>();
    let values=(0..16).map(|i| format!("ldc.i4 {i}\nldc.i4 {}\nldc.i4 41\nldc.i4 42\nnewobj Pair\n",i+1)).collect::<String>();
    let source=format!(".module Wide\n.type Pair\n.field X Int32\n.field Y Int32\n.field Z Int32\n.field W Int32\n.end\n.type Wide\n{fields}.end\n.function Echo(Wide value) -> Wide\nldarg value\nret\n.end\n.function Main() -> Int32\n{values}newobj Wide\ncall Echo(Wide)\nldfld 15\nldfld 3\nret\n.end\n");
    let m=neoclr::assemble(&source).unwrap();
    native_mode(&neoclr::metadata_container::write_module(&m).unwrap(),0,0,42,"Main",true);
}

#[test]
fn erased_wide_integers_preserve_bits_and_nominal_tags_through_generic_calls() {
    let mut source = String::from(".module ErasedWide\n.entry Main\n.type Box<T>\n.field Item T\n.end\n.function Copy<T>(!!0 value) -> !!0\nldarg value\nret\n.end\n.function Unpack<T>(Value value) -> !!0\nldarg value\nvalue.unpack !!0\nret\n.end\n.function Main() -> Int32\n.local Value item\n");
    for (kind, bits) in [("Int64", i64::MIN), ("Int64", i64::MAX), ("Int64", 0x1234567887654321), ("UInt64", -1), ("UInt64", i64::MIN), ("UInt64", 0)] {
        let conversion = if kind == "UInt64" { "conv.u8\n" } else { "" };
        source += &format!("ldc.i8 {bits}\n{conversion}call Copy<{kind}>({kind})\nnewobj Box<{kind}>\nldfld 0\nvalue.pack {kind}\nstloc item\n");
        for target in ["Int32", "Int64", "UInt64", "String"] {
            source += &format!("ldloc item\nvalue.is {target}\n{} fail\n", if kind == target { "brfalse" } else { "brtrue" });
        }
        source += &format!("ldloc item\ncall Unpack<{kind}>(Value)\nldc.i8 {bits}\n{conversion}ceq\nbrfalse fail\n");
        // Differences only above bit 31 must not disappear during erased transport.
        let different = bits ^ (1_i64 << 40);
        source += &format!("ldloc item\ncall Unpack<{kind}>(Value)\nldc.i8 {different}\n{conversion}ceq\nbrtrue fail\n");
    }
    source += "ldc.i4 42\nret\nfail:\nldc.i4 -1\nret\n.end\n";
    let m = neoclr::assemble(&source).unwrap();
    assert_eq!(neoclr::LoadedProgram::new(&m).unwrap().run(neoclr::Limits::default()).unwrap().value, neoclr::Value::Int32(42));
    let bytes = neoclr::metadata_container::write_module(&m).unwrap();
    let temp = Temp::new();
    let compiled = compile_mode(&bytes, &temp, "@entry", true);
    assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    native_mode(&bytes, 0, 0, 42, "@entry", true);
}

#[test]
fn erased_wide_integer_mismatch_faults_without_publishing_result() {
    for (packed, unpacked) in [("Int64", "UInt64"), ("UInt64", "Int64"), ("Int64", "Int32")] {
        let conversion = if packed == "UInt64" { "conv.u8\n" } else { "" };
        let source = format!(".module WrongWide\n.entry Main\n.function Main() -> Int32\nldc.i8 -1\n{conversion}value.pack {packed}\nvalue.unpack {unpacked}\npop\nldc.i4 42\nret\n.end");
        let m = neoclr::assemble(&source).unwrap();
        let error = neoclr::LoadedProgram::new(&m).unwrap().run(neoclr::Limits::default()).unwrap_err();
        assert!(error.to_string().contains("erased value contains"), "{error}");
        let bytes = neoclr::metadata_container::write_module(&m).unwrap();
        let temp = Temp::new();
        let compiled = compile_mode(&bytes, &temp, "@entry", true);
        assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        native_mode(&bytes, 0, 3, 12345, "@entry", true);
    }
}

#[test]
fn selected_value_type_budget_accepts_256_and_rejects_257() {
    for count in [256, 257] {
        let mut source = String::from(".module TypeBudget\n.function Main() -> Int32\n");
        for index in 0..count {
            source += &format!("call Make{index}()\npop\n");
        }
        source += "ldc.i4 42\nret\n.end\n";
        for index in 0..count {
            source += &format!(".type T{index}\n.field Value Int32\n.end\n.function Make{index}() -> T{index}\nldc.i4 1\nnewobj T{index}\nret\n.end\n");
        }
        let module = neoclr::assemble(&source).unwrap();
        let temp = Temp::new();
        let result = compile_mode(&neoclr::metadata_container::write_module(&module).unwrap(), &temp, "Main", true);
        assert_eq!(result.status.success(), count == 256, "{}", String::from_utf8_lossy(&result.stderr));
        if count == 257 {
            assert!(!temp.0.join("value.o").exists());
            assert!(String::from_utf8_lossy(&result.stderr).contains("selected types exceed"));
        }
    }
}

#[test]
fn ordinary_int32_borrow_is_not_authorized_by_a_primitive_like_name() {
    let source = ".module BorrowedInput\n.function System.Int32.ToString(Int32& value) -> Int32\nldarg value\nldobj Int32\nret\n.end\n.function Main() -> Int32\n.local Int32 value\nldc.i4 42\nstloc value\nldloca value\ncall System.Int32.ToString(Int32&)\nret\n.end\n";
    let module = neoclr::assemble(source).unwrap();
    let temp = Temp::new();
    let result = compile_mode(&neoclr::metadata_container::write_module(&module).unwrap(), &temp, "Main", true);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("explicit borrowed parameters require an output contract"));
    assert!(!temp.0.join("value.o").exists());
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn double_arithmetic_comparisons_and_storage_match_interpreter() {
    let cases = [
        ("ldc.r8 7.5\nldc.r8 2\nadd\nldc.r8 9.5\nceq", true),
        ("ldc.r8 7.5\nldc.r8 2\nsub\nldc.r8 5.5\nceq", true),
        ("ldc.r8 3.141592653589793\nldc.r8 2\nmul\nldc.r8 6.283185307179586\nceq", true),
        ("ldc.r8 7.5\nldc.r8 2\ndiv\nldc.r8 3.75\nceq", true),
        ("ldc.r8 1\nldc.r8 -0.0\ndiv\nldc.r8 -inf\nceq", true),
        ("ldc.r8 1e308\nldc.r8 2\nmul\nldc.r8 inf\nceq", true),
        ("ldc.r8 0\nldc.r8 0\ndiv\ndup\nceq", false),
        ("ldc.r8 -0.0\nldc.r8 0.0\nceq", true),
    ];
    for (body, expected) in cases {
        double_check(body, expected);
    }
    for (op, ordered, unordered) in [("ceq", false, false), ("cgt", true, false),
        ("clt", false, false), ("cgt.un", true, true), ("clt.un", false, true)] {
        double_check(&format!("ldc.r8 2\nldc.r8 1\n{op}"), ordered);
        for (left, right) in [("nan", "1"), ("1", "nan"), ("nan", "nan")] {
            double_check(&format!("ldc.r8 {left}\nldc.r8 {right}\n{op}"), unordered);
        }
    }
    for (op, expected) in [("beq", false), ("bne.un", true), ("bgt", false),
        ("blt", false), ("bge", false), ("ble", false), ("bgt.un", true),
        ("blt.un", true), ("bge.un", true), ("ble.un", true)] {
        let source = format!(".module DoubleBranch\n.entry Main\n.function Main() -> Int32\nldc.r8 nan\nldc.r8 1\n{op} yes\nldc.i4 0\nret\nyes:\nldc.i4 1\nret\n.end");
        double_program(&source, i32::from(expected));
    }
    // A nested record, local and call return exercise mixed F64/I32 ABI lanes.
    let source = ".module DoubleStorage\n.entry Main\n.type Inner\n.field Number Double\n.end\n.type Outer\n.field Inner Inner\n.field Count Int32\n.end\n.function Echo(Outer value) -> Outer\nldarg value\nret\n.end\n.function Main() -> Int32\n.local Outer value\nldc.r8 -0.0\nnewobj Inner\nldc.i4 7\nnewobj Outer\ncall Echo(Outer)\nstloc value\nldc.r8 1\nldloc value\nldfld 0\nldfld 0\ndiv\nldc.r8 -inf\nceq\nbrfalse fail\nldloc value\nldfld 1\nret\nfail:\nldc.i4 -1\nret\n.end";
    double_program(source, 7);
    let source = ".module DoubleDefault\n.entry Main\n.function Main() -> Int32\n.local Double value\nldloca value\ninitobj Double\nldc.r8 1\nldloc value\ndiv\nldc.r8 inf\nceq\nbrfalse fail\nldloca value\nldc.r8 -0.0\nstobj Double\nldc.r8 1\nldloca value\nldobj Double\ndiv\nldc.r8 -inf\nceq\nbrfalse fail\nldc.i4 1\nret\nfail:\nldc.i4 0\nret\n.end";
    double_program(source, 1);
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn double_check(body: &str, expected: bool) {
    let source = format!(".module DoubleCheck\n.entry Main\n.function Main() -> Int32\n{body}\nbrtrue yes\nldc.i4 0\nret\nyes:\nldc.i4 1\nret\n.end");
    double_program(&source, i32::from(expected));
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn double_program(source: &str, expected: i32) {
    let m = neoclr::assemble(source).unwrap();
    assert_eq!(neoclr::LoadedProgram::new(&m).unwrap().run(neoclr::Limits::default()).unwrap().value,
        neoclr::Value::Int32(expected), "{source}");
    native(&neoclr::metadata_container::write_module(&m).unwrap(), 0, expected);
}

#[test]
fn double_profile_rejects_unsupported_operations_before_codegen() {
    for body in ["ldc.r8 1\nldc.r8 2\nrem", "ldc.r8 1\nneg", "ldc.r8 1\nconv.i4"] {
        let result_type = if body.ends_with("conv.i4") { "Int32" } else { "Double" };
        let source = format!(".module RejectedDouble\n.entry Main\n.function Work() -> {result_type}\n{body}\nret\n.end\n.function Main() -> Int32\ncall Work()\npop\nldc.i4 0\nret\n.end");
        let m = neoclr::assemble(&source).unwrap();
        let temp = Temp::new();
        let result = compile(&neoclr::metadata_container::write_module(&m).unwrap(), &temp);
        assert!(!result.status.success());
        assert!(!temp.0.join("value.o").exists());
        assert!(!String::from_utf8_lossy(&result.stderr).contains("panicked"));
    }
}
