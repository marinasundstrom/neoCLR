fn main() {
    println!("cargo:rerun-if-changed=../native/interop/imports.c");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos")
        && std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("aarch64")
    {
        cc::Build::new()
            .file("../native/interop/imports.c")
            .compile("neoclr_native_imports");
    }
}
