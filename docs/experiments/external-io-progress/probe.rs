//! Host-side S0 experiment, not a neoCLR runtime or public I/O API.
//! Producers own bytes; invocation-local destinations never cross threads.
use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

type Destination = Rc<RefCell<Vec<u8>>>;

#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    Completed(usize),
    Cancelled,
}

#[derive(Debug)]
struct Event {
    id: u64,
    bytes: Option<Vec<u8>>,
}

enum Request {
    Complete(Vec<u8>),
    Cancel,
}

struct Operation {
    destination: Destination,
    request: Sender<Request>,
    worker: JoinHandle<()>,
}

#[derive(Default)]
struct Counters {
    handles: std::sync::atomic::AtomicUsize,
}

struct FakeHandle(std::sync::Arc<Counters>);
impl Drop for FakeHandle {
    fn drop(&mut self) {
        self.0
            .handles
            .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }
}

struct HostLoop {
    send: Sender<Event>,
    receive: Receiver<Event>,
    operations: BTreeMap<u64, Operation>,
    next_id: u64,
    ready: VecDeque<&'static str>,
    trace: Vec<&'static str>,
    outcomes: BTreeMap<u64, Outcome>,
    counters: std::sync::Arc<Counters>,
}

impl HostLoop {
    fn new() -> Self {
        let (send, receive) = mpsc::channel();
        Self {
            send,
            receive,
            operations: BTreeMap::new(),
            next_id: 0,
            ready: VecDeque::new(),
            trace: Vec::new(),
            outcomes: BTreeMap::new(),
            counters: Default::default(),
        }
    }

    fn start(&mut self, destination: Destination) -> (u64, Sender<Request>) {
        assert!(
            self.operations.len() < 8,
            "experiment pending-operation cap"
        );
        let id = self.next_id;
        self.next_id += 1; // IDs are never reused within this invocation.
        let (request, control) = mpsc::channel();
        let reply = self.send.clone();
        let counters = self.counters.clone();
        counters
            .handles
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let worker = thread::spawn(move || {
            let handle = FakeHandle(counters);
            // This controlled backend waits for readiness or cancellation. It does
            // not access the destination or execute a guest callback.
            let bytes = match control.recv() {
                Ok(Request::Complete(bytes)) => Some(bytes),
                Ok(Request::Cancel) | Err(_) => None,
            };
            // Acknowledge cancellation only after this backend releases its handle.
            drop(handle);
            let _ = reply.send(Event { id, bytes });
        });
        self.operations.insert(
            id,
            Operation {
                destination,
                request: request.clone(),
                worker,
            },
        );
        (id, request)
    }

    fn cancel(&self, id: u64) {
        if let Some(operation) = self.operations.get(&id) {
            // Request, not an immediate terminal transition. Completion may win.
            let _ = operation.request.send(Request::Cancel);
        }
    }

    fn apply(&mut self, event: Event) {
        let Some(operation) = self.operations.remove(&event.id) else {
            return; // Late or duplicate event cannot mutate a completed destination.
        };
        operation
            .worker
            .join()
            .expect("controlled backend panicked");
        let outcome = match event.bytes {
            Some(bytes) => {
                let count = bytes.len();
                *operation.destination.borrow_mut() = bytes;
                Outcome::Completed(count)
            }
            None => Outcome::Cancelled,
        };
        self.outcomes.insert(event.id, outcome);
        self.trace.push("completion");
    }

    fn tick(&mut self) -> bool {
        // Poll between callbacks instead of draining an unbounded ready queue.
        let completed = match self.receive.try_recv() {
            Ok(event) => {
                self.apply(event);
                true
            }
            Err(TryRecvError::Empty) => false,
            Err(TryRecvError::Disconnected) => panic!("event channel disconnected"),
        };
        if let Some(callback) = self.ready.pop_front() {
            self.trace.push(callback);
            return true;
        }
        if completed {
            return true; // Let the caller react before waiting on another operation.
        }
        if self.operations.is_empty() {
            return false;
        }
        // Empty ready queue is not quiescence when producers remain registered.
        // Timeout is a test watchdog, not the proposed timeout or scheduler API.
        let event = self
            .receive
            .recv_timeout(Duration::from_secs(5))
            .expect("pending operation failed to acknowledge within watchdog");
        self.apply(event);
        true
    }

    fn run(&mut self) {
        while self.tick() {}
    }
}

impl Drop for HostLoop {
    fn drop(&mut self) {
        for operation in self.operations.values() {
            let _ = operation.request.send(Request::Cancel);
        }
        // Backend termination precedes release of retained destinations. This
        // cooperative fake backend does not establish bounded OS-I/O cancellation.
        for (_, operation) in std::mem::take(&mut self.operations) {
            operation
                .worker
                .join()
                .expect("controlled backend panicked");
        }
    }
}

