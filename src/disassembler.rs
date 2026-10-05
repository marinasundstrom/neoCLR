//! Read-only native metadata inspection. No load context or runtime is constructed.
use std::{fs, io::Write};

pub(crate) fn execute(args: &[String]) -> Result<Vec<String>, String> {
    if !(2..=3).contains(&args.len()) || args[1..].iter().any(|arg| arg.starts_with("--")) {
        return Err(format!(
            "disassemble requires one metadata input and an optional output path\n{}",
            crate::USAGE
        ));
    }
    let bytes = fs::read(&args[1]).map_err(|error| format!("Cannot read {}: {error}", args[1]))?;
    let module: neoclr::Module = if bytes.starts_with(b"MZ") {
        neoclr::metadata_container::decode(&bytes).map_err(|error| error.to_string())?
    } else if bytes.starts_with(b"NEOX") {
        neoclr::metadata_container::decode_envelope(&bytes).map_err(|error| error.to_string())?
    } else {
        serde_json::from_slice(&bytes)
            .map_err(|error| format!("Invalid native metadata JSON: {error}"))?
    };
    if module.format != 5 {
        return Err("unsupported module format (expected 5); reassemble source".into());
    }
    let listing = render(&module)?;
    if let Some(output) = args.get(2) {
        // Decode and render before publication; never truncate an existing artifact.
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output)
            .map_err(|error| format!("Cannot create {output}: {error}"))?;
        file.write_all(listing.as_bytes())
            .map_err(|error| error.to_string())?;
        Ok(vec![format!("Disassembled {} -> {output}", args[1])])
    } else {
        Ok(vec![listing.trim_end_matches('\n').to_owned()])
    }
}

fn render(module: &neoclr::Module) -> Result<String, String> {
    // Use the canonical serialized model for declaration facts and operands. This
    // preserves new metadata categories without maintaining a second opcode schema.
    let mut metadata = serde_json::to_value(module).map_err(|error| error.to_string())?;
    let functions = metadata
        .as_object_mut()
        .unwrap()
        .remove("functions")
        .unwrap();
    let mut text = String::from(
        "// NeoCLR native metadata disassembly v1\n// Diagnostic listing, not reassemblable neoIL. Instruction labels are decimal indices, not byte offsets.\n// Dependencies are unresolved; method bodies are not verified.\n.module ",
    );
    text.push_str(&serde_json::to_string_pretty(&metadata).map_err(|error| error.to_string())?);
    text.push('\n');
    for (index, function) in functions.as_array().unwrap().iter().enumerate() {
        let mut declaration = function.clone();
        let body = declaration.as_object_mut().unwrap().remove("body").unwrap();
        text.push_str(&format!("\n.method #{index} "));
        text.push_str(
            &serde_json::to_string_pretty(&declaration).map_err(|error| error.to_string())?,
        );
        text.push_str("\n.code\n");
        for (offset, instruction) in body.as_array().unwrap().iter().enumerate() {
            text.push_str(&format!(
                "  I_{offset:04}: {}",
                instruction["op"].as_str().unwrap()
            ));
            if let Some(operand) = instruction.get("arg") {
                text.push(' ');
                text.push_str(&serde_json::to_string(operand).map_err(|error| error.to_string())?);
            }
            text.push('\n');
        }
        text.push_str(".end\n");
    }
    Ok(text)
}
