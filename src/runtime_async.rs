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
    Timer(Instant),
    Completed(i32),
    Cancelled,
}

#[derive(Default)]
pub(crate) struct Operations(BTreeMap<i64, Operation>);

impl Operations {
    pub(crate) fn create(&mut self, milliseconds: Option<i32>) -> Result<i64, Fault> {
        let operation = match milliseconds {
            Some(ms) if ms < 0 => {
                return Err(Fault::new("Runtime async delay must be nonnegative"))
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

    pub(crate) fn complete(&mut self, id: i64, value: i32) -> bool {
        let Some(operation @ Operation::Pending) = self.0.get_mut(&id) else {
            return false;
        };
        *operation = Operation::Completed(value);
        true
    }

    pub(crate) fn cancel(&mut self, id: i64) -> bool {
        match self.0.get_mut(&id) {
            Some(operation @ (Operation::Pending | Operation::Timer(_))) => {
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
            Some(Operation::Pending) => crate::scheduler::POLL_INTERVAL,
            _ => Duration::ZERO,
        }
    }

    pub(crate) fn is_ready(&self, id: i64) -> bool {
        match self.0.get(&id) {
            Some(Operation::Pending) => false,
            Some(Operation::Timer(deadline)) => Instant::now() >= *deadline,
            _ => true,
        }
    }

    pub(crate) fn validate(&self, id: i64) -> Result<(), Fault> {
        if self.0.contains_key(&id) {
            Ok(())
        } else {
            Err(Fault::new("Unknown or consumed runtime async operation"))
        }
    }

    /// Inspection and consumption occur on the same VM dispatcher as completion.
    /// No callback can race between the pending check and boundary installation.
    pub(crate) fn take_ready(&mut self, id: i64) -> Result<Option<i32>, Fault> {
        let result = match self.0.get(&id) {
            Some(Operation::Pending) => return Ok(None),
            Some(Operation::Timer(deadline)) if Instant::now() < *deadline => return Ok(None),
            Some(Operation::Timer(_)) => Ok(Some(0)),
            Some(Operation::Completed(value)) => Ok(Some(*value)),
            Some(Operation::Cancelled) => Err(Fault::coded(
                crate::FaultCode::ExecutionCancelled,
                "Runtime async operation cancelled",
            )),
            None => return Err(Fault::new("Unknown or consumed runtime async operation")),
        };
        self.0.remove(&id);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_are_context_owned_and_capacity_returns_on_consumption() {
        let mut owner = Operations::default();
        let mut other = Operations::default();
        let ids: Vec<_> = (0..CAPACITY).map(|_| owner.create(None).unwrap()).collect();
        assert!(owner.create(None).is_err());
        assert!(!other.complete(ids[0], 42));
        assert!(!other.cancel(ids[0]));
        assert!(other.take_ready(ids[0]).is_err());
        assert!(owner.complete(ids[0], 42));
        assert_eq!(owner.take_ready(ids[0]).unwrap(), Some(42));
        assert!(owner.create(None).is_ok());
    }

    #[test]
    fn zero_timer_and_cancelled_timer_have_terminal_outcomes() {
        let mut operations = Operations::default();
        let ready = operations.create(Some(0)).unwrap();
        assert_eq!(operations.wait_timeout(ready), Duration::ZERO);
        assert_eq!(operations.take_ready(ready).unwrap(), Some(0));
        let pending = operations.create(Some(i32::MAX)).unwrap();
        assert!(!operations.complete(pending, 42), "timer owns its result");
        assert!(operations.cancel(pending));
        assert!(!operations.cancel(pending));
        assert_eq!(
            operations.take_ready(pending).unwrap_err().code,
            crate::FaultCode::ExecutionCancelled
        );
        assert!(operations.validate(pending).is_err());
    }
}
