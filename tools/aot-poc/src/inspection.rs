//! Read-only build diagnostics, not native interface metadata or an ABI contract.
use neoclr::metadata::Instruction as Op;
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub fn report(input: &neoclr::Module, root: &str, closed: bool) -> Value {
    let mut histogram = BTreeMap::<String, usize>::new();
    let functions: Vec<_> = input
        .functions
        .iter()
        .enumerate()
        .map(|(index, f)| {
            let mut calls = vec![];
            for (instruction, op) in f.body.iter().enumerate() {
                let encoded = serde_json::to_value(op).expect("serializable instruction");
                let name = encoded["op"]
                    .as_str()
                    .expect("tagged instruction")
                    .to_owned();
                *histogram.entry(name.clone()).or_default() += 1;
                if let Op::Call(target) | Op::Construct(target) | Op::CallVirtual(target) = op {
                    // Inventory exact references. Never guess binding from a source name,
                    // or silently turn unresolved calls into omitted native code.
                    calls.push(
                        json!({"instruction": instruction, "operation": name, "target": target}),
                    );
                }
            }
            json!({"index": index, "name": f.name, "definition": f.definition,
            "owner": f.owner, "instance": f.instance, "virtual": f.is_virtual,
            "parameters": f.parameters, "returns": f.returns, "noResult": f.no_result,
            "outParameters": f.out_parameters, "calls": calls})
        })
        .collect();
    // Use actual compiler admission rather than maintaining a second capability list.
    // Object bytes are discarded and no linker or emitted code is executed.
    let prepared = if closed {
        super::selection::prepare(input, root).map(Some)
    } else {
        Ok(None)
    };
    let (admission, selection) = match prepared {
        Err(e) => (
            json!({"accepted": false, "phase": "selection", "firstError": e.to_string()}),
            Value::Null,
        ),
        Ok(prepared) => {
            let module = prepared.as_ref().map_or(input, |(module, _)| module);
            let admission = match super::compiler::compile(module, root, false) {
                Ok(_) => json!({"accepted": true}),
                Err(e) => {
                    json!({"accepted": false, "phase": "compilation", "firstError": e.to_string()})
                }
            };
            (
                admission,
                prepared.map_or(Value::Null, |(_, report)| report),
            )
        }
    };
    json!({"schema": "neoclr-aot-inspection-v1", "module": input.name, "root": root,
        "scope": "inventory includes all declarations; admission uses admissionMode",
        "admissionMode": if closed { "closed-world" } else { "whole-module" },
        "selection": selection, "capabilities": {"console": false}, "admission": admission,
        "assemblies": input.assemblies, "types": input.types, "functions": functions, "opcodes": histogram,
        "notice": "Build diagnostics only. Not a deployable metadata sidecar, stable ABI or export manifest."})
}
