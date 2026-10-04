mod debug_terminal;
use neoclr::{ExecutionOptions, StdioConsole, assemble, load};
use std::{
    env, fs,
    io::{self, Write},
    process::ExitCode,
};

const USAGE: &str = "Usage:
  neoclr emit-il <source.neo> [output.neoil]
  neoclr assemble <source.neoil> <output> [--format json|neox] [--module <input>]... [--system <input>]
  neoclr run <input> [System.neo.json] [--module <input>]... [--system <input>] [--gc-stats] [--gc-events] [--show-result] [--instructions <positive-count>] [-- <guest-argument>...]
  neoclr debug <input> [--module <input>]... [--system <input>] [-- <guest-argument>...]
  neoclr check <input> [--module <input>]... [--system <input>]
  neoclr verify <input> [--module <input>]... [--system <input>]
Inputs: .neo is the high-level subset, .neoil is IL source, PE/#Neo and standalone NEOX containers carry native metadata; otherwise JSON artifacts.";

enum Input {
    Text(String),
    MetadataPe(Vec<u8>),
    NativeEnvelope(Vec<u8>),
}
impl Input {
    fn text(&self) -> Result<&str, String> {
        match self {
            Self::Text(text) => Ok(text),
            Self::NativeEnvelope(_) => Err("expected source text, found native envelope".into()),
            Self::MetadataPe(_) => Err("expected source text, found metadata PE".into()),
        }
    }
    fn load(&self) -> Result<neoclr::Module, neoclr::Fault> {
        match self {
            Self::Text(text) => load(text),
            Self::MetadataPe(image) => neoclr::metadata_container::load(image),
            Self::NativeEnvelope(image) => neoclr::metadata_container::load_envelope(image),
        }
    }
    fn module_input(&self, source: bool) -> Result<neoclr::assembler::ModuleInput<'_>, String> {
        if source {
            return Ok(neoclr::assembler::ModuleInput::Source(self.text()?));
        }
        Ok(match self {
            Self::Text(text) => neoclr::assembler::ModuleInput::Json(text),
            Self::MetadataPe(image) => neoclr::assembler::ModuleInput::MetadataPe(image),
            Self::NativeEnvelope(image) => neoclr::assembler::ModuleInput::NativeEnvelope(image),
        })
    }
}
fn read(path: &str) -> Result<Input, String> {
    if path.ends_with(".neoil") {
        neoclr::source::read_source(path).map(Input::Text)
    } else {
        fs::read(path).and_then(|bytes| {
            if bytes.starts_with(b"MZ") {
                Ok(Input::MetadataPe(bytes))
            } else if bytes.starts_with(b"NEOX") {
                Ok(Input::NativeEnvelope(bytes))
            } else {
                String::from_utf8(bytes)
                    .map(Input::Text)
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
            }
        })
    }
    .map_err(|e| format!("Cannot read {path}: {e}"))
}

fn emit_il(args: &[String]) -> Result<Vec<String>, String> {
    if !(2..=3).contains(&args.len())
        || !args[1].ends_with(".neo")
        || args[1..].iter().any(|arg| arg.starts_with("--"))
    {
        return Err(format!(
            "emit-il requires one Neo source and an optional output path\n{USAGE}"
        ));
    }
    let input = read(&args[1])?;
    let source = input.text()?;
    let il = neoclr::frontend::lower_to_il_named(&source, &args[1]).map_err(|e| e.to_string())?;
    // Match ordinary Neo compilation, but retain the original textual lowering.
    // Validate before writing; emission never runs the guest program.
    let module = assemble(&il).map_err(|e| e.to_string())?;
    neoclr::LoadedProgram::new(&module)
        .and_then(|program| program.verify())
        .map_err(|e| e.to_string())?;
    if let Some(output) = args.get(2) {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output)
            .map_err(|e| format!("Cannot create {output}: {e}"))?;
        file.write_all(il.as_bytes()).map_err(|e| e.to_string())?;
        Ok(vec![format!("Emitted {} -> {output}", args[1])])
    } else {
        // main appends one newline; stdout remains reassemblable IL only.
        Ok(vec![il.strip_suffix('\n').unwrap_or(&il).to_owned()])
    }
}

