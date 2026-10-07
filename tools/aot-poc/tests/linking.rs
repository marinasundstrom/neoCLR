use neoclr::metadata::{Instruction as Op, ModuleReference, Visibility};
use std::{
    fs,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temp(std::path::PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "aot-link-{}-{}",
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
fn invoke(dir: &Temp, app: &[u8], libraries: &[Vec<u8>], root: &str, inspect: bool) -> Output {
    let input = dir.0.join("app.neox");
    fs::write(&input, app).unwrap();
    let mut c = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"));
    if inspect {
        c.arg("--inspect")
            .arg(input)
            .arg(root)
            .arg("--closed-world");
    } else {
        c.arg("--closed-world")
            .arg(input)
            .arg(root)
            .arg(dir.0.join("app.o"));
    }
    for (i, bytes) in libraries.iter().enumerate() {
        let p = dir.0.join(format!("lib{i}.neox"));
        fs::write(&p, bytes).unwrap();
        c.arg("--module").arg(p);
    }
    c.output().unwrap()
}
fn encode(m: &neoclr::Module) -> Vec<u8> {
    neoclr::metadata_container::write_module(m).unwrap()
}
fn modules() -> Vec<neoclr::Module> {
    neoclr::assembler::assemble_modules(&[
        ".module App\n.references (Models#r1)\n.entry Main\n.function Main() -> Int32\ncall Create()\ncall Read(Cell)\nret\n.end",
        ".module Models\n.revision r1\n.type Cell\n.field internal Stored Int32\n.end\n.function Create() -> Cell\nldc.i4 42\nnewobj Cell\nret\n.end\n.function Read(Cell value) -> Int32\nldarg value\nldfld 0\nret\n.end\n.function internal Hidden() -> Int32\nldc.i4 99\nret\n.end"
    ]).unwrap()
}
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native(dir: &Temp, value: i32) {
    let host = dir.0.join("host.c");
    fs::write(&host, format!("#include <stdint.h>\nextern int32_t neoclr_entry_v2(int32_t,int32_t*);\nint main(void) {{ int32_t result=-1; return neoclr_entry_v2(0,&result) || result != {value}; }}\n")).unwrap();
    let exe = dir.0.join("app");
    let result = Command::new("clang")
        .args(["-arch", "arm64", "-Wall", "-Wextra", "-Werror"])
        .arg(host)
        .arg(dir.0.join("app.o"))
        .arg("-o")
        .arg(&exe)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(Command::new(exe).env_clear().status().unwrap().success());
    let imports = Command::new("nm")
        .arg("-u")
        .arg(dir.0.join("app.o"))
        .output()
        .unwrap();
    assert!(imports.status.success() && imports.stdout.is_empty());
}
#[test]
fn value_library_preserves_original_identities_and_matches_interpreter() {
    let m = modules();
    let runtime =
        neoclr::LoadedProgram::with_modules(&m[0], neoclr::library::system().unwrap(), &m[1..])
            .unwrap();
    assert_eq!(
        runtime.run(neoclr::Limits::default()).unwrap().value,
        neoclr::Value::Int32(42)
    );
    let dir = Temp::new();
    let inspected = invoke(&dir, &encode(&m[0]), &[encode(&m[1])], "@entry", true);
    assert!(inspected.status.success());
    assert!(!dir.0.join("app.o").exists());
    let inspection: serde_json::Value = serde_json::from_slice(&inspected.stdout).unwrap();
    assert_eq!(inspection["admission"]["accepted"], true, "{inspection}");
    let result = invoke(&dir, &encode(&m[0]), &[encode(&m[1])], "@entry", false);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report, inspection["selection"]);
    assert!(
        report["functions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["definition"]["module"] == "Models")
    );
    assert_eq!(
        report["excludedFunctions"][0]["definition"]["module"],
        "Models"
    );
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    native(&dir, 42);
}
#[test]
fn original_scope_verification_precedes_projection() {
    for case in 0..11 {
        let mut m = modules();
        let mut root = "@entry";
        match case {
            0 => {
                m[0].references = Some(vec![ModuleReference::Exact {
                    name: "Models".into(),
                    revision: "wrong".into(),
                }])
            }
            1 => {
                m[0].functions[0].body[1] = Op::Field(0);
            }
            2 => {
                let Op::Call(mut target) = m[0].functions[0].body[0].clone() else {
                    unreachable!()
                };
                target.name = "Hidden".into();
                target.definition = None;
                m[0].functions[0].body = vec![Op::Call(target), Op::Return];
            }
            3 => m[1].types[0].visibility = Visibility::Internal,
            4 => m[1].functions[0].definition.as_mut().unwrap().index = 999,
            5 => {
                let copy = m[1].clone();
                m.push(copy);
            }
            6 => m[1].entry = "Hidden".into(),
            7 => root = "Hidden",
            8 => m[0].references = Some(vec![]),
            9 => m[0].references = Some(vec![ModuleReference::Name("Missing".into())]),
            _ => {
                if let Op::Call(target) = &mut m[0].functions[0].body[0] {
                    target.definition = Some(neoclr::metadata::MemberId {
                        module: "Models".into(),
                        revision: Some("wrong".into()),
                        index: 0,
                    });
                }
            }
        }
        let dir = Temp::new();
        let libraries: Vec<_> = m[1..].iter().map(encode).collect();
        let result = invoke(&dir, &encode(&m[0]), &libraries, root, false);
        let error = String::from_utf8_lossy(&result.stderr);
        assert!(!result.status.success(), "case {case} accepted");
        assert!(!error.contains("panicked"), "{error}");
        assert!(!dir.0.join("app.o").exists());
        if case == 1 || case == 2 || case == 3 {
            assert!(error.contains("access denied"), "case {case}: {error}");
        }
        if case == 0 {
            assert!(error.contains("revision mismatch"), "{error}");
        }
    }
}
#[test]
fn raven_library_and_application_compile_from_both_native_containers() {
    let app = include_bytes!("../../../docs/experiments/aot-library/App.pe");
    let lib = include_bytes!("../../../docs/experiments/aot-library/Values.pe");
    for envelope in [false, true] {
        let app = if envelope {
            encode(&neoclr::metadata_container::decode(app).unwrap())
        } else {
            app.to_vec()
        };
        let lib = if envelope {
            encode(&neoclr::metadata_container::decode(lib).unwrap())
        } else {
            lib.to_vec()
        };
        let dir = Temp::new();
        let result = invoke(&dir, &app, &[lib], "@entry", false);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        native(&dir, 0);
    }
}
