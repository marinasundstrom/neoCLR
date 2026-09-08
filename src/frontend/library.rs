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

pub(super) fn is_interface(ty: &Ty) -> Result<bool, Fault> {
    let module = crate::library::system()?;
    let metadata = crate::assembler::parse_type(&ty.il())?;
    Ok(module.type_definition(&metadata).is_some_and(|definition| {
        definition.representation == crate::metadata::Representation::Interface
    }))
}

fn owners(module: &crate::Module, ty: &Type) -> Result<Vec<Type>, Fault> {
    if module
        .type_definition(ty)
        .is_some_and(|d| d.representation == crate::metadata::Representation::Record)
    {
        crate::inheritance::lineage(module, ty)
    } else if module
        .type_definition(ty)
        .is_some_and(|d| d.representation == crate::metadata::Representation::Interface)
    {
        let mut result = vec![ty.clone()];
        for base in crate::interfaces::closure(module, ty)? {
            if !result.contains(&base) {
                result.push(base);
            }
        }
        Ok(result)
    } else {
        Ok(vec![ty.clone()])
    }
}
pub(super) fn base_reachable(from: &Ty, to: &Ty) -> Result<bool, Fault> {
    let module = crate::library::system()?;
    let from = crate::assembler::parse_type(&from.il())?;
    let to = crate::assembler::parse_type(&to.il())?;
    if module
        .type_definition(&from)
        .is_some_and(|d| d.representation == crate::metadata::Representation::Record)
    {
        Ok(crate::inheritance::lineage(module, &from)?.contains(&to))
    } else {
        Ok(from == to)
    }
}
pub(super) fn method_signature(function: &crate::metadata::Function) -> Result<String, Fault> {
    Ok(format!(
        "instance {}::{}({})",
        Ty::from_metadata(
            function
                .owner
                .as_ref()
                .ok_or_else(|| Fault::new("instance member requires owner"))?
        )?
        .il(),
        function.name.rsplit('.').next().unwrap(),
        function
            .parameters
            .iter()
            .map(|p| Ty::from_metadata(p).map(|p| p.il()))
            .collect::<Result<Vec<_>, _>>()?
            .join(",")
    ))
}
/// A unique method shape supplies argument context, including a substituted T&.
/// Overloaded names retain the existing exact-signature selection path.
pub(super) fn parameters(ty: &Ty, member: &str, count: usize) -> Result<Option<Vec<Ty>>, Fault> {
    let module = crate::library::system()?;
    let metadata = crate::assembler::parse_type(&ty.il())?;
    for owner in owners(module, &metadata)? {
        let Some(definition) = module.type_definition(&owner) else {
            continue;
        };
        let mut candidates = module.functions.iter().filter(|f| {
            f.instance
                && f.owner.as_ref() == Some(&definition.open_type())
                && f.name.rsplit('.').next() == Some(member)
                && f.parameters.len() == count
                && f.visibility == Visibility::Public
        });
        let Some(method) = candidates.next() else {
            continue;
        };
        if candidates.next().is_some() {
            return Ok(None);
        }
        let arguments = match &owner {
            Type::Constructed { arguments, .. } => arguments.as_slice(),
            _ => &[],
        };
        return method
            .parameters
            .iter()
            .map(|p| Ty::from_metadata(&p.substitute_type_parameters(arguments)?))
            .collect::<Result<Vec<_>, _>>()
            .map(Some);
    }
    Ok(None)
}

