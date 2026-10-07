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

#[test]
fn generic_library_specialization_preserves_library_identity_and_value_copies() {
    let app = include_bytes!("../../../docs/experiments/aot-library/GenericApp.pe");
    let lib = include_bytes!("../../../docs/experiments/aot-library/GenericValues.pe");
    let library = neoclr::metadata_container::decode(lib).unwrap();
    for envelope in [false, true] {
        let app = if envelope {
            encode(&neoclr::metadata_container::decode(app).unwrap())
        } else {
            app.to_vec()
        };
        let lib = if envelope {
            encode(&library)
        } else {
            lib.to_vec()
        };
        let dir = Temp::new();
        let inspect = invoke(&dir, &app, &[lib.clone()], "@entry", true);
        assert!(inspect.status.success());
        assert!(!dir.0.join("app.o").exists());
        let inspected: serde_json::Value = serde_json::from_slice(&inspect.stdout).unwrap();
        assert_eq!(inspected["admission"]["accepted"], true, "{inspected}");
        let result = invoke(&dir, &app, &[lib], "@entry", false);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(report, inspected["selection"]);
        let shapes = report["specialization"]["types"].as_array().unwrap();
        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0]["arguments"], serde_json::json!(["Int32", "Byte"]));
        assert_eq!(shapes[0]["definition"]["module"], library.name);
        assert_eq!(
            shapes[0]["definition"]["revision"],
            library.revision.as_deref().unwrap()
        );
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        native(&dir, 0);
    }
}

#[test]
fn generic_library_rejects_multiple_shapes_references_and_cross_module_access() {
    use neoclr::metadata::Type;
    for case in 0..4 {
        let mut app = neoclr::metadata_container::decode(include_bytes!(
            "../../../docs/experiments/aot-library/GenericApp.pe"
        ))
        .unwrap();
        let mut library = neoclr::metadata_container::decode(include_bytes!(
            "../../../docs/experiments/aot-library/GenericValues.pe"
        ))
        .unwrap();
        match case {
            0 | 1 => {
                let root = app
                    .functions
                    .iter_mut()
                    .find(|f| f.name == app.entry)
                    .unwrap();
                let mut shape = root
                    .locals
                    .iter()
                    .find(|t| matches!(t, Type::Constructed { .. }))
                    .unwrap()
                    .clone();
                if let Type::Constructed { arguments, .. } = &mut shape {
                    arguments[0] = if case == 0 {
                        Type::Boolean
                    } else {
                        Type::String
                    };
                }
                root.locals.push(shape);
            }
            2 => {
                let getter = library
                    .functions
                    .iter_mut()
                    .find(|f| f.returns == Type::TypeParameter(0) && f.parameters.is_empty())
                    .unwrap();
                getter.visibility = Visibility::Internal;
            }
            _ => {
                let root = app
                    .functions
                    .iter_mut()
                    .find(|f| f.name == app.entry)
                    .unwrap();
                let getter_name = &library
                    .functions
                    .iter()
                    .find(|f| f.returns == Type::TypeParameter(0) && f.parameters.is_empty())
                    .unwrap()
                    .name;
                let call = root
                    .body
                    .iter_mut()
                    .find(|op| matches!(op, Op::Call(target) if &target.name == getter_name))
                    .unwrap();
                *call = Op::Field(0);
            }
        }
        let dir = Temp::new();
        let result = invoke(&dir, &encode(&app), &[encode(&library)], "@entry", false);
        let error = String::from_utf8_lossy(&result.stderr);
        assert!(!result.status.success(), "case {case} accepted");
        assert!(!error.contains("panicked"));
        assert!(!dir.0.join("app.o").exists());
        if case == 0 {
            assert!(error.contains("multiple closed instantiations"), "{error}");
        }
        if case >= 2 {
            assert!(error.contains("access denied"), "{error}");
        }
    }
}

