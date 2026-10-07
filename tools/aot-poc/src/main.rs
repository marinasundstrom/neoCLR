mod compiler;
mod inspection;

use std::{
    env, fs,
    io::{Read, Write},
    path::Path,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("aot-poc: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if !(args.len() == 3 || (args.len() == 4 && args[3] == "--console")) {
        return Err(
            "usage: neoclr-aot-poc <input.neoil|input.neox|input.dll> <root-name|@entry> <output.o> [--console]; or --inspect <input> <root-name|@entry>"
                .into(),
        );
    }
    let inspect = args[0] == "--inspect";
    if inspect && args.len() != 3 {
        return Err("usage: neoclr-aot-poc --inspect <input> <root-name|@entry>".into());
    }
    let input_path = &args[usize::from(inspect)];
    // Reuse the runtime's native CIL decoder; never translate binary metadata back
    // through text or interpret CLI projection bodies as neoCLR instructions.
    let mut bytes = Vec::new();
    fs::File::open(input_path)?
        .take(16 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err("input exceeds 16 MiB AOT profile limit".into());
    }
    let input = if bytes.starts_with(b"NEOX") {
        neoclr::metadata_container::decode_envelope(&bytes).map_err(|error| error.to_string())?
    } else if bytes.starts_with(b"MZ") {
        neoclr::metadata_container::decode(&bytes).map_err(|error| error.to_string())?
    } else if Path::new(input_path)
        .extension()
        .is_some_and(|extension| extension == "neoil")
    {
        neoclr::assemble(std::str::from_utf8(&bytes)?).map_err(|error| error.to_string())?
    } else {
        return Err("expected neoCLR CIL in NEOX, PE/#Neo or .neoil source".into());
    };
    let root = args[1 + usize::from(inspect)]
        .to_str()
        .ok_or("root name must be UTF-8")?;
    let root = if root == "@entry" { &input.entry } else { root };
    if inspect {
        println!(
            "{}",
            serde_json::to_string_pretty(&inspection::report(&input, root))?
        );
        return Ok(());
    }
    let object = compiler::compile(&input, root, args.len() == 4)?;
    // Do not clobber an existing artifact, including on failed compilation.
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[2])?;
    output.write_all(&object)?;
    println!(
        "Emitted aarch64-apple-darwin object; C export: int32_t neoclr_entry_v2(int32_t, int32_t *result)"
    );
    Ok(())
}
