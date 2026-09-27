//! Invocation-owned heap coordination. All graph access must hold an Access guard.
//! A participant publishes its roots before releasing access, including before
//! blocking or transferring execution to another native thread. This protocol is
//! internal: a Send SlotReference by itself does not authorize concurrent GC access.
use crate::{CollectionReason, Fault, ManagedHeap};
use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, MutexGuard, Weak},
};

struct Registration {
    roots: Mutex<Vec<usize>>,
    arrays: Mutex<crate::arrays::Usage>,
}
struct State {
    heap: ManagedHeap,
    arrays_used: bool,
    participants: Vec<Weak<Registration>>,
    service_roots: Vec<Weak<Registration>>,
}
struct Shared {
    state: Mutex<State>,
    limit: usize,
}

pub(crate) struct Owner(Arc<Shared>);
#[derive(Clone)]
pub(crate) struct Identity(Weak<Shared>);
impl Identity {
    pub(crate) fn matches(&self, access: &Access<'_>) -> bool {
        Weak::ptr_eq(&self.0, &Arc::downgrade(access.shared))
    }
}
pub(crate) struct Participant {
    shared: Arc<Shared>,
    registration: Arc<Registration>,
}
/// Invocation services keep roots independently of any submitting guest context.
/// This owner holds the heap alive but consumes no executable participant slot.
pub(crate) struct RootSet {
    shared: Arc<Shared>,
    registration: Arc<Registration>,
}
impl RootSet {
    pub(crate) fn publish(&self, access: &mut Access<'_>, roots: Vec<usize>) -> Result<(), Fault> {
        if !Arc::ptr_eq(&self.shared, access.shared) {
            return Err(Fault::new(
                "service roots belong to another invocation heap",
            ));
        }
        validate(&access.state.heap, &roots)?;
        *self
            .registration
            .roots
            .lock()
            .expect("service root lock poisoned") = roots;
        Ok(())
    }
}

pub(crate) struct Access<'a> {
    shared: &'a Arc<Shared>,
    registration: &'a Registration,
    state: MutexGuard<'a, State>,
}

impl Owner {
    pub(crate) fn new(limit: usize) -> Self {
        Self(Arc::new(Shared {
            state: Mutex::new(State {
                heap: ManagedHeap::default(),
                arrays_used: false,
                participants: Vec::new(),
                service_roots: Vec::new(),
            }),
            limit,
        }))
    }

    pub(crate) fn participant(&self) -> Result<Participant, Fault> {
        let mut state = self.0.state.lock().expect("shared heap lock poisoned");
        register(&self.0, &mut state, Vec::new())
    }

    /// Teardown may export the heap only after all participants and service roots have been dropped.
    /// A rejected export leaves the heap owned by the outstanding participants.
    pub(crate) fn into_heap(self) -> Result<ManagedHeap, Fault> {
        let shared = Arc::try_unwrap(self.0).map_err(|_| {
            Fault::new("shared heap still has active participants or service roots")
        })?;
        Ok(shared
            .state
            .into_inner()
            .expect("shared heap lock poisoned")
            .heap)
    }
}

impl Participant {
    /// Native work must not wait indefinitely for the graph gate after teardown
    /// requests cancellation. No guest work runs while waiting for this lock.
    pub(crate) fn enter_cancellable(
        &mut self,
        cancelled: impl Fn() -> bool,
    ) -> Result<Access<'_>, Fault> {
        loop {
            if cancelled() {
                return Err(Fault::coded(
                    crate::FaultCode::ExecutionCancelled,
                    "heap access cancelled",
                ));
            }
            match self.shared.state.try_lock() {
                Ok(state) => {
                    return Ok(Access {
                        shared: &self.shared,
                        registration: &self.registration,
                        state,
                    });
                }
                Err(std::sync::TryLockError::WouldBlock) => {
                    std::thread::park_timeout(std::time::Duration::from_millis(1))
                }
                Err(std::sync::TryLockError::Poisoned(_)) => panic!("shared heap lock poisoned"),
            }
        }
    }
    pub(crate) fn enter(&mut self) -> Access<'_> {
        Access {
            shared: &self.shared,
            registration: &self.registration,
            state: self.shared.state.lock().expect("shared heap lock poisoned"),
        }
    }
}

