//! Private invocation completion driver. TaskQueue remains the callback adapter.
use crate::{ExecutionOptions, Fault, ManagedHeap, Value};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// A coalesced wake hint, not the completion data. Sources retain their outcomes
/// until polled. The latch covers a signal between the last poll and parking.
#[derive(Default)]
pub(crate) struct Wake {
    signalled: Mutex<bool>,
    changed: Condvar,
}
impl Wake {
    pub(crate) fn signal(&self) {
        *self.signalled.lock().unwrap() = true;
        self.changed.notify_one();
    }

    pub(crate) fn park(&self, timeout: Duration) -> bool {
        let guard = self.signalled.lock().unwrap();
        let (mut guard, _) = self
            .changed
            .wait_timeout_while(guard, timeout, |ready| !*ready)
            .unwrap();
        let signalled = *guard;
        *guard = false;
        signalled
    }
}

trait Source {
    fn poll(&mut self, heap: &ManagedHeap) -> Result<Option<Value>, Fault>;
    fn pending(&self) -> bool;
    fn trace_roots(&self, roots: &mut Vec<usize>);
}
impl Source for crate::workers::Workers {
    fn poll(&mut self, _: &ManagedHeap) -> Result<Option<Value>, Fault> {
        Ok(self.poll_notification())
    }
    fn pending(&self) -> bool {
        self.has_notifications()
    }
    fn trace_roots(&self, roots: &mut Vec<usize>) {
        self.trace_roots(roots);
    }
}
impl Source for crate::socket_io::Sockets {
    fn poll(&mut self, heap: &ManagedHeap) -> Result<Option<Value>, Fault> {
        self.poll(heap)
    }
    fn pending(&self) -> bool {
        self.pending()
    }
    fn trace_roots(&self, roots: &mut Vec<usize>) {
        self.trace_roots(roots);
    }
}

impl Source for crate::name_resolution::Resolver {
    fn poll(&mut self, _: &ManagedHeap) -> Result<Option<Value>, Fault> {
        Ok(self.poll())
    }
    fn pending(&self) -> bool {
        self.pending()
    }
    fn trace_roots(&self, roots: &mut Vec<usize>) {
        self.trace_roots(roots);
    }
}

#[derive(Debug, PartialEq)]
enum Progress {
    Ready(Value),
    Pending,
    Idle,
}
#[derive(Default)]
struct Arbitration {
    next: usize,
}
impl Arbitration {
    // One completion per safe VM boundary; rotate after each delivered source.
    fn poll(
        &mut self,
        sources: &mut [&mut dyn Source],
        heap: &ManagedHeap,
    ) -> Result<Progress, Fault> {
        for offset in 0..sources.len() {
            let index = (self.next + offset) % sources.len();
            if let Some(work) = sources[index].poll(heap)? {
                self.next = (index + 1) % sources.len();
                return Ok(Progress::Ready(work));
            }
        }
        Ok(if sources.iter().any(|source| source.pending()) {
            Progress::Pending
        } else {
            Progress::Idle
        })
    }
}

/// Owned runnable registration for the current callback adapter. The destination
/// is explicit even though only the invocation's default queue is supported today.
struct Ready {
    callback: Value,
    destination: Value,
}

pub(crate) struct Scheduler {
    pub(crate) workers: crate::workers::Workers,
    pub(crate) sockets: crate::socket_io::Sockets,
    pub(crate) resolver: crate::name_resolution::Resolver,
    ready: Option<Ready>,
    arbitration: Arbitration,
    wake: Arc<Wake>,
}
impl Default for Scheduler {
    fn default() -> Self {
        let wake = Arc::new(Wake::default());
        Self {
            workers: crate::workers::Workers::with_wake(wake.clone()),
            sockets: Default::default(),
            resolver: crate::name_resolution::Resolver::with_wake(wake.clone()),
            ready: None,
            arbitration: Default::default(),
            wake,
        }
    }
}
impl Scheduler {
    fn progress(&mut self, heap: &ManagedHeap) -> Result<Progress, Fault> {
        let sources: &mut [&mut dyn Source] =
            &mut [&mut self.workers, &mut self.sockets, &mut self.resolver];
        self.arbitration.poll(sources, heap)
    }

