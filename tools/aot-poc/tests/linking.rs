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
fn generic_library_specializes_multiple_shapes_without_weakening_access() {
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
        if case < 2 {
            assert!(result.status.success(), "{error}");
            let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
            assert!(report["specialization"]["types"].as_array().unwrap().len() >= 2);
            continue;
        }
        assert!(!result.status.success(), "case {case} accepted");
        assert!(!error.contains("panicked"));
        assert!(!dir.0.join("app.o").exists());
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
    if case != 1 && case != 11 {
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
    if case >= 9 { c.arg("--compile-system"); }
    if case == 10 { c.arg("--compile-system"); }
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
fn unused_generic_methods_are_verified_and_selected_primitive_ones_compile() {
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
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    native(&dir, 42);
}

fn interface_modules() -> Vec<neoclr::Module> {
    neoclr::assembler::assemble_modules(&[
        ".module App\n.references (Models)\n.entry Main\n.function Main() -> Int32\n.local Cell c\nldc.i4 42\nnewobj Cell\nstloc c\nldloca c\ncall instance Cell::Get()\nret\n.end",
        ".module Models\n.interface Read\n.method instance byref Get() -> Int32\n.end\n.end\n.type Cell\n.implements Read\n.field Value Int32\n.method instance byref Get() -> Int32\nldarg this\nldfld Cell::Value\nret\n.end\n.end"
    ]).unwrap()
}

#[test]
fn verified_interface_contracts_allow_direct_value_calls() {
    let m = interface_modules();
    let dir = Temp::new();
    let result = invoke(&dir, &encode(&m[0]), &[encode(&m[1])], "@entry", false);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    let relationships = report["verifiedInterfaceRelationships"].as_array().unwrap();
    assert_eq!(relationships.len(), 1);
    assert_eq!(relationships[0]["definition"]["module"], "Models");
    assert_eq!(relationships[0]["name"], "Cell");
    assert!(
        !report["types"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["name"] == "Read")
    );
    let inspection = invoke(&dir, &encode(&m[0]), &[encode(&m[1])], "@entry", true);
    let inspection: serde_json::Value = serde_json::from_slice(&inspection.stdout).unwrap();
    assert_eq!(inspection["selection"], report);
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    native(&dir, 42);
}

#[test]
fn invalid_original_interface_contracts_never_emit() {
    for case in 0..3 {
        let mut m = interface_modules();
        let contract = m[1]
            .functions
            .iter_mut()
            .find(|f| f.name == "Read.Get")
            .unwrap();
        match case {
            0 => contract.name = "Read.Missing".into(),
            1 => contract.returns = neoclr::metadata::Type::Byte,
            2 => contract.parameters.push(neoclr::metadata::Type::Int32),
            _ => unreachable!(),
        }
        let dir = Temp::new();
        let result = invoke(&dir, &encode(&m[0]), &[encode(&m[1])], "@entry", false);
        assert!(!result.status.success(), "case {case} accepted");
        assert!(!dir.0.join("app.o").exists());
    }
}

#[test]
fn selected_interface_storage_and_dead_dispatch_stay_unsupported() {
    for dispatch in [false, true] {
        let mut m = interface_modules();
        if dispatch {
            m[0].functions[0].body.push(Op::CallVirtual(
                neoclr::assembler::parse_function_ref("instance Read::Get()").unwrap(),
            ));
        } else {
            m[0].functions[0].local_names.push(Some("view".into()));
            m[0].functions[0]
                .locals
                .push(neoclr::metadata::Type::InterfaceRef(Box::new(
                    neoclr::metadata::Type::Named("Read".into()),
                )));
        }
        let dir = Temp::new();
        let result = invoke(&dir, &encode(&m[0]), &[encode(&m[1])], "@entry", false);
        assert!(!result.status.success());
        assert!(!dir.0.join("app.o").exists());
        let error = String::from_utf8_lossy(&result.stderr);
        assert!(
            error.contains(if dispatch {
                "virtual calls"
            } else {
                "reference-free"
            }),
            "{error}"
        );
    }
}

#[test]
fn metadata_only_interface_arguments_do_not_consume_native_shapes() {
    let m = neoclr::assembler::assemble_modules(&[
        ".module App\n.references (Models)\n.entry Main\n.function Main() -> Int32\n.local First a\n.local Second b\nldc.i4 42\nret\n.end",
        ".module Models\n.interface Marker<T>\n.end\n.type First\n.implements Marker<Void>\n.end\n.type Second\n.implements Marker<Int32>\n.end"
    ]).unwrap();
    let dir = Temp::new();
    let result = invoke(&dir, &encode(&m[0]), &[encode(&m[1])], "@entry", false);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    let relationships = report["verifiedInterfaceRelationships"].as_array().unwrap();
    assert_eq!(relationships.len(), 2);
    assert_eq!(
        relationships[0]["interfaces"][0]["Constructed"]["arguments"][0],
        "Void"
    );
    assert_eq!(
        relationships[1]["interfaces"][0]["Constructed"]["arguments"][0],
        "Int32"
    );
    assert!(
        report["specialization"]["types"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    native(&dir, 42);
}

fn static_owner_modules(generic: bool) -> Vec<neoclr::Module> {
    let cell = if generic { "Cell<Int32>" } else { "Cell" };
    let definition = if generic {
        ".type Cell<T>\n.field Value T"
    } else {
        ".type Cell\n.field Value Int32"
    };
    neoclr::assembler::assemble_modules(&[
        ".module App\n.references (Models)\n.entry Main\n.function Main() -> Int32\ncall Factory::Make()\nldfld 0\nret\n.end",
        &format!(".module Models\n{definition}\n.end\n.type class abstract Factory\n.sealed\n.method static Make() -> {cell}\nldc.i4 42\nnewobj {cell}\nret\n.end\n.end")
    ]).unwrap()
}

#[test]
fn empty_static_owners_allow_direct_calls_with_and_without_specialization() {
    for generic in [false, true] {
        let m = static_owner_modules(generic);
        let program =
            neoclr::LoadedProgram::with_modules(&m[0], neoclr::library::system().unwrap(), &m[1..])
                .unwrap();
        assert_eq!(
            program.run(neoclr::Limits::default()).unwrap().value,
            neoclr::Value::Int32(42)
        );
        let dir = Temp::new();
        let result = invoke(&dir, &encode(&m[0]), &[encode(&m[1])], "@entry", false);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert!(
            report["types"]
                .as_array()
                .unwrap()
                .iter()
                .any(|t| t["name"] == "Factory")
        );
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        native(&dir, 42);
    }
}

#[test]
fn static_owners_do_not_enable_reference_values_or_bypass_access() {
    for generic in [false, true] {
        for case in 0..6 {
            let mut m = static_owner_modules(generic);
            let owner = m[1].types.iter_mut().find(|t| t.name == "Factory").unwrap();
            match case {
                0 => owner.is_sealed = false,
                1 => owner.is_abstract = false,
                2 => owner.fields.push(neoclr::metadata::Field {
                    name: "state".into(),
                    ty: neoclr::metadata::Type::Int32,
                    deferred: false,
                    visibility: Visibility::Public,
                }),
                3 => {
                    m[0].functions[0]
                        .locals
                        .push(neoclr::metadata::Type::Named("Factory".into()));
                    m[0].functions[0].local_names.push(Some("forbidden".into()));
                }
                4 => m[1].functions[0].visibility = Visibility::Private,
                5 => owner.visibility = Visibility::Internal,
                _ => unreachable!(),
            }
            let dir = Temp::new();
            let result = invoke(&dir, &encode(&m[0]), &[encode(&m[1])], "@entry", false);
            assert!(!result.status.success(), "generic={generic} case={case}");
            assert!(!dir.0.join("app.o").exists());
            let error = String::from_utf8_lossy(&result.stderr);
            assert!(!error.contains("panicked"), "{error}");
            if case >= 4 {
                assert!(error.contains("access denied"), "{error}");
            }
        }
    }
}

fn generic_method_modules() -> Vec<neoclr::Module> {
    neoclr::assembler::assemble_modules(&[
        ".module App\n.references (Helpers#r1)\n.entry Main\n.function Main() -> Int32\n.local Value item\nldc.i4 255\nvalue.pack Byte\nstloc item\nldloc item\ncall Forward<Byte>(Value)\nbrfalse fail\nldloc item\ncall Is<Int32>(Value)\nbrtrue fail\nldvoid\nvalue.pack Void\ncall Is<Void>(Value)\nbrfalse fail\nldc.bool true\nvalue.pack Boolean\ncall Is<Boolean>(Value)\nbrfalse fail\nldloc item\ncall Unpack<Byte>(Value)\nret\nfail:\nldc.i4 -1\nret\n.end",
        ".module Helpers\n.revision r1\n.function Is<T>(Value item) -> Boolean\nldarg item\nvalue.is T\nret\n.end\n.function Forward<T>(Value item) -> Boolean\nldarg item\ncall Is<T>(Value)\nret\n.end\n.function Unpack<T>(Value item) -> T\nldarg item\nvalue.unpack T\nret\n.end"
    ]).unwrap()
}

#[test]
fn primitive_generic_methods_clone_exact_shapes_and_preserve_library_identities() {
    let m = generic_method_modules();
    let program =
        neoclr::LoadedProgram::with_modules(&m[0], neoclr::library::system().unwrap(), &m[1..])
            .unwrap();
    assert_eq!(
        program.run(neoclr::Limits::default()).unwrap().value,
        neoclr::Value::Int32(255)
    );
    let dir = Temp::new();
    let inspected = invoke(&dir, &encode(&m[0]), &[encode(&m[1])], "@entry", true);
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
    let methods = report["specialization"]["methods"].as_array().unwrap();
    assert_eq!(methods.len(), 6); // Four Is shapes, Forward<Byte>, Unpack<Byte>.
    assert!(
        methods
            .iter()
            .all(|m| m["definition"]["module"] == "Helpers" && m["definition"]["revision"] == "r1")
    );
    let tests: Vec<_> = report["functions"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["name"] == "Is")
        .collect();
    assert_eq!(tests.len(), 4);
    assert!(
        tests.iter().all(|m| m["definition"]["index"] == 0
            && m["methodArguments"].as_array().unwrap().len() == 1)
    );
    assert_eq!(
        tests
            .iter()
            .map(|m| m["compiledName"].as_str().unwrap())
            .collect::<std::collections::HashSet<_>>()
            .len(),
        4
    );
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    native(&dir, 255);
}

#[test]
fn generic_method_specialization_rejects_bad_identity_access_and_shapes() {
    for case in 0..7 {
        let mut m = generic_method_modules();
        let call = m[0].functions[0]
            .body
            .iter_mut()
            .find_map(|op| {
                if let Op::Call(target) = op {
                    Some(target)
                } else {
                    None
                }
            })
            .unwrap();
        match case {
            0 => call.generic_arguments[0] = neoclr::metadata::Type::String,
            1 => call.generic_arguments.push(neoclr::metadata::Type::Int32),
            2 => call.parameters[0] = neoclr::metadata::Type::Int32,
            3 => {
                call.definition = Some(neoclr::metadata::MemberId {
                    module: "Helpers".into(),
                    revision: Some("wrong".into()),
                    index: 1,
                })
            }
            4 => m[1].functions[1].visibility = Visibility::Private,
            5 => m[1].functions[1]
                .generic_constraints
                .push(neoclr::metadata::GenericConstraint {
                    parameter: 0,
                    kind: neoclr::metadata::ConstraintKind::ValueType,
                }),
            6 => {
                let recursive = neoclr::assemble(".module Recursive\n.function Is<T>(Value item) -> Boolean\nldarg item\ncall Is<T>(Value)\nret\n.end").unwrap();
                m[1].functions[0].body = recursive.functions[0].body.clone();
            }
            _ => unreachable!(),
        }
        let dir = Temp::new();
        let result = invoke(&dir, &encode(&m[0]), &[encode(&m[1])], "@entry", false);
        assert!(!result.status.success(), "accepted case {case}");
        assert!(!dir.0.join("app.o").exists());
        assert!(!String::from_utf8_lossy(&result.stderr).contains("panicked"));
    }
}

#[test]
fn explicit_system_code_compiles_managed_seed_helpers_and_reports_source_identity() {
    let (_, mut m) = runtime_context_fixture();
    let seed = neoclr::assemble(".module System\n.references ()\n.function SeedEcho<T>(T value) -> T\nldarg value\nret\n.end").unwrap();
    m[0].functions[0].body = vec![Op::Int(42), Op::Call(neoclr::assembler::parse_function_ref("SeedEcho<Int32>(Int32)").unwrap()), Op::Return];
    let root = neoclr::metadata::TypeDefId { module: "Core".into(), revision: Some("r1".into()), index: 0 };
    let program = neoclr::LoadedProgram::with_modules_and_object_root(&m[0], &seed, &m[1..], &root).unwrap();
    assert_eq!(program.run(neoclr::Limits::default()).unwrap().value, neoclr::Value::Int32(42));
    let dir = Temp::new();
    let missing = invoke_context(&dir, &seed, &m, 0, false);
    assert!(!missing.status.success());
    assert!(!dir.0.join("app.o").exists());
    let inspected = invoke_context(&dir, &seed, &m, 9, true);
    let inspection: serde_json::Value = serde_json::from_slice(&inspected.stdout).unwrap();
    assert_eq!(inspection["admission"]["accepted"], true, "{inspection}");
    let emitted = invoke_context(&dir, &seed, &m, 9, false);
    assert!(emitted.status.success(), "{}", String::from_utf8_lossy(&emitted.stderr));
    let report: serde_json::Value = serde_json::from_slice(&emitted.stdout).unwrap();
    assert_eq!(report, inspection["selection"]);
    assert_eq!(report["loadSet"]["runtimeContext"]["compileSystem"], true);
    assert_eq!(report["specialization"]["methods"][0]["definition"]["module"], "System");
    assert!(report["loadSet"]["modules"].as_array().unwrap().iter().any(|m| m["name"] == "System"));
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    native(&dir, 42);
    for case in [10, 11] {
        let dir = Temp::new();
        let rejected = invoke_context(&dir, &seed, &m, case, false);
        assert!(!rejected.status.success());
        assert!(String::from_utf8_lossy(&rejected.stderr).contains("--compile-system"));
        assert!(!dir.0.join("app.o").exists());
    }
}

#[test]
fn system_code_opt_in_does_not_enable_internal_calls_or_nonpublic_access() {
    for native_service in [false, true] {
        let (_, mut m) = runtime_context_fixture();
        let seed = neoclr::assemble(if native_service {
            ".module System\n.references ()\n.function neoCLR.Runtime.ConsoleReadByte() -> Value\n.methodimpl InternalCall\n.end"
        } else {
            ".module System\n.references ()\n.function internal Hidden() -> Int32\nldc.i4 42\nret\n.end"
        }).unwrap();
        m[0].functions[0].body = if native_service {
            vec![Op::Call(neoclr::assembler::parse_function_ref("neoCLR.Runtime.ConsoleReadByte()").unwrap()), Op::UnpackValue(neoclr::metadata::Type::Byte), Op::Return]
        } else {
            vec![Op::Call(neoclr::assembler::parse_function_ref("Hidden()").unwrap()), Op::Return]
        };
        let dir = Temp::new();
        let rejected = invoke_context(&dir, &seed, &m, 9, false);
        let error = String::from_utf8_lossy(&rejected.stderr);
        assert!(!rejected.status.success());
        assert!(error.contains(if native_service { "unsupported value member contract" } else { "access denied" }), "{error}");
        assert!(!dir.0.join("app.o").exists());
    }
}