fn validate(heap: &ManagedHeap, roots: &[usize]) -> Result<(), Fault> {
    for root in roots {
        // Root IDs come from this invocation's trusted VM root walker, not guests
        // or other heaps. Existence validation does not establish provenance.
        heap.address(*root)?;
    }
    Ok(())
}

fn register(
    shared: &Arc<Shared>,
    state: &mut State,
    roots: Vec<usize>,
) -> Result<Participant, Fault> {
    validate(&state.heap, &roots)?;
    state.participants.retain(|entry| entry.strong_count() != 0);
    if state.participants.len() >= shared.limit {
        return Err(Fault::new("shared heap participant limit exceeded"));
    }
    let registration = Arc::new(Registration {
        roots: Mutex::new(roots),
        arrays: Mutex::new(Default::default()),
    });
    state.participants.push(Arc::downgrade(&registration));
    Ok(Participant {
        shared: shared.clone(),
        registration,
    })
}

impl Access<'_> {
    pub(crate) fn arrays_used(&self) -> bool {
        self.state.arrays_used
    }

    /// Inline payloads are private to a parked participant. Heap payloads are
    /// measured separately once, regardless of the number of reference aliases.
    pub(crate) fn publish_arrays(&mut self, usage: crate::arrays::Usage, enabled: bool) {
        self.state.arrays_used |= enabled || !usage.is_empty();
        *self.registration.arrays.lock().expect("array accounting lock poisoned") = usage;
    }

    pub(crate) fn check_arrays(
        &self,
        mut current: crate::arrays::Usage,
        limits: &crate::Limits,
    ) -> Result<(), Fault> {
        for entry in &self.state.participants {
            if let Some(entry) = entry.upgrade() {
                if !std::ptr::eq(entry.as_ref(), self.registration) {
                    current.add(
                        *entry.arrays.lock().expect("array accounting lock poisoned"),
                        limits,
                    )?;
                }
            }
        }
        self.state.heap.array_usage(&mut current, limits)
    }

    pub(crate) fn service_roots(&mut self) -> RootSet {
        self.state
            .service_roots
            .retain(|entry| entry.strong_count() != 0);
        let registration = Arc::new(Registration {
            roots: Mutex::new(Vec::new()),
            arrays: Mutex::new(Default::default()),
        });
        self.state.service_roots.push(Arc::downgrade(&registration));
        RootSet {
            shared: self.shared.clone(),
            registration,
        }
    }

    pub(crate) fn identity(&self) -> Identity {
        Identity(Arc::downgrade(self.shared))
    }

    /// Validate native result provenance, including nested values and delegates.
    /// Allocation IDs alone cannot distinguish two invocations with the same ID.
    fn value_roots(&self, value: &crate::Value) -> Result<Vec<usize>, Fault> {
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
                Value::Delegate(delegate) => {
                    pending.extend(delegate.receiver.as_deref());
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
                self.state.heap.check_reference_owner(reference)?;
                roots.push(reference.allocation_id().expect("checked heap reference"));
            }
        }
        Ok(roots)
    }

    /// Captures must be actual values so provenance and inline payload cannot be
    /// omitted by a submitting VM. Its own roots/usage must already be current.
    pub(crate) fn fork_values(
        &mut self,
        values: &[crate::Value],
        limits: &crate::Limits,
    ) -> Result<Participant, Fault> {
        let mut roots = Vec::new();
        let mut usage = crate::arrays::Usage::default();
        for value in values {
            roots.extend(self.value_roots(value)?);
            crate::arrays::measure(value, &mut usage, limits)?;
        }
        let participant = register(self.shared, &mut self.state, roots)?;
        *participant
            .registration
            .arrays
            .lock()
            .expect("array accounting lock poisoned") = usage;
        self.state.arrays_used |= !usage.is_empty();
        let current = *self
            .registration
            .arrays
            .lock()
            .expect("array accounting lock poisoned");
        self.check_arrays_collecting(current, limits)?;
        Ok(participant)
    }

    pub(crate) fn publish_payload(
        &mut self,
        value: &crate::Value,
        limits: &crate::Limits,
    ) -> Result<(), Fault> {
        let roots = self.value_roots(value)?;
        let mut usage = crate::arrays::Usage::default();
        crate::arrays::measure(value, &mut usage, limits)?;
        self.publish(roots)?;
        self.publish_arrays(usage, false);
        self.check_arrays_collecting(usage, limits)
    }

    fn check_arrays_collecting(
        &mut self,
        usage: crate::arrays::Usage,
        limits: &crate::Limits,
    ) -> Result<(), Fault> {
        if self.check_arrays(usage, limits).is_err() {
            let roots = self
                .registration
                .roots
                .lock()
                .expect("root lock poisoned")
                .clone();
            self.collect(roots, CollectionReason::AllocationPressure)?;
            self.check_arrays(usage, limits)?;
        }
        Ok(())
    }

    /// Move a completion's private charge without counting it twice. The caller
    /// must publish its current private usage before receiving, just as for roots.
    pub(crate) fn receive_payload(
        &mut self,
        producer: &Participant,
        value: &crate::Value,
        mut roots: Vec<usize>,
        limits: &crate::Limits,
    ) -> Result<(), Fault> {
        if !Arc::ptr_eq(&producer.shared, self.shared)
            || std::ptr::eq(producer.registration.as_ref(), self.registration)
        {
            return Err(Fault::new(
                "completion requires a distinct participant in the same heap",
            ));
        }
        roots.extend(self.value_roots(value)?);
        let current = *self
            .registration
            .arrays
            .lock()
            .expect("array accounting lock poisoned");
        let transferred = *producer
            .registration
            .arrays
            .lock()
            .expect("array accounting lock poisoned");
        let mut combined = current;
        combined.add(transferred, limits)?;
        self.publish(roots)?;
        self.check_arrays_collecting(current, limits)?;
        self.publish_arrays(combined, false);
        *producer
            .registration
            .arrays
            .lock()
            .expect("array accounting lock poisoned") = Default::default();
        Ok(())
    }

    /// Publish queued captures before the submitting participant can release them.
    /// Dropping a rejected/unsubmitted participant never reacquires the heap lock.
    #[cfg(test)]
    pub(crate) fn fork(&mut self, roots: Vec<usize>) -> Result<Participant, Fault> {
        register(self.shared, &mut self.state, roots)
    }

    /// The root walker must include every live frame, continuation and result.
    /// Call before releasing this guard whenever work can resume later.
    pub(crate) fn publish(&mut self, roots: Vec<usize>) -> Result<(), Fault> {
        validate(&self.state.heap, &roots)?;
        *self.registration.roots.lock().expect("root lock poisoned") = roots;
        Ok(())
    }

    pub(crate) fn collect(
        &mut self,
        roots: Vec<usize>,
        reason: CollectionReason,
    ) -> Result<(), Fault> {
        self.publish(roots)?;
        let mut all = Vec::new();
        self.state.participants.retain(|entry| {
            if let Some(entry) = entry.upgrade() {
                all.extend_from_slice(&entry.roots.lock().expect("root lock poisoned"));
                true
            } else {
                false
            }
        });
        self.state.service_roots.retain(|entry| {
            if let Some(entry) = entry.upgrade() {
                all.extend_from_slice(&entry.roots.lock().expect("service root lock poisoned"));
                true
            } else {
                false
            }
        });
        self.state.heap.collect(all, reason)
    }
}

