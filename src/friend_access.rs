//! Load-snapshot friend grants. No attribute parsing occurs on warmed access checks.
use crate::{
    Module,
    metadata::{AttributeArgument, Type},
    metadata_origin::AssemblyMetadata,
};
use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

type Scope<'a> = Option<(&'a str, Option<&'a str>)>;
type FriendScope = (Option<String>, Option<usize>);

#[derive(Debug, Default)]
pub(crate) struct Cache(OnceLock<Index>);
// Module is mutable input; a clone must never inherit decisions from an old snapshot.
impl Clone for Cache {
    fn clone(&self) -> Self {
        Self::default()
    }
}
#[derive(Debug, Default)]
struct Index {
    scopes: HashMap<String, Vec<FriendScope>>,
    grants: HashSet<(usize, usize)>,
}
impl Index {
    fn assembly(&self, scope: Scope<'_>) -> Option<usize> {
        let (name, revision) = scope?;
        self.scopes
            .get(name)?
            .iter()
            .find(|(v, _)| v.as_deref() == revision)?
            .1
    }
    fn add_scope(&mut self, name: &str, revision: Option<&str>, assembly: usize) {
        let entries = self.scopes.entry(name.to_owned()).or_default();
        if let Some((_, prior)) = entries.iter_mut().find(|(v, _)| v.as_deref() == revision) {
            if *prior != Some(assembly) {
                *prior = None;
            }
        } else {
            entries.push((revision.map(str::to_owned), Some(assembly)));
        }
    }
    fn build(module: &Module) -> Self {
        let mut result = Self::default();
        // Origins associate resolved definition scopes with assembly identities. Legacy
        // IL has no origins, so its exact manifest module names supply the association.
        for (id, origin) in module
            .types
            .iter()
            .filter_map(|t| {
                t.definition.as_ref().map(|d| {
                    (
                        (d.module.as_str(), d.revision.as_deref()),
                        t.origin.as_ref(),
                    )
                })
            })
            .chain(module.functions.iter().filter_map(|f| {
                f.definition.as_ref().map(|d| {
                    (
                        (d.module.as_str(), d.revision.as_deref()),
                        f.origin.as_ref(),
                    )
                })
            }))
        {
            for (index, assembly) in module.assemblies.iter().enumerate() {
                let matches = origin.map_or_else(
                    || assembly.modules.iter().any(|m| m == id.0),
                    |o| o.assembly == assembly.full_name,
                );
                if matches {
                    result.add_scope(id.0, id.1, index);
                }
            }
        }
        for (declaring, assembly) in module.assemblies.iter().enumerate() {
            if !unsigned(assembly) {
                continue;
            }
            for attribute in &assembly.custom_attributes {
                let Some(owner) = attribute
                    .constructor
                    .owner
                    .as_ref()
                    .and_then(|ty| module.type_definition(ty))
                else {
                    continue;
                };
                let name = owner
                    .origin
                    .as_ref()
                    .map_or(owner.name.as_str(), |o| o.name.as_str());
                if name != "System.Runtime.CompilerServices.InternalsVisibleToAttribute"
                    || attribute.constructor.parameters != [Type::String]
                    || !attribute.named_arguments.is_empty()
                {
                    continue;
                }
                let [AttributeArgument::String(Some(friend))] = attribute.arguments.as_slice()
                else {
                    continue;
                };
                // Native signing/full-key identities and non-ASCII identity folding are
                // not admitted yet. Never strip qualifiers and accidentally grant by name.
                if friend.is_empty()
                    || !friend.is_ascii()
                    || friend.trim() != friend
                    || friend
                        .chars()
                        .any(|c| c.is_control() || matches!(c, ',' | '=' | '\\' | '"' | '\''))
                {
                    continue;
                }
                for (requesting, candidate) in module.assemblies.iter().enumerate() {
                    if unsigned(candidate) && candidate.name.eq_ignore_ascii_case(friend) {
                        result.grants.insert((declaring, requesting));
                    }
                }
            }
        }
        result
    }
}
fn unsigned(assembly: &AssemblyMetadata) -> bool {
    if assembly.full_name == assembly.name {
        return true;
    }
    let cli: Vec<_> = assembly.full_name.split(", ").collect();
    if cli.len() == 4
        && cli[0] == assembly.name
        && cli[1].starts_with("Version=")
        && cli[2].starts_with("Culture=")
        && cli[3] == "PublicKeyToken=null"
    {
        return true;
    }
    serde_json::from_str::<Vec<String>>(&assembly.full_name).is_ok_and(|parts| {
        parts.len() == 5 && parts[0] == assembly.name && parts[3].is_empty() && parts[4] == "0"
    })
}
pub(crate) fn permits(module: &Module, requesting: Scope<'_>, declaring: Scope<'_>) -> bool {
    if requesting.is_none() || declaring.is_none() {
        return false;
    }
    let index = module.friend_access.0.get_or_init(|| Index::build(module));
    index
        .assembly(requesting)
        .zip(index.assembly(declaring))
        .is_some_and(|(requesting, declaring)| index.grants.contains(&(declaring, requesting)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn warm_pair_checks_reuse_index_and_clones_discard_it() {
        let mut module =
            crate::assemble(".module Test\n.function Main() -> Int32\nldc.i4 0\nret\n.end")
                .unwrap();
        module.friend_access = Cache::default();
        let mut index = Index::default();
        index.add_scope("Provider", None, 0);
        index.add_scope("Friend", None, 1);
        index.add_scope("Other", None, 2);
        index.grants.insert((0, 1));
        module.friend_access.0.set(index).unwrap();
        let original = module.friend_access.0.get().unwrap() as *const Index;
        for _ in 0..100_000 {
            assert!(permits(
                &module,
                Some(("Friend", None)),
                Some(("Provider", None))
            ));
            assert!(!permits(
                &module,
                Some(("Other", None)),
                Some(("Provider", None))
            ));
        }
        assert_eq!(
            module.friend_access.0.get().unwrap() as *const Index,
            original
        );
        assert!(module.clone().friend_access.0.get().is_none());
    }
    #[test]
    fn ambiguous_scope_never_inherits_a_grant() {
        let mut index = Index::default();
        index.add_scope("Shared", Some("r1"), 0);
        index.add_scope("Shared", Some("r1"), 1);
        assert_eq!(index.assembly(Some(("Shared", Some("r1")))), None);
    }
}
