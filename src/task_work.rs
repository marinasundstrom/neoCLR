//! Bounded native work ownership for the shared-context Task.Run backend.
//! This component does not yet invoke guest callbacks. Workers must publish roots
//! before releasing heap access and must not hold access during blocking host work.
//! Shutdown/join must run outside heap access (never from a worker owned here).
use crate::{
    CancellationToken, Fault, FaultCode, Value,
    shared_heap::{Access, Identity, Participant},
};
use std::thread::{self, JoinHandle};

type Outcome = Result<Completion, Fault>;

#[derive(Clone)]
pub(crate) struct Control {
    invocation: CancellationToken,
    stop: CancellationToken,
}
impl Control {
    pub(crate) fn is_cancelled(&self) -> bool {
        self.invocation.is_cancelled() || self.stop.is_cancelled()
    }
}

pub(crate) struct Context {
    participant: Participant,
    identity: Identity,
    control: Control,
}
impl Context {
    /// Intermediate execution intervals must publish their complete roots before
    /// returning. Blocking host operations belong outside this guard.
    pub(crate) fn enter(&mut self) -> Result<Access<'_>, Fault> {
        self.participant
            .enter_cancellable(|| self.control.is_cancelled())
    }

    /// Produce and root the result under one access interval, closing the gap
    /// between creating a reference result and handing it to the native joiner.
    pub(crate) fn complete(
        mut self,
        operation: impl FnOnce(&mut Access<'_>) -> Result<Value, Fault>,
    ) -> Outcome {
        let value = {
            let mut access = self
                .participant
                .enter_cancellable(|| self.control.is_cancelled())?;
            let value = operation(&mut access)?;
            access.publish_value(&value)?;
            value
        };
        Ok(Completion {
            value,
            participant: self.participant,
            identity: self.identity,
        })
    }
}

pub(crate) struct Completion {
    value: Value,
    // Ownership retains registered result roots even after native thread exit.
    participant: Participant,
    identity: Identity,
}
impl Completion {
    /// Publish into the receiver before releasing the completion registration.
    /// The caller supplies all its other live roots, not just the new result.
    pub(crate) fn receive(
        self,
        access: &mut Access<'_>,
        mut roots: Vec<usize>,
    ) -> Result<Value, Fault> {
        if !self.identity.matches(access) {
            return Err(Fault::new("task completion belongs to another invocation"));
        }
        crate::gc::trace(&self.value, &mut roots);
        access.publish(roots)?;
        let Self {
            value, participant, ..
        } = self;
        drop(participant);
        Ok(value)
    }
}

pub(crate) struct Work {
    identity: Identity,
    cancellation: Control,
    jobs: Vec<Option<JoinHandle<Outcome>>>,
    limit: usize,
}
impl Work {
    pub(crate) fn new(access: &Access<'_>, limit: usize, cancellation: CancellationToken) -> Self {
        Self {
            identity: access.identity(),
            cancellation: Control {
                invocation: cancellation,
                stop: CancellationToken::new(),
            },
            jobs: Vec::new(),
            limit,
        }
    }

    /// The initial bound is total submissions for this owner, like isolated workers.
    /// Joined slots are not reused, so a stale handle can never name a later job.
    pub(crate) fn submit(
        &mut self,
        access: &mut Access<'_>,
        captures: Vec<usize>,
        callback: impl FnOnce(Context, &Control) -> Outcome + Send + 'static,
    ) -> Result<usize, Fault> {
        if !self.identity.matches(access) {
            return Err(Fault::new("task submission belongs to another invocation"));
        }
        cancelled(&self.cancellation)?;
        if self.jobs.len() >= self.limit {
            return Err(Fault::new("native task submission limit exceeded"));
        }
        let participant = access.fork(captures)?;
        let cancellation = self.cancellation.clone();
        let identity = self.identity.clone();
        let worker = thread::Builder::new()
            .name("neoclr-task".into())
            .spawn(move || {
                cancelled(&cancellation)?;
                let completion = callback(
                    Context {
                        participant,
                        identity,
                        control: cancellation.clone(),
                    },
                    &cancellation,
                )?;
                cancelled(&cancellation)?;
                Ok(completion)
            })
            .map_err(|error| Fault::new(format!("cannot start native task: {error}")))?;
        let id = self.jobs.len();
        self.jobs.push(Some(worker));
        Ok(id)
    }

