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
}
struct State {
    heap: ManagedHeap,
    participants: Vec<Weak<Registration>>,
}
struct Shared {
    state: Mutex<State>,
    limit: usize,
}

pub(crate) struct Owner(Arc<Shared>);
pub(crate) struct Participant {
    shared: Arc<Shared>,
    registration: Arc<Registration>,
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
                participants: Vec::new(),
            }),
            limit,
        }))
    }

    pub(crate) fn participant(&self) -> Result<Participant, Fault> {
        let mut state = self.0.state.lock().expect("shared heap lock poisoned");
        register(&self.0, &mut state, Vec::new())
    }

    /// Teardown may export the heap only after every participant has been dropped.
    /// A rejected export leaves the heap owned by the outstanding participants.
    pub(crate) fn into_heap(self) -> Result<ManagedHeap, Fault> {
        let shared = Arc::try_unwrap(self.0)
            .map_err(|_| Fault::new("shared heap still has active participants"))?;
        Ok(shared
            .state
            .into_inner()
            .expect("shared heap lock poisoned")
            .heap)
    }
}

impl Participant {
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
    });
    state.participants.push(Arc::downgrade(&registration));
    Ok(Participant {
        shared: shared.clone(),
        registration,
    })
}

impl Access<'_> {
    /// Publish queued captures before the submitting participant can release them.
    /// Dropping a rejected/unsubmitted participant never reacquires the heap lock.
    #[allow(dead_code)] // Submission is connected when native Task.Run is added.
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
    fn zero_participant_budget_rejects_admission() {
        assert!(Owner::new(0).participant().is_err());
    }
}
