//! Explicit native service implementations after original-scope verification and
//! closed-world selection. Ordinary managed wrappers are never replaced.
use neoclr::metadata::{Instruction as Op, Type};
use serde_json::{Value, json};
type Error = Box<dyn std::error::Error>;

pub fn user_fault(input: &mut neoclr::Module, selection: &Value) -> Result<Value, Error> {
    let mut bindings = vec![];
    for row in selection["functions"]
        .as_array()
        .ok_or("missing selection inventory")?
    {
        let legacy =
            row["name"] == "neoCLR.Runtime.Fault" && row["definition"]["module"] == "System";
        let source_owned = row["name"] == "neoCLR.Runtime.Fail";
        if !legacy && !source_owned {
            continue;
        }
        let f = &mut input.functions[row["compiledIndex"]
            .as_u64()
            .ok_or("missing compiled index")? as usize];
        if f.name != row["name"].as_str().unwrap()
            || f.owner.is_some()
            || f.instance
            || f.receiver_byref
            || f.receiver_readonly
            || f.parameters != [Type::String]
            || f.returns != Type::Void
            || f.no_result != source_owned
            || f.impl_flags != 0x1000
            || f.pinvoke.is_some()
            || !f.body.is_empty()
            || !f.locals.is_empty()
            || f.is_virtual
            || f.is_override
            || f.is_abstract
            || !f.generic_parameters.is_empty()
            || !f.generic_arguments.is_empty()
            || !f.generic_constraints.is_empty()
            || !f.interface_implementations.is_empty()
            || !f.out_parameters.is_empty()
            || !f.out_when_true.is_empty()
            || !f.readonly_parameters.is_empty()
        {
            return Err("native UserFault binding requires exact neoCLR.Runtime.Fault(String) -> Void or neoCLR.Runtime.Fail(String) -> noresult InternalCall contract".into());
        }
        // The supplied String remains a normal argument. Diagnostic lowering uses
        // the explicit binding index to capture that message at the terminal site.
        f.impl_flags = 0;
        f.body = vec![Op::Fault("native UserFault service".into())];
        bindings.push(json!({"definition": row["definition"], "name": row["name"],
            "compiledIndex": row["compiledIndex"], "implementation": "terminal-user-fault-v1",
            "status": 4, "diagnostics": "UTF-8 message and managed frames; caller-owned ABI v3"}));
    }
    Ok(json!(bindings))
}

/// Bind only the verified reserved service, never its public managed wrapper.
pub fn console_read_byte(input: &mut neoclr::Module, selection: &Value) -> Result<Vec<Value>, Error> {
    let mut bindings = vec![];
    for row in selection["functions"].as_array().ok_or("missing selection inventory")? {
        if row["name"] != "neoCLR.Runtime.ConsoleReadByte" { continue; }
        let f = &mut input.functions[row["compiledIndex"].as_u64().ok_or("missing compiled index")? as usize];
        if f.name != "neoCLR.Runtime.ConsoleReadByte"
            || f.owner.is_some() || f.instance || f.receiver_byref || f.receiver_readonly
            || !f.parameters.is_empty() || f.returns != Type::Value || f.no_result
            || f.impl_flags != 0x1000 || f.pinvoke.is_some() || !f.body.is_empty()
            || !f.locals.is_empty() || f.is_virtual || f.is_override || f.is_abstract
            || !f.generic_parameters.is_empty() || !f.generic_arguments.is_empty()
            || !f.generic_constraints.is_empty() || !f.interface_implementations.is_empty()
            || !f.out_parameters.is_empty() || !f.out_when_true.is_empty() || !f.readonly_parameters.is_empty()
        { return Err("native byte input binding requires exact neoCLR.Runtime.ConsoleReadByte() -> System.Value InternalCall contract".into()); }
        // A verifiable private placeholder. Only this reported definition is lowered
        // to the native service; no opcode or guest name is an intrinsic by itself.
        f.impl_flags = 0;
        f.body = vec![Op::Void, Op::PackValue(Type::Void), Op::Return];
        bindings.push(json!({"definition": row["definition"], "name": row["name"],
            "compiledIndex": row["compiledIndex"], "implementation": "console-read-byte-v1",
            "symbol": "neoclr_console_read_byte_v1"}));
    }
    Ok(bindings)
}
