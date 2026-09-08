//! Single-threaded, non-moving tracing heap. Object identities are never reused.
use crate::{Fault, Value};
use std::collections::{BTreeMap, HashSet, VecDeque};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionReason {
    AllocationPressure,
    ExecutionCompleted,
}

/// A bounded history entry; roots count incoming edges, not unique allocations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollectionEvent {
    pub sequence: usize,
    pub reason: CollectionReason,
    pub roots: usize,
    pub before: usize,
    pub after: usize,
    pub reclaimed: usize,
}

#[derive(Debug, Default)]
pub struct ManagedHeap {
    objects: BTreeMap<usize, crate::slots::Cell>,
    next_identity: usize,
    collections: usize,
    reclaimed: usize,
    peak_objects: usize,
    events: VecDeque<CollectionEvent>,
}

impl ManagedHeap {
    pub(crate) fn debug_cells(&self) -> impl Iterator<Item = (&usize, &crate::slots::Cell)> {
        self.objects.iter()
    }
    /// Most recent 64 collections, in execution order. No guest values are retained.
    pub fn collection_events(&self) -> impl ExactSizeIterator<Item = &CollectionEvent> {
        self.events.iter()
    }
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
    /// Copy a live object's current value for host inspection without exposing
    /// a mutable cell or a borrow spanning execution.
    pub fn get(&self, identity: usize) -> Option<Value> {
        self.objects
            .get(&identity)
            .and_then(|cell| cell.borrow().get().ok())
    }
    /// Inspect a heap-backed reference, including an interior field, only within
    /// the execution that owns it. This does not create a guest invocation handle.
    pub fn read_reference(&self, reference: &crate::SlotReference) -> Result<Value, Fault> {
        let cell = reference
            .allocation_id()
            .and_then(|id| self.objects.get(&id))
            .ok_or_else(|| Fault::new("reference does not belong to this managed heap"))?;
        if !reference.belongs_to_heap_cell(cell) {
            return Err(Fault::new("reference does not belong to this managed heap"));
        }
        reference.read()
    }
    pub(crate) fn address(&self, identity: usize) -> Result<crate::SlotReference, Fault> {
        let cell = self
            .objects
            .get(&identity)
            .ok_or_else(|| Fault::new("invalid reference"))?;
        Ok(crate::SlotReference::heap(cell, identity))
    }
    pub fn collections(&self) -> usize {
        self.collections
    }
    pub fn reclaimed_objects(&self) -> usize {
        self.reclaimed
    }
    pub(crate) fn array_usage(
        &self,
        usage: &mut crate::arrays::Usage,
        limits: &crate::Limits,
    ) -> Result<(), Fault> {
        for cell in self.objects.values() {
            cell.borrow().array_usage(usage, limits)?;
        }
        Ok(())
    }
    pub(crate) fn allocate(&mut self, value: Value) -> Result<usize, Fault> {
        value.ensure_heap_references()?;
        let identity = self.next_identity;
        self.next_identity = identity
            .checked_add(1)
            .ok_or_else(|| Fault::new("managed heap identity budget exhausted"))?;
        self.objects
            .insert(identity, crate::slots::Slot::new(value.ty(), Some(value)));
        self.peak_objects = self.peak_objects.max(self.len());
        Ok(identity)
    }
    pub(crate) fn collect(
        &mut self,
        mut pending: Vec<usize>,
        reason: CollectionReason,
    ) -> Result<(), Fault> {
        let roots = pending.len();
        let mut live = HashSet::new();
        while let Some(identity) = pending.pop() {
            if !live.insert(identity) {
                continue;
            }
            let value = self
                .objects
                .get(&identity)
                .ok_or_else(|| Fault::new("invalid managed heap reference during collection"))?;
            value.borrow().trace_heap(&mut pending);
        }
        let before = self.len();
        self.objects.retain(|identity, _| live.contains(identity));
        self.reclaimed = self.reclaimed.saturating_add(before - self.len());
        self.collections = self.collections.saturating_add(1);
        if self.events.len() == 64 {
            self.events.pop_front();
        }
        self.events.push_back(CollectionEvent {
            sequence: self.collections,
            reason,
            roots,
            before,
            after: self.len(),
            reclaimed: before - self.len(),
        });
        Ok(())
    }
}

/// Traverse inline values without recursively traversing reference graphs.
/// Frame-backed targets are scanned through their owning frames. Heap-backed
/// references and interface views mark their owner, including interior references.
pub(crate) fn trace(value: &Value, references: &mut Vec<usize>) {
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        match value {
            Value::SlotReference(reference)
            | Value::SlotInterface {
                receiver: reference,
                ..
            } => {
                if let Some(identity) = reference.allocation_id() {
                    references.push(identity);
                }
            }
            Value::Object { fields, .. }
            | Value::Array {
                elements: fields, ..
            } => pending.extend(fields),
            Value::Erased(value) => pending.push(value),
            Value::Uninitialized(_)
            | Value::Void
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
            | Value::InterfaceRef { .. } => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_roots_do_not_partially_sweep_and_identities_are_not_reused() {
        let mut heap = ManagedHeap::default();
        let first = heap.allocate(Value::Int32(7)).unwrap();
        assert!(
            heap.collect(vec![first, 100], CollectionReason::AllocationPressure)
                .is_err()
        );
        assert_eq!(heap.get(first), Some(Value::Int32(7)));
        heap.collect(vec![], CollectionReason::AllocationPressure)
            .unwrap();
        let second = heap.allocate(Value::Int32(42)).unwrap();
        assert_ne!(first, second);
        assert!(heap.get(first).is_none());
    }
    #[test]
    fn cycles_are_traced_once_and_reclaimed_when_unreachable() {
        let mut heap = ManagedHeap::default();
        let first = heap.allocate(Value::Erased(Box::new(Value::Void))).unwrap();
        let second = heap
            .allocate(Value::Erased(Box::new(Value::SlotReference(
                heap.address(first).unwrap(),
            ))))
            .unwrap();
        heap.address(first)
            .unwrap()
            .write(Value::Erased(Box::new(Value::SlotReference(
                heap.address(second).unwrap(),
            ))))
            .unwrap();
        heap.collect(vec![first], CollectionReason::AllocationPressure)
            .unwrap();
        assert_eq!(heap.len(), 2);
        let observer = heap.address(first).unwrap();
        heap.collect(vec![], CollectionReason::AllocationPressure)
            .unwrap();
        assert!(observer.read().is_err());
        assert!(heap.is_empty());
        assert_eq!(heap.reclaimed_objects(), 2);
    }
}
