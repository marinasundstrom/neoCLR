mod bindings;
mod compiler;
mod fault_details;
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
    let compile_system_count = args.iter().filter(|a| *a == "--compile-system").count();
    if compile_system_count > 1 {
        return Err("duplicate --compile-system option".into());
    }
    let compile_system = compile_system_count == 1;
    args.retain(|a| a != "--compile-system");
    let bind_user_fault_count = args.iter().filter(|a| *a == "--bind-user-fault").count();
    if bind_user_fault_count > 1 {
        return Err("duplicate --bind-user-fault option".into());
    }
    let bind_user_fault = bind_user_fault_count == 1;
    args.retain(|a| a != "--bind-user-fault");
    if bind_user_fault && !compile_system {
        return Err("--bind-user-fault requires --compile-system".into());
    }
    let input_count = args.iter().filter(|a| *a == "--bind-console-read-byte").count();
    if input_count > 1 { return Err("duplicate --bind-console-read-byte option".into()); }
    let bind_console_read_byte = input_count == 1;
    args.retain(|a| a != "--bind-console-read-byte");
    if bind_console_read_byte && !compile_system {
        return Err("--bind-console-read-byte requires --compile-system".into());
    }
    let fault_details_count = args.iter().filter(|a| *a == "--fault-details").count();
    if fault_details_count > 1 { return Err("duplicate --fault-details option".into()); }
    let fault_details = fault_details_count == 1 || bind_user_fault || bind_console_read_byte;
    args.retain(|a| a != "--fault-details");
    let dependency_args = args
        .iter()
        .position(|a| a == "--module" || a == "--system" || a == "--object-root")
        .map(|i| args.split_off(i))
        .unwrap_or_default();
    if dependency_args.len() > 20 || dependency_args.len() % 2 != 0 {
        return Err("expected trailing --module, --system or --object-root input pairs".into());
    }
    let mut module_paths = vec![];
    let mut system_path = None;
    let mut object_path = None;
    for pair in dependency_args.chunks(2) {
        match pair[0].to_str() {
            Some("--module") if module_paths.len() < 8 => module_paths.push(&pair[1]),
            Some("--system") if system_path.is_none() => system_path = Some(&pair[1]),
            Some("--object-root") if object_path.is_none() => object_path = Some(&pair[1]),
            _ => return Err("invalid, duplicate or excessive load-context option".into()),
        }
    }
    if compile_system && system_path.is_none() {
        return Err("--compile-system requires explicit --system".into());
    }
    if object_path.is_some() && system_path.is_none() {
        return Err("--object-root requires explicit --system".into());
    }
    if system_path.is_some() && module_paths.is_empty() {
        return Err("explicit runtime context requires at least one --module dependency".into());
    }
    let inspect = args.first().is_some_and(|a| a == "--inspect");
    let inspect_closed = inspect && args.get(3).is_some_and(|a| a == "--closed-world");
    let closed = args.first().is_some_and(|a| a == "--closed-world");
    if !(args.len() == 3
        || (args.len() == 4 && (args[3] == "--console" || closed || inspect_closed)))
    {
        return Err(
            "usage: neoclr-aot-poc <input.neoil|input.neox|input.dll> <root-name|@entry> <output.o> [--console]; or --inspect <input> <root-name|@entry> [--closed-world]; or --closed-world <input> <root-name|@entry> <output.o>; closed-world modes accept trailing --module <library>, --system <seed>, --object-root <dependency> pairs; --compile-system opts supplied System managed bodies into closed-world selection; --bind-user-fault binds exact supplied neoCLR.Runtime.Fault/Fail services to UserFault with details; --bind-console-read-byte binds the exact supplied input service to a linked C adapter; --fault-details exports ABI v3 with caller-owned diagnostics"
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
    let dependencies = module_paths
        .iter()
        .map(|path| read_module(Path::new(path)))
        .collect::<Result<Vec<_>, _>>()?;
    let object_root = if let Some(path) = object_path {
        if Path::new(path).extension().is_some_and(|e| e == "neoil") {
            return Err("--object-root requires compiled metadata".into());
        }
        let selected = fs::canonicalize(path)?;
        let matches: Vec<_> = module_paths
            .iter()
            .enumerate()
            .filter(|(_, p)| fs::canonicalize(p).is_ok_and(|p| p == selected))
            .map(|(i, _)| i)
            .collect();
        let [index] = matches.as_slice() else {
            return Err("--object-root must identify exactly one explicit --module input".into());
        };
        let module = &dependencies[*index];
        let roots: Vec<_> = module
            .types
            .iter()
            .enumerate()
            .filter(|(_, t)| t.name == "System.Object")
            .map(|(i, _)| i)
            .collect();
        let [index] = roots.as_slice() else {
            return Err("Object root artifact must contain exactly one System.Object".into());
        };
        Some(neoclr::metadata::TypeDefId {
            module: module.name.clone(),
            revision: module.revision.clone(),
            index: *index as u32,
        })
    } else {
        None
    };
    let context = system_path
        .map(|path| {
            read_module(Path::new(path)).map(|system| linking::RuntimeContext {
                system,
                object_root,
                compile_system,
                bind_user_fault,
                bind_console_read_byte,
            })
        })
        .transpose()?;
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
                &dependencies,
                context.as_ref(),
                fault_details
            ))?
        );
        return Ok(());
    }
    let selection = if closed {
        Some(if dependencies.is_empty() {
            selection::prepare(&input, root)?
        } else {
            linking::prepare(&input, &dependencies, root, context.as_ref())?
        })
    } else {
        None
    };
    let compile_input = selection.as_ref().map_or(&input, |(module, _)| module);
    let details = fault_details.then(|| fault_details::Options::from_report(selection.as_ref().map(|(_, r)| r)));
    let object = compiler::compile(compile_input, root, !closed && args.len() == 4, details.as_ref())?;
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
    println!("Emitted aarch64-apple-darwin object; C export: {}", if fault_details { "neoclr_entry_v3 with caller-owned fault details" } else { "neoclr_entry_v2" });
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
