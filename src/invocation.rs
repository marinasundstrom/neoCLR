//! Shared services whose ownership must survive individual Task.Run contexts.
//! File locking is allowed only outside managed graph access. Interning never
//! waits on host work and may run under graph access. No file operation takes
//! the graph gate, preventing a reverse lock order during blocking I/O.
use crate::{Limits, invocation_budget::Budget};
use std::sync::{Arc, Mutex};

pub(crate) struct Invocation {
    pub(crate) dispatch: Mutex<Dispatch>,
    pub(crate) wake: Arc<crate::scheduler::Wake>,
    pub(crate) memory: crate::memory::SharedMemory,
    pub(crate) native_libraries: Arc<Mutex<Option<crate::interop::NativeLibraries>>>,
    pub(crate) limits: Limits,
    pub(crate) budget: Arc<Budget>,
    pub(crate) files: Mutex<crate::file_streams::Files>,
    pub(crate) interned: Mutex<crate::string_interning::Pool>,
}
pub(crate) struct Dispatch {
    pub(crate) scheduler: crate::scheduler::Scheduler,
    pub(crate) default_task_queue: Option<crate::Value>,
    roots: Option<crate::shared_heap::RootSet>,
    pub(crate) owner: std::sync::Weak<()>,
}
impl Dispatch {
    pub(crate) fn bind(
        &mut self,
        heap: &mut crate::shared_heap::Access<'_>,
    ) -> Result<(), crate::Fault> {
        if self.roots.is_none() {
            self.roots = Some(heap.service_roots());
        }
        self.publish(heap)
    }

    pub(crate) fn publish(
        &self,
        heap: &mut crate::shared_heap::Access<'_>,
    ) -> Result<(), crate::Fault> {
        let mut roots = Vec::new();
        self.scheduler.trace_roots(&mut roots);
        if let Some(queue) = &self.default_task_queue {
            crate::gc::trace(queue, &mut roots);
        }
        self.roots
            .as_ref()
            .expect("bound dispatch roots")
            .publish(heap, roots)
    }

    pub(crate) fn claim_owner(owner: &mut std::sync::Weak<()>, context: &Arc<()>) -> bool {
        if let Some(owner) = owner.upgrade() {
            return Arc::ptr_eq(&owner, context);
        }
        *owner = Arc::downgrade(context);
        true
    }
}

impl Invocation {
    pub(crate) fn park(&self) {
        self.wake.park(std::time::Duration::from_millis(10));
    }

