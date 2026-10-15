//! Invocation-owned, one-shot monotonic timers. Polling never invokes guest code.
use crate::{Fault, Value};
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicI64, Ordering},
    time::{Duration, Instant},
};

const MAX_TIMERS: usize = 64;
static NEXT_ID: AtomicI64 = AtomicI64::new(1);

#[derive(Clone, Copy)]
pub(crate) enum Operation {
    Start,
    Cancel,
}

struct Timer {
    deadline: Instant,
    callback: Value,
}

#[derive(Default)]
pub(crate) struct Timers {
    entries: BTreeMap<i64, Timer>,
}

impl Timers {
    pub(crate) fn invoke(&mut self, operation: Operation, args: &[Value]) -> Result<Value, Fault> {
        match (operation, args) {
            (Operation::Start, [Value::Int32(delay), callback]) => self
                .start(*delay, callback.clone(), Instant::now())
                .map(Value::Int64),
            (Operation::Cancel, [Value::Int64(id)]) => Ok(Value::Boolean(self.cancel(*id))),
            _ => Err(Fault::new("Invalid timer arguments")),
        }
    }

    pub(crate) fn start(
        &mut self,
        milliseconds: i32,
        callback: Value,
        now: Instant,
    ) -> Result<i64, Fault> {
        if milliseconds < 0 {
            return Err(Fault::new("Timer delay must be nonnegative"));
        }
        if !matches!(callback, Value::Function(_))
            || callback.ty() != crate::assembler::parse_type("fn<Void>")?
        {
            return Err(Fault::new("Timer requires a unit callback"));
        }
        if self.entries.len() >= MAX_TIMERS {
            return Err(Fault::new("Pending timer limit exceeded"));
        }
        let deadline = now
            .checked_add(Duration::from_millis(milliseconds as u64))
            .ok_or_else(|| Fault::new("Timer deadline overflow"))?;
        // Process-unique IDs prevent stale or foreign invocation handles cancelling new work.
        #[allow(deprecated)]
        let id = NEXT_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| Fault::new("Timer handle limit exceeded"))?;
        self.entries.insert(id, Timer { deadline, callback });
        Ok(id)
    }

    pub(crate) fn cancel(&mut self, id: i64) -> bool {
        self.entries.remove(&id).is_some()
    }

    pub(crate) fn poll(&mut self, now: Instant) -> Option<Value> {
        // Earliest deadline first; unique IDs provide stable ordering for equal deadlines.
        let id = self
            .entries
            .iter()
            .filter(|(_, timer)| timer.deadline <= now)
            .min_by_key(|(id, timer)| (timer.deadline, **id))
            .map(|(id, _)| *id)?;
        self.entries.remove(&id).map(|timer| timer.callback)
    }

    /// Scheduling hint only: observing a deadline never consumes an operation.
    pub(crate) fn next_deadline(&self) -> Option<Instant> {
        self.entries.values().map(|timer| timer.deadline).min()
    }

    pub(crate) fn pending(&self) -> bool {
        !self.entries.is_empty()
    }

    pub(crate) fn trace_roots(&self, roots: &mut Vec<usize>) {
        for timer in self.entries.values() {
            crate::gc::trace(&timer.callback, roots);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn callback() -> Value {
        Value::Function(crate::Function {
            view: None,
            identity: std::sync::Arc::new(()),
            ty: crate::assembler::parse_type("fn<Void>").unwrap(),
            target: Some(crate::assembler::parse_function_ref("Ready()").unwrap()),
            native: None,
            receiver: None,
        })
    }

    #[test]
    fn deadlines_are_one_shot_and_zero_delay_is_deferred_until_poll() {
        let now = Instant::now();
        let mut timers = Timers::default();
        let late = callback();
        let early = callback();
        let zero = callback();
        let late_id = timers.start(i32::MAX, late.clone(), now).unwrap();
        timers.start(10, early.clone(), now).unwrap();
        timers.start(0, zero.clone(), now).unwrap();
        assert_eq!(timers.poll(now), Some(zero));
        assert_eq!(timers.poll(now + Duration::from_millis(9)), None);
        assert_eq!(timers.poll(now + Duration::from_millis(10)), Some(early));
        assert_eq!(
            timers.poll(now + Duration::from_millis(i32::MAX as u64)),
            Some(late)
        );
        assert!(!timers.cancel(late_id));
        assert!(!timers.pending());
    }

    #[test]
    fn cancellation_releases_capacity_and_stale_handles_do_not_alias() {
        let now = Instant::now();
        let mut timers = Timers::default();
        let first = timers.start(0, callback(), now).unwrap();
        for _ in 1..MAX_TIMERS {
            timers.start(0, callback(), now).unwrap();
        }
        assert!(timers.start(0, callback(), now).is_err());
        assert!(timers.cancel(first));
        assert!(!timers.cancel(first));
        let next = timers.start(0, callback(), now).unwrap();
        assert_ne!(first, next);
        let mut other = Timers::default();
        let foreign = other.start(0, callback(), now).unwrap();
        assert!(!timers.cancel(foreign));
        assert!(!other.cancel(next));
        assert!(other.pending());
    }

    #[test]
    fn equal_deadlines_preserve_registration_order() {
        let now = Instant::now();
        let mut timers = Timers::default();
        let first = callback();
        let second = callback();
        timers.start(0, first.clone(), now).unwrap();
        timers.start(0, second.clone(), now).unwrap();
        assert_eq!(timers.poll(now), Some(first));
        assert_eq!(timers.poll(now), Some(second));
        assert_eq!(timers.poll(now), None);
    }

    #[test]
    fn invalid_submission_does_not_retain_work() {
        let now = Instant::now();
        let mut timers = Timers::default();
        assert!(timers.start(-1, callback(), now).is_err());
        assert!(timers.start(0, Value::Int32(42), now).is_err());
        assert!(!timers.pending());
        assert!(!timers.cancel(0));
    }
}
