//! Invocation-owned statics. The key is the resolved closed type, not its spelling.
//! Lock order: managed graph access, then this table. Never wait while holding either.
use crate::{Fault, Module, Value, metadata::Type, type_identity::TypeIdentity};
use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct Storage {
    entries: HashMap<TypeIdentity, Vec<Value>>,
    pub(crate) initialization: crate::type_initialization::Registry<TypeIdentity>,
    roots: Option<crate::shared_heap::RootSet>,
    heap: Option<crate::shared_heap::Identity>,
}

impl Storage {
    fn bind(&mut self, heap: &mut crate::shared_heap::Access<'_>) -> Result<(), Fault> {
        if let Some(identity) = &self.heap {
            if !identity.matches(heap) {
                return Err(Fault::new(
                    "static storage belongs to another invocation heap",
                ));
            }
        } else {
            self.roots = Some(heap.service_roots());
            self.heap = Some(heap.identity());
        }
        Ok(())
    }

    fn publish(
        &self,
        heap: &mut crate::shared_heap::Access<'_>,
        limits: &crate::Limits,
    ) -> Result<(), Fault> {
        let mut roots = Vec::new();
        let mut usage = crate::arrays::Usage::default();
        for fields in self.entries.values() {
            for value in fields {
                crate::gc::trace(value, &mut roots);
                crate::arrays::measure(value, &mut usage, limits)?;
            }
        }
        self.roots
            .as_ref()
            .expect("bound static roots")
            .publish_payload(heap, roots, usage)
    }
    fn key(
        &mut self,
        module: &Module,
        owner: &Type,
        heap: &mut crate::shared_heap::Access<'_>,
        limits: &crate::Limits,
    ) -> Result<TypeIdentity, Fault> {
        let key = crate::type_identity::resolve(module, owner)?;
        if !matches!(key, TypeIdentity::Definition { .. }) {
            return Err(Fault::new("static storage requires a closed nominal type"));
        }
        self.bind(heap)?;
        if !self.entries.contains_key(&key) {
            if self.entries.len() >= limits.heap_objects {
                return Err(Fault::new("static storage type limit exceeded"));
            }
            let definition = module
                .type_definition(owner)
                .ok_or_else(|| Fault::new("missing static owner"))?;
            let fields = definition
                .static_fields
                .iter()
                .map(|f| {
                    let ty = f.ty.substitute_type_parameters(owner.generic_arguments())?;
                    if matches!(ty, Type::Function(_)) {
                        crate::initialization::default_value(module, &ty)
                    } else {
                        crate::initialization::object_field_value(module, &ty)
                    }
                })
                .collect::<Result<Vec<_>, Fault>>()?;
            self.entries.insert(key.clone(), fields);
        }
        Ok(key)
    }
    pub(crate) fn begin_initialization(
        &mut self,
        module: &Module,
        owner: &Type,
        execution: &std::sync::Arc<()>,
        heap: &mut crate::shared_heap::Access<'_>,
        limits: &crate::Limits,
    ) -> Result<crate::type_initialization::Begin, Fault> {
        let key = self.key(module, owner, heap, limits)?;
        self.initialization.begin(&key, execution)
    }
    pub(crate) fn read(
        &mut self,
        module: &Module,
        owner: &Type,
        index: usize,
        heap: &mut crate::shared_heap::Access<'_>,
        limits: &crate::Limits,
    ) -> Result<Value, Fault> {
        let key = self.key(module, owner, heap, limits)?;
        let value = self.entries[&key]
            .get(index)
            .ok_or_else(|| Fault::new("static field index out of range"))?;
        if matches!(value, Value::Uninitialized(_)) {
            return Err(Fault::new("static field is uninitialized"));
        }
        Ok(value.clone())
    }
    pub(crate) fn write(
        &mut self,
        module: &Module,
        owner: &Type,
        index: usize,
        value: Value,
        heap: &mut crate::shared_heap::Access<'_>,
        limits: &crate::Limits,
    ) -> Result<(), Fault> {
        let key = self.key(module, owner, heap, limits)?;
        let destination = self
            .entries
            .get_mut(&key)
            .and_then(|fields| fields.get_mut(index))
            .ok_or_else(|| Fault::new("static field index out of range"))?;
        let old = std::mem::replace(destination, value);
        if let Err(error) = self.publish(heap, limits) {
            self.entries.get_mut(&key).unwrap()[index] = old;
            return Err(error);
        }
        Ok(())
    }
}