fn runtime_context_fixture() -> (neoclr::Module, Vec<neoclr::Module>) {
    use neoclr::assembler::{ModuleInput, read_modules_with_object_root};
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let root = r#"
.module Core
.revision r1
.references ()
.type class abstract System.Object
.method instance virtual ToString() -> String
ldstr "root"
ret
.end
.method instance virtual Equals(System.Object other) -> Boolean
ldc.bool false
ret
.end
.method instance virtual GetHashCode() -> Int32
ldc.i4 -1
ret
.end
.end
.type Cell
.field Value Int32
.end
.function Create() -> Cell
ldc.i4 42
newobj Cell
ret
.end
"#;
    let app = ".module App\n.references (Core#r1)\n.entry Main\n.function Main() -> Int32\ncall Create()\nldfld 0\nret\n.end";
    let id = neoclr::metadata::TypeDefId {
        module: "Core".into(),
        revision: Some("r1".into()),
        index: 0,
    };
    let modules = read_modules_with_object_root(
        &[ModuleInput::Source(app), ModuleInput::Source(root)],
        &seed,
        &id,
    )
    .unwrap();
    (seed, modules)
}
fn invoke_context(
    dir: &Temp,
    seed: &neoclr::Module,
    m: &[neoclr::Module],
    case: usize,
    inspect: bool,
) -> Output {
    let app = dir.0.join("app.neox");
    let lib = dir.0.join("core.neox");
    let system = dir.0.join("system.neox");
    fs::write(&app, encode(&m[0])).unwrap();
    fs::write(&lib, encode(&m[1])).unwrap();
    fs::write(&system, encode(seed)).unwrap();
    let mut c = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"));
    if inspect {
        c.arg("--inspect")
            .arg(&app)
            .args(["@entry", "--closed-world"]);
    } else {
        c.arg("--closed-world")
            .arg(&app)
            .arg("@entry")
            .arg(dir.0.join("app.o"));
    }
    c.arg("--module").arg(&lib);
    if case != 1 {
        c.arg("--system").arg(&system);
    }
    c.arg("--object-root")
        .arg(if case == 2 { &app } else { &lib });
    if case == 3 {
        c.arg("--module").arg(&lib);
    }
    if case == 4 {
        c.arg("--system").arg(&system);
    }
    c.output().unwrap()
}
#[test]
fn explicit_runtime_object_context_preserves_validation_and_standalone_values() {
    let (seed, m) = runtime_context_fixture();
    let dir = Temp::new();
    let inspect = invoke_context(&dir, &seed, &m, 0, true);
    assert!(inspect.status.success());
    let inspection: serde_json::Value = serde_json::from_slice(&inspect.stdout).unwrap();
    assert_eq!(inspection["admission"]["accepted"], true, "{inspection}");
    assert!(!dir.0.join("app.o").exists());
    let result = invoke_context(&dir, &seed, &m, 0, false);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report, inspection["selection"]);
    assert_eq!(
        report["loadSet"]["runtimeContext"]["objectRoot"]["module"],
        "Core"
    );
    assert_eq!(report["loadSet"]["runtimeContext"]["explicit"], true);
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    native(&dir, 42);
}
#[test]
fn invalid_runtime_contexts_never_emit() {
    for case in 1..9 {
        let (mut seed, mut m) = runtime_context_fixture();
        match case {
            5 => seed.name = "NotSystem".into(),
            6 => m[1].types[0].is_abstract = false,
            7 => m[1].types[0].name = "OtherObject".into(),
            8 => {
                let method = m[1]
                    .functions
                    .iter_mut()
                    .find(|f| f.name.ends_with("GetHashCode"))
                    .unwrap();
                method.is_virtual = false;
            }
            _ => (),
        }
        let dir = Temp::new();
        let result = invoke_context(&dir, &seed, &m, case, false);
        let error = String::from_utf8_lossy(&result.stderr);
        assert!(!result.status.success(), "case {case} accepted");
        assert!(!error.contains("panicked"), "{error}");
        assert!(!dir.0.join("app.o").exists());
    }
}

#[test]
fn unused_generic_methods_are_verified_but_selected_ones_still_fail() {
    let (seed, mut m) = runtime_context_fixture();
    let template = neoclr::assemble(
        ".module Template\n.function Unused<T>(T value) -> T\nldarg value\nret\n.end",
    )
    .unwrap();
    let mut method = template.functions[0].clone();
    method.definition = None;
    m[1].functions.push(method);
    let dir = Temp::new();
    let result = invoke_context(&dir, &seed, &m, 0, false);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert!(
        report["excludedFunctions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["name"] == "Unused")
    );
    m[0].functions[0].body = vec![
        Op::Int(42),
        Op::Call(neoclr::assembler::parse_function_ref("Unused<Int32>(Int32)").unwrap()),
        Op::Return,
    ];
    let dir = Temp::new();
    let result = invoke_context(&dir, &seed, &m, 0, false);
    assert!(!result.status.success());
    assert!(!dir.0.join("app.o").exists());
}