    pub(crate) fn take_ready(&mut self, id: usize) -> Result<Option<Completion>, Fault> {
        let job = self
            .jobs
            .get(id)
            .and_then(Option::as_ref)
            .ok_or_else(|| Fault::new("invalid or consumed native task handle"))?;
        if !job.is_finished() {
            return Ok(None);
        }
        self.join(id).map(Some)
    }

    /// Blocking host operation: never call with a shared heap Access guard held.
    pub(crate) fn join(&mut self, id: usize) -> Outcome {
        let job = self
            .jobs
            .get_mut(id)
            .and_then(Option::take)
            .ok_or_else(|| Fault::new("invalid or consumed native task handle"))?;
        join(job)
    }

    /// Cancel and join every outstanding worker, even when one fails. Normal
    /// invocation teardown must inspect this result; Drop is the unwinding fallback.
    pub(crate) fn shutdown(&mut self) -> Result<(), Fault> {
        self.cancellation.stop.cancel();
        let mut failure = None;
        for job in &mut self.jobs {
            if let Some(job) = job.take() {
                if let Err(error) = join(job) {
                    if error.code != FaultCode::ExecutionCancelled && failure.is_none() {
                        failure = Some(error);
                    }
                }
            }
        }
        failure.map_or(Ok(()), Err)
    }
}
impl Drop for Work {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}
fn join(job: JoinHandle<Outcome>) -> Outcome {
    job.join().map_err(|_| Fault::new("native task panicked"))?
}
fn cancelled(token: &Control) -> Result<(), Fault> {
    if token.is_cancelled() {
        Err(Fault::coded(
            FaultCode::ExecutionCancelled,
            "native task cancelled",
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CollectionReason, shared_heap::Owner};
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    };

    #[test]
    fn blocking_work_allows_caller_progress_and_retains_shared_capture_and_result() {
        let owner = Owner::new(2);
        let mut parent = owner.participant().unwrap();
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let (mut work, id, job) = {
            let mut access = parent.enter();
            let id = access.allocate(Value::Int32(1)).unwrap();
            let mut work = Work::new(&access, 1, CancellationToken::new());
            let job = work
                .submit(&mut access, vec![id], move |mut context, token| {
                    {
                        let mut access = context.enter()?;
                        let scratch = access.allocate(Value::String("suspended local".into()))?;
                        access.publish(vec![id, scratch])?;
                    }
                    started_tx.send(()).unwrap();
                    // A blocking operation with a cooperative cancellation check.
                    loop {
                        match release_rx.recv_timeout(std::time::Duration::from_millis(10)) {
                            Ok(()) => break,
                            Err(mpsc::RecvTimeoutError::Timeout) => cancelled(token)?,
                            Err(mpsc::RecvTimeoutError::Disconnected) => {
                                return Err(Fault::new("test peer disconnected"));
                            }
                        }
                    }
                    context.complete(|access| {
                        let reference = access.address(id)?;
                        assert_eq!(reference.read()?, Value::Int32(41));
                        reference.write(Value::Int32(42))?;
                        Ok(Value::SlotReference(reference))
                    })
                })
                .unwrap();
            (work, id, job)
        };
        started_rx.recv().unwrap();
        assert!(work.take_ready(job).unwrap().is_none());
        {
            let mut access = parent.enter();
            access
                .collect(vec![], CollectionReason::ExplicitRequest)
                .unwrap();
            assert_eq!(access.len(), 2);
            access.address(id).unwrap().write(Value::Int32(41)).unwrap();
        }
        release_tx.send(()).unwrap();
        let completion = work.join(job).unwrap();
        assert!(work.join(job).is_err());
        {
            let mut access = parent.enter();
            access
                .collect(vec![], CollectionReason::ExplicitRequest)
                .unwrap();
            let result = completion.receive(&mut access, vec![]).unwrap();
            let Value::SlotReference(reference) = result else {
                panic!()
            };
            assert_eq!(reference.allocation_id(), Some(id));
            assert_eq!(reference.read().unwrap(), Value::Int32(42));
            access
                .collect(vec![id], CollectionReason::ExplicitRequest)
                .unwrap();
            access
                .collect(vec![], CollectionReason::ExplicitRequest)
                .unwrap();
            assert!(access.is_empty());
        }
        work.shutdown().unwrap();
        drop(parent);
        owner.into_heap().unwrap();
    }

    #[test]
    fn completion_roots_fresh_reference_result_before_native_return() {
        let owner = Owner::new(2);
        let mut parent = owner.participant().unwrap();
        let (mut work, job) = {
            let mut access = parent.enter();
            let mut work = Work::new(&access, 1, CancellationToken::new());
            let job = work
                .submit(&mut access, vec![], |context, _| {
                    context.complete(|access| {
                        let id = access.allocate(Value::String("result".into()))?;
                        Ok(Value::SlotReference(access.address(id)?))
                    })
                })
                .unwrap();
            (work, job)
        };
        let completion = work.join(job).unwrap();
        let mut access = parent.enter();
        access
            .collect(vec![], CollectionReason::ExplicitRequest)
            .unwrap();
        assert_eq!(access.len(), 1);
        let value = completion.receive(&mut access, vec![]).unwrap();
        let Value::SlotReference(reference) = value else {
            panic!()
        };
        assert_eq!(reference.read().unwrap(), Value::String("result".into()));
        drop(access);
    }

    #[test]
    fn rejection_does_not_start_callback_and_handles_are_never_reused() {
        let owner = Owner::new(2);
        let mut parent = owner.participant().unwrap();
        let (mut work, job) = {
            let mut access = parent.enter();
            let mut work = Work::new(&access, 1, CancellationToken::new());
            let job = work
                .submit(&mut access, vec![], |context, _| {
                    context.complete(|_| Ok(Value::Void))
                })
                .unwrap();
            assert!(
                work.submit(&mut access, vec![], |_, _| panic!("must not run"))
                    .is_err()
            );
            (work, job)
        };
        drop(work.join(job).unwrap());
        {
            let mut access = parent.enter();
            assert!(
                work.submit(&mut access, vec![], |_, _| panic!("must not run"))
                    .is_err()
            );
        }
        assert!(work.take_ready(99).is_err());
    }

    #[test]
    fn shutdown_cancels_and_joins_work_without_cancelling_the_host_token() {
        let owner = Owner::new(2);
        let mut parent = owner.participant().unwrap();
        let host = CancellationToken::new();
        let stopped = Arc::new(AtomicBool::new(false));
        let (started_tx, started_rx) = mpsc::channel();
        let mut work = {
            let mut access = parent.enter();
            let mut work = Work::new(&access, 1, host.clone());
            let stopped = stopped.clone();
            work.submit(&mut access, vec![], move |context, token| {
                started_tx.send(()).unwrap();
                while !token.is_cancelled() {
                    thread::yield_now();
                }
                stopped.store(true, Ordering::Release);
                context.complete(|_| Ok(Value::Void))
            })
            .unwrap();
            work
        };
        started_rx.recv().unwrap();
        work.shutdown().unwrap();
        assert!(stopped.load(Ordering::Acquire));
        assert!(!host.is_cancelled());
        drop(parent);
        owner.into_heap().unwrap();
    }

    #[test]
    fn faults_stay_faults_and_cleanup_still_joins_other_jobs() {
        let owner = Owner::new(3);
        let mut parent = owner.participant().unwrap();
        let (ready_tx, ready_rx) = mpsc::channel();
        let mut work = {
            let mut access = parent.enter();
            let mut work = Work::new(&access, 2, CancellationToken::new());
            let failed_tx = ready_tx.clone();
            work.submit(&mut access, vec![], move |_, _| {
                failed_tx.send(()).unwrap();
                Err(Fault::coded(FaultCode::UserFault, "callback failed"))
            })
            .unwrap();
            work.submit(&mut access, vec![], move |context, token| {
                ready_tx.send(()).unwrap();
                while !token.is_cancelled() {
                    thread::yield_now();
                }
                context.complete(|_| Ok(Value::Void))
            })
            .unwrap();
            work
        };
        ready_rx.recv().unwrap();
        ready_rx.recv().unwrap();
        let fault = work.shutdown().unwrap_err();
        assert_eq!(fault.code, FaultCode::UserFault);
        assert_eq!(fault.message, "callback failed");
        drop(parent);
        owner.into_heap().unwrap();
    }

    #[test]
    fn foreign_invocation_submission_and_completion_are_rejected() {
        let first = Owner::new(2);
        let second = Owner::new(1);
        let mut parent = first.participant().unwrap();
        let mut foreign = second.participant().unwrap();
        let (mut work, job) = {
            let mut access = parent.enter();
            let mut work = Work::new(&access, 1, CancellationToken::new());
            let job = work
                .submit(&mut access, vec![], |context, _| {
                    context.complete(|_| Ok(Value::Int32(42)))
                })
                .unwrap();
            (work, job)
        };
        let completion = work.join(job).unwrap();
        {
            let mut access = foreign.enter();
            assert!(
                work.submit(&mut access, vec![], |_, _| panic!("must not run"))
                    .is_err()
            );
            assert!(completion.receive(&mut access, vec![]).is_err());
        }
    }

    #[test]
    fn reference_result_from_another_heap_with_the_same_id_is_rejected() {
        let owner = Owner::new(2);
        let foreign = Owner::new(1);
        let mut parent = owner.participant().unwrap();
        let mut other = foreign.participant().unwrap();
        let value = {
            let mut access = other.enter();
            let id = access.allocate(Value::Int32(99)).unwrap();
            access.publish(vec![id]).unwrap();
            Value::SlotReference(access.address(id).unwrap())
        };
        let (mut work, job) = {
            let mut access = parent.enter();
            let id = access.allocate(Value::Int32(1)).unwrap();
            access.publish(vec![id]).unwrap();
            let mut work = Work::new(&access, 1, CancellationToken::new());
            let job = work
                .submit(&mut access, vec![], move |context, _| {
                    context.complete(|_| Ok(value))
                })
                .unwrap();
            (work, job)
        };
        match work.join(job) {
            Err(fault) => assert!(fault.message.contains("does not belong")),
            Ok(_) => panic!("foreign heap result accepted"),
        }
    }

    #[test]
    fn cancellation_releases_a_worker_waiting_for_heap_access() {
        let owner = Owner::new(2);
        let mut parent = owner.participant().unwrap();
        let (ready_tx, ready_rx) = mpsc::channel();
        let mut access = parent.enter();
        let mut work = Work::new(&access, 1, CancellationToken::new());
        work.submit(&mut access, vec![], move |context, _| {
            ready_tx.send(()).unwrap();
            context.complete(|_| Ok(Value::Void))
        })
        .unwrap();
        ready_rx.recv().unwrap();
        // Keep access held to exercise cancellation of the worker's gate wait.
        work.shutdown().unwrap();
        access
            .collect(vec![], CollectionReason::ExplicitRequest)
            .unwrap();
        assert!(access.is_empty());
    }

    #[test]
    fn prior_host_cancellation_rejects_admission() {
        let owner = Owner::new(1);
        let mut parent = owner.participant().unwrap();
        let host = CancellationToken::new();
        host.cancel();
        let mut access = parent.enter();
        let mut work = Work::new(&access, 1, host);
        assert_eq!(
            work.submit(&mut access, vec![], |_, _| panic!("must not run"))
                .unwrap_err()
                .code,
            FaultCode::ExecutionCancelled
        );
    }
}
