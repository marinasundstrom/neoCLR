//! Non-moving tracing heap. Object identities are never reused. The caller must
//! exclude graph mutation during collection; the VM uses shared_heap::Access.
use crate::{Fault, Value};
use std::collections::{BTreeMap, HashSet, VecDeque};

/// Collector selected before an execution heap is created. Only the existing
/// nonmoving mark/sweep implementation is included today; pacing is independent.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum GarbageCollector {
    #[default]
    MarkSweep,
}

impl GarbageCollector {
    /// Stable name accepted by the command-line selector.
    pub const fn name(self) -> &'static str {
        match self {
            Self::MarkSweep => "mark-sweep",
        }
    }

    /// Collectors included in this runtime build.
    pub const fn available() -> &'static [Self] {
        &[Self::MarkSweep]
    }
}

impl std::str::FromStr for GarbageCollector {
    type Err = String;
    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Self::available()
            .iter()
            .copied()
            .find(|gc| gc.name() == name)
            .ok_or_else(|| format!("Unavailable garbage collector '{name}'; available: mark-sweep"))
    }
}

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
    /// Synchronous collection requested through System.Runtime.GC.
    ExplicitRequest,
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
    collector: GarbageCollector,
    pub(crate) tls: crate::tls::Resources,
    objects: BTreeMap<usize, crate::slots::Cell>,
    next_identity: usize,
    collections: usize,
    reclaimed: usize,
    peak_objects: usize,
    events: VecDeque<CollectionEvent>,
}

/// Experimental dependent root: retain `value` only if `owner` is independently
/// marked. The value may refer back to its owner without making that cycle live.
/// The caller must remove entries whose owners were reclaimed after collection,
/// and must exclude graph mutation for the entire collection and cleanup.
#[cfg(test)]
pub(crate) struct ConditionalRoot<'a> {
    pub owner: &'a crate::SlotReference,
    pub value: &'a Value,
}

impl ManagedHeap {
    /// Create an empty heap with a collector fixed for its lifetime.
    pub fn with_collector(collector: GarbageCollector) -> Self {
        match collector {
            GarbageCollector::MarkSweep => Self {
                collector,
                ..Self::default()
            },
        }
    }
    /// Return this heap's selected collector.
    pub fn collector(&self) -> GarbageCollector {
        self.collector
    }

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
    pub(crate) fn contains(&self, identity: usize) -> bool {
        self.objects.contains_key(&identity)
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
        self.check_reference_owner(reference)?;
        reference.read()
    }
    pub(crate) fn check_reference_owner(
        &self,
        reference: &crate::SlotReference,
    ) -> Result<(), Fault> {
        let cell = reference
            .allocation_id()
            .and_then(|id| self.objects.get(&id))
            .ok_or_else(|| Fault::new("reference does not belong to this managed heap"))?;
        if !reference.belongs_to_heap_cell(cell) {
            return Err(Fault::new("reference does not belong to this managed heap"));
        }
        Ok(())
    }
    /// Validate native result provenance, including nested values and functions.
    /// Allocation IDs alone cannot distinguish two invocations with the same ID.
    pub(crate) fn value_roots(&self, value: &crate::Value) -> Result<Vec<usize>, Fault> {
        use crate::Value;
        value.ensure_heap_references()?;
        let mut pending = vec![value];
        let mut roots = Vec::new();
        while let Some(value) = pending.pop() {
            let reference = match value {
                Value::ObjectReference(object) => Some(&object.reference),
                Value::SlotReference(reference)
                | Value::SlotInterface {
                    receiver: reference,
                    ..
                } => Some(reference),
                Value::Function(function_object) => {
                    pending.extend(function_object.receiver.as_deref());
                    None
                }
                Value::Object { fields, .. }
                | Value::Array {
                    elements: fields, ..
                } => {
                    pending.extend(fields);
                    None
                }
                Value::Erased(value) => {
                    pending.push(value);
                    None
                }
                _ => None,
            };
            if let Some(reference) = reference {
                self.check_reference_owner(reference)?;
                roots.push(reference.allocation_id().expect("checked heap reference"));
            }
        }
        Ok(roots)
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
        self.next_identity = identity.checked_add(1).ok_or_else(|| {
            Fault::coded(
                crate::FaultCode::HeapLimitExceeded,
                "managed heap identity budget exhausted",
            )
        })?;
        self.objects
            .insert(identity, crate::slots::Slot::cell(value.ty(), Some(value)));
        self.peak_objects = self.peak_objects.max(self.len());
        Ok(identity)
    }
    #[cfg(test)]
    pub(crate) fn collect(
        &mut self,
        pending: Vec<usize>,
        reason: CollectionReason,
    ) -> Result<(), Fault> {
        self.collect_with_conditional_roots(pending, &[], reason)
    }

