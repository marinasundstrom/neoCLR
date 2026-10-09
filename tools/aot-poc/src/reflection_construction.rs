//! Explicit, source-identity-based retention for the bounded construction bridge.
use neoclr::metadata::{
    FunctionRef, Instruction as Op, MemberId, Representation, Type, TypeDefId, Visibility,
};
use serde_json::{Value, json};
type Error = Box<dyn std::error::Error>;

pub fn bind(
    input: &mut neoclr::Module,
    source: &neoclr::Module,
    definitions: &[TypeDefId],
    methods: &[MemberId],
    config: &Value,
) -> Result<Value, Error> {
    let config = config
        .as_object()
        .ok_or("reflection roots must be an object")?;
    let version = config
        .get("schemaVersion")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if config.len() != 2 || !matches!(version, 1 | 2) {
        return Err("reflection roots require schemaVersion 1 or 2 and types".into());
    }
    let roots = config
        .get("types")
        .and_then(Value::as_array)
        .ok_or("reflection roots require types array")?;
    if roots.is_empty() || roots.len() > 64 {
        return Err("reflection roots require one to 64 types".into());
    }
    let mut selected = vec![];
    let mut rows = vec![];
    for root in roots {
        let fields = root
            .as_object()
            .ok_or("reflection root must be an object")?;
        if fields.len() != if version == 1 { 2 } else { 5 }
            || !fields.contains_key("definition")
            || !fields.contains_key("construct")
            || (version == 2
                && ["properties", "getters", "setters"]
                    .iter()
                    .any(|k| !fields.contains_key(*k)))
        {
            return Err("reflection root fields do not match the requested schema".into());
        }
        let definition: TypeDefId = serde_json::from_value(root["definition"].clone())?;
        let construct = root["construct"]
            .as_bool()
            .ok_or("construct must be Boolean")?;
        let mut property_policy = [false; 3];
        if version == 2 {
            for (i, key) in ["properties", "getters", "setters"].iter().enumerate() {
                property_policy[i] = root[*key]
                    .as_bool()
                    .ok_or("reflection property root policy must be Boolean")?;
            }
            if !property_policy[0] && (property_policy[1] || property_policy[2]) {
                return Err("reflection accessor roots require property metadata".into());
            }
        }
        let index = definitions
            .iter()
            .position(|d| *d == definition)
            .ok_or("reflection root definition is not in the explicit source load set")?;
        if selected.iter().any(|(i, _, _, _)| *i == index) {
            return Err("duplicate reflection root definition".into());
        }
        let ty = &source.types[index];
        if !ty.generic_parameters.is_empty() {
            return Err(
                "reflection construction roots currently require nongeneric declarations".into(),
            );
        }
        let owner = Type::Named(ty.name.clone());
        let candidates: Vec<_> = source
            .functions
            .iter()
            .enumerate()
            .filter(|(i, f)| {
                f.owner.as_ref() == Some(&owner)
                    && f.instance
                    && f.name.ends_with("..ctor")
                    && f.parameters.is_empty()
                    && methods[*i].module == definition.module
                    && methods[*i].revision == definition.revision
            })
            .map(|(i, _)| i)
            .collect();
        let mut status = 0;
        let mut target = None;
        if !ty.is_reference_type || ty.representation != Representation::Record || ty.is_abstract {
            status = 2;
        } else if candidates.len() != 1 {
            status = 4;
        } else {
            let index = candidates[0];
            let f = &source.functions[index];
            if ty.visibility != Visibility::Public
                || f.visibility != Visibility::Public
                || ty
                    .origin
                    .as_ref()
                    .is_some_and(|o| o.publicly_visible != Some(true))
                || f.origin.as_ref().is_some_and(|o| {
                    o.member_access != Some(neoclr::metadata_origin::SourceAccess::Public)
                })
            {
                status = 3;
            } else {
                if !f.no_result
                    || f.returns != Type::Void
                    || f.receiver_byref
                    || f.receiver_readonly
                    || f.is_internal_call()
                    || f.pinvoke.is_some()
                    || !f.generic_parameters.is_empty()
                {
                    return Err("retained constructor has an unsupported native contract".into());
                }
                let lowered = &input.functions[index];
                target = Some(FunctionRef {
                    definition: lowered.definition.clone(),
                    name: lowered.name.clone(),
                    owner: lowered.owner.clone(),
                    instance: true,
                    generic_arguments: vec![],
                    parameters: vec![],
                });
            }
        }
        rows.push(json!({"definition":definition,"sourceName":ty.name,"construct":construct,
            "properties":property_policy[0],"getters":property_policy[1],"setters":property_policy[2],
            "checkStatus":status,"constructor":target.as_ref().and_then(|_| candidates.first()).map(|i| &methods[*i]),
            "policy":"explicit source identity; metadata check does not retain constructor body unless invocation is requested"}));
        selected.push((index, construct, status, target));
    }
    for f in &mut input.functions {
        let check = f.name == "neoCLR.Runtime.ReflectionConstructionCheck";
        if !check && f.name != "neoCLR.Runtime.ReflectionConstruct" {
            continue;
        }
        let returns = if check {
            Type::Int32
        } else {
            Type::Named("System.Object".into())
        };
        if f.owner.is_some()
            || f.instance
            || f.parameters != vec![Type::RuntimeTypeHandle]
            || f.returns != returns
            || f.no_result
            || f.impl_flags != 0x1000
            || !f.body.is_empty()
            || !f.locals.is_empty()
            || f.pinvoke.is_some()
            || f.receiver_byref
            || f.receiver_readonly
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
            return Err(
                "native construction requires the exact reserved InternalCall contract".into(),
            );
        }
        f.impl_flags = 0;
        let mut body = vec![];
        for (index, construct, status, target) in &selected {
            let branch = body.len() + 3;
            body.extend([
                Op::Arg(0),
                Op::LoadTypeToken(Type::from_name(&source.types[*index].name)),
                Op::Equal,
                Op::BranchFalse(0),
            ]);
            if check {
                body.extend([Op::Int(*status), Op::Return]);
            } else if *status != 0 {
                let reason = match status {
                    2 => "UnsupportedType",
                    3 => "AccessDenied",
                    _ => "MissingConstructor",
                };
                body.push(Op::Fault(format!(
                    "reflection construction rejected: {reason}"
                )));
            } else if *construct {
                body.extend([
                    Op::Construct(target.clone().unwrap()),
                    Op::CastClass(Type::Named("System.Object".into())),
                    Op::Return,
                ]);
            } else {
                body.push(Op::Fault(
                    "native reflection constructor invocation was not retained".into(),
                ));
            }
            body[branch] = Op::BranchFalse(body.len());
        }
        body.push(Op::Fault(
            "native reflection type metadata was not retained; configure --reflection-roots".into(),
        ));
        f.body = body;
    }
    Ok(json!({"schemaVersion":version,"types":rows}))
}
