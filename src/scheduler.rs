//! Private invocation runnable/completion driver. TaskQueue is a legacy source adapter.
use crate::{ExecutionOptions, Fault, ManagedHeap, Value};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

// Socket readiness and host cancellation do not yet subscribe to the wake latch.
pub(crate) const POLL_INTERVAL: Duration = Duration::from_millis(10);

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

impl Source for crate::timers::Timers {
    fn poll(&mut self, _: &ManagedHeap) -> Result<Option<Value>, Fault> {
        Ok(self.poll(std::time::Instant::now()))
    }
    fn pending(&self) -> bool {
        self.pending()
    }
    fn trace_roots(&self, roots: &mut Vec<usize>) {
        self.trace_roots(roots);
    }
}

// Keep ready values inline; boxing would add an allocation to each completion.
#[allow(clippy::large_enum_variant)]
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
/// is absent for invocation-owned work; explicit queues belong to the legacy adapter.
struct Ready {
    callback: Value,
    destination: Option<Value>,
}

#[derive(Debug, PartialEq)]
pub(crate) enum CompletionState {
    Ready,
    Pending,
    Idle,
}

pub(crate) struct Scheduler {
    pub(crate) workers: crate::workers::Workers,
    pub(crate) sockets: crate::socket_io::Sockets,
    pub(crate) resolver: crate::name_resolution::Resolver,
    pub(crate) timers: crate::timers::Timers,
    destination: Option<i64>,
    ready: Option<Ready>,
    submitted: std::collections::VecDeque<Value>,
    registrations: std::collections::BTreeMap<i64, Value>,
    prefer_submission: bool,
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
            timers: Default::default(),
            destination: None,
            ready: None,
            submitted: Default::default(),
            registrations: Default::default(),
            prefer_submission: true,
            arbitration: Default::default(),
            wake,
        }
    }
}
impl Scheduler {
    /// Host wait policy, separate from parking and guest execution. A context that
    /// cannot own dispatch must not spin on another context's ready work/deadlines.
    pub(crate) fn wait_timeout(&self, now: Instant, can_dispatch: bool) -> Duration {
        if !can_dispatch {
            return POLL_INTERVAL;
        }
        if self.ready.is_some() || !self.submitted.is_empty() {
            return Duration::ZERO;
        }
        self.timers
            .next_deadline()
            .map_or(POLL_INTERVAL, |deadline| {
                deadline.saturating_duration_since(now).min(POLL_INTERVAL)
            })
    }

    pub(crate) fn wake(&self) -> Arc<Wake> {
        self.wake.clone()
    }

    fn progress(&mut self, heap: &ManagedHeap) -> Result<Progress, Fault> {
        let sources: &mut [&mut dyn Source] = &mut [
            &mut self.workers,
            &mut self.sockets,
            &mut self.resolver,
            &mut self.timers,
        ];
        self.arbitration.poll(sources, heap)
    }

    /// Opaque invocation identity, not a queue, root or transferable authority.
    pub(crate) fn capture_destination(&mut self) -> Result<i64, Fault> {
        use std::sync::atomic::{AtomicI64, Ordering};
        static NEXT: AtomicI64 = AtomicI64::new(1);
        if let Some(destination) = self.destination {
            return Ok(destination);
        }
        // try_update is newer than the supported Rust 1.85 minimum.
        #[allow(deprecated)]
        let destination = NEXT
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .map_err(|_| Fault::new("Scheduler destination identity exhausted"))?;
        self.destination = Some(destination);
        Ok(destination)
    }

    pub(crate) fn submit_to(&mut self, destination: i64, callback: Value) -> Result<(), Fault> {
        if destination <= 0 || self.destination != Some(destination) {
            return Err(Fault::new("Scheduler destination is not current"));
        }
        self.submit(callback)
    }