    /// Value/provenance adapter used by collector contract tests. Production
    /// participants publish the same conditional relationships as validated ids.
    #[cfg(test)]
    pub(crate) fn collect_with_conditional_roots(
        &mut self,
        pending: Vec<usize>,
        conditional: &[ConditionalRoot<'_>],
        reason: CollectionReason,
    ) -> Result<(), Fault> {
        // Index by owner so chains reach a fixed point through the mark worklist,
        // without repeatedly scanning every dormant registration. Validate all
        // owners before marking/sweeping, including unreachable and foreign ones.
        let mut dependents: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for entry in conditional {
            self.check_reference_owner(entry.owner)?;
            dependents
                .entry(entry.owner.allocation_id().expect("validated heap owner"))
                .or_default()
                .extend(self.value_roots(entry.value)?);
        }
        self.collect_with_dependencies(pending, dependents, reason)
    }

    pub(crate) fn collect_with_dependencies(
        &mut self,
        mut pending: Vec<usize>,
        mut dependents: BTreeMap<usize, Vec<usize>>,
        reason: CollectionReason,
    ) -> Result<(), Fault> {
        let roots = pending.len();
        for (&owner, values) in &dependents {
            if !self.objects.contains_key(&owner)
                || values.iter().any(|id| !self.objects.contains_key(id))
            {
                return Err(Fault::new("invalid conditional heap reference"));
            }
        }
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
            if let Some(values) = dependents.remove(&identity) {
                pending.extend(values);
            }
        }
        let before = self.len();
        self.objects.retain(|identity, _| live.contains(identity));
        self.tls.reap(&live);
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
            Value::Function(d) => pending.extend(d.receiver.as_deref()),
            Value::ObjectReference(object) => references.push(object.allocation_id()),
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
            | Value::NullObjectReference(_)
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
            | Value::RuntimeTypeHandle(_)
            | Value::Pointer(_)
            | Value::InterfaceRef { .. } => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference(heap: &ManagedHeap, identity: usize) -> Value {
        Value::SlotReference(heap.address(identity).unwrap())
    }

    #[test]
    fn conditional_back_reference_survives_only_while_owner_is_rooted() {
        let mut heap = ManagedHeap::default();
        let owner = heap.allocate(Value::Int32(42)).unwrap();
        let continuation = heap.allocate(reference(&heap, owner)).unwrap();
        let owner_ref = heap.address(owner).unwrap();
        let callback = reference(&heap, continuation);
        let entries = [ConditionalRoot {
            owner: &owner_ref,
            value: &callback,
        }];
        heap.collect_with_conditional_roots(
            vec![owner],
            &entries,
            CollectionReason::AllocationPressure,
        )
        .unwrap();
        assert_eq!(
            heap.len(),
            2,
            "live operation retains its waiting execution"
        );
        assert_eq!(heap.collection_events().last().unwrap().roots, 1);
        heap.collect_with_conditional_roots(vec![], &entries, CollectionReason::AllocationPressure)
            .unwrap();
        assert!(
            heap.is_empty(),
            "callback's back reference must not root its owner"
        );
        assert!(owner_ref.read().is_err());
    }

    #[test]
    fn conditional_chains_and_multiple_waiters_reach_a_fixed_point() {
        let mut heap = ManagedHeap::default();
        let first = heap.allocate(Value::Int32(1)).unwrap();
        let second = heap.allocate(Value::Int32(2)).unwrap();
        let last = heap.allocate(Value::Int32(3)).unwrap();
        let sibling = heap.allocate(Value::Int32(4)).unwrap();
        // The first continuation makes another operation reachable indirectly.
        let continuation = heap.allocate(reference(&heap, second)).unwrap();
        let first_ref = heap.address(first).unwrap();
        let second_ref = heap.address(second).unwrap();
        let continuation_ref = reference(&heap, continuation);
        let last_ref = reference(&heap, last);
        let sibling_ref = reference(&heap, sibling);
        // Deliberately reverse dependency order; a single linear scan is wrong.
        let entries = [
            ConditionalRoot {
                owner: &second_ref,
                value: &last_ref,
            },
            ConditionalRoot {
                owner: &first_ref,
                value: &continuation_ref,
            },
            ConditionalRoot {
                owner: &first_ref,
                value: &sibling_ref,
            },
        ];
        heap.collect_with_conditional_roots(
            vec![first],
            &entries,
            CollectionReason::AllocationPressure,
        )
        .unwrap();
        assert_eq!(heap.len(), 5);
        heap.collect_with_conditional_roots(vec![], &entries, CollectionReason::AllocationPressure)
            .unwrap();
        assert!(heap.is_empty());
    }

    #[test]
    fn unrooted_mutually_dependent_owners_do_not_resurrect_each_other() {
        let mut heap = ManagedHeap::default();
        let first = heap.allocate(Value::Int32(1)).unwrap();
        let second = heap.allocate(Value::Int32(2)).unwrap();
        let first_ref = heap.address(first).unwrap();
        let second_ref = heap.address(second).unwrap();
        let first_value = reference(&heap, first);
        let second_value = reference(&heap, second);
        let entries = [
            ConditionalRoot {
                owner: &first_ref,
                value: &second_value,
            },
            ConditionalRoot {
                owner: &second_ref,
                value: &first_value,
            },
        ];
        heap.collect_with_conditional_roots(vec![], &entries, CollectionReason::AllocationPressure)
            .unwrap();
        assert!(heap.is_empty());
    }

    #[test]
    fn independently_rooted_continuation_retains_its_operation() {
        let mut heap = ManagedHeap::default();
        let owner = heap.allocate(Value::Int32(42)).unwrap();
        let continuation = heap.allocate(reference(&heap, owner)).unwrap();
        let owner_ref = heap.address(owner).unwrap();
        let callback = reference(&heap, continuation);
        let entries = [ConditionalRoot {
            owner: &owner_ref,
            value: &callback,
        }];
        // Models a runnable callback or a host-rooted execution, not a dormant
        // registration. Such roots must remain strong even with conditional edges.
        heap.collect_with_conditional_roots(
            vec![continuation],
            &entries,
            CollectionReason::AllocationPressure,
        )
        .unwrap();
        assert_eq!(heap.len(), 2);
    }

    #[test]
    fn conditional_owner_validation_is_heap_specific_and_failure_is_atomic() {
        let mut heap = ManagedHeap::default();
        let mut foreign = ManagedHeap::default();
        let local_id = heap.allocate(Value::Int32(1)).unwrap();
        let foreign_id = foreign.allocate(Value::Int32(2)).unwrap();
        assert_eq!(
            local_id, foreign_id,
            "numeric identities alone are insufficient"
        );
        let foreign_ref = foreign.address(foreign_id).unwrap();
        let value = Value::Void;
        let before = heap.statistics();
        assert!(
            heap.collect_with_conditional_roots(
                vec![],
                &[ConditionalRoot {
                    owner: &foreign_ref,
                    value: &value
                }],
                CollectionReason::AllocationPressure,
            )
            .is_err()
        );
        assert_eq!(heap.statistics(), before);
        assert_eq!(heap.collection_events().len(), 0);

        let local_ref = heap.address(local_id).unwrap();
        let foreign_value = Value::Erased(Box::new(reference(&foreign, foreign_id)));
        assert!(
            heap.collect_with_conditional_roots(
                vec![local_id],
                &[ConditionalRoot {
                    owner: &local_ref,
                    value: &foreign_value
                }],
                CollectionReason::AllocationPressure,
            )
            .is_err(),
            "dependent values also require heap provenance, even when nested"
        );
        assert_eq!(heap.statistics(), before);

        let expired_ref = heap.address(local_id).unwrap();
        heap.collect(vec![], CollectionReason::AllocationPressure)
            .unwrap();
        let before = heap.statistics();
        assert!(
            heap.collect_with_conditional_roots(
                vec![],
                &[ConditionalRoot {
                    owner: &expired_ref,
                    value: &value
                }],
                CollectionReason::AllocationPressure,
            )
            .is_err(),
            "callers must prune expired entries before the next collection"
        );
        assert_eq!(heap.statistics(), before);
    }

    #[test]
    fn invalid_conditional_value_does_not_partially_sweep() {
        let mut heap = ManagedHeap::default();
        let owner = heap.allocate(Value::Int32(1)).unwrap();
        let expired = heap.allocate(Value::Int32(2)).unwrap();
        let stale_value = reference(&heap, expired);
        heap.collect(vec![owner], CollectionReason::AllocationPressure)
            .unwrap();
        let owner_ref = heap.address(owner).unwrap();
        let before = heap.statistics();
        assert!(
            heap.collect_with_conditional_roots(
                vec![owner],
                &[ConditionalRoot {
                    owner: &owner_ref,
                    value: &stale_value
                }],
                CollectionReason::AllocationPressure,
            )
            .is_err()
        );
        assert_eq!(heap.statistics(), before);
        assert!(heap.get(owner).is_some());
    }

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
