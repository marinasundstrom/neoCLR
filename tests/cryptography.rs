//! Private runtime service integration; managed API and native lowering are separate gates.
use neoclr::{
    Limits, LoadedProgram, RuntimeService, Value, assemble, assembler::parse_function_ref,
};
const SERVICES: &str = include_str!("../runtime/neoCLR/Runtime/Cryptography.neoil");
fn module(body: &str, returns: &str) -> neoclr::Module {
    let (locals, instructions): (Vec<_>, Vec<_>) =
        body.lines().partition(|line| line.starts_with(".local "));
    let body = locals.join("\n") + "\n" + &instructions.join("\n");
    assemble(&format!(
        ".module CryptoTests\n{SERVICES}\n.function Run() -> {returns}\n{body}\nret\n.end"
    ))
    .unwrap()
}
fn run(body: &str, returns: &str) -> Value {
    let module = module(body, returns);
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    program
        .resolve_function(&parse_function_ref("Run()").unwrap())
        .unwrap()
        .invoke(vec![], Limits::default())
        .unwrap()
        .value
}
fn array(name: &str, bytes: &[u8]) -> String {
    let mut s = format!(
        ".local arrayref<Byte> {name}\nldc.i4 {}\nnewarr Byte\nstloc {name}\n",
        bytes.len()
    );
    for (index, byte) in bytes.iter().enumerate() {
        s += &format!("ldloc {name}\nldc.i4 {index}\nldc.i4 {byte}\nconv.u1\nstelem Byte\n");
    }
    s
}
fn check_bytes(name: &str, bytes: &[u8]) -> String {
    let mut s = String::new();
    for (index, byte) in bytes.iter().enumerate() {
        s += &format!(
            "ldloc {name}\nldc.i4 {index}\nldelem Byte\nldc.i4 {byte}\nconv.u1\nbeq match_{name}_{index}\nfault \"wrong output\"\nmatch_{name}_{index}:\n"
        );
    }
    s
}
#[test]
fn sha256_known_answer_preserves_tail_and_supports_aliasing() {
    let expected = [
        0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22,
        0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00,
        0x15, 0xad, 0xa5,
    ];
    let code = array("input", b"abc")
        + &array("output", &[0xa5; 33])
        + "ldloc input\nldloc output\ncall neoCLR.Runtime.CryptoSha256(arrayref<Byte>,arrayref<Byte>)\npop\n"
        + &check_bytes("output", &expected)
        + "ldc.i4 1\n";
    assert_eq!(run(&code, "Int32"), Value::Int32(1));
    // Hash the same full array into itself; expected SHA-256 of 32 zero bytes.
    let expected = [
        0x66, 0x68, 0x7a, 0xad, 0xf8, 0x62, 0xbd, 0x77, 0x6c, 0x8f, 0xc1, 0x8b, 0x8e, 0x9f, 0x8e,
        0x20, 0x08, 0x97, 0x14, 0x85, 0x6e, 0xe2, 0x33, 0xb3, 0x90, 0x2a, 0x59, 0x1d, 0x0d, 0x5f,
        0x29, 0x25,
    ];
    let code = array("output", &[0; 32])
        + "ldloc output\nldloc output\ncall neoCLR.Runtime.CryptoSha256(arrayref<Byte>,arrayref<Byte>)\npop\n"
        + &check_bytes("output", &expected)
        + "ldc.i4 1\n";
    assert_eq!(run(&code, "Int32"), Value::Int32(1));
}
#[test]
fn hmac_round_trip_rejects_modified_tag() {
    let base = array("key", &[0x0b; 20])
        + &array("input", b"Hi There")
        + &array("tag", &[0; 32])
        + "ldloc key\nldloc input\nldloc tag\ncall neoCLR.Runtime.CryptoHmacSha256(arrayref<Byte>,arrayref<Byte>,arrayref<Byte>)\npop\n";
    let expected: Vec<u8> = "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect();
    let base = base + &check_bytes("tag", &expected);
    let verify = "ldloc key\nldloc input\nldloc tag\ncall neoCLR.Runtime.CryptoHmacSha256Verify(arrayref<Byte>,arrayref<Byte>,arrayref<Byte>)\n";
    assert_eq!(
        run(&(base.clone() + verify), "Boolean"),
        Value::Boolean(true)
    );
    assert_eq!(
        run(
            &(base + "ldloc tag\nldc.i4 0\nldc.i4 0\nconv.u1\nstelem Byte\n" + verify),
            "Boolean"
        ),
        Value::Boolean(false)
    );
}
#[test]
fn short_destination_is_recoverable_and_unchanged() {
    let code = array("input", b"abc")
        + &array("output", &[0xa5; 31])
        + ".local Int32 status\nldloc input\nldloc output\ncall neoCLR.Runtime.CryptoSha256(arrayref<Byte>,arrayref<Byte>)\nstloc status\n"
        + &check_bytes("output", &[0xa5; 31])
        + "ldloc status\n";
    assert_eq!(run(&code, "Int32"), Value::Int32(-1));
}
#[test]
fn random_fill_accepts_empty_and_allocated_buffers() {
    for size in [0, 32] {
        let code = array("output", &vec![0; size])
            + "ldloc output\ncall neoCLR.Runtime.CryptoRandomFill(arrayref<Byte>)\n";
        assert_eq!(run(&code, "Int32"), Value::Int32(0));
    }
}
#[test]
fn exact_signature_is_required() {
    for declaration in [
        ".function neoCLR.Runtime.CryptoSha256(arrayref<Int32>,arrayref<Byte>) -> Int32",
        ".function neoCLR.Runtime.CryptoRandomFill(arrayref<Byte>) -> Boolean",
    ] {
        assert!(
            assemble(&format!(
                ".module Invalid\n{declaration}\n.methodimpl InternalCall\n.end"
            ))
            .is_err()
        );
    }
}
#[test]
fn reachability_reports_cryptography_and_array_capabilities() {
    let module = module(
        &(array("output", &[0; 32])
            + "ldloc output\ncall neoCLR.Runtime.CryptoRandomFill(arrayref<Byte>)\n"),
        "Int32",
    );
    let program = LoadedProgram::new(&module).unwrap();
    let graph = program
        .analyze_reachability(&[parse_function_ref("Run()").unwrap()], 16)
        .unwrap();
    assert!(
        graph
            .required_services()
            .contains(&RuntimeService::Cryptography)
    );
    assert!(
        graph
            .required_services()
            .contains(&RuntimeService::ManagedArrays)
    );
}

#[test]
fn reserved_output_is_initialized_but_uninitialized_input_faults() {
    let code = ".local arrayref<Byte> input\n.local arrayref<Byte> output\nldc.i4 0\nnewarr Byte\nstloc input\nldc.i4 32\narray.reserve Byte\nstloc output\nldloc input\nldloc output\ncall neoCLR.Runtime.CryptoSha256(arrayref<Byte>,arrayref<Byte>)\npop\nldloc output\nldc.i4 0\nldelem Byte\n";
    assert_eq!(run(code, "Byte"), Value::Byte(0xe3));
    let code = code.replace("ldc.i4 0\nnewarr Byte", "ldc.i4 1\narray.reserve Byte");
    let module = module(&code, "Byte");
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    let error = program
        .resolve_function(&parse_function_ref("Run()").unwrap())
        .unwrap()
        .invoke(vec![], Limits::default())
        .err()
        .expect("uninitialized input must fault");
    assert!(error.message.contains("initialized"), "{error:?}");
}
