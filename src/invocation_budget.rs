//! Shared instruction fuel and live-frame admission for one invocation.
//! Waiting consumes no fuel. Dropping a frame releases capacity, never instruction fuel.
use crate::{Fault, FaultCode, Limits};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

pub(crate) struct Budget {
    remaining: AtomicUsize,
    frames: AtomicUsize,
    frame_limit: usize,
}
pub(crate) struct FramePermit(Arc<Budget>);
impl Drop for FramePermit {
    fn drop(&mut self) {
        self.0.frames.fetch_sub(1, Ordering::Relaxed);
    }
}
impl FramePermit {
    pub(crate) fn belongs_to(&self, budget: &Arc<Budget>) -> bool {
        Arc::ptr_eq(&self.0, budget)
    }
}
impl Budget {
    pub(crate) fn new(limits: Limits) -> Arc<Self> {
        Arc::new(Self {
            remaining: AtomicUsize::new(limits.instructions),
            frames: AtomicUsize::new(0),
            frame_limit: limits.frames,
        })
    }

    pub(crate) fn remaining(&self) -> usize {
        self.remaining.load(Ordering::Relaxed)
    }

    // The replacement try_update is newer than our Rust 1.85 minimum.
    #[allow(deprecated)]
    pub(crate) fn charge(&self) -> Result<(), Fault> {
        self.remaining
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |fuel| {
                fuel.checked_sub(1)
            })
            .map(|_| ())
            .map_err(|_| {
                Fault::coded(
                    FaultCode::InstructionLimitExceeded,
                    "instruction limit exceeded",
                )
            })
    }

    // The replacement try_update is newer than our Rust 1.85 minimum.
    #[allow(deprecated)]
    pub(crate) fn frame(self: &Arc<Self>) -> Result<FramePermit, Fault> {
        self.frames
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |live| {
                if live < self.frame_limit {
                    Some(live + 1)
                } else {
                    None
                }
            })
            .map_err(|_| {
                Fault::coded(FaultCode::StackOverflow, "invocation frame limit exceeded")
            })?;
        Ok(FramePermit(self.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_consumers_share_exact_fuel_without_underflow() {
        let budget = Budget::new(Limits {
            instructions: 101,
            ..Default::default()
        });
        let workers: Vec<_> = (0..4)
            .map(|_| {
                let budget = budget.clone();
                std::thread::spawn(move || {
                    let mut charged = 0;
                    while budget.charge().is_ok() {
                        charged += 1;
                    }
                    charged
                })
            })
            .collect();
        assert_eq!(
            workers
                .into_iter()
                .map(|worker| worker.join().unwrap())
                .sum::<usize>(),
            101
        );
        assert_eq!(budget.remaining(), 0);
        assert_eq!(
            budget.charge().unwrap_err().code,
            FaultCode::InstructionLimitExceeded
        );
    }

    #[test]
    fn live_frames_are_bounded_and_released_on_unwind() {
        let budget = Budget::new(Limits {
            frames: 1,
            ..Default::default()
        });
        let other = budget.clone();
        let _ = std::panic::catch_unwind(move || {
            let _permit = other.frame().unwrap();
            assert!(other.frame().is_err());
            panic!("simulate a terminal unwind");
        });
        let permit = budget.frame().unwrap();
        assert!(permit.belongs_to(&budget));
        assert!(!permit.belongs_to(&Budget::new(Limits::default())));
        drop(permit);
        assert_eq!(budget.frames.load(Ordering::Relaxed), 0);
    }
}