    pub(crate) fn new(limits: Limits) -> Arc<Self> {
        let scheduler = crate::scheduler::Scheduler::default();
        let wake = scheduler.wake();
        Arc::new(Self {
            dispatch: Mutex::new(Dispatch {
                scheduler,
                default_task_queue: None,
                roots: None,
                owner: std::sync::Weak::new(),
            }),
            wake,
            memory: crate::memory::SharedMemory::new(limits),
            native_libraries: Default::default(),
            limits,
            budget: Budget::new(limits),
            files: Mutex::new(Default::default()),
            interned: Mutex::new(crate::string_interning::Pool::new(
                limits.intern_entries,
                limits.intern_bytes,
            )),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Value, file_streams::Operation};

    #[test]
    fn native_contexts_share_intern_identity_and_quota() {
        let invocation = Invocation::new(Limits {
            intern_entries: 1,
            intern_bytes: 4,
            ..Default::default()
        });
        let other = invocation.clone();
        let first = std::thread::spawn(move || {
            other
                .interned
                .lock()
                .unwrap()
                .intern("same".into())
                .unwrap()
        })
        .join()
        .unwrap();
        let mut pool = invocation.interned.lock().unwrap();
        let second = pool.intern("same".into()).unwrap();
        assert!(first.same_owner(&second));
        assert_eq!(
            pool.intern("next".into()),
            Err(crate::string_interning::Rejected::Entries)
        );
    }

    #[test]
    fn native_contexts_share_file_position_and_close_state() {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "neoclr-shared-file-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::write(&path, b"ab").unwrap();
        let invocation = Invocation::new(Limits::default());
        let Value::Erased(handle) = invocation
            .files
            .lock()
            .unwrap()
            .invoke(
                Operation::OpenRead,
                &[Value::String(path.to_str().unwrap().into())],
                &invocation.limits,
            )
            .unwrap()
        else {
            panic!()
        };
        let other = invocation.clone();
        let passed = *handle.clone();
        let read = std::thread::spawn(move || {
            other
                .files
                .lock()
                .unwrap()
                .invoke(Operation::Read, &[passed, Value::Int32(1)], &other.limits)
                .unwrap()
        })
        .join()
        .unwrap();
        assert_eq!(
            read,
            Value::Erased(Box::new(Value::Array {
                element: crate::metadata::Type::Byte,
                elements: vec![Value::Byte(b'a')]
            }))
        );
        let mut files = invocation.files.lock().unwrap();
        assert_eq!(
            files
                .invoke(Operation::Position, &[*handle.clone()], &invocation.limits)
                .unwrap(),
            Value::Erased(Box::new(Value::Int64(1)))
        );
        files
            .invoke(Operation::Close, &[*handle.clone()], &invocation.limits)
            .unwrap();
        drop(files);
        let other = invocation.clone();
        let result = std::thread::spawn(move || {
            other
                .files
                .lock()
                .unwrap()
                .invoke(Operation::Read, &[*handle, Value::Int32(1)], &other.limits)
                .unwrap()
        })
        .join()
        .unwrap();
        assert_eq!(result, Value::Erased(Box::new(Value::Byte(6))));
        std::fs::remove_file(path).unwrap();
    }
}

#[cfg(test)]
mod dispatch_tests {
    use super::*;
    use crate::{CollectionReason, Value, metadata::Type, shared_heap::Owner};

    fn object(heap: &mut crate::shared_heap::Access<'_>) -> Value {
        let id = heap
            .allocate(Value::Object {
                ty: Type::from_name("Capture"),
                fields: vec![],
            })
            .unwrap();
        Value::ObjectReference(crate::value::ObjectReference {
            reference: heap.address(id).unwrap(),
            view: None,
        })
    }

    #[test]
    fn pending_source_and_queue_outlive_the_submitting_participant() {
        use crate::socket_io::Operation;
        let owner = Owner::new(2);
        let invocation = Invocation::new(Limits::default());
        let mut submitter = owner.participant().unwrap();
        let mut collector = owner.participant().unwrap();
        let port = {
            let mut heap = submitter.enter();
            let mut dispatch = invocation.dispatch.lock().unwrap();
            dispatch.bind(&mut heap).unwrap();
            dispatch.default_task_queue = Some(object(&mut heap));
            let callback = Value::Delegate(crate::Delegate {
                ty: crate::assembler::parse_type("System.Func<Void>").unwrap(),
                target: crate::assembler::parse_function_ref("instance Capture::Callback()")
                    .unwrap(),
                receiver: Some(Box::new(object(&mut heap))),
            });
            let Value::Erased(listener) = dispatch
                .scheduler
                .sockets
                .invoke(
                    Operation::Listen,
                    &[
                        Value::String("127.0.0.1".into()),
                        Value::Int32(0),
                        Value::Int32(1),
                    ],
                    &heap,
                )
                .unwrap()
            else {
                panic!()
            };
            let Value::Erased(port) = dispatch
                .scheduler
                .sockets
                .invoke(Operation::LocalPort, &[*listener.clone()], &heap)
                .unwrap()
            else {
                panic!()
            };
            let Value::Int32(port) = *port else { panic!() };
            dispatch
                .scheduler
                .sockets
                .invoke(Operation::Accept, &[*listener, callback], &heap)
                .unwrap();
            dispatch.publish(&mut heap).unwrap();
            port
        };
        drop(submitter);
        collector
            .enter()
            .collect(vec![], CollectionReason::ExplicitRequest)
            .unwrap();
        assert_eq!(collector.enter().len(), 2);
        let _peer = std::net::TcpStream::connect(("127.0.0.1", port as u16)).unwrap();
        let mut installed = false;
        for _ in 0..100 {
            let mut heap = collector.enter();
            let mut dispatch = invocation.dispatch.lock().unwrap();
            let queue = dispatch.default_task_queue.clone();
            let state = dispatch
                .scheduler
                .completion_state(&heap, queue.as_ref(), &Default::default())
                .unwrap();
            if state == crate::scheduler::CompletionState::Ready {
                dispatch
                    .scheduler
                    .install_ready(|destination, callback| {
                        assert_eq!(Some(destination), queue.as_ref());
                        let Value::Delegate(callback) = callback else {
                            panic!()
                        };
                        let Some(receiver) = &callback.receiver else {
                            panic!()
                        };
                        let Value::ObjectReference(receiver) = receiver.as_ref() else {
                            panic!()
                        };
                        receiver.reference.read().unwrap();
                        Ok(())
                    })
                    .unwrap();
                assert!(
                    !dispatch
                        .scheduler
                        .install_ready(|_, _| panic!("duplicate completion"))
                        .unwrap()
                );
                dispatch.publish(&mut heap).unwrap();
                installed = true;
                break;
            }
            drop(dispatch);
            drop(heap);
            invocation.park();
        }
        assert!(installed);
        collector
            .enter()
            .collect(vec![], CollectionReason::ExplicitRequest)
            .unwrap();
        assert_eq!(
            collector.enter().len(),
            1,
            "only the invocation queue remains"
        );
        drop(invocation);
        collector
            .enter()
            .collect(vec![], CollectionReason::ExplicitRequest)
            .unwrap();
        assert!(collector.enter().is_empty());
        drop(collector);
        owner.into_heap().unwrap();
    }

    #[test]
    fn dispatch_rejects_a_foreign_heap_and_hands_off_owner_after_exit() {
        let invocation = Invocation::new(Limits::default());
        let first = Owner::new(1);
        let second = Owner::new(1);
        let mut a = first.participant().unwrap();
        let mut b = second.participant().unwrap();
        let mut dispatch = invocation.dispatch.lock().unwrap();
        dispatch.bind(&mut a.enter()).unwrap();
        assert!(dispatch.bind(&mut b.enter()).is_err());
        let one = Arc::new(());
        let two = Arc::new(());
        assert!(Dispatch::claim_owner(&mut dispatch.owner, &one));
        assert!(!Dispatch::claim_owner(&mut dispatch.owner, &two));
        assert!(Dispatch::claim_owner(&mut dispatch.owner, &one));
        drop(one);
        assert!(Dispatch::claim_owner(&mut dispatch.owner, &two));
    }
}
