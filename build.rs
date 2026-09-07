#[path = "src/source.rs"]
mod source;

fn main() {
    println!("cargo:rerun-if-changed=runtime");
    println!("cargo:rerun-if-changed=src/source.rs");
    let text = source::read_source("runtime/System.neoil").expect("expand System library sources");
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    std::fs::write(output.join("System.neoil"), text).expect("write bundled System source");
}
