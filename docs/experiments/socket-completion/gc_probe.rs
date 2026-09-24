//! Real TCP plus actual managed roots; still a test-only invocation adapter.
use super::{array, bytes, callback, identity};
use crate::{CollectionReason, ManagedHeap, Value, metadata::Type};
use std::{
    collections::{BTreeMap, VecDeque},
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    time::{Duration, Instant},
};

#[derive(Debug, PartialEq, Eq)]
enum Error {
    InvalidBuffer,
    InvalidCallback,
    InvalidRange,
    Closed,
    Busy,
    Limit,
    Cancelled,
    Io,
}
struct Pending {
    socket: u64,
    destination: Value,
    callback: Value,
    offset: usize,
    bytes: Vec<u8>,
}
struct Ready {
    callback: Value,
    outcome: Result<usize, Error>,
}
struct Invocation {
    heap: ManagedHeap,
    sockets: BTreeMap<u64, TcpStream>,
    pending: BTreeMap<u64, Pending>,
    ready: VecDeque<Ready>,
    next_socket: u64,
    next_operation: u64,
    reserved: usize,
}
impl Invocation {
    fn new() -> Self {
        Self {
            heap: ManagedHeap::default(),
            sockets: BTreeMap::new(),
            pending: BTreeMap::new(),
            ready: VecDeque::new(),
            next_socket: 0,
            next_operation: 0,
            reserved: 0,
        }
    }
    fn attach(&mut self, socket: TcpStream) -> Result<u64, Error> {
        if self.sockets.len() >= 2 {
            return Err(Error::Limit);
        }
        let next = self.next_socket.checked_add(1).ok_or(Error::Limit)?;
        socket.set_nonblocking(true).map_err(|_| Error::Io)?;
        let id = self.next_socket;
        self.next_socket = next;
        self.sockets.insert(id, socket);
        Ok(id)
    }
    fn begin(
        &mut self,
        socket: u64,
        destination: Value,
        offset: i32,
        count: i32,
        callback: Value,
    ) -> Result<u64, Error> {
        if !self.sockets.contains_key(&socket) {
            return Err(Error::Closed);
        }
        if self.pending.values().any(|p| p.socket == socket) {
            return Err(Error::Busy);
        }
        let Value::ObjectReference(object) = &destination else {
            return Err(Error::InvalidBuffer);
        };
        let Value::Array {
            element: Type::Byte,
            elements,
        } = self
            .heap
            .read_reference(&object.reference)
            .map_err(|_| Error::InvalidBuffer)?
        else {
            return Err(Error::InvalidBuffer);
        };
        let offset = usize::try_from(offset).map_err(|_| Error::InvalidRange)?;
        let count = usize::try_from(count).map_err(|_| Error::InvalidRange)?;
        if offset > elements.len() || count > elements.len() - offset {
            return Err(Error::InvalidRange);
        }
        // Harness-created delegate only. This is not production signature binding.
        let Value::Delegate(delegate) = &callback else {
            return Err(Error::InvalidCallback);
        };
        let Some(Value::ObjectReference(receiver)) = delegate.receiver.as_deref() else {
            return Err(Error::InvalidCallback);
        };
        self.heap
            .read_reference(&receiver.reference)
            .map_err(|_| Error::InvalidCallback)?;
        // Ready callbacks consume admission slots too, until dispatched/released.
        if self.pending.len() + self.ready.len() >= 2 {
            return Err(Error::Limit);
        }
        let reserved = self
            .reserved
            .checked_add(count)
            .filter(|n| *n <= 8)
            .ok_or(Error::Limit)?;
        let next = self.next_operation.checked_add(1).ok_or(Error::Limit)?;
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(count).map_err(|_| Error::Limit)?;
        bytes.resize(count, 0);
        let id = self.next_operation;
        self.next_operation = next;
        self.reserved = reserved;
        self.pending.insert(
            id,
            Pending {
                socket,
                destination,
                callback,
                offset,
                bytes,
            },
        );
        Ok(id)
    }
    fn settle(&mut self, id: u64, outcome: Result<usize, Error>) {
        let Some(pending) = self.pending.remove(&id) else {
            return;
        };
        self.reserved -= pending.bytes.len();
        // No GC safepoint between removing pending roots and publishing ready roots.
        self.ready.push_back(Ready {
            callback: pending.callback,
            outcome,
        });
    }
    fn poll(&mut self, id: u64) {
        let Some(pending) = self.pending.get_mut(&id) else {
            return;
        };
        let socket = self
            .sockets
            .get_mut(&pending.socket)
            .expect("close settles pending reads");
        let result = if pending.bytes.is_empty() {
            Ok(0)
        } else {
            socket.read(&mut pending.bytes)
        };
        let outcome = match result {
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                ) =>
            {
                return;
            }
            Err(_) => Err(Error::Io),
            Ok(count) => {
                let Value::ObjectReference(object) = &pending.destination else {
                    unreachable!()
                };
                let mut replacement = self
                    .heap
                    .read_reference(&object.reference)
                    .expect("pending destination rooted");
                let Value::Array { elements, .. } = &mut replacement else {
                    unreachable!()
                };
                for (slot, byte) in elements[pending.offset..pending.offset + count]
                    .iter_mut()
                    .zip(&pending.bytes)
                {
                    *slot = Value::Byte(*byte);
                }
                object.reference.write(replacement).unwrap();
                Ok(count)
            }
        };
        self.settle(id, outcome);
    }
    fn cancel(&mut self, id: u64) {
        self.settle(id, Err(Error::Cancelled));
    }
    fn close(&mut self, socket: u64) {
        self.sockets.remove(&socket);
        let ids: Vec<_> = self
            .pending
            .iter()
            .filter(|(_, p)| p.socket == socket)
            .map(|(id, _)| *id)
            .collect();
        for id in ids {
            self.settle(id, Err(Error::Closed));
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
    fn shutdown(&mut self) {
        self.sockets.clear();
        self.pending.clear();
        self.ready.clear();
        self.reserved = 0;
    }
}
fn pair() -> (TcpStream, TcpStream) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let peer = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    peer.set_write_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    (peer, listener.accept().unwrap().0)
}
fn finish(invocation: &mut Invocation, id: u64) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while invocation.pending.contains_key(&id) {
        invocation.poll(id);
        assert!(Instant::now() < deadline, "socket completion watchdog");
        if invocation.pending.contains_key(&id) {
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}

#[test]
fn real_receive_transfers_gc_roots_and_exact_range_to_ready_callback() {
    let (mut peer, stream) = pair();
    let mut invocation = Invocation::new();
    let socket = invocation.attach(stream).unwrap();
    let destination = array(&mut invocation.heap, &[9; 4]);
    let destination_id = identity(&destination);
    let done = callback(&mut invocation.heap, destination.clone());
    let id = invocation.begin(socket, destination, 1, 2, done).unwrap();
    invocation.poll(id);
    assert_eq!(invocation.pending.len(), 1);
    for _ in 0..100 {
        array(&mut invocation.heap, &[0]);
    }
    invocation.collect();
    assert_eq!(invocation.heap.len(), 2);
    assert_eq!(invocation.heap.statistics().reclaimed_objects, 100);
    peer.write_all(&[4]).unwrap();
    finish(&mut invocation, id);
    invocation.collect();
    assert_eq!(bytes(&invocation.heap, destination_id), [9, 4, 9, 9]);
    assert_eq!(invocation.ready.front().unwrap().outcome, Ok(1));
    invocation.cancel(id);
    invocation.poll(id);
    assert_eq!(invocation.ready.len(), 1);
    invocation.ready.clear();
    invocation.collect();
    assert!(invocation.heap.is_empty());
    assert_eq!(invocation.reserved, 0);
}

#[test]
fn cancellation_preserves_socket_and_unconsumed_bytes_for_the_next_read() {
    let (mut peer, stream) = pair();
    let mut invocation = Invocation::new();
    let socket = invocation.attach(stream).unwrap();
    let destination = array(&mut invocation.heap, &[9; 4]);
    let destination_id = identity(&destination);
    let done = callback(&mut invocation.heap, destination.clone());
    let first = invocation
        .begin(socket, destination.clone(), 0, 4, done.clone())
        .unwrap();
    peer.write_all(&[7]).unwrap();
    invocation.cancel(first);
    invocation.collect();
    assert_eq!(bytes(&invocation.heap, destination_id), [9; 4]);
    assert_eq!(
        invocation.ready.pop_front().unwrap().outcome,
        Err(Error::Cancelled)
    );
    let second = invocation.begin(socket, destination, 2, 2, done).unwrap();
    assert_ne!(first, second);
    invocation.cancel(first);
    finish(&mut invocation, second);
    invocation.collect();
    assert_eq!(bytes(&invocation.heap, destination_id), [9, 9, 7, 9]);
    assert_eq!(invocation.ready.front().unwrap().outcome, Ok(1));
}

#[test]
fn destination_and_callback_are_independent_roots_and_close_publishes_once() {
    let (_peer, stream) = pair();
    let mut invocation = Invocation::new();
    let socket = invocation.attach(stream).unwrap();
    let destination = array(&mut invocation.heap, &[9]);
    let destination_id = identity(&destination);
    let marker = array(&mut invocation.heap, &[42]);
    let marker_id = identity(&marker);
    let done = callback(&mut invocation.heap, marker);
    let id = invocation.begin(socket, destination, 0, 1, done).unwrap();
    invocation.collect();
    assert_eq!(invocation.heap.len(), 3);
    invocation.close(socket);
    invocation.close(socket);
    invocation.cancel(id);
    invocation.poll(id);
    invocation.collect();
    assert!(invocation.heap.get(destination_id).is_none());
    assert_eq!(bytes(&invocation.heap, marker_id), [42]);
    assert_eq!(invocation.ready.len(), 1);
    assert_eq!(
        invocation.ready.front().unwrap().outcome,
        Err(Error::Closed)
    );
    invocation.ready.clear();
    invocation.collect();
    assert!(invocation.heap.is_empty());
}

#[test]
fn admission_checks_real_heap_membership_ranges_and_socket_aliases() {
    let (_peer, stream) = pair();
    let mut invocation = Invocation::new();
    let socket = invocation.attach(stream).unwrap();
    let buffer = array(&mut invocation.heap, &[9; 4]);
    let done = callback(&mut invocation.heap, buffer.clone());
    let large = array(&mut invocation.heap, &[0; 9]);
    assert_eq!(
        invocation.begin(socket, large, 0, 9, done.clone()),
        Err(Error::Limit)
    );
    assert_eq!(
        invocation.begin(socket, Value::Int32(0), 0, 0, done.clone()),
        Err(Error::InvalidBuffer)
    );
    assert_eq!((invocation.reserved, invocation.next_operation), (0, 0));
    let mut foreign = ManagedHeap::default();
    let foreign_buffer = array(&mut foreign, &[9; 4]);
    assert_eq!(
        invocation.begin(socket, foreign_buffer, 0, 1, done.clone()),
        Err(Error::InvalidBuffer)
    );
    assert_eq!(
        invocation.begin(socket, buffer.clone(), -1, 1, done.clone()),
        Err(Error::InvalidRange)
    );
    assert_eq!(
        invocation.begin(socket, buffer.clone(), 3, 2, done.clone()),
        Err(Error::InvalidRange)
    );
    assert_eq!(
        invocation.begin(socket, buffer.clone(), 0, 1, Value::Int32(1)),
        Err(Error::InvalidCallback)
    );
    let first = invocation
        .begin(socket, buffer.clone(), 0, 1, done.clone())
        .unwrap();
    assert_eq!(
        invocation.begin(socket, buffer.clone(), 0, 1, done.clone()),
        Err(Error::Busy)
    );
    invocation.cancel(first);
    let second = invocation
        .begin(socket, buffer.clone(), 0, 1, done.clone())
        .unwrap();
    invocation.cancel(second);
    assert_eq!(
        invocation.begin(socket, buffer, 0, 1, done),
        Err(Error::Limit)
    );
    assert_eq!(invocation.reserved, 0);
    assert!(invocation.sockets.contains_key(&socket));
}

#[test]
fn later_socket_completes_and_shutdown_releases_all_roots_and_connections() {
    let (mut idle_peer, idle) = pair();
    let (mut peer, stream) = pair();
    let mut invocation = Invocation::new();
    let idle_socket = invocation.attach(idle).unwrap();
    let ready_socket = invocation.attach(stream).unwrap();
    let mut ids = vec![];
    for socket in [idle_socket, ready_socket] {
        let buffer = array(&mut invocation.heap, &[0; 4]);
        let done = callback(&mut invocation.heap, buffer.clone());
        ids.push(invocation.begin(socket, buffer, 0, 4, done).unwrap());
    }
    invocation.poll(ids[0]);
    peer.write_all(&[3]).unwrap();
    finish(&mut invocation, ids[1]);
    invocation.collect();
    assert_eq!(invocation.heap.len(), 4);
    assert!(invocation.pending.contains_key(&ids[0]));
    invocation.shutdown();
    invocation.collect();
    assert!(invocation.heap.is_empty());
    assert_eq!(invocation.reserved, 0);
    assert_eq!(idle_peer.read(&mut [0]).unwrap(), 0);
    assert_eq!(peer.read(&mut [0]).unwrap(), 0);
}
