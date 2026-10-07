mod compiler;
mod inspection;
mod linking;
mod selection;
mod specialization;

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
    let mut args: Vec<_> = env::args_os().skip(1).collect();
    let dependency_args = args
        .iter()
        .position(|a| a == "--module")
        .map(|i| args.split_off(i))
        .unwrap_or_default();
    if dependency_args.len() > 16
        || dependency_args.len() % 2 != 0
        || dependency_args.chunks(2).any(|pair| pair[0] != "--module")
    {
        return Err("expected up to eight trailing --module <input> pairs".into());
    }
    let inspect = args.first().is_some_and(|a| a == "--inspect");
    let inspect_closed = inspect && args.get(3).is_some_and(|a| a == "--closed-world");
    let closed = args.first().is_some_and(|a| a == "--closed-world");
    if !(args.len() == 3
        || (args.len() == 4 && (args[3] == "--console" || closed || inspect_closed)))
    {
        return Err(
            "usage: neoclr-aot-poc <input.neoil|input.neox|input.dll> <root-name|@entry> <output.o> [--console]; or --inspect <input> <root-name|@entry> [--closed-world]; or --closed-world <input> <root-name|@entry> <output.o>; closed-world modes accept trailing --module <library> pairs"
                .into(),
        );
    }
    if inspect && !(args.len() == 3 || inspect_closed) {
        return Err(
            "usage: neoclr-aot-poc --inspect <input> <root-name|@entry> [--closed-world]".into(),
        );
    }
    if closed && args.len() != 4 {
        return Err(
            "usage: neoclr-aot-poc --closed-world <input> <root-name|@entry> <output.o>".into(),
        );
    }
    let shifted = inspect || closed;
    let input_path = &args[usize::from(shifted)];
    if !dependency_args.is_empty() && !(closed || inspect_closed) {
        return Err("--module requires explicit closed-world emission or inspection".into());
    }
    let input = read_module(Path::new(input_path))?;
    let dependencies = dependency_args
        .chunks(2)
        .map(|pair| read_module(Path::new(&pair[1])))
        .collect::<Result<Vec<_>, _>>()?;
    let root = args[1 + usize::from(shifted)]
        .to_str()
        .ok_or("root name must be UTF-8")?;
    let root = if root == "@entry" { &input.entry } else { root };
    if inspect {
        println!(
            "{}",
            serde_json::to_string_pretty(&inspection::report(
                &input,
                root,
                inspect_closed,
                &dependencies
            ))?
        );
        return Ok(());
    }
    let selection = if closed {
        Some(if dependencies.is_empty() {
            selection::prepare(&input, root)?
        } else {
            linking::prepare(&input, &dependencies, root)?
        })
    } else {
        None
    };
    let compile_input = selection.as_ref().map_or(&input, |(module, _)| module);
    let object = compiler::compile(compile_input, root, !closed && args.len() == 4)?;
    // Do not clobber an existing artifact, including on failed compilation.
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[2 + usize::from(closed)])?;
    output.write_all(&object)?;
    if let Some((_, report)) = selection {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }
    println!(
        "Emitted aarch64-apple-darwin object; C export: int32_t neoclr_entry_v2(int32_t, int32_t *result)"
    );
    Ok(())
}

fn read_module(input_path: &Path) -> Result<neoclr::Module, Box<dyn std::error::Error>> {
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
    } else if input_path
        .extension()
        .is_some_and(|extension| extension == "neoil")
    {
        neoclr::assemble(std::str::from_utf8(&bytes)?).map_err(|error| error.to_string())?
    } else {
        return Err("expected neoCLR CIL in NEOX, PE/#Neo or .neoil source".into());
    };
    Ok(input)
}
