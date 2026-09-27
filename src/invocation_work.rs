//! Invocation-owned native jobs. Lock order is graph -> service; shutdown takes
//! the work owner out of the service before joining, and holds neither lock.
use crate::{
    CancellationToken, Fault, Limits, Value,
    shared_heap::Access,
    task_work::{Completion, Context, Control, Work},
};
use std::sync::{Arc, Mutex};

struct State {
    work: Option<Work>,
    closed: bool,
}
#[derive(Default)]
struct Output {
    lines: Vec<String>,
    bytes: [Vec<u8>; 2],
}
pub(crate) struct Service {
    output: Mutex<Output>,
    revision: Arc<std::sync::atomic::AtomicUsize>,
    state: Mutex<State>,
    limits: Limits,
    cancellation: CancellationToken,
    wake: Arc<crate::scheduler::Wake>,
    failure: Arc<Mutex<Option<Fault>>>,
}
impl Service {
    pub(crate) fn new(
        limits: Limits,
        cancellation: CancellationToken,
        wake: Arc<crate::scheduler::Wake>,
    ) -> Self {
        Self {
            output: Mutex::new(Output::default()),
            revision: Default::default(),
            state: Mutex::new(State {
                work: None,
                closed: false,
            }),
            limits,
            cancellation,
            wake,
            failure: Arc::new(Mutex::new(None)),
        }
    }

    #[allow(dead_code)] // Guest Task.Run binding is the next integration slice.
    pub(crate) fn submit(
        &self,
        access: &mut Access<'_>,
        captures: Vec<Value>,
        callback: impl FnOnce(Context, Vec<Value>, &Control) -> Result<Completion, Fault>
        + Send
        + 'static,
    ) -> Result<usize, Fault> {
        self.check()?;
        let mut state = self.state.lock().expect("task service lock poisoned");
        if state.closed {
            return Err(Fault::new("invocation task service is closed"));
        }
        let work = state.work.get_or_insert_with(|| {
            // Initial bound reuses frame capacity; no unbounded native submission.
            let mut work = Work::new(
                access,
                self.limits.frames,
                self.cancellation.clone(),
                self.limits,
            );
            work.observe(
                self.wake.clone(),
                self.failure.clone(),
                self.revision.clone(),
            );
            work
        });
        work.submit(access, captures, callback)
    }

    #[allow(dead_code)]
    pub(crate) fn take_ready(&self, id: usize) -> Result<Option<Completion>, Fault> {
        let mut state = self.state.lock().expect("task service lock poisoned");
        state
            .work
            .as_mut()
            .ok_or_else(|| Fault::new("invocation has no active task service"))?
            .take_ready(id)
    }

    pub(crate) fn record_output(&self, lines: &mut Vec<String>, bytes: &mut [Vec<u8>; 2]) {
        let mut output = self.output.lock().expect("task output lock poisoned");
        output.lines.append(lines);
        for (target, source) in output.bytes.iter_mut().zip(bytes) {
            target.append(source);
        }
    }

    pub(crate) fn drain_output(&self, lines: &mut Vec<String>, bytes: &mut [Vec<u8>; 2]) {
        let mut output = self.output.lock().expect("task output lock poisoned");
        lines.append(&mut output.lines);
        for (target, source) in bytes.iter_mut().zip(&mut output.bytes) {
            target.append(source);
        }
    }

    pub(crate) fn activity(&self) -> (bool, usize) {
        let state = self.state.lock().expect("task service lock poisoned");
        let running = state.work.as_ref().is_some_and(Work::running);
        (
            running,
            self.revision.load(std::sync::atomic::Ordering::Acquire),
        )
    }

    pub(crate) fn check(&self) -> Result<(), Fault> {
        self.failure
            .lock()
            .expect("task failure lock poisoned")
            .clone()
            .map_or(Ok(()), Err)
    }

    /// Invocation's root driver calls this outside graph access, including on error.
    pub(crate) fn shutdown(&self) -> Result<(), Fault> {
        let work = {
            let mut state = self.state.lock().expect("task service lock poisoned");
            state.closed = true;
            state.work.take()
        };
        let result = match work {
            Some(mut work) => work.shutdown(),
            None => Ok(()),
        };
        self.check().and(result)
    }
}

/// Root-driver scope breaks the job -> invocation lifetime dependency on success,
/// failure and unwinding. Never create or drop this scope inside a worker or graph guard.
pub(crate) struct Scope(pub(crate) Arc<crate::invocation::Invocation>);
impl Drop for Scope {
    fn drop(&mut self) {
        let _ = self.0.work.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FaultCode, invocation::Invocation, shared_heap::Owner};
    use std::time::{Duration, Instant};

