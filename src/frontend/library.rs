//! The companion compiler targets bundled System, whose implementation is trusted.
//! Union coverage requires its marker, constructor cases and typed public accessors.
use super::Ty;
use crate::{
    Fault,
    metadata::{Type, Visibility},
};

pub(super) struct Case {
    pub name: String,
    pub test: String,
    pub extract: String,
    pub payload: Option<(Ty, String)>,
}

pub(super) fn resolve(signature: &str) -> Result<crate::metadata::Function, Fault> {
    let function = crate::vm::resolve(
        crate::library::system()?,
        &crate::assembler::parse_function_ref(signature)?,
    )?;
    if function.visibility != Visibility::Public
        || function.receiver_byref
        || !function.out_parameters.is_empty()
        || !function.out_when_true.is_empty()
    {
        return Err(Fault::new("unsupported library call contract"));
    }
    Ok(function)
}

pub(super) fn cases(ty: &Ty) -> Result<Vec<Case>, Fault> {
    let module = crate::library::system()?;
    let metadata = crate::assembler::parse_type(&ty.il())?;
    let definition = module
        .type_definition(&metadata)
        .ok_or_else(|| Fault::new("match requires a bundled System union"))?;
    if !definition.custom_attributes.iter().any(|a| {
        a.constructor.owner.as_ref().and_then(Type::definition_name)
            == Some("System.Runtime.CompilerServices.UnionAttribute")
    }) {
        return Err(Fault::new("match requires a bundled System union marker"));
    }
    let arguments = match &metadata {
        Type::Constructed { arguments, .. } => arguments.as_slice(),
        _ => &[],
    };
    let mut cases = Vec::new();
    for constructor in module.functions.iter().filter(|f| {
        f.owner.as_ref() == Some(&definition.open_type())
            && f.instance
            && f.name.ends_with("..ctor")
    }) {
        if constructor.visibility != Visibility::Public || constructor.parameters.len() != 1 {
            return Err(Fault::new("unsupported union constructor contract"));
        }
        let case_type = constructor.parameters[0].substitute_type_parameters(arguments)?;
        let case = Ty::from_metadata(&case_type)?;
        let name = case_type
            .definition_name()
            .and_then(|n| n.rsplit('.').next())
            .ok_or_else(|| Fault::new("invalid union case type"))?
            .to_owned();
        if cases.iter().any(|c: &Case| c.name == name) {
            return Err(Fault::new("ambiguous union case name"));
        }
        let mut accessors = None;
        for test_suffix in ["Case", ""] {
            for extract_suffix in ["Case", ""] {
                let test = format!("instance {}::get_Is{name}{test_suffix}()", ty.il());
                let extract = format!("instance {}::Get{name}{extract_suffix}()", ty.il());
                if let (Ok(test_function), Ok(extract_function)) =
                    (resolve(&test), resolve(&extract))
                {
                    if test_function.returns == Type::Boolean
                        && extract_function.returns == case_type
                    {
                        accessors = Some((test, extract));
                        break;
                    }
                }
            }
            if accessors.is_some() {
                break;
            }
        }
        let (test, extract) = accessors
            .ok_or_else(|| Fault::new("union case lacks typed test/extraction accessors"))?;
        module
            .type_definition(&case_type)
            .ok_or_else(|| Fault::new("missing union case definition"))?;
        let fields = module.instantiated_fields(&case_type)?;
        let payload = if fields.is_empty() {
            None
        } else {
            if fields.len() != 1 {
                return Err(Fault::new("only single-payload union cases are supported"));
            }
            let accessor = format!("instance {}::get_Value()", case.il());
            let function = resolve(&accessor)?;
            if function.returns != fields[0].ty {
                return Err(Fault::new("union payload accessor type mismatch"));
            }
            Some((Ty::from_metadata(&function.returns)?, accessor))
        };
        cases.push(Case {
            name,
            test,
            extract,
            payload,
        });
    }
    if cases.is_empty() {
        return Err(Fault::new("union has no supported cases"));
    }
    Ok(cases)
}