fn execute(args: &[String], exit_status: &mut i32) -> Result<Vec<String>, String> {
    let command = args.first().map(String::as_str).ok_or(USAGE)?;
    if command == "emit-il" {
        return emit_il(args);
    }
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
    let mut show_result = false;
    let mut output_format = None;
    let mut instruction_limit = None;
    let mut options = args[required..].iter();
    while let Some(option) = options.next() {
        match option.as_str() {
            "--" if matches!(command, "run" | "debug") => {
                guest_arguments.extend(options.cloned());
                break;
            }
            "--format" if command == "assemble" && output_format.is_none() => {
                let format = options
                    .next()
                    .map(String::as_str)
                    .filter(|value| matches!(*value, "json" | "neox"))
                    .ok_or_else(|| format!("Expected --format json or neox\n{USAGE}"))?;
                output_format = Some(format);
            }
            "--instructions" if command == "run" && instruction_limit.is_none() => {
                let value = options.next().ok_or("Missing count for --instructions")?;
                instruction_limit = Some(
                    value
                        .bytes()
                        .all(|byte| byte.is_ascii_digit())
                        .then(|| value.parse::<usize>().ok())
                        .flatten()
                        .filter(|count| *count > 0)
                        .ok_or("Expected a positive integer count for --instructions")?,
                );
            }
            "--show-result" if command == "run" && !show_result => show_result = true,
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
            neoclr::frontend::compile_named(texts[0].text()?, paths[0])
        } else if command == "assemble" || paths[0].ends_with(".neoil") {
            assemble(texts[0].text()?)
        } else {
            texts[0].load()
        }
        .map_err(|e| e.to_string())?;
        let program = neoclr::LoadedProgram::new(&module).map_err(|e| e.to_string())?;
        (vec![module], program)
    } else {
        let system = if let Some(path) = system_path {
            let text = read(path)?;
            if path.ends_with(".neoil") {
                assemble(text.text()?)
            } else {
                text.load()
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
            .map(|(index, input)| {
                input.module_input(
                    (command == "assemble" && index == 0) || paths[index].ends_with(".neoil"),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let modules =
            neoclr::assembler::read_modules(&inputs, &system).map_err(|e| e.to_string())?;
        let program = neoclr::LoadedProgram::with_modules(&modules[0], &system, &modules[1..])
            .map_err(|e| e.to_string())?;
        (modules, program)
    };
    let module = &modules[0];
    match command {
        "debug" => debug_terminal::run(
            program,
            paths[0],
            texts[0].text().unwrap_or(""),
            guest_arguments,
        ),
        "assemble" => {
            let output = &args[2];
            let image = if output_format == Some("neox") {
                program.verify().map_err(|e| e.to_string())?;
                neoclr::metadata_container::write_module(module).map_err(|e| e.to_string())?
            } else {
                serde_json::to_vec_pretty(module).map_err(|e| e.to_string())?
            };
            // Validate everything before creating the one requested artifact; preserve no-overwrite behavior.
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(output)
                .map_err(|e| format!("Cannot create {output}: {e}"))?;
            file.write_all(&image).map_err(|e| e.to_string())?;
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
                    limits: neoclr::Limits {
                        instructions: instruction_limit
                            .unwrap_or(neoclr::Limits::default().instructions),
                        ..neoclr::Limits::default()
                    },
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
            if show_result {
                writeln!(io::stderr().lock(), "=> {:?}", execution.value)
                    .map_err(|error| format!("Result diagnostics output failed: {error}"))?;
            }
            if let neoclr::Value::Int32(value) = execution.value {
                *exit_status = value;
            }
            Ok(execution.output)
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<_> = env::args().skip(1).collect();
    let mut exit_status = 0;
    match execute(&args, &mut exit_status) {
        Ok(lines) => {
            let mut stdout = io::stdout().lock();
            for line in lines {
                if let Err(error) = writeln!(stdout, "{line}") {
                    eprintln!("Output failed: {error}");
                    return ExitCode::from(2);
                }
            }
            drop(stdout);
            std::process::exit(exit_status)
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
