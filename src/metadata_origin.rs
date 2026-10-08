//! Source identity and reflection admission, distinct from executable neoIL definition IDs.
use crate::{Fault, Module};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssemblyMetadata {
    /// Logical declaration containers; None denotes legacy namespace projection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declaration_modules: Option<DeclarationModules>,
    pub name: String,
    pub full_name: String,
    pub modules: Vec<String>,
    pub references: Vec<String>,
    /// External nominal value categories required by metadata-only CLI projection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub value_type_references: Vec<String>,
    /// Explicit compiler-reference to native-module linkage for translated libraries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub native_module_bindings: Vec<NativeModuleBinding>,
    /// Physical CLI type scopes retained for reference projection; execution uses native_name.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub native_type_bindings: Vec<NativeTypeBinding>,
    /// Explicit nominal descriptor backing managed vector storage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub array_backing: Option<crate::metadata::TypeDefId>,
    /// Compile-time assembly-level literals, with exact binary64 bits and no execution storage.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constants: Vec<AssemblyConstant>,
}

/// Versioned logical module table, independent of physical image names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeclarationModules {
    pub version: u32,
    pub names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssemblyConstant {
    pub namespace: String,
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub bits: String,
    pub visibility: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeModuleBinding {
    pub assembly: String,
    pub module: String,
    pub revision: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeTypeBinding {
    pub native_name: String,
    pub assembly: String,
    pub namespace: String,
    pub name: String,
    pub arity: usize,
    pub value_type: bool,
    pub declaring: Option<String>,
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
    /// Optional source-qualified method name, independent of lowered target names.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
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
    pub field_access: Vec<SourceAccess>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Init-only storage flags, also enforced by managed field operations. Missing flags remain mutable.
    pub field_readonly: Vec<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub property_tokens: Vec<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameter_tokens: Vec<u32>,
    /// Explicit .NET nullable transform facts; -1 denotes the return value.
    /// These annotations do not affect physical signatures or runtime checks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nullable_annotations: Vec<NullableAnnotation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NullableAnnotation {
    pub position: i32,
    pub flags: Vec<u8>,
    pub uniform: bool,
}

fn text(value: &str) -> bool {
    !value.trim().is_empty() && !value.chars().any(char::is_control)
}

pub(crate) fn validate(module: &Module) -> Result<(), Fault> {
    let bindings: Vec<_> = module
        .assemblies
        .iter()
        .filter_map(|a| a.array_backing.as_ref().map(|id| (a, id)))
        .collect();
    if bindings.len() > 1 {
        return Err(Fault::new("multiple nominal array backing selections"));
    }
    if let Some((assembly, id)) = bindings.first() {
        let shape = module
            .types
            .iter()
            .find(|t| t.definition.as_ref() == Some(*id))
            .ok_or_else(|| Fault::new("nominal array backing definition is missing"))?;
        if !shape.is_reference_type
            || shape.is_abstract
            || shape.base.as_ref().is_some_and(|base| {
                base != &crate::metadata::Type::from_name("System.Object")
                    || module.type_definition(base).is_none_or(|parent| {
                        module.object_root.is_none()
                            || parent.definition.as_ref() != module.object_root.as_ref()
                            || !parent.fields.is_empty()
                    })
            })
            || shape.declaring_type.is_some()
            || !matches!(
                shape.representation,
                crate::metadata::Representation::Record
            )
            || shape.generic_parameters.len() != 1
            || !shape.generic_constraints.is_empty()
            || shape.fields.len() != 1
            || shape.fields[0].ty
                != crate::metadata::Type::ArrayRef(Box::new(crate::metadata::Type::TypeParameter(
                    0,
                )))
            || !shape.origin.as_ref().is_some_and(|o| {
                o.assembly == assembly.full_name && o.field_access == vec![SourceAccess::Private]
            })
        {
            return Err(Fault::new("invalid nominal array backing storage contract"));
        }
    }
    let mut assemblies = HashSet::new();
    for assembly in &module.assemblies {
        let mut constants = HashSet::new();
        let name = |s: &str| {
            !s.trim().is_empty()
                && s.chars().count() <= 1024
                && !s.chars().any(|c| c.is_control() || c == '.')
        };
        if let Some(table) = &assembly.declaration_modules {
            let names: HashSet<_> = table.names.iter().map(String::as_str).collect();
            if table.version != 1
                || names.len() != table.names.len()
                || names.len() > 4096
                || names
                    .iter()
                    .any(|n| n.chars().count() > 1024 || (!n.is_empty() && !n.split('.').all(name)))
            {
                return Err(Fault::new("invalid declaration module table"));
            }
            let type_owners = module
                .types
                .iter()
                .filter(|ty| ty.declaring_type.is_none())
                .filter_map(|ty| ty.origin.as_ref())
                .filter(|origin| origin.assembly == assembly.full_name)
                .map(|origin| origin.name.rsplit_once('.').map_or("", |(owner, _)| owner));
            let function_owners = module
                .functions
                .iter()
                .filter(|f| f.owner.is_none())
                .filter(|f| {
                    f.origin
                        .as_ref()
                        .is_some_and(|origin| origin.assembly == assembly.full_name)
                })
                .map(|f| f.namespace.as_str());
            if type_owners
                .chain(function_owners)
                .chain(assembly.constants.iter().map(|c| c.namespace.as_str()))
                .any(|owner| !names.contains(owner))
            {
                return Err(Fault::new("missing declaration module owner"));
            }
        }
        for constant in &assembly.constants {
            if assembly.constants.len() > 4096
                || !name(&constant.name)
                || constant.namespace.chars().count() > 1024
                || (!constant.namespace.is_empty() && !constant.namespace.split('.').all(name))
                || !constants.insert((&constant.namespace, &constant.name))
                || constant.ty != "Double"
                || !matches!(constant.visibility.as_str(), "public" | "internal")
                || constant.bits.len() != 16
                || !constant
                    .bits
                    .bytes()
                    .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
                || !u64::from_str_radix(&constant.bits, 16)
                    .is_ok_and(|bits| f64::from_bits(bits).is_finite())
            {
                return Err(Fault::new("invalid or duplicate assembly-level constant"));
            }
        }
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
    for assembly in &module.assemblies {
        if assembly.native_module_bindings.len() > 256 || assembly.native_type_bindings.len() > 4096
        {
            return Err(Fault::new("native reference binding limit exceeded"));
        }
        let mut bound_assemblies = HashSet::new();
        for binding in &assembly.native_module_bindings {
            if !text(&binding.assembly)
                || !text(&binding.module)
                || binding.revision.as_ref().is_some_and(|r| !text(r))
                || !assembly.references.contains(&binding.assembly)
                || !bound_assemblies.insert(&binding.assembly)
            {
                return Err(Fault::new("invalid native module binding"));
            }
        }
        let mut bound_types = HashSet::new();
        for binding in &assembly.native_type_bindings {
            if !text(&binding.native_name)
                || !text(&binding.name)
                || binding.arity > 32
                || (!binding.namespace.is_empty() && !text(&binding.namespace))
                || binding.declaring.as_ref().is_some_and(|d| !text(d))
                || !bound_assemblies.contains(&binding.assembly)
                || !bound_types.insert((&binding.native_name, binding.arity))
            {
                return Err(Fault::new("invalid native type binding"));
            }
        }
        let mut seen = HashSet::new();
        if assembly.value_type_references.len() > 4096 {
            return Err(Fault::new("value type reference limit exceeded"));
        }
        for name in &assembly.value_type_references {
            if !text(name) || !seen.insert(name) {
                return Err(Fault::new("invalid or duplicate value type reference"));
            }
            let definition = module
                .types
                .iter()
                .find(|ty| ty.name == *name)
                .ok_or_else(|| Fault::new("missing imported value type declaration"))?;
            if definition.is_reference_type
                || definition.representation == crate::metadata::Representation::Interface
            {
                return Err(Fault::new("imported value type category mismatch"));
            }
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
            || !origin.field_access.is_empty() && origin.field_access.len() != fields
            || !origin.field_readonly.is_empty() && origin.field_readonly.len() != fields
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
        if origin.full_name.as_ref().is_some_and(|name| !text(name)) {
            return Err(Fault::new("invalid source-qualified member name"));
        }
        if table != 0x02 && origin.declaring_type_token.is_some() {
            return Err(Fault::new(
                "declaring type origin applies only to type definitions",
            ));
        }
        let mut nullable_positions = HashSet::new();
        if origin.nullable_annotations.len() > 257
            || origin.nullable_annotations.iter().any(|annotation| {
                table != 0x06
                    || annotation.position < -1
                    || annotation.position >= parameters as i32
                    || annotation.flags.is_empty()
                    || annotation.flags.len() > 4096
                    || annotation.flags.iter().any(|flag| *flag > 2)
                    || annotation.uniform && annotation.flags.len() != 1
                    || !nullable_positions.insert(annotation.position)
            })
        {
            return Err(Fault::new("invalid callable nullable annotation metadata"));
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
            crate::metadata::Visibility::Protected => SourceAccess::Family,
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

/// Source field access when retained; legacy imports retain their descriptive visibility.
pub(crate) fn field_access(owner: &crate::metadata::TypeDef, index: usize) -> SourceAccess {
    owner
        .origin
        .as_ref()
        .and_then(|o| o.field_access.get(index))
        .copied()
        .unwrap_or_else(|| match owner.fields[index].visibility {
            crate::metadata::Visibility::Public => SourceAccess::Public,
            crate::metadata::Visibility::Private => SourceAccess::Private,
            crate::metadata::Visibility::Internal => SourceAccess::Assembly,
            crate::metadata::Visibility::Protected => SourceAccess::Family,
        })
}
