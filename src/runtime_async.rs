//! Private interpreter POC. Handles belong to one execution context; there is no
//! host completion ABI or managed result rooting contract yet.
use crate::Fault;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{Duration, Instant};

static NEXT: AtomicI64 = AtomicI64::new(1);
const CAPACITY: usize = 64;

enum Operation {
    Pending,
    Body,
    Timer(Instant),
    Completed(i32),
    CompletedUnit,
    Cancelled,
}

#[derive(Default)]
pub(crate) struct Operations(BTreeMap<i64, Operation>, BTreeMap<i64, usize>);

impl Operations {
    pub(crate) fn own(&mut self, id: i64, owner: usize) {
        self.1.insert(id, owner);
    }
    pub(crate) fn abandon(&mut self, id: i64) {
        self.0.remove(&id);
        self.1.remove(&id);
    }
    pub(crate) fn reap(&mut self, heap: &crate::shared_heap::Access<'_>) {
        let dead: Vec<_> = self
            .1
            .iter()
            .filter(|(_, owner)| !heap.contains(**owner))
            .map(|(id, _)| *id)
            .collect();
        for id in dead {
            self.abandon(id);
        }
    }
    pub(crate) fn create(&mut self, milliseconds: Option<i32>) -> Result<i64, Fault> {
        let operation = match milliseconds {
            Some(ms) if ms < 0 => {
                return Err(Fault::new("Runtime async delay must be nonnegative"));
            }
            Some(ms) => Operation::Timer(Instant::now() + Duration::from_millis(ms as u64)),
            None => Operation::Pending,
        };
        if self.0.len() >= CAPACITY {
            return Err(Fault::new("Runtime async operation limit exceeded"));
        }
        let id = NEXT
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| Fault::new("Runtime async identity exhausted"))?;
        self.0.insert(id, operation);
        Ok(id)
    }

    pub(crate) fn start(&mut self) -> Result<i64, Fault> {
        let id = self.create(None)?;
        self.0.insert(id, Operation::Body);
        Ok(id)
    }

    pub(crate) fn complete_body(&mut self, id: i64, value: i32) {
        if let Some(operation @ Operation::Body) = self.0.get_mut(&id) {
            *operation = Operation::Completed(value);
        }
        // Cancellation retains terminal authority over a later body return.
    }

    pub(crate) fn complete(&mut self, id: i64, value: i32) -> bool {
        let Some(operation @ Operation::Pending) = self.0.get_mut(&id) else {
            return false;
        };
        *operation = Operation::Completed(value);
        true
    }

    pub(crate) fn complete_unit(&mut self, id: i64) -> bool {
        let Some(operation @ Operation::Pending) = self.0.get_mut(&id) else {
            return false;
        };
        *operation = Operation::CompletedUnit;
        true
    }

    pub(crate) fn complete_body_unit(&mut self, id: i64) {
        if let Some(operation @ Operation::Body) = self.0.get_mut(&id) {
            *operation = Operation::CompletedUnit;
        }
    }

    pub(crate) fn read_unit(&mut self, id: i64) -> Result<Option<()>, Fault> {
        if !self.is_completed(id)? {
            return Ok(None);
        }
        match self.0.get(&id) {
            Some(Operation::CompletedUnit) => Ok(Some(())),
            Some(Operation::Cancelled) => Err(Fault::coded(
                crate::FaultCode::ExecutionCancelled,
                "Runtime async operation cancelled",
            )),
            _ => Err(Fault::new(
                "Runtime async operation does not have a unit result",
            )),
        }
    }

    pub(crate) fn cancel(&mut self, id: i64) -> bool {
        match self.0.get_mut(&id) {
            Some(operation @ (Operation::Pending | Operation::Body | Operation::Timer(_))) => {
                *operation = Operation::Cancelled;
                true
            }
            _ => false,
        }
    }

    pub(crate) fn wait_timeout(&self, id: i64) -> Duration {
        match self.0.get(&id) {
            Some(Operation::Timer(deadline)) => deadline
                .saturating_duration_since(Instant::now())
                .min(crate::scheduler::POLL_INTERVAL),
            Some(Operation::Pending | Operation::Body) => crate::scheduler::POLL_INTERVAL,
            _ => Duration::ZERO,
        }
    }

    pub(crate) fn is_ready(&self, id: i64) -> bool {
        match self.0.get(&id) {
            Some(Operation::Pending | Operation::Body) => false,
            Some(Operation::Timer(deadline)) => Instant::now() >= *deadline,
            _ => true,
        }
    }

    pub(crate) fn is_completed(&mut self, id: i64) -> Result<bool, Fault> {
        self.validate(id)?;
        if let Some(Operation::Timer(deadline)) = self.0.get(&id) {
            if Instant::now() >= *deadline {
                self.0.insert(id, Operation::Completed(0));
            }
        }
        Ok(matches!(
            self.0.get(&id),
            Some(Operation::Completed(_) | Operation::CompletedUnit | Operation::Cancelled)
        ))
    }

    pub(crate) fn is_cancelled(&self, id: i64) -> Result<bool, Fault> {
        self.validate(id)?;
        Ok(matches!(self.0.get(&id), Some(Operation::Cancelled)))
    }

    /// Awaiter adapters need delivery for every terminal outcome, without reading
    /// a cancelled operation's result and terminating the enclosing invocation.
    pub(crate) fn read_completion(&mut self, id: i64) -> Result<Option<bool>, Fault> {
        if self.is_completed(id)? {
            Ok(Some(!self.is_cancelled(id)?))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn validate(&self, id: i64) -> Result<(), Fault> {
        if self.0.contains_key(&id) {
            Ok(())
        } else {
            Err(Fault::new("Unknown or released runtime async operation"))
        }
    }

    /// VM callers additionally reject release while any activation still uses it.
    pub(crate) fn release(&mut self, id: i64) -> bool {
        if !matches!(
            self.0.get(&id),
            Some(Operation::Completed(_) | Operation::CompletedUnit | Operation::Cancelled)
        ) {
            return false;
        }
        self.abandon(id);
        true
    }

    /// Outcome reads serialize on the same VM dispatcher as producer completion.
    /// No callback can race between the pending check and boundary installation.
    pub(crate) fn read_ready(&mut self, id: i64) -> Result<Option<i32>, Fault> {
        let result = match self.0.get(&id) {
            Some(Operation::Pending | Operation::Body) => return Ok(None),
            Some(Operation::Timer(deadline)) if Instant::now() < *deadline => return Ok(None),
            Some(Operation::Timer(_)) => Ok(Some(0)),
            Some(Operation::Completed(value)) => Ok(Some(*value)),
            Some(Operation::CompletedUnit) => Err(Fault::new(
                "Runtime async operation does not have an Int32 result",
            )),
            Some(Operation::Cancelled) => Err(Fault::coded(
                crate::FaultCode::ExecutionCancelled,
                "Runtime async operation cancelled",
            )),
            None => return Err(Fault::new("Unknown or released runtime async operation")),
        };
        if let Ok(Some(value)) = result {
            // Materialize timer completion so a later cancel cannot change it.
            self.0.insert(id, Operation::Completed(value));
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_results_preserve_producer_authority_and_cancellation() {
        let mut operations = Operations::default();
        let body = operations.start().unwrap();
        assert_eq!(operations.read_unit(body).unwrap(), None);
        assert!(!operations.complete_unit(body));
        operations.complete_body_unit(body);
        assert_eq!(operations.read_unit(body).unwrap(), Some(()));
        assert_eq!(operations.read_unit(body).unwrap(), Some(()));
        assert_eq!(operations.read_completion(body).unwrap(), Some(true));
        assert!(operations.read_ready(body).is_err());
        assert!(operations.release(body));
        assert!(operations.read_unit(body).is_err());

        let cancelled = operations.start().unwrap();
        assert!(operations.cancel(cancelled));
        operations.complete_body_unit(cancelled);
        assert_eq!(operations.read_completion(cancelled).unwrap(), Some(false));
        assert_eq!(
            operations.read_unit(cancelled).unwrap_err().code,
            crate::FaultCode::ExecutionCancelled
        );
        assert!(operations.release(cancelled));
    }

    #[test]
    fn completion_observation_preserves_cancelled_and_successful_outcomes() {
        let mut operations = Operations::default();
        let id = operations.create(None).unwrap();
        assert_eq!(operations.read_completion(id).unwrap(), None);
        assert!(!operations.is_cancelled(id).unwrap());
        assert!(operations.cancel(id));
        assert_eq!(operations.read_completion(id).unwrap(), Some(false));
        assert_eq!(operations.read_completion(id).unwrap(), Some(false));
        assert!(operations.is_cancelled(id).unwrap());
        assert!(operations.read_ready(id).is_err());
        assert!(operations.release(id));
        assert!(operations.is_cancelled(id).is_err());
        assert!(operations.read_completion(id).is_err());

        let timer = operations.create(Some(0)).unwrap();
        assert_eq!(operations.read_completion(timer).unwrap(), Some(true));
        assert!(!operations.cancel(timer));
        assert!(!operations.is_cancelled(timer).unwrap());
        assert_eq!(operations.read_ready(timer).unwrap(), Some(0));
        assert!(Operations::default().read_completion(timer).is_err());
    }

    #[test]
    fn handles_are_context_owned_and_capacity_returns_only_on_release() {
        let mut owner = Operations::default();
        let mut other = Operations::default();
        let ids: Vec<_> = (0..CAPACITY).map(|_| owner.create(None).unwrap()).collect();
        assert!(owner.create(None).is_err());
        assert!(!other.complete(ids[0], 42));
        assert!(!other.cancel(ids[0]));
        assert!(other.read_ready(ids[0]).is_err());
        assert!(owner.complete(ids[0], 42));
        assert_eq!(owner.read_ready(ids[0]).unwrap(), Some(42));
        assert_eq!(owner.read_ready(ids[0]).unwrap(), Some(42));
        assert!(owner.create(None).is_err());
        assert!(owner.release(ids[0]));
        assert!(!owner.release(ids[0]));
        assert!(owner.create(None).is_ok());
    }

    #[test]
    fn body_owns_completion_and_cancellation_wins_over_late_return() {
        let mut operations = Operations::default();
        let body = operations.start().unwrap();
        assert!(!operations.complete(body, 99));
        assert!(!operations.is_ready(body));
        assert!(operations.cancel(body));
        operations.complete_body(body, 42);
        assert_eq!(
            operations.read_ready(body).unwrap_err().code,
            crate::FaultCode::ExecutionCancelled
        );
        let completed = operations.start().unwrap();
        operations.complete_body(completed, 42);
        operations.complete_body(completed, 99);
        assert_eq!(operations.read_ready(completed).unwrap(), Some(42));
    }

    #[test]
    fn readiness_observes_terminal_state_without_consuming_result() {
        let mut operations = Operations::default();
        let pending = operations.create(None).unwrap();
        assert!(!operations.is_completed(pending).unwrap());
        assert!(operations.complete(pending, 42));
        assert!(operations.is_completed(pending).unwrap());
        assert_eq!(operations.read_ready(pending).unwrap(), Some(42));
        let cancelled = operations.create(None).unwrap();
        assert!(operations.cancel(cancelled));
        assert!(operations.is_completed(cancelled).unwrap());
        let timer = operations.create(Some(0)).unwrap();
        assert!(operations.is_completed(timer).unwrap());
        assert!(!operations.cancel(timer));
        assert_eq!(operations.read_ready(timer).unwrap(), Some(0));
        assert!(operations.release(timer));
        assert!(operations.is_completed(timer).is_err());
        assert!(operations.is_completed(0).is_err());
    }

    #[test]
    fn zero_timer_and_cancelled_timer_have_terminal_outcomes() {
        let mut operations = Operations::default();
        let ready = operations.create(Some(0)).unwrap();
        assert_eq!(operations.wait_timeout(ready), Duration::ZERO);
        assert_eq!(operations.read_ready(ready).unwrap(), Some(0));
        let pending = operations.create(Some(i32::MAX)).unwrap();
        assert!(!operations.complete(pending, 42), "timer owns its result");
        assert!(operations.cancel(pending));
        assert!(!operations.cancel(pending));
        assert_eq!(
            operations.read_ready(pending).unwrap_err().code,
            crate::FaultCode::ExecutionCancelled
        );
        assert!(operations.validate(pending).is_ok());
        assert!(operations.release(pending));
        assert!(operations.validate(pending).is_err());
    }
}
