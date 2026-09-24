//! S0 research harness using the real managed heap, not the interpreter event loop.
//! Compiled only by `cargo test --lib external_io_gc_probe`.
use crate::{CollectionReason, Fault, ManagedHeap, Value, metadata::Type};
use std::collections::{BTreeMap, VecDeque};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
    mpsc::{self, Receiver, SyncSender},
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[derive(Debug, PartialEq)]
enum Outcome {
    Copied(usize),
    Cancelled,
    Failed,
}

// Only owned host data crosses the thread boundary. No Value, heap handle or
// guest callback is Send; Rust checks the producer capture list accordingly.
enum Command {
    Bytes(Vec<u8>),
    Cancel,
    Fail,
}
struct Event {
    id: u64,
    command: Command,
}
struct Pending {
    destination: Value,
    callback: Value,
    offset: usize,
    count: usize,
    control: SyncSender<Command>,
    worker: JoinHandle<()>,
}
struct Ready {
    callback: Value,
    outcome: Outcome,
}
struct Resource(Arc<AtomicUsize>);
impl Drop for Resource {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

struct Invocation {
    heap: ManagedHeap,
    pending: BTreeMap<u64, Pending>,
    ready: VecDeque<Ready>,
    send: SyncSender<Event>,
    receive: Receiver<Event>,
    next_id: u64,
    resources: Arc<AtomicUsize>,
}
impl Invocation {
    fn new() -> Self {
        // At most eight producers; each sends exactly one terminal event.
        let (send, receive) = mpsc::sync_channel(8);
        Self {
            heap: ManagedHeap::default(),
            pending: BTreeMap::new(),
            ready: VecDeque::new(),
            send,
            receive,
            next_id: 0,
            resources: Arc::default(),
        }
    }

    fn begin(
        &mut self,
        destination: Value,
        offset: i32,
        count: i32,
        callback: Value,
    ) -> Result<(u64, SyncSender<Command>), Fault> {
        if self.pending.len() >= 8 {
            return Err(Fault::new("pending operation limit"));
        }
        let Value::ObjectReference(object) = &destination else {
            return Err(Fault::new("expected managed array object"));
        };
        // Check actual heap membership, not just an allocation number or view type.
        let Value::Array {
            element: Type::Byte,
            elements,
        } = self.heap.read_reference(&object.reference)?
        else {
            return Err(Fault::new("expected byte array"));
        };
        let offset = usize::try_from(offset).map_err(|_| Fault::new("invalid range"))?;
        let count = usize::try_from(count).map_err(|_| Fault::new("invalid range"))?;
        if offset > elements.len() || count > elements.len() - offset || count > 16 {
            return Err(Fault::new("invalid range or payload limit"));
        }
        // A real bridge must also bind/validate the callback signature. This
        // harness constructs delegates internally and checks their receiver root.
        let Value::Delegate(delegate) = &callback else {
            return Err(Fault::new("expected callback"));
        };
        let Some(Value::ObjectReference(receiver)) = delegate.receiver.as_deref() else {
            return Err(Fault::new("expected heap receiver"));
        };
        self.heap.read_reference(&receiver.reference)?;
        let id = self.next_id;
        self.next_id = id
            .checked_add(1)
            .ok_or_else(|| Fault::new("operation ID exhausted"))?;
        let (control, commands) = mpsc::sync_channel(1);
        let reply = self.send.clone();
        self.resources.fetch_add(1, Ordering::SeqCst);
        let resource = Resource(self.resources.clone());
        let worker = thread::spawn(move || {
            let command = commands.recv().unwrap_or(Command::Cancel);
            drop(resource); // Quiescent before terminal acknowledgement.
            let _ = reply.send(Event { id, command });
        });
        self.pending.insert(
            id,
            Pending {
                destination,
                callback,
                offset,
                count,
                control: control.clone(),
                worker,
            },
        );
        Ok((id, control))
    }

    fn cancel(&self, id: u64) {
        if let Some(pending) = self.pending.get(&id) {
            // A queued completion wins over a later cancellation request.
            let _ = pending.control.try_send(Command::Cancel);
        }
    }

