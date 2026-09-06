//! Small line-oriented assembler. The serialized output is the prototype metadata format.
use crate::{
    Fault, Module,
    metadata::{Field, Function, FunctionRef, Instruction, Representation, Type, TypeDef},
};
use std::collections::HashMap;

struct PendingFunction {
    function: Function,
    labels: HashMap<String, usize>,
    branches: Vec<(usize, Option<usize>, String, usize)>,
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
        format: 3,
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
                    for (pc, slot, label, source_line) in pending.branches {
                        let target = *pending.labels.get(&label).ok_or_else(|| {
                            Fault::new(format!(
                                "undefined label {label} used on line {source_line}"
                            ))
                        })?;
                        match &mut pending.function.body[pc] {
                            Instruction::Branch(i)
                            | Instruction::BranchTrue(i)
                            | Instruction::BranchFalse(i) => *i = target,
                            Instruction::Switch(targets) => {
                                targets[slot.ok_or_else(|| Fault::new("invalid switch fixup"))?] =
                                    target
                            }
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
            if let (true, Some(def)) = (function.is_none() && word != ".method", typedef.as_mut()) {
                if word != ".field" {
                    return Err(Fault::new("expected .field, .method or .end"));
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
                if word == ".pinvoke" {
                    if pending.function.pinvoke.is_some()
                        || !pending.function.body.is_empty()
                        || !pending.labels.is_empty()
                    {
                        return Err(Fault::new(
                            "expected one .pinvoke directive before any body",
                        ));
                    }
                    let mut strings =
                        serde_json::Deserializer::from_str(rest).into_iter::<String>();
                    let library = strings
                        .next()
                        .ok_or_else(|| Fault::new("expected quoted native library"))?
                        .map_err(|_| Fault::new("expected quoted native library"))?;
                    let entry_point = strings
                        .next()
                        .ok_or_else(|| Fault::new("expected quoted native entry point"))?
                        .map_err(|_| Fault::new("expected quoted native entry point"))?;
                    if rest[strings.byte_offset()..].trim() != "cdecl" {
                        return Err(Fault::new("expected cdecl calling convention"));
                    }
                    pending.function.pinvoke = Some(crate::metadata::NativeImport {
                        library,
                        entry_point,
                        calling_convention: crate::metadata::CallingConvention::Cdecl,
                    });
                } else if word == ".methodimpl" {
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
                    let (name, ty) = parse_slot(rest)?;
                    if word == ".param" {
                        pending.function.parameters.push(ty);
                        pending.function.parameter_names.push(name);
                    } else {
                        pending.function.locals.push(ty);
                        pending.function.local_names.push(name);
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
                        "ldc.i8" => Some(serde_json::json!(
                            rest.parse::<i64>()
                                .map_err(|_| Fault::new("expected Int64 literal"))?
                        )),
                        "ldc.r4" => Some(serde_json::json!({"bits":
                            rest.parse::<f32>().map_err(|_| Fault::new("expected Float32 literal"))?.to_bits()
                        })),
                        "ldc.r8" => Some(serde_json::json!({"bits":
                            rest.parse::<f64>().map_err(|_| Fault::new("expected Float64 literal"))?.to_bits()
                        })),
                        "ldc.bool" => Some(serde_json::json!(
                            rest.parse::<bool>()
                                .map_err(|_| Fault::new("expected true or false"))?
                        )),
                        "ldstr" | "error" | "fault" => Some(serde_json::json!(
                            serde_json::from_str::<String>(rest)
                                .map_err(|_| Fault::new("expected JSON-quoted string"))?
                        )),
                        "ldarg" | "ldloc" | "stloc" => Some(serde_json::json!(resolve_slot(
                            &pending.function,
                            word,
                            rest
                        )?)),
                        "ldfld" | "stfld" | "ldflda" => Some(serde_json::json!(
                            rest.parse::<usize>()
                                .map_err(|_| Fault::new("expected nonnegative index"))?
                        )),
                        "br" | "brtrue" | "brfalse" => {
                            identifier(rest)?;
                            pending.branches.push((pc, None, rest.into(), line_number));
                            Some(serde_json::json!(0))
                        }
                        "switch" => {
                            let labels = rest
                                .strip_prefix('(')
                                .and_then(|s| s.strip_suffix(')'))
                                .ok_or_else(|| {
                                Fault::new("expected switch (Label, Label, ...)")
                            })?;
                            let mut targets = vec![];
                            if !labels.trim().is_empty() {
                                for label in labels.split(',') {
                                    let label = label.trim();
                                    identifier(label)?;
                                    pending.branches.push((
                                        pc,
                                        Some(targets.len()),
                                        label.into(),
                                        line_number,
                                    ));
                                    targets.push(0usize);
                                }
                            }
                            Some(serde_json::json!(targets))
                        }
                        "call" => Some(
                            serde_json::to_value(parse_function_ref(rest)?)
                                .map_err(|e| Fault::new(e.to_string()))?,
                        ),
                        "newobj" | "is.case" | "ldcase" => {
                            identifier(rest)?;
                            Some(serde_json::json!(rest))
                        }
                        "none" | "ok" | "err" | "sizeof" | "alignof" | "heap.alloc"
                        | "ptr.null" | "ptr.cast" | "ptr.fromint" | "ldobj" | "stobj" | "cpobj"
                        | "initobj" => Some(
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
                    let ty = Type::from_name(rest);
                    typedef = Some(TypeDef {
                        name: ty.definition_name().unwrap_or(rest).into(),
                        fields: vec![],
                        representation: if ty.is_primitive() {
                            Representation::Runtime
                        } else {
                            Representation::Record
                        },
                    });
                }
                ".function" | ".method" => {
                    let (owner, instance, declaration) = if word == ".method" {
                        let def = typedef
                            .as_ref()
                            .ok_or_else(|| Fault::new(".method requires an enclosing .type"))?;
                        let (kind, signature) =
                            rest.split_once(char::is_whitespace).ok_or_else(|| {
                                Fault::new("expected .method static/instance Name(...) -> Type")
                            })?;
                        if kind != "static" && kind != "instance" {
                            return Err(Fault::new("expected static or instance method"));
                        }
                        (
                            Some(Type::from_name(&def.name)),
                            kind == "instance",
                            signature.trim(),
                        )
                    } else {
                        (None, false, rest)
                    };
                    let (name, result) = declaration
                        .split_once("->")
                        .ok_or_else(|| Fault::new("expected .function Name -> Type"))?;
                    let inline_parameters = name.contains('(');
                    let (target, parameter_names) = if inline_parameters {
                        parse_callable(name.trim(), true)?
                    } else {
                        identifier(name.trim())?;
                        (
                            FunctionRef {
                                name: name.trim().into(),
                                parameters: vec![],
                                owner: None,
                                instance: false,
                            },
                            vec![],
                        )
                    };
                    if target.owner.is_some() || target.instance {
                        return Err(Fault::new(
                            "declaration names must not contain an owner or instance prefix",
                        ));
                    }
                    if owner.is_some() && target.name.contains('.') {
                        return Err(Fault::new(
                            "method name must be unqualified within its type",
                        ));
                    }
                    let name = match &owner {
                        Some(ty) => format!(
                            "{}.{}",
                            ty.definition_name()
                                .ok_or_else(|| Fault::new("invalid method owner"))?,
                            target.name
                        ),
                        None => target.name,
                    };
                    function = Some(PendingFunction {
                        function: Function {
                            name,
                            owner,
                            instance,
                            parameters: target.parameters,
                            parameter_names,
                            returns: parse_type(result)?,
                            locals: vec![],
                            local_names: vec![],
                            body: vec![],
                            impl_flags: 0,
                            pinvoke: None,
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
        if let Some(element) = text.strip_suffix('*') {
            return Ok(Type::Ptr(Box::new(parse(element, depth + 1)?)));
        }
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
                ("Ptr", [t]) => Ok(Type::Ptr(Box::new(parse(t, depth + 1)?))),
                ("Result", [t, e]) => Ok(Type::Result(
                    Box::new(parse(t, depth + 1)?),
                    Box::new(parse(e, depth + 1)?),
                )),
                _ => Err(Fault::new(
                    "expected Option<T>, Ref<T>, Ptr<T> or Result<T,E>",
                )),
            };
        }
        identifier(text)?;
        Ok(Type::from_name(text))
    }
    parse(text, 0)
}

/// Parse an explicit call signature, including nested constructed parameter types.
pub fn parse_function_ref(text: &str) -> Result<FunctionRef, Fault> {
    parse_callable(text, false).map(|(target, _)| target)
}

fn parse_callable(text: &str, named: bool) -> Result<(FunctionRef, Vec<Option<String>>), Fault> {
    let (name, parameters) = text.trim().split_once('(').ok_or_else(|| {
        Fault::new("call requires an explicit signature: Name(Type, ...) or Name()")
    })?;
    let (instance, name) = match name.trim().strip_prefix("instance ") {
        Some(name) => (true, name.trim()),
        None => (false, name.trim()),
    };
    let (owner, name) = if let Some((owner, member)) = name.split_once("::") {
        identifier(member.trim())?;
        if member.trim().contains('.') {
            return Err(Fault::new("expected an unqualified member name"));
        }
        let owner = parse_type(owner)?;
        let full_name = format!(
            "{}.{}",
            owner
                .definition_name()
                .ok_or_else(|| Fault::new("constructed method owners are not supported yet"))?,
            member.trim()
        );
        (Some(owner), full_name)
    } else {
        identifier(name)?;
        (None, name.to_owned())
    };
    let parameters = parameters
        .strip_suffix(')')
        .ok_or_else(|| Fault::new("unclosed call signature"))?;
    let mut types = vec![];
    let mut names = vec![];
    let mut parameter = |text: &str| -> Result<(), Fault> {
        let (name, ty) = if named {
            parse_slot(text)?
        } else {
            (None, parse_type(text)?)
        };
        names.push(name);
        types.push(ty);
        Ok(())
    };
    if !parameters.trim().is_empty() {
        let mut nesting = 0i32;
        let mut start = 0;
        for (index, character) in parameters.char_indices() {
            match character {
                '<' => nesting += 1,
                '>' => nesting -= 1,
                ',' if nesting == 0 => {
                    parameter(&parameters[start..index])?;
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
        parameter(&parameters[start..])?;
    }
    Ok((
        FunctionRef {
            name,
            owner,
            instance,
            parameters: types,
        },
        names,
    ))
}

fn parse_slot(text: &str) -> Result<(Option<String>, Type), Fault> {
    match text.split_once(':') {
        Some((name, ty)) => {
            let name = name.trim();
            if !crate::metadata::valid_slot_name(name) {
                return Err(Fault::new("invalid parameter/local name"));
            }
            Ok((Some(name.into()), parse_type(ty)?))
        }
        None => Ok((None, parse_type(text)?)),
    }
}

fn resolve_slot(function: &Function, op: &str, operand: &str) -> Result<usize, Fault> {
    if let Ok(index) = operand.parse::<usize>() {
        return Ok(index);
    }
    if op == "ldarg" && function.instance && operand == "this" {
        return Ok(0);
    }
    let (names, offset) = if op == "ldarg" {
        (&function.parameter_names, usize::from(function.instance))
    } else {
        (&function.local_names, 0)
    };
    names
        .iter()
        .position(|name| name.as_deref() == Some(operand))
        .map(|index| index + offset)
        .ok_or_else(|| {
            Fault::new(format!(
                "unknown {} name {operand:?}",
                if op == "ldarg" { "parameter" } else { "local" }
            ))
        })
}
