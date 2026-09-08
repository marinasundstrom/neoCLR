mod debug_terminal;
use neoclr::{ExecutionOptions, StdioConsole, assemble, load};
use std::{
    env, fs,
    io::{self, Write},
    process::ExitCode,
};

const USAGE: &str = "Usage:
  neoclr assemble <source.neoil> <output.neo.json> [--module <input>]... [--system <input>]
  neoclr run <input> [System.neo.json] [--module <input>]... [--system <input>] [--gc-stats] [--gc-events] [-- <guest-argument>...]
  neoclr debug <input> [--module <input>]... [--system <input>] [-- <guest-argument>...]
  neoclr check <input> [--module <input>]... [--system <input>]
  neoclr verify <input> [--module <input>]... [--system <input>]
Inputs: .neo is the high-level subset, .neoil is IL source, otherwise JSON artifacts.";

fn read(path: &str) -> Result<String, String> {
    if path.ends_with(".neoil") {
        neoclr::source::read_source(path)
    } else {
        fs::read_to_string(path)
    }
    .map_err(|e| format!("Cannot read {path}: {e}"))
}

fn execute(args: &[String]) -> Result<Vec<String>, String> {
    let command = args.first().map(String::as_str).ok_or(USAGE)?;
    if !matches!(command, "assemble" | "run" | "debug" | "check" | "verify") {
        return Err(USAGE.into());
    }
    let required = if command == "assemble" { 3 } else { 2 };
    if args.len() < required || args[1..required].iter().any(|s| s.starts_with("--")) {
        return Err(USAGE.into());
    }
    let mut guest_arguments = vec![args[1].clone()];
    let mut paths = vec![args[1].as_str()];
    let mut system_path = None;
    let mut gc_stats = false;
    let mut gc_events = false;
    let mut options = args[required..].iter();
    while let Some(option) = options.next() {
        match option.as_str() {
            "--" if matches!(command, "run" | "debug") => {
                guest_arguments.extend(options.cloned());
                break;
            }
            "--gc-stats" if command == "run" && !gc_stats => gc_stats = true,
            "--gc-events" if command == "run" && !gc_events => gc_events = true,
            "--module" | "--system" => {
                let path = options
                    .next()
                    .filter(|path| !path.starts_with("--"))
                    .ok_or_else(|| format!("Missing input for {option}\n{USAGE}"))?;
                if option == "--module" {
                    paths.push(path.as_str());
                } else if system_path.replace(path.as_str()).is_some() {
                    return Err("System input specified more than once".into());
                }
            }
            path if command == "run" && !path.starts_with('-') && system_path.is_none() => {
                system_path = Some(path);
            }
            _ => return Err(format!("Unexpected argument {option}\n{USAGE}")),
        }
    }

    if paths[0].ends_with(".neo") && (paths.len() != 1 || system_path.is_some()) {
        return Err(
            "high-level source currently supports only bundled System and one input file".into(),
        );
    }
    let texts = paths
        .iter()
        .map(|path| read(path))
        .collect::<Result<Vec<_>, _>>()?;
    let (modules, program) = if paths.len() == 1 && system_path.is_none() {
        // Preserve standalone System assembly/analysis and existing single-input behavior.
        let module = if paths[0].ends_with(".neo") {
            neoclr::frontend::compile_named(&texts[0], paths[0])
        } else if command == "assemble" || paths[0].ends_with(".neoil") {
            assemble(&texts[0])
        } else {
            load(&texts[0])
        }
        .map_err(|e| e.to_string())?;
        let program = neoclr::LoadedProgram::new(&module).map_err(|e| e.to_string())?;
        (vec![module], program)
    } else {
        let system = if let Some(path) = system_path {
            let text = read(path)?;
            if path.ends_with(".neoil") {
                assemble(&text)
            } else {
                load(&text)
            }
            .map_err(|e| e.to_string())?
        } else {
            neoclr::library::system()
                .map_err(|e| e.to_string())?
                .clone()
        };
        let inputs: Vec<_> = texts
            .iter()
            .enumerate()
            .map(|(index, text)| {
                if (command == "assemble" && index == 0) || paths[index].ends_with(".neoil") {
                    neoclr::assembler::ModuleInput::Source(text)
                } else {
                    neoclr::assembler::ModuleInput::Json(text)
                }
            })
            .collect();
        let modules =
            neoclr::assembler::read_modules(&inputs, &system).map_err(|e| e.to_string())?;
        let program = neoclr::LoadedProgram::with_modules(&modules[0], &system, &modules[1..])
            .map_err(|e| e.to_string())?;
        (modules, program)
    };
    let module = &modules[0];
    match command {
        "debug" => debug_terminal::run(program, paths[0], &texts[0], guest_arguments),
        "assemble" => {
            let output = &args[2];
            let json = serde_json::to_string_pretty(module).map_err(|e| e.to_string())?;
            // Validate everything before creating the one requested artifact; preserve no-overwrite behavior.
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(output)
                .map_err(|e| format!("Cannot create {output}: {e}"))?;
            file.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
            Ok(vec![format!("Assembled {} -> {output}", module.name)])
        }
        "check" => Ok(vec![format!(
            "{}: metadata valid (execution types checked at runtime)",
            module.name
        )]),
        "verify" => {
            let report = program.verify().map_err(|e| e.to_string())?;
            let maximum = report
                .functions
                .iter()
                .map(|f| f.maximum_stack)
                .max()
                .unwrap_or(0);
            Ok(vec![format!(
                "{}: typed-stack/control-flow verification passed ({} IL functions; maximum stack {maximum}; runtime value checks remain)",
                module.name,
                report.functions.len()
            )])
        }
        _ => {
            // SAFETY: CLI run treats the user-selected program and native imports as trusted code.
            let execution = unsafe {
                program.run_with_native(ExecutionOptions {
                    arguments: guest_arguments,
                    console: Some(std::sync::Arc::new(StdioConsole)),
                    ..ExecutionOptions::default()
                })
            }
            .map_err(|e| e.to_string())?;
            if gc_stats {
                let stats = execution.heap.statistics();
                writeln!(
                    io::stderr().lock(),
                    "GC: allocated={} live={} peak={} collections={} reclaimed={}",
                    stats.allocated_objects,
                    stats.live_objects,
                    stats.peak_objects,
                    stats.collections,
                    stats.reclaimed_objects
                )
                .map_err(|error| format!("GC diagnostics output failed: {error}"))?;
            }
            if gc_events {
                for event in execution.heap.collection_events() {
                    writeln!(
                        io::stderr().lock(),
                        "GC #{} {:?}: roots={} before={} after={} reclaimed={}",
                        event.sequence,
                        event.reason,
                        event.roots,
                        event.before,
                        event.after,
                        event.reclaimed
                    )
                    .map_err(|error| format!("GC diagnostics output failed: {error}"))?;
                }
            }
            let mut lines = execution.output;
            lines.push(format!("=> {:?}", execution.value));
            Ok(lines)
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<_> = env::args().skip(1).collect();
    match execute(&args) {
        Ok(lines) => {
            let mut stdout = io::stdout().lock();
            for line in lines {
                if let Err(error) = writeln!(stdout, "{line}") {
                    eprintln!("Output failed: {error}");
                    return ExitCode::from(2);
                }
            }
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