// Existing allocation/inspection helpers operate on ManagedHeap. The VM must use
// Access::collect for collection, never dereference to bypass registered roots.
impl Deref for Access<'_> {
    type Target = ManagedHeap;
    fn deref(&self) -> &Self::Target {
        &self.state.heap
    }
}
impl DerefMut for Access<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state.heap
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Value, metadata::Type};

    fn object(access: &mut Access<'_>, value: i32) -> usize {
        access
            .allocate(Value::Object {
                ty: Type::from_name("Capture"),
                fields: vec![Value::Int32(value)],
            })
            .unwrap()
    }

    #[test]
    fn queued_capture_survives_parent_release_and_transfers_to_native_execution() {
        let owner = Owner::new(2);
        let mut parent = owner.participant().unwrap();
        let (mut queued, id) = {
            let mut access = parent.enter();
            let id = object(&mut access, 1);
            let queued = access.fork(vec![id]).unwrap();
            access
                .collect(vec![], CollectionReason::ExplicitRequest)
                .unwrap();
            assert_eq!(access.len(), 1);
            (queued, id)
        };
        let done = std::thread::spawn(move || {
            {
                let mut access = queued.enter();
                access
                    .address(id)
                    .unwrap()
                    .write_field(0, Type::Int32, Value::Int32(42))
                    .unwrap();
                access.publish(vec![id]).unwrap();
            }
            queued // Completion retains the result until its consumer takes it.
        })
        .join()
        .unwrap();
        {
            let mut access = parent.enter();
            access
                .collect(vec![], CollectionReason::ExplicitRequest)
                .unwrap();
            assert_eq!(
                access.address(id).unwrap().read_field(0).unwrap(),
                Value::Int32(42)
            );
            access.publish(vec![id]).unwrap(); // Receiving owner publishes first.
            drop(done); // Must not reacquire the heap lock.
            access
                .collect(vec![id], CollectionReason::ExplicitRequest)
                .unwrap();
            assert_eq!(access.len(), 1);
            access
                .collect(vec![], CollectionReason::ExplicitRequest)
                .unwrap();
            assert!(access.is_empty());
        }
        drop(parent);
        assert!(owner.into_heap().unwrap().is_empty());
    }

    #[test]
    fn mutation_guard_excludes_collection_and_published_graph_is_traced() {
        let owner = Owner::new(2);
        let mut parent = owner.participant().unwrap();
        let (mut worker, id) = {
            let mut access = parent.enter();
            let id = object(&mut access, 1);
            (access.fork(vec![id]).unwrap(), id)
        };
        let (locked_tx, locked_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            {
                let mut access = worker.enter();
                let child = object(&mut access, 42);
                let parent_object = access.address(id).unwrap();
                // Publish a transitive graph while mutation owns the heap gate.
                let child = access.address(child).unwrap();
                let holder = access
                    .allocate(Value::Object {
                        ty: Type::from_name("Holder"),
                        fields: vec![
                            Value::SlotReference(parent_object),
                            Value::SlotReference(child),
                        ],
                    })
                    .unwrap();
                access.publish(vec![holder]).unwrap();
                locked_tx.send(()).unwrap();
                release_rx.recv().unwrap();
            }
            worker
        });
        locked_rx.recv().unwrap();
        assert!(matches!(
            owner.0.state.try_lock(),
            Err(std::sync::TryLockError::WouldBlock)
        ));
        release_tx.send(()).unwrap();
        let completion = worker.join().unwrap();
        {
            let mut access = parent.enter();
            access
                .collect(vec![], CollectionReason::ExplicitRequest)
                .unwrap();
            assert_eq!(access.len(), 3);
            drop(completion);
            access
                .collect(vec![], CollectionReason::ExplicitRequest)
                .unwrap();
            assert_eq!(access.len(), 0);
        }
    }

    #[test]
    fn abandoned_submission_releases_roots_without_locking_or_leaking_capacity() {
        let owner = Owner::new(2);
        let mut parent = owner.participant().unwrap();
        let mut access = parent.enter();
        let id = object(&mut access, 1);
        for _ in 0..128 {
            let rejected = access.fork(vec![id]).unwrap();
            assert!(access.fork(vec![]).is_err());
            drop(rejected);
        }
        access
            .collect(vec![], CollectionReason::ExplicitRequest)
            .unwrap();
        assert_eq!(access.len(), 0);
        assert_eq!(access.state.participants.len(), 1);
    }

    #[test]
    fn failed_root_publication_preserves_previous_roots_and_does_not_sweep() {
        let owner = Owner::new(1);
        let mut participant = owner.participant().unwrap();
        let mut access = participant.enter();
        let id = object(&mut access, 1);
        access.publish(vec![id]).unwrap();
        assert!(
            access
                .collect(vec![usize::MAX], CollectionReason::ExplicitRequest)
                .is_err()
        );
        assert_eq!(*access.registration.roots.lock().unwrap(), vec![id]);
        assert_eq!(access.len(), 1);
        assert_eq!(access.collections(), 0);
    }

    #[test]
    fn heap_export_requires_all_participants_to_be_released() {
        let owner = Owner::new(1);
        let mut participant = owner.participant().unwrap();
        assert!(owner.into_heap().is_err());
        let mut access = participant.enter();
        let id = object(&mut access, 42);
        access
            .collect(vec![id], CollectionReason::ExplicitRequest)
            .unwrap();
        assert_eq!(access.len(), 1);
    }

    #[test]
    fn service_roots_prevent_export_without_consuming_a_participant_slot() {
        let owner = Owner::new(1);
        let mut participant = owner.participant().unwrap();
        let roots = participant.enter().service_roots();
        drop(participant);
        assert!(owner.into_heap().is_err());
        drop(roots);
    }

    #[test]
    fn zero_participant_budget_rejects_admission() {
        assert!(Owner::new(0).participant().is_err());
    }
}

