//! Build the C-ABI sample with the same Rust toolchain used for the runtime.
use std::{path::PathBuf, process::Command};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let directory = root.join("target/native");
    std::fs::create_dir_all(&directory)?;
    let output = directory.join(libloading::library_filename("neoclr_sample"));
    let status = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
        .args([
            "--edition=2024",
            "--crate-type=cdylib",
            "--crate-name=neoclr_sample",
        ])
        .arg(root.join("examples/native/library.rs"))
        .arg("-o")
        .arg(&output)
        .status()?;
    if !status.success() {
        return Err("native sample compilation failed".into());
    }
    println!("Built {}", output.display());
    Ok(())
}
