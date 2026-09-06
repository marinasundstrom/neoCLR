//! Small line-oriented assembler. The serialized output is the prototype metadata format.
use crate::{
    Fault, Module,
    metadata::{Field, Function, FunctionRef, Instruction, Type, TypeDef},
};
use std::collections::HashMap;

struct PendingFunction {
    function: Function,
    labels: HashMap<String, usize>,
    branches: Vec<(usize, String, usize)>,
    inline_parameters: bool,
}

/// Assemble neoIL into a validated module. Errors include source line numbers.
pub fn assemble(source: &str) -> Result<Module, Fault> {
    let module = parse_module(source)?;
    crate::vm::validate(&module)?;
    Ok(module)
}

pub(crate) fn parse_module(source: &str) -> Result<Module, Fault> {
    let mut module = Module {
        format: 2,
        name: String::new(),
        entry: String::new(),
        types: vec![],
        functions: vec![],
    };
    let mut function: Option<PendingFunction> = None;
    let mut typedef: Option<TypeDef> = None;
    for (index, raw) in source.lines().enumerate() {
        let line_number = index + 1;
        // Comments occupy their own lines, keeping quoted strings unambiguous.
        let line = raw.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }
        let (word, rest) = line
            .split_once(char::is_whitespace)
            .map_or((line, ""), |(a, b)| (a, b.trim()));
        let result = (|| -> Result<(), Fault> {
            if word == ".end" {
                if !rest.is_empty() {
                    return Err(Fault::new(".end has no operand"));
                }
                if let Some(mut pending) = function.take() {
                    for (pc, label, source_line) in pending.branches {
                        let target = *pending.labels.get(&label).ok_or_else(|| {
                            Fault::new(format!(
                                "undefined label {label} used on line {source_line}"
                            ))
                        })?;
                        match &mut pending.function.body[pc] {
                            Instruction::Branch(i) | Instruction::BranchTrue(i) => *i = target,
                            _ => return Err(Fault::new("invalid branch fixup")),
                        }
                    }
                    module.functions.push(pending.function);
                } else if let Some(def) = typedef.take() {
                    module.types.push(def);
                } else {
                    return Err(Fault::new("unexpected .end"));
                }
                return Ok(());
            }
            if let Some(def) = typedef.as_mut() {
                if word != ".field" {
                    return Err(Fault::new("expected .field or .end"));
                }
                let (name, ty) = rest
                    .split_once(char::is_whitespace)
                    .ok_or_else(|| Fault::new("expected .field Name Type"))?;
                identifier(name)?;
                def.fields.push(Field {
                    name: name.into(),
                    ty: parse_type(ty)?,
                });
                return Ok(());
            }
            if let Some(pending) = function.as_mut() {
                if word == ".methodimpl" {
                    if rest != "InternalCall" || pending.function.impl_flags != 0 {
                        return Err(Fault::new(
                            "expected one .methodimpl InternalCall directive",
                        ));
                    }
                    if !pending.function.body.is_empty() || !pending.labels.is_empty() {
                        return Err(Fault::new(".methodimpl must precede any body"));
                    }
                    pending.function.impl_flags = crate::metadata::INTERNAL_CALL;
                } else if word == ".param" || word == ".local" {
                    if !pending.function.body.is_empty() || !pending.labels.is_empty() {
                        return Err(Fault::new(
                            "parameters and locals must precede instructions",
                        ));
                    }
                    if word == ".param" && pending.inline_parameters {
                        return Err(Fault::new(
                            "inline parameters cannot be combined with .param",
                        ));
                    }
                    let ty = parse_type(rest)?;
                    if word == ".param" {
                        pending.function.parameters.push(ty);
                    } else {
                        pending.function.locals.push(ty);
                    }
                } else if let Some(label) = line.strip_suffix(':') {
                    identifier(label)?;
                    if pending
                        .labels
                        .insert(label.into(), pending.function.body.len())
                        .is_some()
                    {
                        return Err(Fault::new("duplicate label"));
                    }
                } else {
                    let pc = pending.function.body.len();
                    let argument = match word {
                        "ldc.i4" => Some(serde_json::json!(
                            rest.parse::<i32>()
                                .map_err(|_| Fault::new("expected Int32 literal"))?
                        )),
                        "ldc.bool" => Some(serde_json::json!(
                            rest.parse::<bool>()
                                .map_err(|_| Fault::new("expected true or false"))?
                        )),
                        "ldstr" | "error" | "fault" => Some(serde_json::json!(
                            serde_json::from_str::<String>(rest)
                                .map_err(|_| Fault::new("expected JSON-quoted string"))?
                        )),
                        "ldarg" | "ldloc" | "stloc" | "ldfld" | "stfld" => Some(serde_json::json!(
                            rest.parse::<usize>()
                                .map_err(|_| Fault::new("expected nonnegative index"))?
                        )),
                        "br" | "brtrue" => {
                            identifier(rest)?;
                            pending.branches.push((pc, rest.into(), line_number));
                            Some(serde_json::json!(0))
                        }
                        "call" => Some(
                            serde_json::to_value(parse_function_ref(rest)?)
                                .map_err(|e| Fault::new(e.to_string()))?,
                        ),
                        "newobj" | "is.case" | "ldcase" => {
                            identifier(rest)?;
                            Some(serde_json::json!(rest))
                        }
                        "none" | "ok" | "err" => Some(
                            serde_json::to_value(parse_type(rest)?)
                                .map_err(|e| Fault::new(e.to_string()))?,
                        ),
                        _ => {
                            if !rest.is_empty() {
                                return Err(Fault::new("unexpected instruction operand"));
                            }
                            None
                        }
                    };
                    let mut json = serde_json::json!({"op": word});
                    if let Some(argument) = argument {
                        json["arg"] = argument;
                    }
                    pending.function.body.push(
                        serde_json::from_value(json)
                            .map_err(|e| Fault::new(format!("invalid instruction: {e}")))?,
                    );
                }
                return Ok(());
            }
            match word {
                ".module" => {
                    identifier(rest)?;
                    if !module.name.is_empty() {
                        return Err(Fault::new("duplicate .module"));
                    }
                    module.name = rest.into();
                }
                ".entry" => {
                    identifier(rest)?;
                    if !module.entry.is_empty() {
                        return Err(Fault::new("duplicate .entry"));
                    }
                    module.entry = rest.into();
                }
                ".type" => {
                    identifier(rest)?;
                    typedef = Some(TypeDef {
                        name: rest.into(),
                        fields: vec![],
                    });
                }
                ".function" => {
                    let (name, result) = rest
                        .split_once("->")
                        .ok_or_else(|| Fault::new("expected .function Name -> Type"))?;
                    let inline_parameters = name.contains('(');
                    let target = if inline_parameters {
                        parse_function_ref(name.trim())?
                    } else {
                        identifier(name.trim())?;
                        FunctionRef {
                            name: name.trim().into(),
                            parameters: vec![],
                        }
                    };
                    function = Some(PendingFunction {
                        function: Function {
                            name: target.name,
                            parameters: target.parameters,
                            returns: parse_type(result)?,
                            locals: vec![],
                            body: vec![],
                            impl_flags: 0,
                        },
                        labels: HashMap::new(),
                        branches: vec![],
                        inline_parameters,
                    });
                }
                _ => return Err(Fault::new("expected .module, .entry, .type or .function")),
            }
            Ok(())
        })();
        result.map_err(|error| Fault::new(format!("line {line_number}: {}", error.message)))?;
    }
    if function.is_some() || typedef.is_some() {
        return Err(Fault::new("missing .end"));
    }
    if module.name.is_empty() {
        return Err(Fault::new(".module is required"));
    }
    Ok(module)
}