    pub(crate) fn submit(&mut self, callback: Value) -> Result<(), Fault> {
        if !matches!(callback, Value::Function(_))
            || callback.ty() != crate::assembler::parse_type("fn<Void>")?
        {
            return Err(Fault::new("Scheduler submission requires a unit callback"));
        }
        // Runnable storage grows independently of dormant registrations. Reserve
        // enough spare entries for every promised activation before accepting work.
        self.submitted
            .try_reserve(self.registrations.len() + 1)
            .map_err(|_| {
                Fault::coded(
                    crate::FaultCode::NativeMemoryLimitExceeded,
                    "Scheduler runnable allocation failed",
                )
            })?;
        self.submitted.push_back(callback);
        self.wake.signal();
        Ok(())
    }

    pub(crate) fn register(&mut self, destination: i64, callback: Value) -> Result<i64, Fault> {
        use std::sync::atomic::{AtomicI64, Ordering};
        static NEXT: AtomicI64 = AtomicI64::new(1);
        if destination <= 0 || self.destination != Some(destination) {
            return Err(Fault::new("Scheduler destination is not current"));
        }
        if !matches!(callback, Value::Function(_))
            || callback.ty() != crate::assembler::parse_type("fn<Void>")?
        {
            return Err(Fault::new(
                "Scheduler registration requires a unit callback",
            ));
        }
        if self.registrations.len() >= 64 {
            return Err(Fault::coded(
                crate::FaultCode::NativeMemoryLimitExceeded,
                "Scheduler registration capacity exceeded",
            ));
        }
        self.submitted
            .try_reserve(self.registrations.len() + 1)
            .map_err(|_| {
                Fault::coded(
                    crate::FaultCode::NativeMemoryLimitExceeded,
                    "Scheduler registration allocation failed",
                )
            })?;
        // try_update is newer than the supported Rust 1.85 minimum.
        #[allow(deprecated)]
        let id = NEXT
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .map_err(|_| Fault::new("Scheduler registration identity exhausted"))?;
        self.registrations.insert(id, callback);
        Ok(id)
    }

    pub(crate) fn activate(&mut self, id: i64) -> bool {
        let Some(callback) = self.registrations.remove(&id) else {
            return false;
        };
        // Registration already reserved capacity. Transfer ownership, never clone.
        self.submitted.push_back(callback);
        self.wake.signal();
        true
    }

    pub(crate) fn cancel_registration(&mut self, id: i64) -> bool {
        self.registrations.remove(&id).is_some()
    }

    pub(crate) fn install_submission(
        &mut self,
        install: impl FnOnce(&Value) -> Result<(), Fault>,
    ) -> Result<bool, Fault> {
        if self.ready.is_some() && !self.prefer_submission {
            return Ok(false);
        }
        let Some(callback) = self.submitted.front() else {
            return Ok(false);
        };
        install(callback)?;
        self.submitted.pop_front();
        self.prefer_submission = false;
        Ok(true)
    }

    pub(crate) fn trace_roots(&self, roots: &mut Vec<usize>) {
        for callback in self.registrations.values() {
            crate::gc::trace(callback, roots);
        }
        for callback in &self.submitted {
            crate::gc::trace(callback, roots);
        }
        if let Some(ready) = &self.ready {
            crate::gc::trace(&ready.callback, roots);
            if let Some(destination) = &ready.destination {
                crate::gc::trace(destination, roots);
            }
        }
        Source::trace_roots(&self.workers, roots);
        Source::trace_roots(&self.sockets, roots);
        Source::trace_roots(&self.resolver, roots);
        Source::trace_roots(&self.timers, roots);
    }

    fn stage(&mut self, callback: Value, destination: Option<&Value>) {
        assert!(
            self.ready.is_none(),
            "ready work must be installed before admitting more"
        );
        self.ready = Some(Ready {
            callback,
            destination: destination.cloned(),
        });
    }

    pub(crate) fn poll(&mut self, heap: &ManagedHeap, destination: &Value) -> Result<bool, Fault> {
        if self.ready.is_some() {
            return Ok(true);
        }
        if let Progress::Ready(callback) = self.progress(heap)? {
            self.stage(callback, Some(destination));
        }
        Ok(self.ready.is_some())
    }

