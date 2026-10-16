//! Provisional dynamic invocation for the selected closed function inventory.
use neoclr::metadata::Instruction as Op;
use serde_json::{Value, json};

pub fn bind(
    input: &mut neoclr::Module,
    report: &mut Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let shapes: Vec<_> = input
        .functions
        .iter()
        .flat_map(|f| &f.body)
        .filter_map(|op| {
            if let Op::BindFunction { function_type, .. } = op {
                Some(function_type.clone())
            } else {
                None
            }
        })
        .collect();
    for index in 0..input.functions.len() {
        if input.functions[index].name != "neoCLR.Runtime.InvokeFunction" {
            continue;
        }
        input.functions[index] =
            neoclr::function_invocation::lower(input, &input.functions[index], &shapes)
                .map_err(|error| error.to_string())?;
        report["nativeBindings"].as_array_mut().ok_or("missing native binding inventory")?.push(json!({
            "compiledIndex": index, "name": "neoCLR.Runtime.InvokeFunction", "implementation": "dynamic-function-closed-v1",
            "policy": "selected closed function bindings; checked boxed arguments and typed invocation; no runtime code generation"
        }));
    }
    Ok(())
}
