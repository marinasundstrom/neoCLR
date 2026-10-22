use neoclr::{
    Limits, Value,
    metadata::{Instruction as Op, StaticField, Type, Visibility},
};
use std::{fs, path::PathBuf, process::Command};

fn command(command: &mut Command) -> std::process::Output {
    let result = command.output().unwrap();
    assert!(
        result.status.success(),
        "{command:?}\n{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    result
}
fn body(m: &mut neoclr::Module, name: &str, body: Vec<Op>) {
    m.functions
        .iter_mut()
        .find(|f| f.name == name)
        .unwrap()
        .body = body;
}
fn load(owner: &Type) -> Op {
    Op::StaticField {
        owner: owner.clone(),
        index: 0,
    }
}
fn store(owner: &Type) -> Op {
    Op::SetStaticField {
        owner: owner.clone(),
        index: 0,
    }
}
fn fixture(case: &str) -> neoclr::Module {
    let generic = case == "generic";
    let suffix = if generic { "<T>" } else { "" };
    let extra = match case {
        "cycle" => {
            ".type class Peer\n.method private static .cctor() -> void\nret\n.end\n.end\n"
        }
        "reference" => {
            ".type class Payload\n.field Value Int32\n.method instance .ctor(Int32 value) -> void\nldarg 0\nldarg 1\nstfld 0\nret\n.end\n.end\n"
        }
        _ => "",
    };
    let mut m = neoclr::assemble(&format!(".module Initializers\n.entry Main\n.type class Cache{suffix}\n.method private static .cctor() -> void\nret\n.end\n.method static Read() -> Int32\nldc.i4 42\nret\n.end\n.end\n{extra}.function Main() -> Int32\nldc.i4 0\nret\n.end")).unwrap();
    if case == "strict" || case == "relaxed" {
        // Empty storage owners still need an initialization state; this faults if run.
        body(
            &mut m,
            "Cache..cctor",
            vec![
                Op::Int(i32::MAX),
                Op::Int(1),
                Op::AddChecked,
                Op::Pop,
                Op::Return,
            ],
        );
        body(
            &mut m,
            "Main",
            vec![
                Op::Call(neoclr::assembler::parse_function_ref("Cache::Read()").unwrap()),
                Op::Return,
            ],
        );
        m.types[0].before_field_init = case == "relaxed";
    } else {
        m.types[0].static_fields.push(StaticField {
            name: "Value".into(),
            ty: Type::Int32,
            visibility: Visibility::Public,
            readonly: true,
        });
        let owner = if generic {
            Type::Constructed {
                definition: "Cache".into(),
                arguments: vec![Type::TypeParameter(0)],
            }
        } else {
            Type::Named("Cache".into())
        };
        body(
            &mut m,
            "Cache..cctor",
            vec![
                load(&owner),
                Op::Int(if generic { 21 } else { 42 }),
                Op::Add,
                store(&owner),
                Op::Return,
            ],
        );
        let a = if generic {
            Type::Constructed {
                definition: "Cache".into(),
                arguments: vec![Type::Int32],
            }
        } else {
            owner
        };
        let mut ops = vec![load(&a), Op::Pop, load(&a)];
        if generic {
            ops.extend([
                load(&Type::Constructed {
                    definition: "Cache".into(),
                    arguments: vec![Type::String],
                }),
                Op::Add,
            ]);
        }
        ops.push(Op::Return);
        body(&mut m, "Main", ops);
    }
    if case == "cycle" {
        let field = m.types[0].static_fields[0].clone();
        m.types[1].static_fields.push(field);
        let a = Type::Named("Cache".into());
        let b = Type::Named("Peer".into());
        body(
            &mut m,
            "Cache..cctor",
            vec![load(&b), Op::Int(2), Op::Add, store(&a), Op::Return],
        );
        body(
            &mut m,
            "Peer..cctor",
            vec![load(&a), Op::Int(20), Op::Add, store(&b), Op::Return],
        );
        body(
            &mut m,
            "Main",
            vec![load(&a), load(&b), Op::Add, Op::Return],
        );
    }
    if case == "reference" {
        let owner = Type::Named("Cache".into());
        let payload = Type::Named("Payload".into());
        m.types[0].static_fields[0].ty = payload.clone();
        let constructor =
            neoclr::assembler::parse_function_ref("instance Payload::.ctor(Int32)").unwrap();
        body(
            &mut m,
            "Cache..cctor",
            vec![
                Op::Int(42),
                Op::Construct(constructor.clone()),
                store(&owner),
                Op::Int(0),
                Op::Store(0),
                Op::Int(0),
                Op::Construct(constructor.clone()),
                Op::Pop,
                Op::Load(0),
                Op::Int(1),
                Op::Add,
                Op::Store(0),
                Op::Load(0),
                Op::Int(6000),
                Op::Less,
                Op::BranchTrue(5),
                Op::Return,
            ],
        );
        m.functions
            .iter_mut()
            .find(|f| f.name == "Cache..cctor")
            .unwrap()
            .locals
            .push(Type::Int32);
        body(
            &mut m,
            "Main",
            vec![
                Op::Int(7),
                Op::Construct(constructor),
                load(&owner),
                Op::Field(0),
                Op::Store(0),
                Op::Field(0),
                Op::Load(0),
                Op::Add,
                Op::Int(7),
                Op::Sub,
                Op::Return,
            ],
        );
        m.functions
            .iter_mut()
            .find(|f| f.name == "Main")
            .unwrap()
            .locals
            .push(Type::Int32);
    }
    m
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn readonly_initializers_agree_in_interpreter_jit_and_aot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir =
        std::env::temp_dir().join(format!("neoclr-type-initialization-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    fs::write(
        dir.join("seed.neox"),
        neoclr::metadata_container::write_module(&seed).unwrap(),
    )
    .unwrap();
    fs::write(
        dir.join("helpers.neoil"),
        ".module Helpers\n.references ()\n",
    )
    .unwrap();
    let console = root.join("docs/experiments/aot-console");
    let host = root.join("docs/validation/interface-variance/native-host.c");
    let support = [
        console.join("native-gc.c"),
        console.join("root-probe.c"),
        console.join("text-arena.c"),
        console.join("native-stack.c"),
        root.join("docs/experiments/aot-fault-details/render.c"),
    ];
    let mut link = Command::new("xcrun");
    link.args([
        "--sdk",
        "macosx",
        "clang",
        "-std=c11",
        "-Wall",
        "-Wextra",
        "-Werror",
        "-DNEOCLR_NATIVE_GC",
        "-DJIT_HOST",
        "-dynamiclib",
        "-I",
    ])
    .arg(&console)
    .arg(&host)
    .args(&support)
    .arg("-o")
    .arg(dir.join("host.dylib"));
    command(&mut link);
    for case in [
        "field",
        "generic",
        "strict",
        "relaxed",
        "cycle",
        "reference",
    ] {
        let app = fixture(case);
        neoclr::verify(&app).unwrap();
        let result = neoclr::run(&app, Limits::default());
        if case == "strict" {
            assert_eq!(
                result.unwrap_err().code,
                neoclr::FaultCode::ArithmeticOverflow
            );
        } else {
            assert_eq!(result.unwrap().value, Value::Int32(42));
        }
        let image = dir.join(format!("{case}.neox"));
        fs::write(
            &image,
            neoclr::metadata_container::write_module(&app).unwrap(),
        )
        .unwrap();
        let options = [
            "--compile-system",
            "--reference-arena",
            "--native-gc",
            "--native-stack-budget",
        ];
        let object = dir.join(format!("{case}.o"));
        let mut compiler = Command::new(env!("CARGO_BIN_EXE_neoclr-codegen"));
        compiler
            .arg("--closed-world")
            .arg(&image)
            .arg("Main")
            .arg(&object)
            .arg("--system")
            .arg(dir.join("seed.neox"))
            .arg("--module")
            .arg(dir.join("helpers.neoil"))
            .args(options);
        command(&mut compiler);
        let binary = dir.join(case);
        let mut link = Command::new("xcrun");
        link.args([
            "--sdk",
            "macosx",
            "clang",
            "-std=c11",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-fsanitize=undefined,bounds",
            "-DNEOCLR_NATIVE_GC",
            "-I",
        ])
        .arg(&console)
        .arg(&host)
        .arg(&object)
        .args(&support)
        .arg("-o")
        .arg(&binary);
        command(&mut link);
        let expected = if case == "strict" { 2 } else { 42 };
        let result = Command::new(binary).output().unwrap();
        assert_eq!(
            result.status.code(),
            Some(expected),
            "{case}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        if case == "strict" {
            assert!(String::from_utf8_lossy(&result.stderr).contains("status=2 fault=2 result=-1"));
        }
        let mut jit = Command::new(env!("CARGO_BIN_EXE_neoclr-codegen"));
        jit.args(["--jit-managed", "--closed-world"])
            .arg(&image)
            .arg("Main")
            .arg(dir.join("host.dylib"))
            .arg("--system")
            .arg(dir.join("seed.neox"))
            .arg("--module")
            .arg(dir.join("helpers.neoil"))
            .args(options);
        let result = command(&mut jit);
        let stderr = String::from_utf8_lossy(&result.stderr);
        let report: serde_json::Value =
            serde_json::from_str(stderr.split("JIT_RESULT ").nth(1).unwrap().trim()).unwrap();
        assert_eq!(report["guestExitCode"], expected, "{case}: {stderr}");
        if case == "strict" {
            assert!(stderr.contains("status=2 fault=2 result=-1"));
        }
    }
    fs::remove_dir_all(dir).unwrap();
}
