mod bindings;
mod boxing;
mod compiler;
mod fault_details;
mod inspection;
mod linking;
mod limits;
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
    let probe_count = args.iter().filter(|a| *a == "--probe-stack-roots").count();
    if probe_count > 1 { return Err("duplicate --probe-stack-roots option".into()); }
    let gc_count = args.iter().filter(|a| *a == "--native-gc").count();
    if gc_count > 1 { return Err("duplicate --native-gc option".into()); }
    let native_gc = gc_count == 1;
    let stack_count = args.iter().filter(|a| *a == "--native-stack-budget").count();
    if stack_count > 1 { return Err("duplicate --native-stack-budget option".into()); }
    let native_stack_budget = stack_count == 1;
    if native_stack_budget && !native_gc { return Err("--native-stack-budget requires --native-gc".into()); }
    args.retain(|a| a != "--native-stack-budget");
    if native_gc && probe_count != 0 { return Err("--native-gc and --probe-stack-roots are distinct modes".into()); }
    args.retain(|a| a != "--native-gc");
    let probe_stack_roots = probe_count == 1 || native_gc;
    args.retain(|a| a != "--probe-stack-roots");
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
    let output_count = args.iter().filter(|a| *a == "--bind-console-write-line").count();
    if output_count > 1 { return Err("duplicate --bind-console-write-line option".into()); }
    let bind_console_write_line = output_count == 1;
    args.retain(|a| a != "--bind-console-write-line");
    if bind_console_write_line && !compile_system {
        return Err("--bind-console-write-line requires --compile-system".into());
    }
    let stream_output_count = args.iter().filter(|a| *a == "--bind-console-stream-output").count();
    if stream_output_count > 1 { return Err("duplicate --bind-console-stream-output option".into()); }
    let bind_console_stream_output = stream_output_count == 1;
    args.retain(|a| a != "--bind-console-stream-output");
    if bind_console_stream_output && !compile_system {
        return Err("--bind-console-stream-output requires --compile-system".into());
    }
    let format_count = args.iter().filter(|a| *a == "--bind-int32-to-string").count();
    if format_count > 1 { return Err("duplicate --bind-int32-to-string option".into()); }
    let bind_int32_to_string = format_count == 1;
    args.retain(|a| a != "--bind-int32-to-string");
    if bind_int32_to_string && !compile_system {
        return Err("--bind-int32-to-string requires --compile-system".into());
    }
    let utf8_count = args.iter().filter(|a| *a == "--bind-utf8-text").count();
    if utf8_count > 1 { return Err("duplicate --bind-utf8-text option".into()); }
    let bind_utf8_text = utf8_count == 1;
    args.retain(|a| a != "--bind-utf8-text");
    if bind_utf8_text && !compile_system {
        return Err("--bind-utf8-text requires --compile-system".into());
    }
    let character_count = args.iter().filter(|a| *a == "--bind-character-text").count();
    if character_count > 1 { return Err("duplicate --bind-character-text option".into()); }
    let bind_character_text = character_count == 1;
    args.retain(|a| a != "--bind-character-text");
    if bind_character_text && !compile_system {
        return Err("--bind-character-text requires --compile-system".into());
    }
    let integer_count = args.iter().filter(|a| *a == "--bind-integer-text").count();
    if integer_count > 1 { return Err("duplicate --bind-integer-text option".into()); }
    let bind_integer_text = integer_count == 1;
    args.retain(|a| a != "--bind-integer-text");
    if bind_integer_text && !compile_system {
        return Err("--bind-integer-text requires --compile-system".into());
    }
    let task_count = args.iter().filter(|a| *a == "--bind-task-queue").count();
    if task_count > 1 { return Err("duplicate --bind-task-queue option".into()); }
    let bind_task_queue = task_count == 1;
    args.retain(|a| a != "--bind-task-queue");
    if bind_task_queue && (!compile_system || !native_gc) {
        return Err("--bind-task-queue requires --compile-system and --native-gc".into());
    }
    let accept_count = args.iter().filter(|a| *a == "--bind-socket-accept").count();
    if accept_count > 1 { return Err("duplicate --bind-socket-accept option".into()); }
    let bind_socket_accept = accept_count == 1;
    args.retain(|a| a != "--bind-socket-accept");
    if bind_socket_accept && (!compile_system || !native_gc) {
        return Err("--bind-socket-accept requires --compile-system and --native-gc".into());
    }
    let transfer_count = args.iter().filter(|a| *a == "--bind-socket-transfer").count();
    if transfer_count > 1 { return Err("duplicate --bind-socket-transfer option".into()); }
    let bind_socket_transfer = transfer_count == 1;
    args.retain(|a| a != "--bind-socket-transfer");
    if bind_socket_transfer && (!compile_system || !native_gc) {
        return Err("--bind-socket-transfer requires --compile-system and --native-gc".into());
    }
    let socket_count = args.iter().filter(|a| *a == "--bind-socket-listener").count();
    if socket_count > 1 { return Err("duplicate --bind-socket-listener option".into()); }
    let bind_socket_listener = socket_count == 1;
    args.retain(|a| a != "--bind-socket-listener");
    if bind_socket_listener && !compile_system {
        return Err("--bind-socket-listener requires --compile-system".into());
    }
    let reference_count = args.iter().filter(|a| *a == "--reference-arena").count();
    if reference_count > 1 { return Err("duplicate --reference-arena option".into()); }
    let reference_arena = reference_count == 1;
    args.retain(|a| a != "--reference-arena");
    if bind_socket_listener && !reference_arena { return Err("--bind-socket-listener requires --reference-arena".into()); }
    if native_gc && !reference_arena { return Err("--native-gc requires --reference-arena".into()); }
    if reference_arena && !compile_system {
        return Err("--reference-arena requires --compile-system".into());
    }
    if bind_console_stream_output && !reference_arena {
        return Err("--bind-console-stream-output requires --reference-arena".into());
    }
    if bind_utf8_text && !reference_arena { return Err("--bind-utf8-text requires --reference-arena".into()); }
    let fault_details_count = args.iter().filter(|a| *a == "--fault-details").count();
    if fault_details_count > 1 { return Err("duplicate --fault-details option".into()); }
    let fault_details = fault_details_count == 1 || bind_user_fault || bind_console_read_byte || bind_console_write_line || bind_console_stream_output || bind_int32_to_string || bind_character_text || bind_integer_text || reference_arena;
    args.retain(|a| a != "--fault-details");
    if probe_stack_roots && !fault_details {
        return Err("--probe-stack-roots requires --fault-details or an existing context-enabled binding".into());
    }
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
            "usage: neoclr-aot-poc <input.neoil|input.neox|input.dll> <root-name|@entry> <output.o> [--console]; or --inspect <input> <root-name|@entry> [--closed-world]; or --closed-world <input> <root-name|@entry> <output.o>; closed-world modes accept trailing --module <library>, --system <seed>, --object-root <dependency> pairs; --compile-system opts supplied System managed bodies into closed-world selection; --bind-user-fault binds exact supplied neoCLR.Runtime.Fault/Fail services to UserFault with details; --bind-console-read-byte binds the exact supplied input service to a linked C adapter; --bind-console-write-line binds the exact supplied output service to a linked UTF-8 adapter; --bind-console-stream-output binds raw byte Write/Flush with --reference-arena; --bind-int32-to-string binds formatting with caller-owned text arena ABI v4; --bind-utf8-text binds UTF-8 encoding/decoding, concatenation, ordinal predicates, byte counts and scalar-boundary slices with --reference-arena; --bind-character-text binds exact UTF-8 grapheme character services; --bind-integer-text binds Int32 parsing, signed/unsigned 64-bit formatting and native-width conversion services; --bind-socket-transfer binds Receive/Send/TransferResult and request-deadline services with --native-gc; --bind-task-queue binds exact closed TaskQueue services with --native-gc; --bind-socket-accept binds Accept/ConnectResult/Cancel with --native-gc; --bind-socket-listener binds Listen/LocalPort/Close to an explicit host socket scope (requires --reference-arena); --reference-arena admits bounded invocation-owned reference objects in ABI v4; --fault-details exports ABI v3 with caller-owned diagnostics; --probe-stack-roots adds a read-only pre-operation spill callback (requires a context-enabled profile, not a collector); --native-gc enables experimental nonmoving collection with --reference-arena and a matching statically linked GC adapter; --native-stack-budget opts into guarded recursion with --native-gc and the matching macOS ARM64 stack adapter"
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
                bind_console_write_line,
                bind_console_stream_output,
                bind_int32_to_string,
                bind_utf8_text,
                bind_character_text,
                bind_integer_text,
                bind_socket_listener,
                bind_socket_accept,
                bind_task_queue,
                bind_socket_transfer,
                reference_arena,
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
                fault_details,
                probe_stack_roots,
                native_gc,
                native_stack_budget
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
    let details = details.map(|mut d| { d.probe_stack_roots = probe_stack_roots; d.native_gc = native_gc; d.native_stack_budget = native_stack_budget; d });
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
