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

/// Assemble sources together with bundled System. The first source is the root;
/// remaining sources are library modules. Returns source artifacts in input order.
pub fn assemble_modules(sources: &[&str]) -> Result<Vec<Module>, Fault> {
    let inputs: Vec<_> = sources
        .iter()
        .map(|source| ModuleInput::Source(source))
        .collect();
    read_modules(&inputs, crate::library::system()?)
}

/// An explicitly classified source or serialized metadata artifact.
pub enum ModuleInput<'a> {
    Source(&'a str),
    Json(&'a str),
}

/// Parse and validate a mixed module set against a supplied System artifact.
/// First input is the root; remaining inputs are dependencies. Returned source
/// artifacts retain their identities and scopes, including absent legacy rows.
pub fn read_modules(inputs: &[ModuleInput<'_>], library: &Module) -> Result<Vec<Module>, Fault> {
    if inputs.is_empty() {
        return Err(Fault::new("expected at least one module input"));
    }
    let parsed = inputs
        .iter()
        .map(|input| match input {
            ModuleInput::Source(source) => parse_parts(source),
            ModuleInput::Json(json) => crate::decode_module(json).map(|module| (module, vec![])),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut context = library.clone();
    context.normalize_definition_ids()?;
    for (module, _) in &parsed {
        let mut normalized = module.clone();
        normalized.normalize_definition_ids()?;
        context.types.extend(normalized.types);
    }
    let mut modules = Vec::new();
    for (mut module, fixups) in parsed {
        resolve_fields(&mut module, &context, fixups)?;
        modules.push(module);
    }
    crate::library::link_modules(&modules[0], library, &modules[1..])?;
    Ok(modules)
}

type FieldFixup = (usize, usize, Type, String, usize);

pub(crate) fn parse_module(source: &str) -> Result<Module, Fault> {
    let (mut module, fixups) = parse_parts(source)?;
    let context = module.clone();
    resolve_fields(&mut module, &context, fixups)?;
    Ok(module)
}

fn parse_parts(source: &str) -> Result<(Module, Vec<FieldFixup>), Fault> {
    let mut module = Module {
        format: 4,
        name: String::new(),
        revision: None,
        references: None,
        entry: String::new(),
        types: vec![],
        functions: vec![],
    };
    let mut field_fixups: Vec<FieldFixup> = vec![];
    let mut function: Option<PendingFunction> = None;
    let mut typedef: Option<TypeDef> = None;
    let mut enclosing_types = Vec::new();
    let mut nesting_fixups = Vec::new();
    let mut property: Option<crate::metadata::Property> = None;
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
            if let Some(pending) = property.as_mut() {
                if word == ".end" {
                    if !rest.is_empty() {
                        return Err(Fault::new("unexpected property .end operand"));
                    }
                    typedef
                        .as_mut()
                        .ok_or_else(|| Fault::new("property requires type"))?
                        .properties
                        .push(property.take().unwrap());
                } else {
                    let slot = match word {
                        ".get" => &mut pending.getter,
                        ".set" => &mut pending.setter,
                        _ => return Err(Fault::new("expected .get, .set or .end in property")),
                    };
                    if slot.is_some() {
                        return Err(Fault::new("duplicate property accessor"));
                    }
                    *slot = Some(parse_function_ref(rest)?);
                    let def = typedef
                        .as_ref()
                        .ok_or_else(|| Fault::new("property requires type"))?;
                    pending.map_types(|ty| {
                        Ok(bind_type_parameters(ty.clone(), &def.generic_parameters))
                    })?;
                }
                return Ok(());
            }
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
                            | Instruction::BranchFalse(i)
                            | Instruction::BranchEqual(i)
                            | Instruction::BranchNotEqual(i)
                            | Instruction::BranchGreater(i)
                            | Instruction::BranchGreaterUnsigned(i)
                            | Instruction::BranchLess(i)
                            | Instruction::BranchLessUnsigned(i)
                            | Instruction::BranchGreaterEqual(i)
                            | Instruction::BranchGreaterEqualUnsigned(i)
                            | Instruction::BranchLessEqual(i)
                            | Instruction::BranchLessEqualUnsigned(i) => *i = target,
                            Instruction::Switch(targets) => {
                                targets[slot.ok_or_else(|| Fault::new("invalid switch fixup"))?] =
                                    target
                            }
                            _ => return Err(Fault::new("invalid branch fixup")),
                        }
                    }
                    if let Some(def) = &typedef {
                        let owner = pending.function.owner.clone();
                        pending.function = pending.function.map_types(|ty| {
                            Ok(bind_type_parameters(ty.clone(), &def.generic_parameters))
                        })?;
                        pending.function.owner = owner;
                    }
                    module.functions.push(pending.function);
                } else if let Some(def) = typedef.take() {
                    module.types.push(def);
                    typedef = enclosing_types.pop();
                } else {
                    return Err(Fault::new("unexpected .end"));
                }
                return Ok(());
            }
            if let (true, Some(def)) = (
                function.is_none() && word != ".method" && word != ".type" && word != ".interface",
                typedef.as_mut(),
            ) {
                if word == ".property" {
                    let (kind, signature) =
                        rest.split_once(char::is_whitespace).ok_or_else(|| {
                            Fault::new("expected .property static/instance Name(...) -> Type")
                        })?;
                    if !matches!(kind, "static" | "instance") {
                        return Err(Fault::new("expected static or instance property"));
                    }
                    let (name, ty) = signature
                        .split_once("->")
                        .ok_or_else(|| Fault::new("expected property signature and return type"))?;
                    let target = parse_function_ref(name.trim())?;
                    if target.owner.is_some()
                        || target.instance
                        || target.definition.is_some()
                        || target.name.contains('.')
                    {
                        return Err(Fault::new("property name must be unqualified"));
                    }
                    let mut pending = crate::metadata::Property {
                        name: target.name,
                        instance: kind == "instance",
                        parameters: target.parameters,
                        ty: parse_type(ty)?,
                        getter: None,
                        setter: None,
                    };
                    pending.map_types(|ty| {
                        Ok(bind_type_parameters(ty.clone(), &def.generic_parameters))
                    })?;
                    property = Some(pending);
                    return Ok(());
                }
                if word == ".custom" {
                    def.custom_attributes
                        .push(crate::metadata::CustomAttribute {
                            constructor: parse_function_ref(rest)?,
                        });
                    return Ok(());
                }
                if word == ".pack" {
                    if def.packing.is_some() {
                        return Err(Fault::new("duplicate .pack"));
                    }
                    def.packing = Some(
                        rest.parse()
                            .map_err(|_| Fault::new("expected unsigned packing size"))?,
                    );
                    return Ok(());
                }
                if word == ".size" {
                    if def.minimum_size.is_some() {
                        return Err(Fault::new("duplicate .size"));
                    }
                    def.minimum_size = Some(
                        rest.parse()
                            .map_err(|_| Fault::new("expected unsigned record size"))?,
                    );
                    return Ok(());
                }
                if word == ".implements" {
                    def.implements.push(bind_type_parameters(
                        parse_type(rest)?,
                        &def.generic_parameters,
                    ));
                    return Ok(());
                }
                if word != ".field" {
                    return Err(Fault::new(
                        "expected .type, .interface, .implements, .field, .property, .pack, .size, .custom, .method or .end",
                    ));
                }
                let (name, ty) = rest
                    .split_once(char::is_whitespace)
                    .ok_or_else(|| Fault::new("expected .field Name Type"))?;
                let (visibility, name, ty) = match name {
                    "public" | "internal" | "private" => {
                        let visibility = match name {
                            "internal" => crate::metadata::Visibility::Internal,
                            "private" => crate::metadata::Visibility::Private,
                            _ => crate::metadata::Visibility::Public,
                        };
                        let (name, ty) = ty
                            .trim()
                            .split_once(char::is_whitespace)
                            .ok_or_else(|| Fault::new("expected .field [visibility] Name Type"))?;
                        (visibility, name, ty)
                    }
                    _ => (crate::metadata::Visibility::Public, name, ty),
                };
                identifier(name)?;
                def.fields.push(Field {
                    visibility,
                    name: name.into(),
                    ty: bind_type_parameters(parse_type(ty)?, &def.generic_parameters),
                });
                return Ok(());
            }
            if let Some(pending) = function.as_mut() {
                if word == ".custom" {
                    if !pending.function.body.is_empty() || !pending.labels.is_empty() {
                        return Err(Fault::new(".custom must precede instructions and labels"));
                    }
                    pending
                        .function
                        .custom_attributes
                        .push(crate::metadata::CustomAttribute {
                            constructor: parse_function_ref(rest)?,
                        });
                } else if word == ".pinvoke" {
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
                    if let Some(instruction) =
                        parse_compact_instruction(word, rest, &pending.function)?
                    {
                        pending.function.body.push(instruction);
                        return Ok(());
                    }
                    let pc = pending.function.body.len();
                    let argument = match word {
                        "unaligned." => Some(serde_json::json!(
                            rest.parse::<u8>()
                                .map_err(|_| Fault::new("expected unaligned. 1, 2, or 4"))?
                        )),
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
                        "ldarg" | "ldarga" | "starg" | "ldloc" | "ldloca" | "stloc" => Some(
                            serde_json::json!(resolve_slot(&pending.function, word, rest)?),
                        ),
                        "ldfld" | "stfld" | "ldflda" => {
                            let index = if let Ok(index) = rest.parse::<usize>() {
                                index
                            } else {
                                let (owner, field) = rest.split_once("::").ok_or_else(|| {
                                    Fault::new("expected field index or Type::Field")
                                })?;
                                let owner = bind_type_parameters(
                                    parse_type(owner)?,
                                    &typedef
                                        .as_ref()
                                        .map(|d| d.generic_parameters.clone())
                                        .unwrap_or_default(),
                                );
                                let field = field.trim();
                                identifier(field)?;
                                field_fixups.push((
                                    module.functions.len(),
                                    pc,
                                    owner,
                                    field.into(),
                                    line_number,
                                ));
                                0
                            };
                            Some(serde_json::json!(index))
                        }
                        "br" | "brtrue" | "brfalse" | "beq" | "bne.un" | "bgt" | "bgt.un"
                        | "blt" | "blt.un" | "bge" | "bge.un" | "ble" | "ble.un" => {
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
                        "call" | "callvirt" | "newobj.ctor" => Some(
                            serde_json::to_value(parse_function_ref(rest)?)
                                .map_err(|e| Fault::new(e.to_string()))?,
                        ),
                        "newobj" if rest.contains('(') => Some(
                            serde_json::to_value(parse_function_ref(rest)?)
                                .map_err(|e| Fault::new(e.to_string()))?,
                        ),
                        "newobj" => Some(
                            serde_json::to_value(parse_type(rest)?)
                                .map_err(|e| Fault::new(e.to_string()))?,
                        ),
                        "sizeof" | "alignof" | "heap.alloc" | "ptr.null" | "ptr.cast"
                        | "ptr.fromint" | "ldobj" | "stobj" | "cpobj" | "initobj"
                        | "value.pack" | "value.is" | "value.unpack" | "ldtoken"
                        | "interface.borrow" => Some(
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
                    let opcode = if word == "newobj" && rest.contains('(') {
                        "newobj.ctor"
                    } else {
                        word
                    };
                    let mut json = serde_json::json!({"op": opcode});
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
                ".references" => {
                    if module.references.is_some() {
                        return Err(Fault::new("duplicate .references"));
                    }
                    let names = rest
                        .strip_prefix('(')
                        .and_then(|s| s.strip_suffix(')'))
                        .ok_or_else(|| Fault::new("expected .references (Module, ...)"))?;
                    let mut references = Vec::new();
                    if !names.trim().is_empty() {
                        for name in names.split(',') {
                            references.push(parse_module_reference(name.trim())?);
                        }
                    }
                    module.references = Some(references);
                }
                ".revision" => {
                    if module.revision.is_some() || !crate::metadata::valid_revision(rest) {
                        return Err(Fault::new("invalid or duplicate .revision"));
                    }
                    module.revision = Some(rest.into());
                }
                ".type" | ".interface" => {
                    let (visibility, rest) = match rest.split_once(char::is_whitespace) {
                        Some(("public", rest)) => {
                            (crate::metadata::Visibility::Public, rest.trim())
                        }
                        Some(("internal", rest)) => {
                            (crate::metadata::Visibility::Internal, rest.trim())
                        }
                        Some(("private", rest)) => {
                            (crate::metadata::Visibility::Private, rest.trim())
                        }
                        _ => (crate::metadata::Visibility::Public, rest),
                    };
                    let (mut name, generic_parameters) = parse_type_declaration(rest)?;
                    if let Some(parent) = typedef.take() {
                        if !parent.generic_parameters.is_empty() {
                            return Err(Fault::new(
                                "nesting under generic types is not supported yet",
                            ));
                        }
                        if name.contains('.') {
                            return Err(Fault::new(
                                "nested type declaration requires a simple name",
                            ));
                        }
                        if enclosing_types.len() >= 31 {
                            return Err(Fault::new("type ownership nesting exceeds 32"));
                        }
                        name = format!("{}.{}", parent.name, name);
                        nesting_fixups.push((
                            name.clone(),
                            generic_parameters.len(),
                            parent.name.clone(),
                        ));
                        enclosing_types.push(parent);
                    }
                    let ty = Type::from_name(&name);
                    typedef = Some(TypeDef {
                        visibility,
                        definition: None,
                        declaring_type: None,
                        custom_attributes: vec![],
                        name: ty.definition_name().unwrap_or(&name).into(),
                        generic_parameters,
                        fields: vec![],
                        implements: vec![],
                        properties: vec![],
                        packing: None,
                        minimum_size: None,
                        representation: if word == ".interface" {
                            Representation::Interface
                        } else if ty.is_primitive() {
                            Representation::Runtime
                        } else {
                            Representation::Record
                        },
                    });
                }
                ".function" | ".method" => {
                    let (visibility, rest) = match rest.split_once(char::is_whitespace) {
                        Some(("public", rest)) => {
                            (crate::metadata::Visibility::Public, rest.trim())
                        }
                        Some(("internal", rest)) => {
                            (crate::metadata::Visibility::Internal, rest.trim())
                        }
                        Some(("private", rest)) => {
                            (crate::metadata::Visibility::Private, rest.trim())
                        }
                        _ => (crate::metadata::Visibility::Public, rest),
                    };
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
                        (Some(def.open_type()), kind == "instance", signature.trim())
                    } else {
                        (None, false, rest)
                    };
                    let (receiver_byref, declaration) = match declaration.strip_prefix("byref ") {
                        Some(rest) => (true, rest.trim()),
                        None => (false, declaration),
                    };
                    let (name, result) = declaration
                        .split_once("->")
                        .ok_or_else(|| Fault::new("expected .function Name -> Type"))?;
                    let inline_parameters = name.contains('(');
                    let (target, parameter_names, out_parameters) = if inline_parameters {
                        parse_callable(name.trim(), true)?
                    } else {
                        identifier(name.trim())?;
                        (
                            FunctionRef {
                                definition: None,
                                name: name.trim().into(),
                                parameters: vec![],
                                owner: None,
                                instance: false,
                            },
                            vec![],
                            vec![],
                        )
                    };
                    if target.owner.is_some() || target.instance {
                        return Err(Fault::new(
                            "declaration names must not contain an owner or instance prefix",
                        ));
                    }
                    if owner.is_some() && target.name.contains('.') && target.name != ".ctor" {
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
                            visibility,
                            definition: None,
                            custom_attributes: vec![],
                            name,
                            owner,
                            instance,
                            parameters: target.parameters,
                            parameter_names,
                            out_parameters,
                            receiver_byref,
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
    module.normalize_definition_ids()?;
    for (child, arity, parent) in nesting_fixups {
        let owner = module
            .types
            .iter()
            .find(|def| def.name == parent && def.generic_parameters.is_empty())
            .and_then(|def| def.definition.clone())
            .ok_or_else(|| Fault::new("missing nested type owner"))?;
        for def in &mut module.types {
            if def.name == child && def.generic_parameters.len() == arity {
                def.declaring_type = Some(owner.clone());
            }
        }
    }
    Ok((module, field_fixups))
}

fn resolve_fields(
    module: &mut Module,
    context: &Module,
    field_fixups: Vec<FieldFixup>,
) -> Result<(), Fault> {
    let context = crate::scope::normalize_module(context, context)?;
    // Resolve after all declarations, including other modules in a source group.
    for (function, pc, owner, name, line) in field_fixups {
        let owner = crate::scope::normalize_type(&context, &owner)?;
        let arity = module.functions[function]
            .owner
            .as_ref()
            .and_then(|t| context.type_definition(t))
            .map_or(0, |d| d.generic_parameters.len());
        let fields = crate::vm::record_fields(&context, &owner, arity).map_err(|e| {
            Fault::new(format!(
                "line {line}: invalid field owner {owner:?}: {}",
                e.message
            ))
        })?;
        let index = fields
            .iter()
            .position(|field| field.name == name)
            .ok_or_else(|| {
                Fault::new(format!("line {line}: unknown field {name:?} on {owner:?}"))
            })?;
        crate::references::check_type(&context, module, &owner)?;
        match &mut module.functions[function].body[pc] {
            Instruction::Field(slot)
            | Instruction::SetField(slot)
            | Instruction::FieldAddress(slot) => *slot = index,
            _ => return Err(Fault::new("invalid field fixup")),
        }
    }
    Ok(())
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

fn parse_module_reference(text: &str) -> Result<crate::metadata::ModuleReference, Fault> {
    if let Some((name, revision)) = text.split_once('#') {
        identifier(name)?;
        if !crate::metadata::valid_revision(revision) {
            return Err(Fault::new("invalid module revision"));
        }
        Ok(crate::metadata::ModuleReference::Exact {
            name: name.into(),
            revision: revision.into(),
        })
    } else {
        identifier(text)?;
        Ok(text.into())
    }
}

pub fn parse_type(text: &str) -> Result<Type, Fault> {
    fn parse(text: &str, depth: usize) -> Result<Type, Fault> {
        if depth > 32 {
            return Err(Fault::new("type nesting exceeds 32"));
        }
        let text = text.trim();
        if let Some(index) = text.strip_prefix('!') {
            // Pointer suffixes are parsed first below; !0* is not a bare index.
            if !text.ends_with('*') && !text.ends_with('&') {
                return Ok(Type::TypeParameter(
                    index
                        .parse()
                        .map_err(|_| Fault::new("expected type parameter index !0"))?,
                ));
            }
        }
        if let Some(element) = text.strip_suffix('&') {
            return Ok(Type::ByRef(Box::new(parse(element, depth + 1)?)));
        }
        if let Some(element) = text.strip_suffix('*') {
            return Ok(Type::Ptr(Box::new(parse(element, depth + 1)?)));
        }
        if let Some(scoped) = text.strip_prefix('[') {
            let (module, rest) = scoped
                .split_once(']')
                .ok_or_else(|| Fault::new("unclosed module scope"))?;
            identifier(module)?;
            let (name, arguments) = match parse(rest, depth + 1)? {
                Type::Constructed {
                    definition,
                    arguments,
                } => (definition, arguments),
                ty => (
                    ty.definition_name()
                        .ok_or_else(|| Fault::new("module scope requires a named type definition"))?
                        .to_owned(),
                    vec![],
                ),
            };
            return Ok(Type::Scoped {
                module: module.into(),
                name,
                arguments,
            });
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
                ("InterfaceRef", [t]) => Ok(Type::InterfaceRef(Box::new(parse(t, depth + 1)?))),
                ("Ref", [t]) => Ok(Type::Ref(Box::new(parse(t, depth + 1)?))),
                ("Ptr", [t]) => Ok(Type::Ptr(Box::new(parse(t, depth + 1)?))),
                ("Ref" | "Ptr" | "InterfaceRef", _) => {
                    Err(Fault::new("incorrect built-in generic arity"))
                }
                _ => {
                    identifier(name.trim())?;
                    Ok(Type::Constructed {
                        definition: name.trim().into(),
                        arguments: parts
                            .iter()
                            .map(|part| parse(part, depth + 1))
                            .collect::<Result<_, _>>()?,
                    })
                }
            };
        }
        identifier(text)?;
        Ok(Type::from_name(text))
    }
    parse(text, 0)
}

/// Parse an explicit call signature, including nested constructed parameter types.
pub fn parse_function_ref(text: &str) -> Result<FunctionRef, Fault> {
    parse_callable(text, false).map(|(target, _, _)| target)
}

type Callable = (FunctionRef, Vec<Option<String>>, Vec<usize>);
fn parse_callable(text: &str, named: bool) -> Result<Callable, Fault> {
    let (text, definition) = if !named {
        if let Some((signature, identity)) = text.rsplit_once('@') {
            let (module, index) = identity
                .trim()
                .split_once(':')
                .ok_or_else(|| Fault::new("expected @ Module:index"))?;
            let module = parse_module_reference(module.trim())?;
            let index = index
                .trim()
                .parse::<u32>()
                .map_err(|_| Fault::new("expected unsigned definition index"))?;
            (
                signature.trim(),
                Some(crate::metadata::MemberId {
                    module: module.name().into(),
                    revision: module.revision().map(str::to_owned),
                    index,
                }),
            )
        } else {
            (text, None)
        }
    } else {
        (text, None)
    };
    let (name, parameters) = text.trim().split_once('(').ok_or_else(|| {
        Fault::new("call requires an explicit signature: Name(Type, ...) or Name()")
    })?;
    let (instance, name) = match name.trim().strip_prefix("instance ") {
        Some(name) => (true, name.trim()),
        None => (false, name.trim()),
    };
    let (owner, name) = if let Some((owner, member)) = name.split_once("::") {
        identifier(member.trim())?;
        if member.trim().contains('.') && member.trim() != ".ctor" {
            return Err(Fault::new("expected an unqualified member name"));
        }
        let owner = parse_type(owner)?;
        let full_name = format!(
            "{}.{}",
            match &owner {
                Type::Constructed { definition, .. } => definition.as_str(),
                Type::Scoped { name, .. } => name.as_str(),
                _ => owner
                    .definition_name()
                    .ok_or_else(|| Fault::new("invalid method owner"))?,
            },
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
    let mut out_parameters = vec![];
    let mut parameter = |text: &str| -> Result<(), Fault> {
        let text = if named {
            if let Some(rest) = text.trim().strip_prefix("out ") {
                out_parameters.push(types.len());
                rest.trim()
            } else {
                text
            }
        } else {
            text
        };
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
            definition,
            name,
            owner,
            instance,
            parameters: types,
        },
        names,
        out_parameters,
    ))
}

fn parse_slot(text: &str) -> Result<(Option<String>, Type), Fault> {
    let text = text.trim();
    // Try the complete type first: whitespace inside generic arguments and
    // between pointer suffixes does not introduce a name.
    if let Ok(ty) = parse_type(text) {
        return Ok((None, ty));
    }
    let (ty, name) = text
        .rsplit_once(char::is_whitespace)
        .ok_or_else(|| Fault::new("expected Type or Type name"))?;
    if !crate::metadata::valid_slot_name(name) {
        return Err(Fault::new("invalid parameter/local name"));
    }
    Ok((Some(name.into()), parse_type(ty)?))
}

fn resolve_slot(function: &Function, op: &str, operand: &str) -> Result<usize, Fault> {
    if let Ok(index) = operand.parse::<usize>() {
        return Ok(index);
    }
    if matches!(op, "ldarg" | "ldarga" | "starg") && function.instance && operand == "this" {
        return Ok(0);
    }
    let (names, offset) = if matches!(op, "ldarg" | "ldarga" | "starg") {
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
                if matches!(op, "ldarg" | "ldarga" | "starg") {
                    "parameter"
                } else {
                    "local"
                }
            ))
        })
}

/// Compact source spellings normalize to one canonical metadata instruction.
fn parse_compact_instruction(
    word: &str,
    operand: &str,
    function: &Function,
) -> Result<Option<Instruction>, Fault> {
    if word == "ldc.i4.s" {
        let value = operand
            .parse::<i8>()
            .map_err(|_| Fault::new("ldc.i4.s requires a signed 8-bit literal"))?;
        return Ok(Some(Instruction::Int(value as i32)));
    }
    if let Some(suffix) = word.strip_prefix("ldc.i4.") {
        let value = match suffix {
            "m1" => -1,
            "0" => 0,
            "1" => 1,
            "2" => 2,
            "3" => 3,
            "4" => 4,
            "5" => 5,
            "6" => 6,
            "7" => 7,
            "8" => 8,
            _ => return Err(Fault::new("invalid compact Int32 opcode")),
        };
        if !operand.is_empty() {
            return Err(Fault::new("compact constant has no operand"));
        }
        return Ok(Some(Instruction::Int(value)));
    }
    for base in ["ldarg", "starg", "ldloc", "stloc"] {
        let Some(suffix) = word.strip_prefix(base).and_then(|s| s.strip_prefix('.')) else {
            continue;
        };
        let index = if suffix == "s" {
            let index = resolve_slot(function, base, operand)?;
            if index > u8::MAX as usize {
                return Err(Fault::new("short slot index exceeds 255"));
            }
            index
        } else {
            if base == "starg" || !matches!(suffix, "0" | "1" | "2" | "3") {
                return Err(Fault::new("invalid compact slot opcode"));
            }
            if !operand.is_empty() {
                return Err(Fault::new("compact slot opcode has no operand"));
            }
            suffix.parse::<usize>().unwrap()
        };
        return Ok(Some(match base {
            "ldarg" => Instruction::Arg(index),
            "starg" => Instruction::StoreArg(index),
            "ldloc" => Instruction::Load(index),
            "stloc" => Instruction::Store(index),
            _ => unreachable!(),
        }));
    }
    Ok(None)
}

fn parse_type_declaration(text: &str) -> Result<(String, Vec<Option<String>>), Fault> {
    let Some((name, parameters)) = text.split_once('<') else {
        identifier(text)?;
        return Ok((text.into(), vec![]));
    };
    let name = name.trim();
    identifier(name)?;
    let parameters = parameters
        .strip_suffix('>')
        .ok_or_else(|| Fault::new("unclosed type parameters"))?;
    let mut names = vec![];
    for parameter in parameters.split(',') {
        if names.len() > u16::MAX as usize {
            return Err(Fault::new("too many type parameters"));
        }
        let parameter = parameter.trim();
        if parameter == format!("!{}", names.len()) {
            names.push(None);
        } else if crate::metadata::valid_slot_name(parameter) {
            names.push(Some(parameter.into()));
        } else {
            return Err(Fault::new("expected type parameter name or its index"));
        }
    }
    Ok((name.into(), names))
}

fn bind_type_parameters(ty: Type, names: &[Option<String>]) -> Type {
    match ty {
        Type::Scoped {
            module,
            name,
            arguments,
        } => Type::Scoped {
            module,
            name,
            arguments: arguments
                .into_iter()
                .map(|t| bind_type_parameters(t, names))
                .collect(),
        },
        Type::Named(name) => names
            .iter()
            .position(|n| n.as_deref() == Some(&name))
            .map_or(Type::Named(name), |index| Type::TypeParameter(index as u16)),
        Type::Constructed {
            definition,
            arguments,
        } => Type::Constructed {
            definition,
            arguments: arguments
                .into_iter()
                .map(|t| bind_type_parameters(t, names))
                .collect(),
        },
        Type::Ref(t) => Type::Ref(Box::new(bind_type_parameters(*t, names))),
        Type::InterfaceRef(t) => Type::InterfaceRef(Box::new(bind_type_parameters(*t, names))),
        Type::ByRef(t) => Type::ByRef(Box::new(bind_type_parameters(*t, names))),
        Type::Ptr(t) => Type::Ptr(Box::new(bind_type_parameters(*t, names))),
        other => other,
    }
}