    fn collect(&mut self) {
        let mut roots = vec![];
        for pending in self.pending.values() {
            crate::gc::trace(&pending.destination, &mut roots);
            crate::gc::trace(&pending.callback, &mut roots);
        }
        for ready in &self.ready {
            crate::gc::trace(&ready.callback, &mut roots);
        }
        self.heap
            .collect(roots, CollectionReason::AllocationPressure)
            .unwrap();
    }

    fn apply(&mut self, event: Event) {
        let Some(pending) = self.pending.remove(&event.id) else {
            return; // A terminal operation cannot write or publish twice.
        };
        pending.worker.join().unwrap();
        let outcome = match event.command {
            Command::Bytes(bytes) if bytes.len() <= pending.count => {
                let Value::ObjectReference(object) = &pending.destination else {
                    unreachable!()
                };
                // Validate the complete payload before mutation. Replacement keeps
                // array shape and guest alias identity, using the real slot checks.
                let mut replacement = self.heap.read_reference(&object.reference).unwrap();
                let Value::Array { elements, .. } = &mut replacement else {
                    unreachable!()
                };
                for (index, byte) in bytes.iter().enumerate() {
                    elements[pending.offset + index] = Value::Byte(*byte);
                }
                object.reference.write(replacement).unwrap();
                Outcome::Copied(bytes.len())
            }
            Command::Cancel => Outcome::Cancelled,
            Command::Bytes(_) | Command::Fail => Outcome::Failed,
        };
        // Publish the receiver into another traced owner before dropping it.
        self.ready.push_back(Ready {
            callback: pending.callback,
            outcome,
        });
    }

    fn wait(&mut self) {
        let event = self
            .receive
            .recv_timeout(Duration::from_secs(5))
            .expect("producer watchdog");
        self.apply(event);
    }