pub(super) fn resolve(signature: &str) -> Result<crate::metadata::Function, Fault> {
    let module = crate::library::system()?;
    let mut target = crate::assembler::parse_function_ref(signature)?;
    let chain = if target.instance && !target.name.ends_with("..ctor") {
        owners(module, target.owner.as_ref().unwrap())?
    } else {
        target.owner.iter().cloned().collect()
    };
    let mut result = crate::vm::resolve(module, &target);
    for owner in chain {
        if result.is_ok() {
            break;
        }
        let member = target.name.rsplit('.').next().unwrap().to_owned();
        target.name = format!("{}.{}", owner.definition_name().unwrap(), member);
        target.owner = Some(owner);
        result = crate::vm::resolve(module, &target);
    }
    let function = result?;
    if function.visibility != Visibility::Public {
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

/// Read an ordinary non-indexed property through its declared public getter.
pub(super) fn property(
    ty: &Ty,
    name: &str,
) -> Result<Option<(Ty, String, crate::metadata::Function)>, Fault> {
    let module = crate::library::system()?;
    let metadata = crate::assembler::parse_type(&ty.il())?;
    for owner in owners(module, &metadata)? {
        let Some(definition) = module.type_definition(&owner) else {
            continue;
        };
        let Some(property) = definition
            .properties
            .iter()
            .find(|p| p.name == name && p.instance && p.parameters.is_empty())
        else {
            continue;
        };
        let arguments = match &owner {
            Type::Constructed { arguments, .. } => arguments.as_slice(),
            _ => &[],
        };
        let mut property = property.clone();
        property.map_types(|ty| ty.substitute_type_parameters(arguments))?;
        let getter = property
            .getter
            .ok_or_else(|| Fault::new("property has no getter"))?;
        if !getter.instance
            || getter.owner.as_ref() != Some(&owner)
            || !getter.parameters.is_empty()
        {
            return Err(Fault::new("unsupported property getter contract"));
        }
        let function = crate::vm::resolve(module, &getter)?;
        if function.visibility != Visibility::Public || function.returns != property.ty {
            return Err(Fault::new("unsupported property getter contract"));
        }
        return Ok(Some((
            Ty::from_metadata(&property.ty)?,
            method_signature(&function)?,
            function,
        )));
    }
    Ok(None)
}

/// Declared conformance of a closed bundled type, independent of its storage.
pub(super) fn implements(concrete: &Ty, interface: &Ty) -> Result<bool, Fault> {
    let module = crate::library::system()?;
    let concrete = crate::assembler::parse_type(&concrete.il())?;
    let interface = crate::assembler::parse_type(&interface.il())?;
    if module.type_definition(&concrete).is_none() {
        return Ok(false);
    }
    Ok(crate::interfaces::closure(module, &concrete)?.contains(&interface))
}

/// Resolve the declared single-index Item property, never a method naming guess.
pub(super) fn indexer(ty: &Ty, setter: bool) -> Result<(String, crate::metadata::Function), Fault> {
    let module = crate::library::system()?;
    let metadata = crate::assembler::parse_type(&ty.il())?;
    let (owner, mut property) = owners(module, &metadata)?
        .into_iter()
        .find_map(|owner| {
            module
                .type_definition(&owner)
                .and_then(|d| {
                    d.properties
                        .iter()
                        .find(|p| p.name == "Item" && p.instance && p.parameters.len() == 1)
                })
                .cloned()
                .map(|p| (owner, p))
        })
        .ok_or_else(|| Fault::new("type has no single-index Item property"))?;
    let arguments = match &owner {
        Type::Constructed { arguments, .. } => arguments.as_slice(),
        _ => &[],
    };
    property.map_types(|ty| ty.substitute_type_parameters(arguments))?;
    let accessor = if setter {
        property.setter
    } else {
        property.getter
    }
    .ok_or_else(|| {
        Fault::new(if setter {
            "indexer has no setter"
        } else {
            "indexer has no getter"
        })
    })?;
    let member = accessor
        .name
        .rsplit('.')
        .next()
        .ok_or_else(|| Fault::new("invalid indexer accessor"))?;
    let signature = format!(
        "instance {}::{member}({})",
        Ty::from_metadata(&owner)?.il(),
        accessor
            .parameters
            .iter()
            .map(|t| Ty::from_metadata(t).map(|t| t.il()))
            .collect::<Result<Vec<_>, _>>()?
            .join(",")
    );
    let function = resolve(&signature)?;
    Ok((signature, function))
}

pub(super) fn delegate(ty: &Ty) -> Result<Option<crate::metadata::Function>, Fault> {
    let module = crate::library::system()?;
    let ty = crate::assembler::parse_type(&ty.il())?;
    if module
        .type_definition(&ty)
        .is_some_and(|d| d.representation == crate::metadata::Representation::Delegate)
    {
        crate::delegates::contract(module, &ty).map(Some)
    } else {
        Ok(None)
    }
}

/// Explicit generic arguments and a unique declared arity provide argument context.
pub(super) fn generic_static(
    owner: &str,
    member: &str,
    arguments: &[Ty],
    count: usize,
) -> Result<crate::metadata::Function, Fault> {
    let module = crate::library::system()?;
    let owner = crate::assembler::parse_type(owner)?;
    let def = module
        .type_definition(&owner)
        .ok_or_else(|| Fault::new("unknown static owner"))?;
    let mut methods = module.functions.iter().filter(|f| {
        !f.instance
            && f.owner.as_ref() == Some(&def.open_type())
            && f.name.rsplit('.').next() == Some(member)
            && f.parameters.len() == count
            && f.generic_parameters.len() == arguments.len()
            && f.visibility == Visibility::Public
    });
    let method = methods
        .next()
        .ok_or_else(|| Fault::new("unknown generic static method"))?;
    if methods.next().is_some() {
        return Err(Fault::new("ambiguous generic static method"));
    }
    let type_arguments = match &owner {
        Type::Constructed { arguments, .. } => arguments.as_slice(),
        _ => &[],
    };
    let arguments = arguments
        .iter()
        .map(|t| crate::assembler::parse_type(&t.il()))
        .collect::<Result<Vec<_>, _>>()?;
    method.map_types(|t| t.substitute_parameters(Some(type_arguments), Some(&arguments)))
}