#[test]
fn empty_ready_queue_waits_for_external_completion_and_retains_destination() {
    let mut host = HostLoop::new();
    let destination = Rc::new(RefCell::new(Vec::new()));
    let weak = Rc::downgrade(&destination);
    let (id, request) = host.start(destination.clone());
    drop(destination);
    assert!(weak.upgrade().is_some());
    assert!(matches!(host.receive.try_recv(), Err(TryRecvError::Empty)));
    assert!(host.ready.is_empty());
    let producer = thread::spawn(move || request.send(Request::Complete(vec![1, 2, 3])).unwrap());
    host.run();
    producer.join().unwrap();
    assert_eq!(host.outcomes[&id], Outcome::Completed(3));
    assert!(weak.upgrade().is_none());
    assert_eq!(host.trace, ["completion"]);
}

#[test]
fn ready_work_progresses_while_operation_is_pending() {
    let mut host = HostLoop::new();
    let destination = Rc::new(RefCell::new(vec![9]));
    let (id, request) = host.start(destination.clone());
    host.ready.push_back("unrelated callback");
    assert!(host.tick());
    assert_eq!(host.trace, ["unrelated callback"]);
    assert!(!host.outcomes.contains_key(&id));
    request.send(Request::Complete(vec![4, 5])).unwrap();
    host.run();
    assert_eq!(*destination.borrow(), [4, 5]);
    assert_eq!(host.outcomes[&id], Outcome::Completed(2));
}

#[test]
fn cancellation_requires_acknowledgement_and_never_writes_destination() {
    let mut host = HostLoop::new();
    let destination = Rc::new(RefCell::new(vec![9]));
    let (id, _) = host.start(destination.clone());
    host.cancel(id);
    assert!(!host.outcomes.contains_key(&id));
    assert!(host.operations.contains_key(&id));
    host.run();
    assert_eq!(host.outcomes[&id], Outcome::Cancelled);
    assert_eq!(*destination.borrow(), [9]);
    assert_eq!(
        host.counters
            .handles
            .load(std::sync::atomic::Ordering::SeqCst),
        0
    );
}

#[test]
fn both_race_orders_have_one_terminal_outcome_and_ignore_late_events() {
    for cancel_first in [true, false] {
        let mut host = HostLoop::new();
        let destination = Rc::new(RefCell::new(vec![9]));
        let (id, request) = host.start(destination.clone());
        if cancel_first {
            host.cancel(id);
            let _ = request.send(Request::Complete(vec![1]));
        } else {
            request.send(Request::Complete(vec![1])).unwrap();
            host.cancel(id);
        }
        host.run();
        assert_eq!(
            host.outcomes[&id],
            if cancel_first {
                Outcome::Cancelled
            } else {
                Outcome::Completed(1)
            }
        );
        host.apply(Event {
            id,
            bytes: Some(vec![99]),
        });
        assert_eq!(
            *destination.borrow(),
            if cancel_first { vec![9] } else { vec![1] }
        );
        assert_eq!(host.trace, ["completion"]);
    }
}

#[test]
fn completed_event_is_processed_before_ready_backlog() {
    let mut host = HostLoop::new();
    let (id, request) = host.start(Rc::new(RefCell::new(vec![])));
    request.send(Request::Complete(vec![1])).unwrap();
    // Establish an already-queued completion without sleeps or timing assumptions.
    let event = host.receive.recv_timeout(Duration::from_secs(5)).unwrap();
    host.send.send(event).unwrap();
    host.ready.extend(["one", "two", "three"]);
    host.tick();
    assert_eq!(host.trace, ["completion", "one"]);
    assert_eq!(host.outcomes[&id], Outcome::Completed(1));
    host.run();
}

#[test]
fn invocation_teardown_cancels_pending_work_and_releases_handles_and_buffers() {
    let counters;
    let mut weak_destinations = Vec::new();
    {
        let mut host = HostLoop::new();
        counters = host.counters.clone();
        for _ in 0..8 {
            let destination = Rc::new(RefCell::new(vec![0; 32]));
            weak_destinations.push(Rc::downgrade(&destination));
            host.start(destination);
        }
        assert!(
            weak_destinations
                .iter()
                .all(|weak| weak.upgrade().is_some())
        );
    }
    assert_eq!(
        counters.handles.load(std::sync::atomic::Ordering::SeqCst),
        0
    );
    assert!(
        weak_destinations
            .iter()
            .all(|weak| weak.upgrade().is_none())
    );
}

#[test]
fn later_completion_is_delivered_without_waiting_for_earlier_operation() {
    let mut host = HostLoop::new();
    let (slow, _) = host.start(Rc::new(RefCell::new(vec![])));
    let (fast, request) = host.start(Rc::new(RefCell::new(vec![])));
    request.send(Request::Complete(vec![7])).unwrap();
    let event = host.receive.recv_timeout(Duration::from_secs(5)).unwrap();
    host.send.send(event).unwrap();
    assert!(host.tick());
    assert_eq!(host.outcomes[&fast], Outcome::Completed(1));
    assert!(!host.outcomes.contains_key(&slow));
    host.cancel(slow);
    host.run();
    assert_eq!(host.outcomes[&slow], Outcome::Cancelled);
}