    /// Poll under graph access. Ready includes submitted work and source completions.
    /// Pending requires publishing VM roots before parking; Idle means neither kind
    /// of runnable work nor a pending completion source belongs to this scheduler.
    pub(crate) fn completion_state(
        &mut self,
        heap: &ManagedHeap,
        options: &ExecutionOptions,
    ) -> Result<CompletionState, Fault> {
        options.check_cancellation("Worker.Completion", 0)?;
        if self.ready.is_some() {
            return Ok(CompletionState::Ready);
        }
        match self.progress(heap)? {
            Progress::Ready(callback) => {
                self.stage(callback, None);
                Ok(CompletionState::Ready)
            }
            Progress::Idle | Progress::Pending if !self.submitted.is_empty() => {
                Ok(CompletionState::Ready)
            }
            Progress::Idle => Ok(CompletionState::Idle),
            Progress::Pending => Ok(CompletionState::Pending),
        }
    }

    /// No managed graph access may be held while parking.
    #[cfg(test)]
    pub(crate) fn park(&self) {
        // Cancellation and the socket source have no wake subscription.
        self.wake.park(self.wait_timeout(Instant::now(), true));
    }

    #[cfg(test)]
    fn wait(
        &mut self,
        heap: &ManagedHeap,
        destination: Option<&Value>,
        options: &ExecutionOptions,
    ) -> Result<bool, Fault> {
        loop {
            match self.completion_state(heap, options)? {
                CompletionState::Ready => {
                    // Test adapter: preserve an already-bound destination, or bind
                    // newly admitted invocation work to this test's queue.
                    if let Some(ready) = &mut self.ready {
                        if ready.destination.is_none() {
                            ready.destination = destination.cloned();
                        }
                    }
                    return Ok(true);
                }
                CompletionState::Idle => return Ok(false),
                CompletionState::Pending => self.park(),
            }
        }
    }

