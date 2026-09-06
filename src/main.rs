use neoclr::{Fault, Limits, assemble, load, run};
use std::{
    env, fs,
    io::{self, Write},
    process::ExitCode,
};

fn execute(args: &[String]) -> Result<Vec<String>, String> {
    match args {
        [command, input, output] if command == "assemble" => {
            let source = fs::read_to_string(input).map_err(|e| format!("Cannot read {input}: {e}"))?;
            let module = assemble(&source).map_err(|e| e.to_string())?;
            let json = serde_json::to_string_pretty(&module).map_err(|e| e.to_string())?;
            // Refuse to silently overwrite source or existing artifacts.
            let mut file = fs::OpenOptions::new().write(true).create_new(true).open(output).map_err(|e| format!("Cannot create {output}: {e}"))?;
            file.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
            Ok(vec![format!("Assembled {} -> {output}", module.name)])
        }
        [command, input] if command == "run" || command == "check" => {
            let source = fs::read_to_string(input).map_err(|e| format!("Cannot read {input}: {e}"))?;
            let module = if input.ends_with(".neoil") { assemble(&source) } else { load(&source) }.map_err(|e| e.to_string())?;
            if command == "check" { return Ok(vec![format!("{}: metadata valid (execution types checked at runtime)", module.name)]); }
            run(&module, Limits::default()).map(|execution| {
                let mut lines = execution.output;
                lines.push(format!("=> {:?}", execution.value));
                lines
            }).map_err(|fault: Fault| fault.to_string())
        }
        _ => Err("Usage:\n  neoclr assemble <source.neoil> <output.neo.json>\n  neoclr run <source.neoil|module.neo.json>\n  neoclr check <source.neoil|module.neo.json>".into()),
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
