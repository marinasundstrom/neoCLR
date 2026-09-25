//! Source identity and reflection admission, distinct from executable neoIL definition IDs.
use crate::{Fault, Module};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssemblyMetadata {
    pub name: String,
    pub full_name: String,
    pub modules: Vec<String>,
    pub references: Vec<String>,
}

/// Original CLI member accessibility, independent of lowered helper visibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceAccess {
    Public,
    Private,
    Assembly,
    Family,
    FamilyOrAssembly,
    FamilyAndAssembly,
    CompilerControlled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetadataOrigin {
    pub assembly: String,
    pub module: String,
    pub name: String,
    pub token: u32,
    /// Type and all containing types are public in the imported source metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publicly_visible: Option<bool>,
    /// Original method accessibility. Older origins lack reflection admission data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_access: Option<SourceAccess>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declaring_type_token: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub field_tokens: Vec<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub property_tokens: Vec<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameter_tokens: Vec<u32>,
}

fn text(value: &str) -> bool {
    !value.trim().is_empty() && !value.chars().any(char::is_control)
}

pub(crate) fn validate(module: &Module) -> Result<(), Fault> {
    let mut assemblies = HashSet::new();
    for assembly in &module.assemblies {
        if !text(&assembly.name)
            || !text(&assembly.full_name)
            || !assemblies.insert(&assembly.full_name)
            || assembly.modules.is_empty()
            || assembly.modules.iter().any(|v| !text(v))
            || assembly.references.iter().any(|v| !text(v))
            || assembly.modules.iter().collect::<HashSet<_>>().len() != assembly.modules.len()
            || assembly.references.iter().collect::<HashSet<_>>().len() != assembly.references.len()
        {
            return Err(Fault::new("invalid or duplicate assembly metadata"));
        }
    }
    let mut tokens = HashSet::new();
    let mut check = |origin: &MetadataOrigin, token: u32, table: u32| -> Result<(), Fault> {
        if token >> 24 != table
            || token & 0x00ff_ffff == 0
            || !tokens.insert((origin.assembly.clone(), origin.module.clone(), token))
        {
            return Err(Fault::new(
                "invalid or duplicate module-scoped metadata token",
            ));
        }
        Ok(())
    };
    for (origin, fields, properties, parameters, table) in module
        .types
        .iter()
        .filter_map(|t| {
            t.origin
                .as_ref()
                .map(|o| (o, t.fields.len(), t.properties.len(), 0, 0x02))
        })
        .chain(module.functions.iter().filter_map(|f| {
            f.origin
                .as_ref()
                .map(|o| (o, 0, 0, f.parameters.len(), 0x06))
        }))
    {
        if !text(&origin.name)
            || !module
                .assemblies
                .iter()
                .any(|a| a.full_name == origin.assembly && a.modules.contains(&origin.module))
            || origin.field_tokens.len() != fields
            || origin.property_tokens.len() != properties
            || origin.parameter_tokens.len() != parameters
        {
            return Err(Fault::new(
                "metadata origin does not match its definition or assembly",
            ));
        }
        if table != 0x02 && origin.publicly_visible.is_some()
            || table != 0x06 && origin.member_access.is_some()
        {
            return Err(Fault::new(
                "source access metadata does not match definition kind",
            ));
        }
        if table != 0x02 && origin.declaring_type_token.is_some() {
            return Err(Fault::new(
                "declaring type origin applies only to type definitions",
            ));
        }
        check(origin, origin.token, table)?;
        for &token in &origin.field_tokens {
            check(origin, token, 0x04)?;
        }
        for &token in &origin.property_tokens {
            check(origin, token, 0x17)?;
        }
        // CLI permits parameters without a Param row. Zero denotes that absence,
        // never an invented row or a module-wide identity.
        for &token in &origin.parameter_tokens {
            if token != 0 {
                check(origin, token, 0x08)?;
            }
        }
    }
    for origin in module.types.iter().filter_map(|t| t.origin.as_ref()) {
        let mut current = origin;
        let mut visited = HashSet::from([origin.token]);
        while let Some(parent) = current.declaring_type_token {
            if !visited.insert(parent) {
                return Err(Fault::new("cyclic source declaring type metadata"));
            }
            current = module
                .types
                .iter()
                .filter_map(|t| t.origin.as_ref())
                .find(|p| {
                    p.assembly == origin.assembly && p.module == origin.module && p.token == parent
                })
                .ok_or_else(|| Fault::new("missing source declaring type definition"))?;
        }
    }
    Ok(())
}

pub(crate) fn merge(target: &mut Module, source: &Module) -> Result<(), Fault> {
    for assembly in &source.assemblies {
        if let Some(existing) = target
            .assemblies
            .iter()
            .find(|a| a.full_name == assembly.full_name)
        {
            if existing != assembly {
                return Err(Fault::new("conflicting assembly metadata"));
            }
        } else {
            target.assemblies.push(assembly.clone());
        }
    }
    Ok(())
}

/// Reflection observes original member accessibility; lowering can use broader helpers.
pub(crate) fn member_access(function: &crate::metadata::Function) -> SourceAccess {
    function
        .origin
        .as_ref()
        .and_then(|o| o.member_access)
        .unwrap_or(match function.visibility {
            crate::metadata::Visibility::Public => SourceAccess::Public,
            crate::metadata::Visibility::Private => SourceAccess::Private,
            crate::metadata::Visibility::Internal => SourceAccess::Assembly,
        })
}

pub(crate) fn reflection_public(
    module: &Module,
    owner: &crate::metadata::Type,
    function: &crate::metadata::Function,
) -> bool {
    let type_public = module.type_definition(owner).is_some_and(|t| {
        t.origin
            .as_ref()
            .is_none_or(|o| o.publicly_visible == Some(true))
    });
    type_public
        && function
            .origin
            .as_ref()
            .is_none_or(|o| o.member_access == Some(SourceAccess::Public))
}