#[cfg(test)]
mod array_accounting_tests {
    use super::*;
    use crate::{Limits, Value, arrays::Usage, metadata::Type};

    #[test]
    fn aliases_count_heap_storage_once_and_collection_releases_payload() {
        let owner = Owner::new(2);
        let limits = Limits {
            array_elements: 2,
            ..Limits::default()
        };
        let mut first = owner.participant().unwrap();
        let mut second = owner.participant().unwrap();
        let id = {
            let mut access = first.enter();
            let id = access
                .allocate(Value::Array {
                    element: Type::Int32,
                    elements: vec![Value::Int32(0); 2],
                })
                .unwrap();
            access.publish(vec![id]).unwrap();
            access.publish_arrays(Usage::default(), true);
            id
        };
        {
            let mut access = second.enter();
            access.publish(vec![id]).unwrap();
            access.check_arrays(Usage::default(), &limits).unwrap();
            access
                .collect(vec![id], CollectionReason::ExplicitRequest)
                .unwrap();
            let mut own = Usage::default();
            crate::arrays::measure(
                &Value::Array {
                    element: Type::Byte,
                    elements: vec![Value::Byte(1)],
                },
                &mut own,
                &limits,
            )
            .unwrap();
            assert!(access.check_arrays(own, &limits).is_err());
        }
        drop(first);
        let mut access = second.enter();
        access
            .collect(vec![], CollectionReason::ExplicitRequest)
            .unwrap();
        access
            .check_arrays(
                Usage::default(),
                &Limits {
                    array_elements: 0,
                    ..limits
                },
            )
            .unwrap();
    }

    #[test]
    fn parked_byte_usage_is_aggregated_and_current_publication_is_replaced() {
        let owner = Owner::new(2);
        let limits = Limits {
            array_bytes: std::mem::size_of::<Value>() * 2 + 3,
            ..Limits::default()
        };
        let mut usage = Usage::default();
        crate::arrays::measure(
            &Value::Array {
                element: Type::String,
                elements: vec![Value::String("ab".into())],
            },
            &mut usage,
            &limits,
        )
        .unwrap();
        let mut first = owner.participant().unwrap();
        let mut second = owner.participant().unwrap();
        first.enter().publish_arrays(usage, true);
        {
            let mut access = second.enter();
            access.publish_arrays(usage, true);
            assert!(access.check_arrays(usage, &limits).is_err());
            access.publish_arrays(Usage::default(), true);
            access.check_arrays(Usage::default(), &limits).unwrap();
        }
        drop(first);
        second.enter().check_arrays(usage, &limits).unwrap();
    }
}