    pub(crate) fn trace_roots(&self, roots: &mut Vec<usize>) {
        if let Some(ready) = &self.ready {
            crate::gc::trace(&ready.callback, roots);
            crate::gc::trace(&ready.destination, roots);
        }
        Source::trace_roots(&self.workers, roots);
        Source::trace_roots(&self.sockets, roots);
        Source::trace_roots(&self.resolver, roots);
    }

    fn stage(&mut self, callback: Value, destination: &Value) {
        assert!(
            self.ready.is_none(),
            "ready work must be installed before admitting more"
        );
        self.ready = Some(Ready {
            callback,
            destination: destination.clone(),
        });
    }

    pub(crate) fn poll(&mut self, heap: &ManagedHeap, destination: &Value) -> Result<bool, Fault> {
        if self.ready.is_some() {
            return Ok(true);
        }
        if let Progress::Ready(callback) = self.progress(heap)? {
            self.stage(callback, destination);
        }
        Ok(self.ready.is_some())
    }

    pub(crate) fn wait(
        &mut self,
        heap: &ManagedHeap,
        destination: Option<&Value>,
        options: &ExecutionOptions,
    ) -> Result<bool, Fault> {
        loop {
            options.check_cancellation("Worker.Completion", 0)?;
            if self.ready.is_some() {
                return Ok(true);
            }
            match self.progress(heap)? {
                Progress::Ready(callback) => {
                    let destination =
                        destination.ok_or_else(|| Fault::new("Missing default TaskQueue"))?;
                    self.stage(callback, destination);
                    return Ok(true);
                }
                Progress::Idle => return Ok(false),
                Progress::Pending => {}
            }
            // Cancellation and the socket source have no wake subscription.
            self.wake.park(Duration::from_millis(10));
        }
    }

