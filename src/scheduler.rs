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
#[cfg(test)]
impl Source for crate::socket_vm_probe::Receives {
    fn poll(&mut self, heap: &ManagedHeap) -> Result<Option<Value>, Fault> {
        self.poll(heap)
    }
    fn pending(&self) -> bool {
        self.is_pending()
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

pub(crate) struct Scheduler {
    pub(crate) workers: crate::workers::Workers,
    #[cfg(test)]
    pub(crate) socket_probe: crate::socket_vm_probe::Receives,
    arbitration: Arbitration,
    wake: Arc<Wake>,
}
impl Default for Scheduler {
    fn default() -> Self {
        let wake = Arc::new(Wake::default());
        Self {
            workers: crate::workers::Workers::with_wake(wake.clone()),
            #[cfg(test)]
            socket_probe: Default::default(),
            arbitration: Default::default(),
            wake,
        }
    }
}
impl Scheduler {
    fn progress(&mut self, heap: &ManagedHeap) -> Result<Progress, Fault> {
        let sources: &mut [&mut dyn Source] = &mut [
            &mut self.workers,
            #[cfg(test)]
            &mut self.socket_probe,
        ];
        self.arbitration.poll(sources, heap)
    }

    pub(crate) fn trace_roots(&self, roots: &mut Vec<usize>) {
        Source::trace_roots(&self.workers, roots);
        #[cfg(test)]
        Source::trace_roots(&self.socket_probe, roots);
    }

    pub(crate) fn poll(&mut self, heap: &ManagedHeap) -> Result<Option<Value>, Fault> {
        Ok(match self.progress(heap)? {
            Progress::Ready(work) => Some(work),
            _ => None,
        })
    }

    pub(crate) fn wait(
        &mut self,
        heap: &ManagedHeap,
        options: &ExecutionOptions,
    ) -> Result<Option<Value>, Fault> {
        loop {
            // Preserve the existing host-cancellation fault location for now.
            options.check_cancellation("Worker.Completion", 0)?;
            match self.progress(heap)? {
                Progress::Ready(work) => return Ok(Some(work)),
                Progress::Idle => return Ok(None),
                Progress::Pending => {}
            }
            // Cancellation currently has no wake subscription. Keep a bounded
            // timeout; the test socket source also has no OS readiness notifier.
            self.wake.park(Duration::from_millis(10));
        }
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
}