fn identifier(text: &str) -> Result<(), Fault> {
    if text.is_empty()
        || !text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.'))
    {
        Err(Fault::new(format!("invalid identifier {text:?}")))
    } else {
        Ok(())
    }
}

pub fn parse_type(text: &str) -> Result<Type, Fault> {
    fn parse(text: &str, depth: usize) -> Result<Type, Fault> {
        if depth > 32 {
            return Err(Fault::new("type nesting exceeds 32"));
        }
        let text = text.trim();
        if let Some((name, args)) = text.split_once('<') {
            let args = args
                .strip_suffix('>')
                .ok_or_else(|| Fault::new("unclosed generic type"))?;
            let mut nesting = 0i32;
            let mut start = 0;
            let mut parts = vec![];
            for (index, character) in args.char_indices() {
                match character {
                    '<' => nesting += 1,
                    '>' => nesting -= 1,
                    ',' if nesting == 0 => {
                        parts.push(&args[start..index]);
                        start = index + 1;
                    }
                    _ => (),
                }
                if nesting < 0 {
                    return Err(Fault::new("unbalanced generic type"));
                }
            }
            if nesting != 0 {
                return Err(Fault::new("unbalanced generic type"));
            }
            parts.push(&args[start..]);
            return match (name.trim(), parts.as_slice()) {
                ("Option", [t]) => Ok(Type::Option(Box::new(parse(t, depth + 1)?))),
                ("Ref", [t]) => Ok(Type::Ref(Box::new(parse(t, depth + 1)?))),
                ("Result", [t, e]) => Ok(Type::Result(
                    Box::new(parse(t, depth + 1)?),
                    Box::new(parse(e, depth + 1)?),
                )),
                _ => Err(Fault::new("expected Option<T>, Ref<T> or Result<T,E>")),
            };
        }
        identifier(text)?;
        Ok(match text {
            "Void" | "void" => Type::Void,
            "Int32" | "int32" | "int" => Type::Int32,
            "Boolean" | "boolean" | "bool" => Type::Boolean,
            "String" | "string" => Type::String,
            "Error" => Type::Error,
            _ => Type::Named(text.into()),
        })
    }
    parse(text, 0)
}

/// Parse an explicit call signature, including nested constructed parameter types.
pub fn parse_function_ref(text: &str) -> Result<FunctionRef, Fault> {
    let (name, parameters) = text.trim().split_once('(').ok_or_else(|| {
        Fault::new("call requires an explicit signature: Name(Type, ...) or Name()")
    })?;
    identifier(name.trim())?;
    let parameters = parameters
        .strip_suffix(')')
        .ok_or_else(|| Fault::new("unclosed call signature"))?;
    let mut types = vec![];
    if !parameters.trim().is_empty() {
        let mut nesting = 0i32;
        let mut start = 0;
        for (index, character) in parameters.char_indices() {
            match character {
                '<' => nesting += 1,
                '>' => nesting -= 1,
                ',' if nesting == 0 => {
                    types.push(parse_type(&parameters[start..index])?);
                    start = index + 1;
                }
                _ => (),
            }
            if nesting < 0 {
                return Err(Fault::new("unbalanced call parameter type"));
            }
        }
        if nesting != 0 {
            return Err(Fault::new("unbalanced call parameter type"));
        }
        types.push(parse_type(&parameters[start..])?);
    }
    Ok(FunctionRef {
        name: name.trim().into(),
        parameters: types,
    })
}
