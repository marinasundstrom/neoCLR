//! Single-threaded, non-moving tracing heap. Object identities are never reused.
use crate::{Fault, Value};
use std::collections::{BTreeMap, HashSet};

/// Object-count diagnostics for one execution, including its final collection.
/// These counts exclude inline values and separately tracked native allocations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GcStatistics {
    pub allocated_objects: usize,
    pub live_objects: usize,
    pub peak_objects: usize,
    pub collections: usize,
    pub reclaimed_objects: usize,
}

#[derive(Debug, Default)]
pub struct ManagedHeap {
    objects: BTreeMap<usize, Value>,
    next_identity: usize,
    collections: usize,
    reclaimed: usize,
    peak_objects: usize,
}

impl ManagedHeap {
    pub fn statistics(&self) -> GcStatistics {
        GcStatistics {
            allocated_objects: self.next_identity,
            live_objects: self.len(),
            peak_objects: self.peak_objects,
            collections: self.collections,
            reclaimed_objects: self.reclaimed,
        }
    }
    pub fn len(&self) -> usize {
        self.objects.len()
    }
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }
    /// Look up a live object by the identity carried by a prototype Reference.
    pub fn get(&self, identity: usize) -> Option<&Value> {
        self.objects.get(&identity)
    }
    pub fn collections(&self) -> usize {
        self.collections
    }
    pub fn reclaimed_objects(&self) -> usize {
        self.reclaimed
    }
    pub(crate) fn get_mut(&mut self, identity: usize) -> Option<&mut Value> {
        self.objects.get_mut(&identity)
    }
    pub(crate) fn allocate(&mut self, value: Value) -> Result<usize, Fault> {
        let identity = self.next_identity;
        self.next_identity = identity
            .checked_add(1)
            .ok_or_else(|| Fault::new("managed heap identity budget exhausted"))?;
        self.objects.insert(identity, value);
        self.peak_objects = self.peak_objects.max(self.len());
        Ok(identity)
    }
    pub(crate) fn collect(&mut self, mut pending: Vec<usize>) -> Result<(), Fault> {
        let mut live = HashSet::new();
        while let Some(identity) = pending.pop() {
            if !live.insert(identity) {
                continue;
            }
            let value = self
                .objects
                .get(&identity)
                .ok_or_else(|| Fault::new("invalid managed heap reference during collection"))?;
            trace(value, &mut pending);
        }
        let before = self.len();
        self.objects.retain(|identity, _| live.contains(identity));
        self.reclaimed = self.reclaimed.saturating_add(before - self.len());
        self.collections = self.collections.saturating_add(1);
        Ok(())
    }
}

/// Traverse inline values without recursively traversing reference graphs.
/// Slot-reference targets currently always belong to active frames, whose cells
/// are enumerated separately. Heap-backed ByRef will need a root edge here.
pub(crate) fn trace(value: &Value, references: &mut Vec<usize>) {
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        match value {
            Value::Reference { index, .. } => references.push(*index),
            Value::Object { fields, .. } => pending.extend(fields),
            Value::Erased(value) => pending.push(value),
            Value::Void
            | Value::Single(_)
            | Value::Double(_)
            | Value::Int32(_)
            | Value::SByte(_)
            | Value::Byte(_)
            | Value::Int16(_)
            | Value::UInt16(_)
            | Value::Char(_)
            | Value::UInt32(_)
            | Value::Int64(_)
            | Value::UInt64(_)
            | Value::IntPtr(_)
            | Value::UIntPtr(_)
            | Value::Boolean(_)
            | Value::String(_)
            | Value::Error(_)
            | Value::RuntimeTypeHandle(_)
            | Value::Pointer(_)
            | Value::InterfaceRef { .. }
            | Value::SlotReference(_)
            | Value::SlotInterface { .. } => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::Type;
    #[test]
    fn invalid_roots_do_not_partially_sweep_and_identities_are_not_reused() {
        let mut heap = ManagedHeap::default();
        let first = heap.allocate(Value::Int32(7)).unwrap();
        assert!(heap.collect(vec![first, 100]).is_err());
        assert_eq!(heap.get(first), Some(&Value::Int32(7)));
        heap.collect(vec![]).unwrap();
        let second = heap.allocate(Value::Int32(42)).unwrap();
        assert_ne!(first, second);
        assert!(heap.get(first).is_none());
    }
    #[test]
    fn cycles_are_traced_once_and_reclaimed_when_unreachable() {
        let mut heap = ManagedHeap::default();
        let first = heap.allocate(Value::Void).unwrap();
        let second = heap
            .allocate(Value::Reference {
                index: first,
                target: Type::Void,
            })
            .unwrap();
        *heap.get_mut(first).unwrap() = Value::Reference {
            index: second,
            target: Type::Void,
        };
        heap.collect(vec![first]).unwrap();
        assert_eq!(heap.len(), 2);
        heap.collect(vec![]).unwrap();
        assert!(heap.is_empty());
        assert_eq!(heap.reclaimed_objects(), 2);
    }
}
