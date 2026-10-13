use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "aot-friends-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn compile(dir: &Temp, friend: Option<&str>, private_method: bool) -> std::process::Output {
    let consumer: neoclr::Module =
        serde_json::from_str(include_str!("fixtures/friend-assemblies/Consumer.neo.json")).unwrap();
    let mut provider: neoclr::Module =
        serde_json::from_str(include_str!("fixtures/friend-assemblies/Provider.neo.json")).unwrap();
    if let Some(name) = friend {
        provider.assemblies[0].custom_attributes[0].arguments[0] =
            neoclr::metadata::AttributeArgument::String(Some(name.into()));
    } else {
        provider.assemblies[0].custom_attributes.clear();
    }
    if private_method {
        let method = provider
            .functions
            .iter_mut()
            .find(|f| f.origin.as_ref().is_some_and(|o| o.name == "Answer"))
            .unwrap();
        method.visibility = neoclr::metadata::Visibility::Private;
        method.origin.as_mut().unwrap().member_access =
            Some(neoclr::metadata_origin::SourceAccess::Private);
    }
    fs::write(
        dir.0.join("consumer.neox"),
        neoclr::metadata_container::write_module(&consumer).unwrap(),
    )
    .unwrap();
    fs::write(
        dir.0.join("provider.neox"),
        neoclr::metadata_container::write_module(&provider).unwrap(),
    )
    .unwrap();
    fs::write(dir.0.join("seed.neoil"), ".module System\n.references ()\n").unwrap();
    Command::new(env!("CARGO_BIN_EXE_neoclr-codegen"))
        .arg("--closed-world")
        .arg(dir.0.join("consumer.neox"))
        .arg("@entry")
        .arg(dir.0.join("app.o"))
        .args(["--reference-arena", "--compile-system", "--system"])
        .arg(dir.0.join("seed.neoil"))
        .arg("--module")
        .arg(dir.0.join("provider.neox"))
        .output()
        .unwrap()
}
#[test]
fn friend_compiles_internal_type_constructor_field_and_method() {
    let dir = Temp::new();
    let output = compile(&dir, Some("FriendConsumer"), false);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(dir.0.join("app.o").exists());
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
        fs::write(
            dir.0.join("host.c"),
            r#"
#include "text-arena.h"
int main(void) {
    uint64_t storage[32] = {0};
    neoclr_aot_context context = { .text = {(unsigned char*)storage, sizeof(storage), 0} };
    int32_t result = -1;
    return neoclr_entry_v4(0, &result, &context) || result != 42;
}
"#,
        )
        .unwrap();
        let link = Command::new("clang")
            .args([
                "-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-I",
            ])
            .arg(base.join("aot-console"))
            .arg(dir.0.join("host.c"))
            .arg(base.join("aot-console/text-arena.c"))
            .arg(base.join("aot-fault-details/render.c"))
            .arg(dir.0.join("app.o"))
            .arg("-o")
            .arg(dir.0.join("app"))
            .output()
            .unwrap();
        assert!(
            link.status.success(),
            "{}",
            String::from_utf8_lossy(&link.stderr)
        );
        assert!(
            Command::new(dir.0.join("app"))
                .env_clear()
                .status()
                .unwrap()
                .success()
        );
    }
}
#[test]
fn absent_wrong_and_qualified_grants_produce_no_object() {
    for friend in [
        None,
        Some("Unrelated"),
        Some("FriendConsumer, PublicKeyToken=null"),
    ] {
        let dir = Temp::new();
        let output = compile(&dir, friend, false);
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("access denied"),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!dir.0.join("app.o").exists());
    }
}

#[test]
fn friend_cannot_compile_private_method_access() {
    let dir = Temp::new();
    let output = compile(&dir, Some("FriendConsumer"), true);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("access denied"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!dir.0.join("app.o").exists());
}
