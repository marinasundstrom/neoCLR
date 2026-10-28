//! Initialization ownership is separate from storage allocation. Callers must
//! allocate defaults before starting and must release the graph gate before waiting.
use crate::{
    Fault, Module,
    metadata::{Function, FunctionRef, Representation, Type, Visibility},
};
use std::{collections::HashMap, hash::Hash, sync::Arc};

pub(crate) fn is_initializer(function: &Function) -> bool {
    function.name.ends_with("..cctor")
}

pub(crate) fn initializer(module: &Module, owner: &Type) -> Result<Option<Function>, Fault> {
    let definition = module
        .type_definition(owner)
        .ok_or_else(|| Fault::new("missing initialization owner"))?;
    let name = format!("{}..cctor", definition.name);
    let matches_owner = |f: &&Function| {
        f.name == name
            && f.owner
                .as_ref()
                .and_then(|t| module.type_definition(t))
                .is_some_and(|d| std::ptr::eq(d, definition))
    };
    let candidate = if let Some(indexes) = crate::runtime_lookup::functions(module, &name) {
        indexes
            .iter()
            .map(|i| &module.functions[*i])
            .find(matches_owner)
    } else {
        module.functions.iter().find(matches_owner)
    };
    candidate
        .map(|f| {
            crate::vm::resolve(
                module,
                &FunctionRef {
                    definition: f.definition.clone(),
                    name: f.name.clone(),
                    owner: Some(owner.clone()),
                    instance: false,
                    generic_arguments: vec![],
                    parameters: vec![],
                },
            )
        })
        .transpose()
}

pub(crate) fn entry_owner<'a>(module: &Module, function: &'a Function) -> Option<&'a Type> {
    if is_initializer(function) {
        return None;
    }
    let owner = function.owner.as_ref()?;
    let definition = module.type_definition(owner)?;
    (!definition.before_field_init
        && (!function.instance
            || (definition.representation == Representation::Record
                && !definition.is_reference_type)
            || function.name.ends_with("..ctor")))
    .then_some(owner)
}