    /// Installation must publish into an active traced owner before returning Ok.
    /// On Err, installation must not publish. Keep ready roots on failure.
    /// No source is polled while this slot is occupied.
    pub(crate) fn install_ready(
        &mut self,
        install: impl FnOnce(&Value, &Value) -> Result<(), Fault>,
    ) -> Result<bool, Fault> {
        let Some(ready) = &self.ready else {
            return Ok(false);
        };
        install(&ready.destination, &ready.callback)?;
        self.ready = None;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    struct Fake {
        ready: VecDeque<Value>,
        pending: bool,
    }
    impl Source for Fake {
        fn poll(&mut self, _: &ManagedHeap) -> Result<Option<Value>, Fault> {
            Ok(self.ready.pop_front())
        }
        fn pending(&self) -> bool {
            self.pending || !self.ready.is_empty()
        }
        fn trace_roots(&self, _: &mut Vec<usize>) {}
    }
    fn source(values: &[i32]) -> Fake {
        Fake {
            ready: values.iter().map(|n| Value::Int32(*n)).collect(),
            pending: false,
        }
    }

    #[test]
    fn resolver_callback_transfers_through_ready_slot_and_survives_collection() {
        let mut heap = ManagedHeap::default();
        let mut scheduler = Scheduler::default();
        let (callback, destination) = ready_graphs(&mut heap);
        let id = scheduler
            .resolver
            .submit("127.0.0.1", callback, Duration::from_secs(5))
            .unwrap();
        collect(&mut heap, &scheduler, &[destination.clone()]);
        assert_eq!(heap.statistics().live_objects, 4);
        assert!(
            scheduler
                .wait(&heap, Some(&destination), &ExecutionOptions::default())
                .unwrap()
        );
        collect(&mut heap, &scheduler, &[]);
        assert_eq!(heap.statistics().live_objects, 4);
        assert_eq!(
            scheduler.resolver.take_result(id).unwrap(),
            Some(Ok(vec![std::net::Ipv4Addr::LOCALHOST]))
        );
        let mut active = vec![];
        scheduler
            .install_ready(|queue, callback| {
                active.extend([queue.clone(), callback.clone()]);
                Ok(())
            })
            .unwrap();
        collect(&mut heap, &scheduler, &active);
        assert_eq!(heap.statistics().live_objects, 4);
        collect(&mut heap, &scheduler, &[]);
        assert_eq!(heap.statistics().live_objects, 0);
    }

    #[test]
    fn rotating_sources_share_one_policy_across_dispatch_turns() {
        let mut first = source(&[1, 2, 3]);
        let mut second = source(&[4, 5]);
        let mut arbitration = Arbitration::default();
        let heap = ManagedHeap::default();
        for expected in [1, 4, 2, 5, 3] {
            assert_eq!(
                arbitration
                    .poll(&mut [&mut first, &mut second], &heap)
                    .unwrap(),
                Progress::Ready(Value::Int32(expected))
            );
        }
        assert_eq!(
            arbitration
                .poll(&mut [&mut first, &mut second], &heap)
                .unwrap(),
            Progress::Idle
        );
        first.pending = true;
        assert_eq!(
            arbitration
                .poll(&mut [&mut first, &mut second], &heap)
                .unwrap(),
            Progress::Pending
        );
        second.ready.push_back(Value::Int32(6));
        assert_eq!(
            arbitration
                .poll(&mut [&mut first, &mut second], &heap)
                .unwrap(),
            Progress::Ready(Value::Int32(6))
        );
    }

    #[test]
    fn completion_between_empty_poll_and_park_is_not_lost() {
        let wake = Wake::default();
        let mut source = source(&[]);
        source.pending = true;
        let mut arbitration = Arbitration::default();
        let heap = ManagedHeap::default();
        assert_eq!(
            arbitration.poll(&mut [&mut source], &heap).unwrap(),
            Progress::Pending
        );
        source.ready.push_back(Value::Int32(1));
        wake.signal();
        // Zero timeout makes this deterministic: only a retained signal returns true.
        assert!(wake.park(Duration::ZERO));
        assert_eq!(
            arbitration.poll(&mut [&mut source], &heap).unwrap(),
            Progress::Ready(Value::Int32(1))
        );
        assert!(!wake.park(Duration::ZERO));
    }

    #[test]
    fn coalesced_signals_do_not_discard_source_outcomes() {
        let wake = Wake::default();
        let mut source = source(&[1, 2]);
        wake.signal();
        wake.signal();
        assert!(wake.park(Duration::ZERO));
        let mut arbitration = Arbitration::default();
        let heap = ManagedHeap::default();
        for n in [1, 2] {
            assert_eq!(
                arbitration.poll(&mut [&mut source], &heap).unwrap(),
                Progress::Ready(Value::Int32(n))
            );
        }
        assert!(!wake.park(Duration::ZERO));
    }

    #[test]
    fn notification_on_another_thread_wakes_a_parked_or_about_to_park_owner() {
        let wake = Arc::new(Wake::default());
        let other = wake.clone();
        let producer = std::thread::spawn(move || other.signal());
        assert!(wake.park(Duration::from_secs(5)));
        producer.join().unwrap();
    }
    fn object(heap: &mut ManagedHeap, fields: Vec<Value>) -> Value {
        let id = heap
            .allocate(Value::Object {
                ty: crate::metadata::Type::from_name("TestOwner"),
                fields,
            })
            .unwrap();
        Value::ObjectReference(crate::value::ObjectReference {
            reference: heap.address(id).unwrap(),
            view: None,
        })
    }

    fn ready_graphs(heap: &mut ManagedHeap) -> (Value, Value) {
        let captured = object(heap, vec![Value::Int32(42)]);
        let receiver = object(heap, vec![captured]);
        let callback = Value::Delegate(crate::Delegate {
            ty: crate::assembler::parse_type("System.Func<Void>").unwrap(),
            target: crate::assembler::parse_function_ref("instance TestOwner::Complete()").unwrap(),
            receiver: Some(Box::new(receiver)),
        });
        let marker = object(heap, vec![Value::Int32(7)]);
        (callback, object(heap, vec![marker]))
    }

    fn collect(heap: &mut ManagedHeap, scheduler: &Scheduler, active: &[Value]) {
        let mut roots = vec![];
        scheduler.trace_roots(&mut roots);
        for value in active {
            crate::gc::trace(value, &mut roots);
        }
        heap.collect(roots, crate::CollectionReason::AllocationPressure)
            .unwrap();
    }

    #[test]
    fn ready_roots_survive_failed_install_then_transfer_to_active_owner_once() {
        let mut heap = ManagedHeap::default();
        let mut scheduler = Scheduler::default();
        let (callback, destination) = ready_graphs(&mut heap);
        scheduler.stage(callback, &destination);
        drop(destination);
        for _ in 0..100 {
            object(&mut heap, vec![]);
        }
        collect(&mut heap, &scheduler, &[]);
        assert_eq!(
            heap.len(),
            4,
            "callback and independent destination graphs must survive"
        );
        assert_eq!(heap.statistics().reclaimed_objects, 100);
        assert!(
            scheduler
                .install_ready(|_, _| Err(Fault::new("frame rejected")))
                .is_err()
        );
        collect(&mut heap, &scheduler, &[]);
        assert_eq!(heap.len(), 4);
        let mut active = vec![];
        assert!(
            scheduler
                .install_ready(|queue, callback| {
                    active.extend([queue.clone(), callback.clone()]);
                    Ok(())
                })
                .unwrap()
        );
        assert!(
            !scheduler
                .install_ready(|_, _| panic!("duplicate installation"))
                .unwrap()
        );
        collect(&mut heap, &scheduler, &active);
        assert_eq!(
            heap.len(),
            4,
            "active frame roots take over from ready ownership"
        );
        active.clear();
        collect(&mut heap, &scheduler, &active);
        assert!(heap.is_empty());
    }

    #[test]
    fn occupied_ready_slot_preserves_destination_until_acknowledged() {
        let heap = ManagedHeap::default();
        let mut scheduler = Scheduler::default();
        scheduler.stage(Value::Int32(1), &Value::Int32(2));
        assert!(scheduler.poll(&heap, &Value::Int32(99)).unwrap());
        assert!(
            scheduler
                .wait(&heap, Some(&Value::Int32(98)), &Default::default())
                .unwrap()
        );
        assert!(
            scheduler
                .install_ready(|destination, callback| {
                    assert_eq!(*destination, Value::Int32(2));
                    assert_eq!(*callback, Value::Int32(1));
                    Ok(())
                })
                .unwrap()
        );
        assert!(!scheduler.wait(&heap, None, &Default::default()).unwrap());
    }

    #[test]
    fn cancellation_keeps_ready_roots_until_invocation_teardown() {
        let mut heap = ManagedHeap::default();
        let mut scheduler = Scheduler::default();
        let (callback, destination) = ready_graphs(&mut heap);
        scheduler.stage(callback, &destination);
        drop(destination);
        let token = crate::CancellationToken::new();
        token.cancel();
        let options = ExecutionOptions {
            cancellation: Some(token),
            ..Default::default()
        };
        assert_eq!(
            scheduler.wait(&heap, None, &options).unwrap_err().code,
            crate::FaultCode::ExecutionCancelled
        );
        collect(&mut heap, &scheduler, &[]);
        assert_eq!(heap.len(), 4);
        drop(scheduler);
        heap.collect(vec![], crate::CollectionReason::ExecutionCompleted)
            .unwrap();
        assert!(heap.is_empty());
    }
}