    fn shutdown(&mut self) {
        for pending in self.pending.values() {
            let _ = pending.control.try_send(Command::Cancel);
        }
        for (_, pending) in std::mem::take(&mut self.pending) {
            // This is safe only because the controlled producer cooperates and
            // the eight-event channel has room for every outstanding producer.
            pending.worker.join().unwrap();
        }
        self.ready.clear();
        while self.receive.try_recv().is_ok() {}
    }
}
impl Drop for Invocation {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn object(heap: &mut ManagedHeap, value: Value) -> Value {
    let id = heap.allocate(value).unwrap();
    Value::ObjectReference(crate::value::ObjectReference {
        reference: heap.address(id).unwrap(),
        view: None,
    })
}
fn array(heap: &mut ManagedHeap, bytes: &[u8]) -> Value {
    object(
        heap,
        Value::Array {
            element: Type::Byte,
            elements: bytes.iter().copied().map(Value::Byte).collect(),
        },
    )
}
fn identity(value: &Value) -> usize {
    let Value::ObjectReference(reference) = value else {
        panic!("expected object")
    };
    reference.allocation_id()
}
fn callback(heap: &mut ManagedHeap, captured: Value) -> Value {
    let receiver = object(
        heap,
        Value::Object {
            ty: Type::from_name("CopyConsumer"),
            fields: vec![captured],
        },
    );
    Value::Delegate(crate::Delegate {
        ty: Type::from_name("Completion"),
        target: crate::assembler::parse_function_ref("instance CopyConsumer::Complete(Int32)")
            .unwrap(),
        receiver: Some(Box::new(receiver)),
    })
}
fn bytes(heap: &ManagedHeap, id: usize) -> Vec<u8> {
    let Value::Array { elements, .. } = heap.get(id).unwrap() else {
        panic!("expected array")
    };
    elements
        .iter()
        .map(|value| match value {
            Value::Byte(byte) => *byte,
            _ => panic!("expected byte"),
        })
        .collect()
}

#[test]
fn pending_and_ready_roots_keep_real_heap_graph_alive_then_release_it() {
    let mut invocation = Invocation::new();
    let destination = array(&mut invocation.heap, &[9, 9, 9, 9]);
    let id = identity(&destination);
    let done = callback(&mut invocation.heap, destination.clone());
    let (_, command) = invocation.begin(destination, 1, 2, done).unwrap();
    // No caller Value remains. Unrelated allocation pressure is actually swept.
    for _ in 0..100 {
        array(&mut invocation.heap, &[0]);
    }
    invocation.collect();
    assert_eq!(invocation.heap.len(), 2);
    assert_eq!(invocation.heap.statistics().reclaimed_objects, 100);
    command.send(Command::Bytes(vec![4, 5])).unwrap();
    invocation.wait();
    assert!(invocation.pending.is_empty());
    invocation.collect();
    assert_eq!(bytes(&invocation.heap, id), [9, 4, 5, 9]);
    assert_eq!(
        invocation.ready.front().unwrap().outcome,
        Outcome::Copied(2)
    );
    assert_eq!(invocation.resources.load(Ordering::SeqCst), 0);
    invocation.ready.clear();
    invocation.collect();
    assert!(invocation.heap.is_empty());
}

#[test]
fn missing_root_is_detected_by_the_real_heap_not_hidden_by_rust_handles() {
    let mut heap = ManagedHeap::default();
    let destination = array(&mut heap, &[7]);
    let Value::ObjectReference(reference) = &destination else {
        unreachable!()
    };
    heap.collect(vec![], CollectionReason::AllocationPressure)
        .unwrap();
    assert!(heap.read_reference(&reference.reference).is_err());
    assert!(reference.reference.read().is_err());
}

#[test]
fn destination_and_receiver_are_independent_pending_roots() {
    let mut invocation = Invocation::new();
    let destination = array(&mut invocation.heap, &[9]);
    let destination_id = identity(&destination);
    let marker = array(&mut invocation.heap, &[42]);
    let marker_id = identity(&marker);
    let done = callback(&mut invocation.heap, marker);
    let (_, command) = invocation.begin(destination, 0, 1, done).unwrap();
    invocation.collect();
    assert_eq!(invocation.heap.len(), 3);
    command.send(Command::Bytes(vec![1])).unwrap();
    invocation.wait();
    assert_eq!(bytes(&invocation.heap, destination_id), [1]);
    invocation.collect();
    assert!(invocation.heap.get(destination_id).is_none());
    assert_eq!(bytes(&invocation.heap, marker_id), [42]);
    invocation.ready.clear();
    invocation.collect();
    assert!(invocation.heap.is_empty());
}

#[test]
fn cancellation_waits_for_ack_and_duplicate_completion_cannot_overwrite() {
    for completion_wins in [false, true] {
        let mut invocation = Invocation::new();
        let destination = array(&mut invocation.heap, &[9]);
        let destination_id = identity(&destination);
        let done = callback(&mut invocation.heap, destination.clone());
        let (id, command) = invocation.begin(destination, 0, 1, done).unwrap();
        if completion_wins {
            command.send(Command::Bytes(vec![4])).unwrap();
        }
        invocation.cancel(id);
        assert!(invocation.pending.contains_key(&id));
        invocation.collect();
        invocation.wait();
        invocation.apply(Event {
            id,
            command: Command::Bytes(vec![7]),
        });
        assert_eq!(invocation.ready.len(), 1);
        assert_eq!(
            bytes(&invocation.heap, destination_id),
            if completion_wins { vec![4] } else { vec![9] }
        );
        assert_eq!(
            invocation.ready.front().unwrap().outcome,
            if completion_wins {
                Outcome::Copied(1)
            } else {
                Outcome::Cancelled
            }
        );
    }
}

#[test]
fn oversize_payload_and_failure_leave_destination_unchanged() {
    for command in [Command::Bytes(vec![1, 2, 3]), Command::Fail] {
        let mut invocation = Invocation::new();
        let destination = array(&mut invocation.heap, &[9, 9]);
        let id = identity(&destination);
        let done = callback(&mut invocation.heap, destination.clone());
        let (_, send) = invocation.begin(destination, 0, 2, done).unwrap();
        send.send(command).unwrap();
        invocation.wait();
        invocation.collect();
        assert_eq!(bytes(&invocation.heap, id), [9, 9]);
        assert_eq!(invocation.ready.front().unwrap().outcome, Outcome::Failed);
    }
}

#[test]
fn ranges_types_and_cross_heap_handles_are_rejected_before_registration() {
    let mut invocation = Invocation::new();
    let destination = array(&mut invocation.heap, &[9, 9]);
    let done = callback(&mut invocation.heap, destination.clone());
    for (offset, count) in [(-1, 1), (0, -1), (2, 1), (i32::MAX, 1), (1, i32::MAX)] {
        assert!(
            invocation
                .begin(destination.clone(), offset, count, done.clone())
                .is_err()
        );
    }
    let mut other = ManagedHeap::default();
    let foreign = array(&mut other, &[1, 2]);
    // Equal numeric allocation IDs do not establish ownership.
    assert_eq!(identity(&foreign), identity(&destination));
    assert!(invocation.begin(foreign, 0, 1, done.clone()).is_err());
    let wrong = object(
        &mut invocation.heap,
        Value::Array {
            element: Type::Int32,
            elements: vec![Value::Int32(0)],
        },
    );
    assert!(invocation.begin(wrong, 0, 1, done.clone()).is_err());
    assert!(
        invocation
            .begin(
                Value::NullObjectReference(Type::ArrayRef(Box::new(Type::Byte))),
                0,
                0,
                done.clone()
            )
            .is_err()
    );
    assert!(
        invocation
            .begin(destination.clone(), 0, 1, Value::Void)
            .is_err()
    );
    assert!(invocation.pending.is_empty());
    assert_eq!(invocation.next_id, 0);
    assert_eq!(invocation.resources.load(Ordering::SeqCst), 0);
    let (_, send) = invocation.begin(destination, 2, 0, done).unwrap();
    send.send(Command::Bytes(vec![])).unwrap();
    invocation.wait();
    assert_eq!(
        invocation.ready.front().unwrap().outcome,
        Outcome::Copied(0)
    );
}

#[test]
fn later_completion_does_not_release_an_earlier_operations_roots() {
    let mut invocation = Invocation::new();
    let first = array(&mut invocation.heap, &[1]);
    let first_id = identity(&first);
    let first_done = callback(&mut invocation.heap, first.clone());
    let (first_op, _) = invocation.begin(first, 0, 1, first_done).unwrap();
    let second = array(&mut invocation.heap, &[2]);
    let second_id = identity(&second);
    let second_done = callback(&mut invocation.heap, second.clone());
    let (_, send) = invocation.begin(second, 0, 1, second_done).unwrap();
    send.send(Command::Bytes(vec![3])).unwrap();
    invocation.wait();
    invocation.ready.clear();
    invocation.collect();
    assert_eq!(bytes(&invocation.heap, first_id), [1]);
    assert!(invocation.heap.get(second_id).is_none());
    invocation.cancel(first_op);
    invocation.wait();
}

#[test]
fn teardown_quiesces_all_producers_before_releasing_real_roots() {
    let mut invocation = Invocation::new();
    for _ in 0..8 {
        let destination = array(&mut invocation.heap, &[9]);
        let done = callback(&mut invocation.heap, destination.clone());
        invocation.begin(destination, 0, 1, done).unwrap();
    }
    let extra = array(&mut invocation.heap, &[0]);
    let done = callback(&mut invocation.heap, extra.clone());
    assert!(invocation.begin(extra, 0, 1, done).is_err());
    invocation.collect();
    assert_eq!(invocation.heap.len(), 16);
    invocation.shutdown();
    assert_eq!(invocation.resources.load(Ordering::SeqCst), 0);
    invocation.collect();
    assert!(invocation.heap.is_empty());
}

#[path = "../socket-completion/gc_probe.rs"]
mod socket_completion;