    #[test]
    fn completion_signals_invocation_and_is_received_once() {
        let invocation = Invocation::new(Limits::default());
        let scope = Scope(invocation.clone());
        let owner = Owner::new(2);
        let mut parent = owner.participant().unwrap();
        let id = invocation
            .work
            .submit(
                &mut parent.enter(),
                vec![Value::Int32(42)],
                |context, mut values, _| context.complete(|_| Ok(values.pop().unwrap())),
            )
            .unwrap();
        assert!(invocation.wake.park(Duration::from_secs(2)));
        let deadline = Instant::now() + Duration::from_secs(2);
        let completion = loop {
            if let Some(value) = invocation.work.take_ready(id).unwrap() {
                break value;
            }
            assert!(Instant::now() < deadline);
            invocation.park();
        };
        assert_eq!(
            completion.receive(&mut parent.enter(), vec![]).unwrap(),
            Value::Int32(42)
        );
        assert!(invocation.work.take_ready(id).is_err());
        invocation.work.shutdown().unwrap();
        assert!(
            invocation
                .work
                .submit(&mut parent.enter(), vec![], |_, _, _| panic!(
                    "closed service admitted work"
                ))
                .is_err()
        );
        drop(scope);
        assert_eq!(Arc::strong_count(&invocation), 1);
    }

    #[test]
    fn root_scope_cancels_and_joins_work_without_holding_service_lock() {
        let invocation = Invocation::new(Limits::default());
        let scope = Scope(invocation.clone());
        let owner = Owner::new(2);
        let mut parent = owner.participant().unwrap();
        let worker_invocation = invocation.clone();
        let (started, ready) = std::sync::mpsc::channel();
        let (finished, done) = std::sync::mpsc::channel();
        invocation
            .work
            .submit(&mut parent.enter(), vec![], move |_, _, control| {
                started.send(()).unwrap();
                let deadline = Instant::now() + Duration::from_secs(2);
                while !control.is_cancelled() {
                    assert!(Instant::now() < deadline);
                    std::thread::yield_now();
                }
                // Shutdown must release the service mutex before waiting for us.
                assert!(worker_invocation.work.take_ready(0).is_err());
                finished.send(()).unwrap();
                Err(Fault::coded(FaultCode::ExecutionCancelled, "scope stopped"))
            })
            .unwrap();
        ready.recv_timeout(Duration::from_secs(2)).unwrap();
        let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _scope = scope;
            panic!("root driver unwound");
        }));
        assert!(unwound.is_err());
        done.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(Arc::strong_count(&invocation), 1);
        assert!(owner.participant().is_ok());
    }

    #[test]
    fn faults_and_panics_wake_and_remain_visible_after_job_consumption() {
        for panic in [false, true] {
            let invocation = Invocation::new(Limits::default());
            let scope = Scope(invocation.clone());
            let owner = Owner::new(2);
            let mut parent = owner.participant().unwrap();
            let id = invocation
                .work
                .submit(&mut parent.enter(), vec![], move |_, _, _| {
                    if panic {
                        panic!("native test panic");
                    }
                    Err(Fault::coded(FaultCode::UserFault, "worker failed"))
                })
                .unwrap();
            assert!(invocation.wake.park(Duration::from_secs(2)));
            let failure = invocation.work.check().unwrap_err();
            assert!(failure.message.contains(if panic {
                "native task panicked"
            } else {
                "worker failed"
            }));
            let deadline = Instant::now() + Duration::from_secs(2);
            loop {
                match invocation.work.take_ready(id) {
                    Err(error) => {
                        assert_eq!(error.message, failure.message);
                        break;
                    }
                    Ok(None) => {
                        assert!(Instant::now() < deadline);
                        invocation.park();
                    }
                    Ok(Some(_)) => panic!("failed worker returned success"),
                }
            }
            assert_eq!(
                invocation.work.shutdown().unwrap_err().message,
                failure.message
            );
            drop(scope);
        }
    }

    #[test]
    fn a_worker_fault_stops_siblings_and_survives_shutdown() {
        let invocation = Invocation::new(Limits::default());
        let scope = Scope(invocation.clone());
        let owner = Owner::new(3);
        let mut parent = owner.participant().unwrap();
        let (started, ready) = std::sync::mpsc::channel();
        let (finished, done) = std::sync::mpsc::channel();
        invocation
            .work
            .submit(&mut parent.enter(), vec![], move |_, _, control| {
                started.send(()).unwrap();
                let deadline = Instant::now() + Duration::from_secs(2);
                while !control.is_cancelled() {
                    assert!(Instant::now() < deadline);
                    std::thread::yield_now();
                }
                finished.send(()).unwrap();
                Err(Fault::coded(
                    FaultCode::ExecutionCancelled,
                    "sibling stopped",
                ))
            })
            .unwrap();
        ready.recv_timeout(Duration::from_secs(2)).unwrap();
        invocation
            .work
            .submit(&mut parent.enter(), vec![], |_, _, _| {
                Err(Fault::coded(FaultCode::UserFault, "primary worker fault"))
            })
            .unwrap();
        done.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(
            invocation.work.shutdown().unwrap_err().message,
            "primary worker fault"
        );
        drop(scope);
    }

    #[test]
    fn host_cancellation_reaches_the_shared_work_owner() {
        let token = CancellationToken::new();
        let invocation = Invocation::with_cancellation(Limits::default(), token.clone());
        let scope = Scope(invocation.clone());
        let owner = Owner::new(2);
        let mut parent = owner.participant().unwrap();
        token.cancel();
        let error = invocation
            .work
            .submit(&mut parent.enter(), vec![], |_, _, _| {
                panic!("cancelled job ran")
            })
            .unwrap_err();
        assert_eq!(error.code, FaultCode::ExecutionCancelled);
        drop(scope);
    }
}
