//! Private host resolver. Blocking work owns host data only; the VM owns callbacks.
//! No guest-facing DNS contract is exposed by this checkpoint.

use crate::Value;
use std::{
    collections::BTreeMap,
    net::{Ipv4Addr, ToSocketAddrs},
    sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

const MAX_OPERATIONS: usize = 8;
const MAX_ADDRESSES: usize = 16;
static NEXT_ID: AtomicU64 = AtomicU64::new(1);
static HOST_WORK: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct OperationId(u64);
#[derive(Clone, Copy)]
pub(crate) enum Operation {
    Lookup,
    Result,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Error {
    InvalidName,
    InvalidCallback,
    Limit,
    Failed,
    NoAddress,
    Cancelled,
    TimedOut,
}
type Outcome = Result<Vec<Ipv4Addr>, Error>;
type Lookup = dyn Fn(&str) -> Outcome + Send + Sync;

// Kept by the blocking call, not by its invocation. Cancelling/dropping an
// invocation cannot admit unlimited replacement threads while libc is stuck.
struct Permit(&'static AtomicUsize);
impl Permit {
    fn acquire(count: &'static AtomicUsize) -> Result<Self, Error> {
        count
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| {
                (n < 4).then_some(n + 1)
            })
            .map(|_| Self(count))
            .map_err(|_| Error::Limit)
    }
}
impl Drop for Permit {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

fn host_lookup(name: &str) -> Outcome {
    let addresses = (name, 0).to_socket_addrs().map_err(|_| Error::Failed)?;
    ipv4_addresses(addresses)
}
fn ipv4_addresses(addresses: impl Iterator<Item = std::net::SocketAddr>) -> Outcome {
    let mut result = Vec::new();
    for (index, address) in addresses.enumerate() {
        if index >= 256 {
            return Err(Error::Limit);
        }
        if let std::net::IpAddr::V4(address) = address.ip() {
            if !result.contains(&address) {
                if result.len() == MAX_ADDRESSES {
                    return Err(Error::Limit);
                }
                result.push(address);
            }
        }
    }
    if result.is_empty() {
        Err(Error::NoAddress)
    } else {
        Ok(result)
    }
}
struct Pending {
    callback: Option<Value>,
    receiver: mpsc::Receiver<Outcome>,
    deadline: Instant,
    outcome: Option<Outcome>,
}
pub(crate) struct Resolver {
    operations: BTreeMap<OperationId, Pending>,
    wake: Arc<crate::scheduler::Wake>,
    lookup: Arc<Lookup>,
    permits: &'static AtomicUsize,
}
impl Resolver {
    pub(crate) fn invoke(
        &mut self,
        operation: Operation,
        args: &[Value],
    ) -> Result<Value, crate::Fault> {
        let result = match (operation, args) {
            (Operation::Lookup, [Value::String(name), callback]) => self
                .submit(name, callback.clone(), Duration::from_secs(5))
                .map(|id| Value::Int64(id.0 as i64)),
            (Operation::Result, [Value::Int64(id)]) => self
                .take_result(OperationId(*id as u64))
                .map_err(|_| crate::Fault::new("Unknown resolver operation"))?
                .ok_or_else(|| crate::Fault::new("Resolver result is not ready"))?
                .map(|addresses| Value::Array {
                    element: crate::metadata::Type::String,
                    elements: addresses
                        .into_iter()
                        .map(|ip| Value::String(ip.to_string().into()))
                        .collect(),
                }),
            _ => {
                return Err(crate::Fault::new(
                    "Invalid resolver runtime service arguments",
                ));
            }
        };
        Ok(Value::Erased(Box::new(result.unwrap_or_else(|error| {
            Value::Byte(match error {
                Error::InvalidName => 1,
                Error::Limit => 2,
                Error::NoAddress => 3,
                Error::TimedOut => 4,
                Error::Cancelled => 5,
                _ => 6,
            })
        }))))
    }
    pub(crate) fn with_wake(wake: Arc<crate::scheduler::Wake>) -> Self {
        Self {
            operations: BTreeMap::new(),
            wake,
            lookup: Arc::new(host_lookup),
            permits: &HOST_WORK,
        }
    }
    pub(crate) fn submit(
        &mut self,
        name: &str,
        callback: Value,
        timeout: Duration,
    ) -> Result<OperationId, Error> {
        self.submit_until(name, callback, timeout, None)
    }
    pub(crate) fn submit_until(
        &mut self,
        name: &str,
        callback: Value,
        timeout: Duration,
        until: Option<Instant>,
    ) -> Result<OperationId, Error> {
        if name.is_empty()
            || name.len() > 253
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
        {
            return Err(Error::InvalidName);
        }
        if !matches!(callback, Value::Delegate(_))
            || callback.ty() != crate::assembler::parse_type("System.Func<Void>").unwrap()
        {
            return Err(Error::InvalidCallback);
        }
        if self.operations.len() >= MAX_OPERATIONS {
            return Err(Error::Limit);
        }
        let now = Instant::now();
        let phase_deadline = now.checked_add(timeout).ok_or(Error::Limit)?;
        let deadline = until.map_or(phase_deadline, |end| end.min(phase_deadline));
        if until.is_some_and(|end| now >= end) {
            return Err(Error::TimedOut);
        }
        let permit = Permit::acquire(self.permits)?;
        let id = OperationId(
            NEXT_ID
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| {
                    (id < i64::MAX as u64).then_some(id + 1)
                })
                .map_err(|_| Error::Limit)?,
        );
        let name = name.to_owned();
        let lookup = self.lookup.clone();
        let wake = self.wake.clone();
        let (sender, receiver) = mpsc::channel();
        std::thread::Builder::new()
            .name("neoclr-resolver".into())
            .spawn(move || {
                let outcome =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| lookup(&name)))
                        .unwrap_or(Err(Error::Failed))
                        .and_then(|addresses| {
                            if addresses.is_empty() {
                                Err(Error::NoAddress)
                            } else if addresses.len() > MAX_ADDRESSES {
                                Err(Error::Limit)
                            } else {
                                Ok(addresses)
                            }
                        });
                // Capacity is restored before a caller can consume this completion.
                drop(permit);
                let _ = sender.send(outcome);
                wake.signal();
            })
            .map_err(|_| Error::Limit)?;
        self.operations.insert(
            id,
            Pending {
                callback: Some(callback),
                receiver,
                deadline,
                outcome: None,
            },
        );
        Ok(id)
    }
    #[allow(dead_code)] // Private cancellation; no guest cancellation API yet.
    pub(crate) fn cancel(&mut self, id: OperationId) -> Result<bool, Error> {
        let op = self.operations.get_mut(&id).ok_or(Error::Failed)?;
        if op.outcome.is_some() {
            return Ok(false);
        }
        op.outcome = Some(Err(Error::Cancelled));
        Ok(true)
    }
    pub(crate) fn poll(&mut self) -> Option<Value> {
        self.poll_at(Instant::now())
    }
    fn poll_at(&mut self, now: Instant) -> Option<Value> {
        for op in self.operations.values_mut() {
            if op.callback.is_none() {
                continue;
            }
            if op.outcome.is_none() {
                // Deadline wins if completion wasn't observed before it expired.
                op.outcome = if now >= op.deadline {
                    Some(Err(Error::TimedOut))
                } else {
                    match op.receiver.try_recv() {
                        Ok(result) => Some(result),
                        Err(mpsc::TryRecvError::Empty) => None,
                        Err(mpsc::TryRecvError::Disconnected) => Some(Err(Error::Failed)),
                    }
                };
            }
            if op.outcome.is_some() {
                return op.callback.take();
            }
        }
        None
    }
    pub(crate) fn take_result(&mut self, id: OperationId) -> Result<Option<Outcome>, Error> {
        let op = self.operations.get(&id).ok_or(Error::Failed)?;
        if op.outcome.is_none() || op.callback.is_some() {
            return Ok(None);
        }
        Ok(self.operations.remove(&id).unwrap().outcome)
    }
    pub(crate) fn pending(&self) -> bool {
        self.operations.values().any(|op| op.callback.is_some())
    }
    pub(crate) fn trace_roots(&self, roots: &mut Vec<usize>) {
        for op in self.operations.values() {
            if let Some(callback) = &op.callback {
                crate::gc::trace(callback, roots);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Condvar, Mutex};
    fn callback() -> Value {
        Value::Delegate(crate::Delegate {
            ty: crate::assembler::parse_type("System.Func<Void>").unwrap(),
            target: crate::assembler::parse_function_ref("Ready()").unwrap(),
            receiver: None,
        })
    }
    fn resolver(lookup: impl Fn(&str) -> Outcome + Send + Sync + 'static) -> Resolver {
        Resolver {
            lookup: Arc::new(lookup),
            permits: Box::leak(Box::new(AtomicUsize::new(0))),
            ..Resolver::with_wake(Arc::new(crate::scheduler::Wake::default()))
        }
    }
    fn complete(resolver: &mut Resolver) {
        let end = Instant::now() + Duration::from_secs(5);
        while resolver.poll().is_none() {
            assert!(Instant::now() < end, "resolver did not complete");
            resolver.wake.park(Duration::from_millis(10));
        }
    }
    #[derive(Default)]
    struct Gate(Arc<(Mutex<bool>, Condvar)>);
    impl Gate {
        fn release(&self) {
            *self.0.0.lock().unwrap() = true;
            self.0.1.notify_all();
        }
    }
    impl Drop for Gate {
        fn drop(&mut self) {
            self.release();
        }
    }
    fn blocked() -> (Resolver, Gate, mpsc::Receiver<()>) {
        let gate = Gate::default();
        let worker_gate = gate.0.clone();
        let (entered, receiver) = mpsc::channel();
        let resolver = resolver(move |_| {
            entered.send(()).unwrap();
            let guard = worker_gate.0.lock().unwrap();
            let _guard = worker_gate.1.wait_while(guard, |open| !*open).unwrap();
            Ok(vec![Ipv4Addr::LOCALHOST])
        });
        (resolver, gate, receiver)
    }
    #[test]
    fn native_bridge_validates_arguments_and_consumes_owned_address_result_once() {
        let mut resolver = resolver(|_| Ok(vec![Ipv4Addr::LOCALHOST]));
        assert!(
            resolver
                .invoke(Operation::Lookup, &[Value::Int32(1), callback()])
                .is_err()
        );
        assert_eq!(
            resolver
                .invoke(
                    Operation::Lookup,
                    &[Value::String("bad name".into()), callback()]
                )
                .unwrap(),
            Value::Erased(Box::new(Value::Byte(1)))
        );
        let Value::Erased(payload) = resolver
            .invoke(
                Operation::Lookup,
                &[Value::String("localhost".into()), callback()],
            )
            .unwrap()
        else {
            panic!()
        };
        let Value::Int64(id) = *payload else { panic!() };
        assert!(
            resolver
                .invoke(Operation::Result, &[Value::Int64(id)])
                .is_err()
        );
        complete(&mut resolver);
        assert_eq!(
            resolver
                .invoke(Operation::Result, &[Value::Int64(id)])
                .unwrap(),
            Value::Erased(Box::new(Value::Array {
                element: crate::metadata::Type::String,
                elements: vec![Value::String("127.0.0.1".into())]
            }))
        );
        assert!(
            resolver
                .invoke(Operation::Result, &[Value::Int64(id)])
                .is_err()
        );
    }

    #[test]
    fn host_lookup_numeric_and_localhost_only_no_external_dns() {
        assert_eq!(host_lookup("127.0.0.1"), Ok(vec![Ipv4Addr::LOCALHOST]));
        assert!(
            host_lookup("localhost")
                .unwrap()
                .contains(&Ipv4Addr::LOCALHOST)
        );
        assert_eq!(host_lookup("::1"), Err(Error::NoAddress));
    }
    #[test]
    fn address_selection_preserves_order_deduplicates_and_rejects_truncation() {
        use std::net::{IpAddr, Ipv6Addr, SocketAddr};
        let first = Ipv4Addr::new(127, 0, 0, 2);
        let v4 = |ip| SocketAddr::new(IpAddr::V4(ip), 0);
        let v6 = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 0);
        assert_eq!(
            ipv4_addresses([v6, v4(first), v4(first), v4(Ipv4Addr::LOCALHOST)].into_iter()),
            Ok(vec![first, Ipv4Addr::LOCALHOST])
        );
        assert_eq!(ipv4_addresses([v6].into_iter()), Err(Error::NoAddress));
        assert_eq!(
            ipv4_addresses((0..17).map(|n| v4(Ipv4Addr::new(127, 0, 0, n)))),
            Err(Error::Limit)
        );
        assert_eq!(
            ipv4_addresses(std::iter::repeat_n(v6, 257)),
            Err(Error::Limit)
        );
    }

    #[test]
    fn validation_precedes_host_work_and_results_are_consumed_once() {
        let owner = std::thread::current().id();
        let mut resolver = resolver(move |_| {
            assert_ne!(std::thread::current().id(), owner);
            Ok(vec![Ipv4Addr::LOCALHOST])
        });
        for name in [
            "",
            "name:80",
            "two names",
            "a\0b",
            "é.example",
            &"a".repeat(254),
        ] {
            assert_eq!(
                resolver.submit(name, callback(), Duration::from_secs(5)),
                Err(Error::InvalidName)
            );
        }
        assert_eq!(
            resolver.submit("localhost", Value::Int32(0), Duration::from_secs(5)),
            Err(Error::InvalidCallback)
        );
        assert_eq!(resolver.permits.load(Ordering::Acquire), 0);
        let id = resolver
            .submit("localhost", callback(), Duration::from_secs(5))
            .unwrap();
        assert_eq!(resolver.take_result(id), Ok(None));
        complete(&mut resolver);
        assert!(!resolver.pending());
        assert_eq!(resolver.cancel(id), Ok(false));
        assert_eq!(
            resolver.take_result(id),
            Ok(Some(Ok(vec![Ipv4Addr::LOCALHOST])))
        );
        assert_eq!(resolver.take_result(id), Err(Error::Failed));
        assert!(resolver.poll().is_none());
    }
    #[test]
    fn cancel_and_drop_do_not_free_running_host_capacity_or_deliver_late_results() {
        let (mut resolver, gate, entered) = blocked();
        let counter = resolver.permits;
        for _ in 0..4 {
            let id = resolver
                .submit("localhost", callback(), Duration::from_secs(5))
                .unwrap();
            entered.recv_timeout(Duration::from_secs(5)).unwrap();
            assert!(resolver.cancel(id).unwrap());
            complete(&mut resolver);
            assert_eq!(resolver.take_result(id), Ok(Some(Err(Error::Cancelled))));
        }
        assert_eq!(
            resolver.submit("localhost", callback(), Duration::from_secs(5)),
            Err(Error::Limit)
        );
        assert_eq!(counter.load(Ordering::Acquire), 4);
        let start = Instant::now();
        drop(resolver); // Must not join uninterruptible host resolution.
        assert!(start.elapsed() < Duration::from_secs(1));
        assert_eq!(counter.load(Ordering::Acquire), 4);
        let mut replacement = Resolver::with_wake(Arc::new(crate::scheduler::Wake::default()));
        replacement.permits = counter;
        assert_eq!(
            replacement.submit("localhost", callback(), Duration::from_secs(5)),
            Err(Error::Limit)
        );
        gate.release();
        let end = Instant::now() + Duration::from_secs(5);
        while counter.load(Ordering::Acquire) != 0 {
            assert!(Instant::now() < end);
            std::thread::yield_now();
        }
    }
    #[test]
    fn deadline_wins_and_pending_work_does_not_block_poll() {
        let (mut resolver, gate, entered) = blocked();
        let id = resolver
            .submit("localhost", callback(), Duration::ZERO)
            .unwrap();
        entered.recv_timeout(Duration::from_secs(5)).unwrap();
        complete(&mut resolver);
        assert_eq!(resolver.take_result(id), Ok(Some(Err(Error::TimedOut))));
        gate.release();
        assert!(resolver.poll().is_none());
        let (mut resolver, _gate, entered) = blocked();
        resolver
            .submit("localhost", callback(), Duration::from_secs(5))
            .unwrap();
        entered.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(resolver.pending());
        assert!(resolver.poll().is_none());
    }
    #[test]
    fn shared_deadline_clamps_lookup_and_rejects_expired_work_before_host_admission() {
        let (mut resolver, gate, entered) = blocked();
        let expired = Instant::now();
        assert_eq!(
            resolver.submit_until(
                "localhost",
                callback(),
                Duration::from_secs(5),
                Some(expired)
            ),
            Err(Error::TimedOut)
        );
        assert!(resolver.operations.is_empty());
        assert_eq!(resolver.permits.load(Ordering::Acquire), 0);
        assert!(
            entered.try_recv().is_err(),
            "expired lookup must not start host work"
        );

        let end = Instant::now() + Duration::from_secs(1);
        let id = resolver
            .submit_until("localhost", callback(), Duration::from_secs(5), Some(end))
            .unwrap();
        entered.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(resolver.operations[&id].deadline, end);
        assert!(resolver.poll_at(end - Duration::from_nanos(1)).is_none());
        assert!(resolver.poll_at(end).is_some());
        assert_eq!(resolver.take_result(id), Ok(Some(Err(Error::TimedOut))));
        assert_eq!(
            resolver.permits.load(Ordering::Acquire),
            1,
            "timeout cannot stop libc or release its worker permit"
        );
        gate.release();
        let watchdog = Instant::now() + Duration::from_secs(5);
        while resolver.permits.load(Ordering::Acquire) != 0 {
            assert!(Instant::now() < watchdog);
            std::thread::yield_now();
        }
        assert!(
            resolver.poll_at(end).is_none(),
            "late native result is detached"
        );
    }
    #[test]
    fn errors_panics_and_oversized_results_settle_without_losing_capacity() {
        for (outcome, expected) in [
            (Err(Error::Failed), Error::Failed),
            (Ok(vec![]), Error::NoAddress),
            (Ok(vec![Ipv4Addr::LOCALHOST; 17]), Error::Limit),
        ] {
            let mut resolver = resolver(move |_| outcome.clone());
            let id = resolver
                .submit("localhost", callback(), Duration::from_secs(5))
                .unwrap();
            complete(&mut resolver);
            assert_eq!(resolver.take_result(id), Ok(Some(Err(expected))));
            assert_eq!(resolver.permits.load(Ordering::Acquire), 0);
        }
        let mut resolver = resolver(|_| panic!("injected resolver failure"));
        let id = resolver
            .submit("localhost", callback(), Duration::from_secs(5))
            .unwrap();
        complete(&mut resolver);
        assert_eq!(resolver.take_result(id), Ok(Some(Err(Error::Failed))));
        assert_eq!(resolver.permits.load(Ordering::Acquire), 0);
    }
    #[test]
    fn unconsumed_outcomes_keep_invocation_slots_and_foreign_ids_are_rejected() {
        let mut resolver = resolver(|_| Ok(vec![Ipv4Addr::LOCALHOST]));
        let mut ids = vec![];
        for _ in 0..MAX_OPERATIONS {
            ids.push(
                resolver
                    .submit("localhost", callback(), Duration::from_secs(5))
                    .unwrap(),
            );
            complete(&mut resolver);
        }
        assert_eq!(
            resolver.submit("localhost", callback(), Duration::from_secs(5)),
            Err(Error::Limit)
        );
        let mut other = Resolver::with_wake(Arc::new(crate::scheduler::Wake::default()));
        assert_eq!(other.take_result(ids[0]), Err(Error::Failed));
        resolver.take_result(ids[0]).unwrap();
        assert!(
            resolver
                .submit("localhost", callback(), Duration::from_secs(5))
                .is_ok()
        );
    }
}
