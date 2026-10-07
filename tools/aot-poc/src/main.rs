mod compiler;

use std::{env, fs, io::Write};

fn main() {
    if let Err(error) = run() {
        eprintln!("aot-poc: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err("usage: neoclr-aot-poc <source.neoil> <root-name> <output.o>".into());
    }
    let source = fs::read_to_string(&args[0])?;
    let root = args[1].to_str().ok_or("root name must be UTF-8")?;
    let object = compiler::compile(&source, root)?;
    // Do not clobber an existing artifact, including on failed compilation.
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[2])?;
    output.write_all(&object)?;
    println!("Emitted aarch64-apple-darwin object; C export: int32_t neoclr_entry(int32_t)");
    Ok(())
}