pub(crate) fn validate(module: &Module) -> Result<(), Fault> {
    let mut owners = std::collections::HashSet::new();
    for f in module.functions.iter().filter(|f| is_initializer(f)) {
        let owner = f
            .owner
            .as_ref()
            .ok_or_else(|| Fault::new("type initializer requires an owner"))?;
        let d = module
            .type_definition(owner)
            .ok_or_else(|| Fault::new("type initializer owner is missing"))?;
        if !owners.insert((&d.name, d.generic_parameters.len()))
            || f.instance
            || f.visibility != Visibility::Private
            || !f.parameters.is_empty()
            || !f.generic_parameters.is_empty()
            || f.returns != Type::Void
            || !f.no_result
            || f.receiver_byref
            || f.receiver_readonly
            || f.is_virtual
            || f.is_override
            || f.is_abstract
            || f.impl_flags & !crate::metadata::INLINING_FLAGS != 0
            || f.pinvoke.is_some()
            || !f.interface_implementations.is_empty()
            || d.primitive_reference
            || matches!(owner, Type::ArrayRef(_))
            || !matches!(
                d.representation,
                Representation::Record | Representation::Interface
            )
        {
            return Err(Fault::new(
                "type initializer requires a unique private static parameterless IL no-result signature",
            ));
        }
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Begin {
    Run,
    Ready,
    Recursive,
    Wait,
}
enum State {
    Running(Arc<()>),
    Complete,
    Failed(Fault),
}

pub(crate) struct Registry<K> {
    states: HashMap<K, State>,
    // Wait edges retain both identities so address reuse cannot create false cycles.
    waits: HashMap<usize, (Arc<()>, Arc<()>, K)>,
}
impl<K> Default for Registry<K> {
    fn default() -> Self {
        Self {
            states: HashMap::new(),
            waits: HashMap::new(),
        }
    }
}
fn id(owner: &Arc<()>) -> usize {
    Arc::as_ptr(owner) as usize
}
impl<K: Eq + Hash + Clone> Registry<K> {
    pub(crate) fn begin(&mut self, key: &K, owner: &Arc<()>) -> Result<Begin, Fault> {
        self.waits.remove(&id(owner));
        match self.states.get(key) {
            None => {
                self.states
                    .insert(key.clone(), State::Running(owner.clone()));
                Ok(Begin::Run)
            }
            Some(State::Complete) => Ok(Begin::Ready),
            Some(State::Failed(fault)) => Err(fault.clone()),
            Some(State::Running(running)) => {
                let mut cursor = id(running);
                loop {
                    if cursor == id(owner) {
                        return Ok(Begin::Recursive);
                    }
                    match self.waits.get(&cursor) {
                        Some((_, next, _)) => cursor = id(next),
                        None => break,
                    }
                }
                self.waits
                    .insert(id(owner), (owner.clone(), running.clone(), key.clone()));
                Ok(Begin::Wait)
            }
        }
    }
    pub(crate) fn complete(&mut self, key: &K, owner: &Arc<()>) -> Result<(), Fault> {
        if !matches!(self.states.get(key), Some(State::Running(running)) if Arc::ptr_eq(running, owner))
        {
            return Err(Fault::new(
                "type initializer completed without owning initialization",
            ));
        }
        self.states.insert(key.clone(), State::Complete);
        // Only waiters for this type are released; nested initializers remain active.
        self.waits.retain(|_, (_, _, waiting)| waiting != key);
        Ok(())
    }
    pub(crate) fn fail_owner(&mut self, owner: &Arc<()>, fault: &Fault) {
        for state in self.states.values_mut() {
            if matches!(state, State::Running(running) if Arc::ptr_eq(running, owner)) {
                *state = State::Failed(fault.clone());
            }
        }
        self.waits.retain(|_, (source, target, _)| {
            !Arc::ptr_eq(source, owner) && !Arc::ptr_eq(target, owner)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn once_only_recursion_failure_and_independent_closed_types() {
        let mut r = Registry::default();
        let a = Arc::new(());
        let b = Arc::new(());
        assert_eq!(r.begin(&"Cache<int>", &a).unwrap(), Begin::Run);
        assert_eq!(r.begin(&"Cache<int>", &a).unwrap(), Begin::Recursive);
        assert_eq!(r.begin(&"Cache<int>", &b).unwrap(), Begin::Wait);
        assert_eq!(r.begin(&"Cache<string>", &b).unwrap(), Begin::Run);
        assert!(r.complete(&"Cache<int>", &b).is_err());
        r.complete(&"Cache<int>", &a).unwrap();
        assert_eq!(r.begin(&"Cache<int>", &b).unwrap(), Begin::Ready);
        let fault = Fault::coded(crate::FaultCode::ArithmeticOverflow, "initializer overflow");
        r.fail_owner(&b, &fault);
        assert_eq!(r.begin(&"Cache<string>", &a).unwrap_err(), fault);
        assert_eq!(r.begin(&"Cache<string>", &b).unwrap_err(), fault);
        assert_eq!(r.begin(&"Cache<int>", &a).unwrap(), Begin::Ready);
    }
    #[test]
    fn completion_of_nested_type_keeps_other_wait_edges() {
        let mut r = Registry::default();
        let a = Arc::new(());
        let b = Arc::new(());
        assert_eq!(r.begin(&"Outer", &a).unwrap(), Begin::Run);
        assert_eq!(r.begin(&"Inner", &a).unwrap(), Begin::Run);
        assert_eq!(r.begin(&"Other", &b).unwrap(), Begin::Run);
        assert_eq!(r.begin(&"Outer", &b).unwrap(), Begin::Wait);
        r.complete(&"Inner", &a).unwrap();
        assert_eq!(r.begin(&"Other", &a).unwrap(), Begin::Recursive);
        r.complete(&"Outer", &a).unwrap();
        assert_eq!(r.begin(&"Outer", &b).unwrap(), Begin::Ready);
    }
    #[test]
    fn cross_owner_cycles_do_not_deadlock_or_publish_completion() {
        let mut r = Registry::default();
        let a = Arc::new(());
        let b = Arc::new(());
        let c = Arc::new(());
        assert_eq!(r.begin(&"A", &a).unwrap(), Begin::Run);
        assert_eq!(r.begin(&"B", &b).unwrap(), Begin::Run);
        assert_eq!(r.begin(&"C", &c).unwrap(), Begin::Run);
        assert_eq!(r.begin(&"B", &a).unwrap(), Begin::Wait);
        assert_eq!(r.begin(&"C", &b).unwrap(), Begin::Wait);
        assert_eq!(r.begin(&"A", &c).unwrap(), Begin::Recursive);
        r.complete(&"C", &c).unwrap();
        assert_eq!(r.begin(&"C", &b).unwrap(), Begin::Ready);
        r.complete(&"B", &b).unwrap();
        assert_eq!(r.begin(&"B", &a).unwrap(), Begin::Ready);
        assert_eq!(r.begin(&"A", &c).unwrap(), Begin::Wait);
        r.complete(&"A", &a).unwrap();
        assert_eq!(r.begin(&"A", &c).unwrap(), Begin::Ready);
    }
}