    /// Installation must publish into an active traced owner before returning Ok.
    /// On Err, installation must not publish. Keep ready roots on failure.
    /// No source is polled while this slot is occupied.
    pub(crate) fn install_ready(
        &mut self,
        install: impl FnOnce(Option<&Value>, &Value) -> Result<(), Fault>,
    ) -> Result<bool, Fault> {
        let Some(ready) = &self.ready else {
            return Ok(false);
        };
        install(ready.destination.as_ref(), &ready.callback)?;
        self.ready = None;
        self.prefer_submission = true;
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
    fn wait_policy_respects_deadlines_ready_ownership_and_polling_fallback() {
        let mut heap = ManagedHeap::default();
        let mut scheduler = Scheduler::default();
        let (callback, destination) = ready_graphs(&mut heap);
        let now = Instant::now();
        assert_eq!(scheduler.wait_timeout(now, true), POLL_INTERVAL);
        let late = scheduler.timers.start(20, callback.clone(), now).unwrap();
        let early = scheduler.timers.start(3, callback.clone(), now).unwrap();
        assert_eq!(scheduler.wait_timeout(now, true), Duration::from_millis(3));
        assert_eq!(scheduler.wait_timeout(now, false), POLL_INTERVAL);
        assert_eq!(
            scheduler.wait_timeout(now + Duration::from_millis(4), true),
            Duration::ZERO
        );
        // Computing a wait does not consume or release the timer registration.
        assert!(scheduler.timers.cancel(early));
        assert_eq!(scheduler.wait_timeout(now, true), POLL_INTERVAL);
        assert!(scheduler.timers.cancel(late));
        scheduler.stage(callback, Some(&destination));
        assert_eq!(scheduler.wait_timeout(now, true), Duration::ZERO);
        assert_eq!(scheduler.wait_timeout(now, false), POLL_INTERVAL);
        assert!(scheduler.install_ready(|_, _| Ok(())).unwrap());
        assert_eq!(scheduler.wait_timeout(now, true), POLL_INTERVAL);
    }

    #[test]
    fn timer_callback_roots_transfer_once_and_cancelled_timers_release_them() {
        let mut heap = ManagedHeap::default();
        let mut scheduler = Scheduler::default();
        let (callback, destination) = ready_graphs(&mut heap);
        let id = scheduler
            .timers
            .start(0, callback, std::time::Instant::now())
            .unwrap();
        collect(&mut heap, &scheduler, std::slice::from_ref(&destination));
        assert_eq!(heap.len(), 4);
        assert!(scheduler.poll(&heap, &destination).unwrap());
        assert!(
            !scheduler.timers.cancel(id),
            "delivery already belongs to ready slot"
        );
        collect(&mut heap, &scheduler, &[]);
        assert_eq!(heap.len(), 4);
        assert!(
            scheduler
                .install_ready(|_, _| Err(Fault::new("rejected")))
                .is_err()
        );
        collect(&mut heap, &scheduler, &[]);
        assert_eq!(heap.len(), 4);
        scheduler.install_ready(|_, _| Ok(())).unwrap();
        collect(&mut heap, &scheduler, &[]);
        assert_eq!(heap.len(), 0);
        assert!(!scheduler.poll(&heap, &Value::Void).unwrap());

        let (callback, _) = ready_graphs(&mut heap);
        let id = scheduler
            .timers
            .start(i32::MAX, callback, std::time::Instant::now())
            .unwrap();
        collect(&mut heap, &scheduler, &[]);
        assert_eq!(heap.len(), 2);
        assert!(scheduler.timers.cancel(id));
        collect(&mut heap, &scheduler, &[]);
        assert_eq!(heap.len(), 0);
    }

    #[test]
    fn accept_callback_is_rooted_while_pending_and_staged() {
        use crate::socket_io::Operation;
        let mut heap = ManagedHeap::default();
        let mut scheduler = Scheduler::default();
        let (callback, destination) = ready_graphs(&mut heap);
        let Value::Erased(handle) = scheduler
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
        let Value::Erased(port) = scheduler
            .sockets
            .invoke(Operation::LocalPort, &[*handle.clone()], &heap)
            .unwrap()
        else {
            panic!()
        };
        let Value::Int32(port) = *port else { panic!() };
        let Value::Erased(operation) = scheduler
            .sockets
            .invoke(Operation::Accept, &[*handle, callback], &heap)
            .unwrap()
        else {
            panic!()
        };
        collect(&mut heap, &scheduler, std::slice::from_ref(&destination));
        assert_eq!(heap.statistics().live_objects, 4);
        assert!(!scheduler.poll(&heap, &destination).unwrap());
        let _peer = std::net::TcpStream::connect(("127.0.0.1", port as u16)).unwrap();
        assert!(
            scheduler
                .wait(&heap, Some(&destination), &ExecutionOptions::default())
                .unwrap()
        );
        collect(&mut heap, &scheduler, &[]);
        assert_eq!(heap.statistics().live_objects, 4);
        scheduler
            .sockets
            .invoke(Operation::ConnectResult, &[*operation], &heap)
            .unwrap();
        scheduler.install_ready(|_, _| Ok(())).unwrap();
        collect(&mut heap, &scheduler, &[]);
        assert_eq!(heap.statistics().live_objects, 0);
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
        collect(&mut heap, &scheduler, std::slice::from_ref(&destination));
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
                active.extend([queue.unwrap().clone(), callback.clone()]);
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
        let callback = Value::Function(crate::Function {
            object_view: false,
            identity: std::sync::Arc::new(()),
            ty: crate::assembler::parse_type("fn<Void>").unwrap(),
            target: Some(
                crate::assembler::parse_function_ref("instance TestOwner::Complete()").unwrap(),
            ),
            native: None,
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
        scheduler.stage(callback, Some(&destination));
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
                    active.extend([queue.unwrap().clone(), callback.clone()]);
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
    fn submitted_backlog_is_rooted_and_yields_to_source_delivery() {
        let mut heap = ManagedHeap::default();
        let mut scheduler = Scheduler::default();
        assert!(scheduler.submit(Value::Int32(0)).is_err());
        let (callback, _) = ready_graphs(&mut heap);
        for _ in 0..256 {
            scheduler.submit(callback.clone()).unwrap();
        }
        collect(&mut heap, &scheduler, &[]);
        assert_eq!(heap.len(), 2);
        assert_eq!(scheduler.wait_timeout(Instant::now(), true), Duration::ZERO);
        assert!(
            scheduler
                .install_submission(|_| Err(Fault::new("frame rejected")))
                .is_err()
        );
        assert_eq!(scheduler.submitted.len(), 256);
        scheduler.stage(Value::Void, None);
        assert!(scheduler.install_submission(|_| Ok(())).unwrap());
        assert!(
            !scheduler
                .install_submission(|_| panic!("source must get a turn"))
                .unwrap()
        );
        scheduler.install_ready(|_, _| Ok(())).unwrap();
        assert!(scheduler.install_submission(|_| Ok(())).unwrap());
        for _ in 0..254 {
            assert!(scheduler.install_submission(|_| Ok(())).unwrap());
        }
        assert!(
            !scheduler
                .install_submission(|_| panic!("duplicate delivery"))
                .unwrap()
        );
        collect(&mut heap, &scheduler, &[]);
        assert!(heap.is_empty());
    }

    #[test]
    fn growing_backlog_preserves_reserved_activation_capacity() {
        let mut heap = ManagedHeap::default();
        let mut scheduler = Scheduler::default();
        let destination = scheduler.capture_destination().unwrap();
        let (callback, _) = ready_graphs(&mut heap);
        let registrations: Vec<_> = (0..64)
            .map(|_| scheduler.register(destination, callback.clone()).unwrap())
            .collect();
        for _ in 0..256 {
            scheduler.submit(callback.clone()).unwrap();
            assert!(
                scheduler.submitted.capacity()
                    >= scheduler.submitted.len() + scheduler.registrations.len()
            );
        }
        let capacity = scheduler.submitted.capacity();
        for registration in registrations {
            assert!(scheduler.activate(registration));
            assert_eq!(scheduler.submitted.capacity(), capacity);
        }
        collect(&mut heap, &scheduler, &[]);
        assert_eq!(heap.len(), 2);
        for _ in 0..320 {
            assert!(scheduler.install_submission(|_| Ok(())).unwrap());
        }
        collect(&mut heap, &scheduler, &[]);
        assert!(heap.is_empty());
    }

    #[test]
    fn dormant_registrations_reserve_capacity_transfer_once_and_release_roots() {
        let mut heap = ManagedHeap::default();
        let mut scheduler = Scheduler::default();
        let destination = scheduler.capture_destination().unwrap();
        let (callback, _) = ready_graphs(&mut heap);
        let registrations: Vec<_> = (0..64)
            .map(|_| scheduler.register(destination, callback.clone()).unwrap())
            .collect();
        assert!(scheduler.register(destination, callback.clone()).is_err());
        scheduler.submit(callback).unwrap();
        assert!(scheduler.install_submission(|_| Ok(())).unwrap());
        collect(&mut heap, &scheduler, &[]);
        assert_eq!(heap.len(), 2);
        assert_eq!(
            scheduler
                .completion_state(&heap, &Default::default())
                .unwrap(),
            CompletionState::Idle
        );
        assert_eq!(scheduler.wait_timeout(Instant::now(), true), POLL_INTERVAL);
        let mut foreign = Scheduler::default();
        assert!(!foreign.activate(registrations[0]));
        assert!(!foreign.cancel_registration(registrations[0]));
        assert!(
            scheduler.activate(registrations[0]),
            "activation needs no extra capacity"
        );
        assert!(!scheduler.activate(registrations[0]));
        assert!(!scheduler.cancel_registration(registrations[0]));
        assert_eq!(
            scheduler
                .completion_state(&heap, &Default::default())
                .unwrap(),
            CompletionState::Ready
        );
        assert!(
            scheduler
                .install_submission(|_| Err(Fault::new("frame rejected")))
                .is_err()
        );
        assert!(scheduler.install_submission(|_| Ok(())).unwrap());
        for registration in &registrations[1..] {
            assert!(scheduler.cancel_registration(*registration));
            assert!(!scheduler.cancel_registration(*registration));
            assert!(!scheduler.activate(*registration));
        }
        collect(&mut heap, &scheduler, &[]);
        assert!(heap.is_empty());
        let (callback, _) = ready_graphs(&mut heap);
        let abandoned = scheduler.register(destination, callback).unwrap();
        drop(scheduler);
        assert!(!foreign.activate(abandoned));
        collect(&mut heap, &foreign, &[]);
        assert!(heap.is_empty(), "teardown abandons dormant registrations");
    }

    #[test]
    fn destination_identity_rejects_foreign_and_expired_invocations_before_retaining_work() {
        let mut heap = ManagedHeap::default();
        let (callback, _) = ready_graphs(&mut heap);
        let mut owner = Scheduler::default();
        let destination = owner.capture_destination().unwrap();
        assert!(destination > 0);
        assert_eq!(owner.capture_destination().unwrap(), destination);
        let mut other = Scheduler::default();
        let foreign = other.capture_destination().unwrap();
        assert_ne!(foreign, destination);
        for rejected in [0, -1, foreign] {
            assert!(owner.submit_to(rejected, callback.clone()).is_err());
        }
        assert!(owner.submitted.is_empty());
        assert!(other.submit_to(destination, callback.clone()).is_err());
        owner.submit_to(destination, callback.clone()).unwrap();
        collect(&mut heap, &owner, &[]);
        assert_eq!(heap.len(), 2);
        owner.install_submission(|_| Ok(())).unwrap();
        drop(owner);
        let mut replacement = Scheduler::default();
        assert_ne!(replacement.capture_destination().unwrap(), destination);
        assert!(replacement.submit_to(destination, callback).is_err());
        collect(&mut heap, &replacement, &[]);
        assert!(
            heap.is_empty(),
            "rejected destination must retain no callback graph"
        );
    }

    #[test]
    fn submissions_prevent_false_idle_and_pending_without_hiding_ready_sources() {
        let mut heap = ManagedHeap::default();
        let mut scheduler = Scheduler::default();
        let options = ExecutionOptions::default();
        let (callback, _) = ready_graphs(&mut heap);
        scheduler.submit(callback.clone()).unwrap();
        assert_eq!(
            scheduler.completion_state(&heap, &options).unwrap(),
            CompletionState::Ready
        );
        assert!(
            scheduler
                .install_submission(|_| Err(Fault::new("not installed")))
                .is_err()
        );
        assert_eq!(
            scheduler.completion_state(&heap, &options).unwrap(),
            CompletionState::Ready
        );
        scheduler.install_submission(|_| Ok(())).unwrap();
        assert_eq!(
            scheduler.completion_state(&heap, &options).unwrap(),
            CompletionState::Idle
        );

        let timer = scheduler
            .timers
            .start(i32::MAX, callback.clone(), Instant::now())
            .unwrap();
        assert_eq!(
            scheduler.completion_state(&heap, &options).unwrap(),
            CompletionState::Pending
        );
        scheduler.submit(callback.clone()).unwrap();
        assert_eq!(
            scheduler.completion_state(&heap, &options).unwrap(),
            CompletionState::Ready
        );
        scheduler.install_submission(|_| Ok(())).unwrap();
        assert_eq!(
            scheduler.completion_state(&heap, &options).unwrap(),
            CompletionState::Pending
        );
        assert!(scheduler.timers.cancel(timer));

        // A continuously replenished submission queue must still poll and yield
        // to an already-due timer. The caller supplies no source-readiness hint.
        scheduler.submit(callback.clone()).unwrap();
        scheduler
            .timers
            .start(0, callback.clone(), Instant::now())
            .unwrap();
        assert_eq!(
            scheduler.completion_state(&heap, &options).unwrap(),
            CompletionState::Ready
        );
        assert!(
            !scheduler
                .install_submission(|_| panic!("source turn"))
                .unwrap()
        );
        assert!(scheduler.install_ready(|_, _| Ok(())).unwrap());
        assert_eq!(
            scheduler.completion_state(&heap, &options).unwrap(),
            CompletionState::Ready
        );
        assert!(scheduler.install_submission(|_| Ok(())).unwrap());
        assert_eq!(
            scheduler.completion_state(&heap, &options).unwrap(),
            CompletionState::Idle
        );

        scheduler.submit(callback).unwrap();
        let token = crate::CancellationToken::new();
        token.cancel();
        let cancelled = ExecutionOptions {
            cancellation: Some(token),
            ..Default::default()
        };
        assert_eq!(
            scheduler
                .completion_state(&heap, &cancelled)
                .unwrap_err()
                .code,
            crate::FaultCode::ExecutionCancelled
        );
        collect(&mut heap, &scheduler, &[]);
        assert_eq!(
            heap.len(),
            2,
            "cancelled dispatch retains the submitted receiver until teardown"
        );
        drop(scheduler);
        heap.collect(vec![], crate::CollectionReason::ExecutionCompleted)
            .unwrap();
        assert!(heap.is_empty());
    }

    #[test]
    fn invocation_completion_needs_no_queue_and_survives_adapter_failure() {
        let mut heap = ManagedHeap::default();
        let mut scheduler = Scheduler::default();
        let (callback, _) = ready_graphs(&mut heap);
        scheduler.timers.start(0, callback, Instant::now()).unwrap();
        assert_eq!(
            scheduler
                .completion_state(&heap, &Default::default())
                .unwrap(),
            CompletionState::Ready
        );
        collect(&mut heap, &scheduler, &[]);
        assert_eq!(
            heap.len(),
            2,
            "only the callback graph belongs to ready work"
        );
        assert!(
            scheduler
                .install_ready(|destination, _| {
                    assert!(destination.is_none());
                    Err(Fault::new("adapter unavailable"))
                })
                .is_err()
        );
        collect(&mut heap, &scheduler, &[]);
        assert_eq!(heap.len(), 2);
        // A later legacy poll must not rewrite an already-admitted destination.
        assert!(scheduler.poll(&heap, &Value::Int32(99)).unwrap());
        let mut active = vec![];
        assert!(
            scheduler
                .install_ready(|destination, callback| {
                    assert!(destination.is_none());
                    active.push(callback.clone());
                    Ok(())
                })
                .unwrap()
        );
        assert!(
            !scheduler
                .install_ready(|_, _| panic!("duplicate delivery"))
                .unwrap()
        );
        collect(&mut heap, &scheduler, &active);
        assert_eq!(heap.len(), 2);
        active.clear();
        collect(&mut heap, &scheduler, &active);
        assert!(heap.is_empty());
        assert_eq!(
            scheduler
                .completion_state(&heap, &Default::default())
                .unwrap(),
            CompletionState::Idle
        );
    }

    #[test]
    fn occupied_ready_slot_preserves_destination_until_acknowledged() {
        let heap = ManagedHeap::default();
        let mut scheduler = Scheduler::default();
        scheduler.stage(Value::Int32(1), Some(&Value::Int32(2)));
        assert!(scheduler.poll(&heap, &Value::Int32(99)).unwrap());
        assert!(
            scheduler
                .wait(&heap, Some(&Value::Int32(98)), &Default::default())
                .unwrap()
        );
        assert!(
            scheduler
                .install_ready(|destination, callback| {
                    assert_eq!(destination, Some(&Value::Int32(2)));
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
        scheduler.stage(callback, Some(&destination));
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
