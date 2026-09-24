//! Private owned TCP receive backend. Public addressing and Socket APIs are pending.
use crate::{Fault, ManagedHeap, Value, metadata::Type};
use std::{
    collections::BTreeMap,
    io::{self, Read},
    net::TcpStream,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
fn next_id() -> Result<u64, Error> {
    NEXT_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        .map_err(|_| Error::Limit)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SocketId(u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct OperationId(u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Error {
    Closed,
    UnknownOperation,
    Busy,
    InvalidBuffer,
    InvalidCallback,
    InvalidRange,
    Limit,
    Cancelled,
    Io,
}
#[derive(Clone, Copy)]
struct Budget {
    sockets: usize,
    operations: usize,
    bytes: usize,
}
impl Default for Budget {
    fn default() -> Self {
        Self {
            sockets: 64,
            operations: 64,
            bytes: 64 * 1024,
        }
    }
}
struct Receive {
    socket: SocketId,
    destination: Option<Value>,
    callback: Option<Value>,
    offset: usize,
    buffer: Vec<u8>,
    outcome: Option<Result<usize, Error>>,
}
pub(crate) struct Sockets {
    sockets: BTreeMap<SocketId, TcpStream>,
    operations: BTreeMap<OperationId, Receive>,
    reserved: usize,
    budget: Budget,
}
impl Default for Sockets {
    fn default() -> Self {
        Self {
            sockets: BTreeMap::new(),
            operations: BTreeMap::new(),
            reserved: 0,
            budget: Budget::default(),
        }
    }
}
// Admission/result methods are used by the test-only VM adapter until the public
// library bridge is added. Polling and root ownership already use this backend.
#[allow(dead_code)]
impl Sockets {
    pub(crate) fn adopt(&mut self, socket: TcpStream) -> Result<SocketId, Error> {
        if self.sockets.len() >= self.budget.sockets {
            return Err(Error::Limit);
        }
        socket.set_nonblocking(true).map_err(|_| Error::Io)?;
        let id = SocketId(next_id()?);
        self.sockets.insert(id, socket);
        Ok(id)
    }
    pub(crate) fn receive(
        &mut self,
        socket: SocketId,
        heap: &ManagedHeap,
        destination: Value,
        offset: i32,
        count: i32,
        callback: Value,
    ) -> Result<OperationId, Error> {
        if !self.sockets.contains_key(&socket) {
            return Err(Error::Closed);
        }
        if self
            .operations
            .values()
            .any(|op| op.socket == socket && op.outcome.is_none())
        {
            return Err(Error::Busy);
        }
        let Value::ObjectReference(object) = &destination else {
            return Err(Error::InvalidBuffer);
        };
        let Value::Array {
            element: Type::Byte,
            elements,
        } = heap
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
        if !matches!(&callback, Value::Delegate(_))
            || callback.ty() != crate::assembler::parse_type("System.Func<Void>").unwrap()
        {
            return Err(Error::InvalidCallback);
        }
        // Pending and completed-but-unconsumed operations occupy the same slots.
        if self.operations.len() >= self.budget.operations {
            return Err(Error::Limit);
        }
        let reserved = self
            .reserved
            .checked_add(count)
            .filter(|n| *n <= self.budget.bytes)
            .ok_or(Error::Limit)?;
        let mut buffer = Vec::new();
        buffer.try_reserve_exact(count).map_err(|_| Error::Limit)?;
        buffer.resize(count, 0);
        let id = OperationId(next_id()?);
        self.operations.insert(
            id,
            Receive {
                socket,
                destination: Some(destination),
                callback: Some(callback),
                offset,
                buffer,
                outcome: None,
            },
        );
        self.reserved = reserved;
        Ok(id)
    }
    fn settle(op: &mut Receive, reserved: &mut usize, outcome: Result<usize, Error>) {
        *reserved -= op.buffer.len();
        op.buffer = Vec::new();
        op.destination = None;
        op.outcome = Some(outcome);
    }
    pub(crate) fn cancel(&mut self, id: OperationId) -> Result<bool, Error> {
        let op = self
            .operations
            .get_mut(&id)
            .ok_or(Error::UnknownOperation)?;
        if op.outcome.is_some() {
            return Ok(false);
        }
        // No native read is in flight concurrently: all calls run on the owner.
        Self::settle(op, &mut self.reserved, Err(Error::Cancelled));
        Ok(true)
    }
    pub(crate) fn close(&mut self, socket: SocketId) {
        self.sockets.remove(&socket);
        for op in self
            .operations
            .values_mut()
            .filter(|op| op.socket == socket && op.outcome.is_none())
        {
            Self::settle(op, &mut self.reserved, Err(Error::Closed));
        }
    }
    pub(crate) fn take_result(
        &mut self,
        id: OperationId,
    ) -> Result<Option<Result<usize, Error>>, Error> {
        let op = self.operations.get(&id).ok_or(Error::UnknownOperation)?;
        if op.callback.is_some() || op.outcome.is_none() {
            return Ok(None);
        }
        Ok(self.operations.remove(&id).unwrap().outcome)
    }
    pub(crate) fn pending(&self) -> bool {
        self.operations.values().any(|op| op.callback.is_some())
    }
    pub(crate) fn trace_roots(&self, roots: &mut Vec<usize>) {
        for op in self.operations.values() {
            if let Some(destination) = &op.destination {
                crate::gc::trace(destination, roots);
            }
            if let Some(callback) = &op.callback {
                crate::gc::trace(callback, roots);
            }
        }
    }
    pub(crate) fn poll(&mut self, heap: &ManagedHeap) -> Result<Option<Value>, Fault> {
        for op in self.operations.values_mut() {
            if op.callback.is_none() {
                continue;
            }
            if op.outcome.is_none() {
                let socket = self
                    .sockets
                    .get_mut(&op.socket)
                    .expect("close settles pending operations");
                let read = if op.buffer.is_empty() {
                    Ok(0)
                } else {
                    socket.read(&mut op.buffer)
                };
                let outcome = match read {
                    Err(e)
                        if matches!(
                            e.kind(),
                            io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                        ) =>
                    {
                        continue;
                    }
                    Err(_) => Err(Error::Io),
                    Ok(count) => {
                        let Some(Value::ObjectReference(object)) = &op.destination else {
                            unreachable!()
                        };
                        let mut replacement = heap.read_reference(&object.reference)?;
                        let Value::Array { elements, .. } = &mut replacement else {
                            unreachable!()
                        };
                        for (slot, byte) in elements[op.offset..op.offset + count]
                            .iter_mut()
                            .zip(&op.buffer)
                        {
                            *slot = Value::Byte(*byte);
                        }
                        object.reference.write(replacement)?;
                        Ok(count)
                    }
                };
                Self::settle(op, &mut self.reserved, outcome);
            }
            return Ok(op.callback.take());
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::Write,
        net::TcpListener,
        time::{Duration, Instant},
    };
    fn pair() -> (TcpStream, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let peer = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        peer.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        peer.set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        (peer, listener.accept().unwrap().0)
    }
    fn array(heap: &mut ManagedHeap) -> Value {
        let id = heap
            .allocate(Value::Array {
                element: Type::Byte,
                elements: vec![Value::Byte(9); 4],
            })
            .unwrap();
        Value::ObjectReference(crate::value::ObjectReference {
            reference: heap.address(id).unwrap(),
            view: None,
        })
    }
    fn callback() -> Value {
        Value::Delegate(crate::Delegate {
            ty: crate::assembler::parse_type("System.Func<Void>").unwrap(),
            target: crate::assembler::parse_function_ref("Ready()").unwrap(),
            receiver: None,
        })
    }
    fn bytes(heap: &ManagedHeap, buffer: &Value) -> Vec<Value> {
        let Value::ObjectReference(object) = buffer else {
            unreachable!()
        };
        let Value::Array { elements, .. } = heap.read_reference(&object.reference).unwrap() else {
            unreachable!()
        };
        elements
    }
    fn deliver(sockets: &mut Sockets, heap: &ManagedHeap) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while sockets.poll(heap).unwrap().is_none() {
            assert!(Instant::now() < deadline, "receive watchdog");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn cancelled_read_preserves_connection_and_bytes_for_a_new_operation() {
        let (mut peer, stream) = pair();
        let mut sockets = Sockets::default();
        let socket = sockets.adopt(stream).unwrap();
        let mut heap = ManagedHeap::default();
        let buffer = array(&mut heap);
        let first = sockets
            .receive(socket, &heap, buffer.clone(), 0, 4, callback())
            .unwrap();
        peer.write_all(b"H").unwrap();
        assert!(sockets.cancel(first).unwrap());
        assert!(!sockets.cancel(first).unwrap());
        assert_eq!(
            sockets.take_result(first),
            Ok(None),
            "completion must be delivered before consumption"
        );
        deliver(&mut sockets, &heap);
        assert_eq!(sockets.take_result(first), Ok(Some(Err(Error::Cancelled))));
        assert_eq!(bytes(&heap, &buffer), vec![Value::Byte(9); 4]);
        let second = sockets
            .receive(socket, &heap, buffer.clone(), 1, 2, callback())
            .unwrap();
        assert_ne!(first, second);
        assert_eq!(sockets.cancel(first), Err(Error::UnknownOperation));
        deliver(&mut sockets, &heap);
        assert_eq!(sockets.take_result(second), Ok(Some(Ok(1))));
        assert_eq!(
            bytes(&heap, &buffer),
            vec![
                Value::Byte(9),
                Value::Byte(b'H'),
                Value::Byte(9),
                Value::Byte(9)
            ]
        );
        assert_eq!(sockets.reserved, 0);
    }

    #[test]
    fn completed_unconsumed_result_holds_admission_slot_and_is_taken_once() {
        let (_peer, stream) = pair();
        let mut sockets = Sockets::default();
        sockets.budget.operations = 1;
        let socket = sockets.adopt(stream).unwrap();
        let mut heap = ManagedHeap::default();
        let buffer = array(&mut heap);
        let first = sockets
            .receive(socket, &heap, buffer.clone(), 0, 0, callback())
            .unwrap();
        deliver(&mut sockets, &heap);
        assert!(!sockets.pending());
        assert_eq!(sockets.poll(&heap).unwrap(), None);
        assert!(
            !sockets.cancel(first).unwrap(),
            "completion wins over late cancellation"
        );
        assert_eq!(
            sockets.receive(socket, &heap, buffer.clone(), 0, 1, callback()),
            Err(Error::Limit)
        );
        assert_eq!(sockets.take_result(first), Ok(Some(Ok(0))));
        assert_eq!(sockets.take_result(first), Err(Error::UnknownOperation));
        assert!(
            sockets
                .receive(socket, &heap, buffer, 0, 1, callback())
                .is_ok()
        );
    }

    #[test]
    fn socket_and_operation_handles_do_not_alias_another_registry() {
        let (_peer, stream) = pair();
        let (_other_peer, other_stream) = pair();
        let mut first = Sockets::default();
        let mut second = Sockets::default();
        let a = first.adopt(stream).unwrap();
        let b = second.adopt(other_stream).unwrap();
        assert_ne!(a, b);
        let mut heap = ManagedHeap::default();
        let buffer = array(&mut heap);
        let op = first
            .receive(a, &heap, buffer.clone(), 0, 1, callback())
            .unwrap();
        assert_eq!(
            second.receive(a, &heap, buffer.clone(), 0, 1, callback()),
            Err(Error::Closed)
        );
        assert_eq!(second.cancel(op), Err(Error::UnknownOperation));
        second.close(a);
        assert!(second.receive(b, &heap, buffer, 0, 1, callback()).is_ok());
    }

    #[test]
    fn invalid_admission_preserves_socket_and_aggregate_budget() {
        let (_peer, stream) = pair();
        let (_peer2, stream2) = pair();
        let mut sockets = Sockets::default();
        sockets.budget.bytes = 4;
        let a = sockets.adopt(stream).unwrap();
        let b = sockets.adopt(stream2).unwrap();
        let mut heap = ManagedHeap::default();
        let buffer = array(&mut heap);
        let mut foreign = ManagedHeap::default();
        assert_eq!(
            sockets.receive(a, &heap, array(&mut foreign), 0, 1, callback()),
            Err(Error::InvalidBuffer)
        );
        for (offset, count) in [(-1, 1), (0, -1), (3, 2)] {
            assert_eq!(
                sockets.receive(a, &heap, buffer.clone(), offset, count, callback()),
                Err(Error::InvalidRange)
            );
        }
        assert_eq!(
            sockets.receive(a, &heap, buffer.clone(), 0, 1, Value::Int32(1)),
            Err(Error::InvalidCallback)
        );
        assert_eq!(sockets.reserved, 0);
        let first = sockets
            .receive(a, &heap, buffer.clone(), 0, 4, callback())
            .unwrap();
        assert_eq!(
            sockets.receive(a, &heap, buffer.clone(), 0, 0, callback()),
            Err(Error::Busy)
        );
        assert_eq!(
            sockets.receive(b, &heap, buffer.clone(), 0, 1, callback()),
            Err(Error::Limit)
        );
        sockets.cancel(first).unwrap();
        assert!(sockets.receive(b, &heap, buffer, 0, 4, callback()).is_ok());
    }

    #[test]
    fn close_completes_pending_read_once_and_frees_native_resource() {
        let (mut peer, stream) = pair();
        let mut sockets = Sockets::default();
        let socket = sockets.adopt(stream).unwrap();
        let mut heap = ManagedHeap::default();
        let buffer = array(&mut heap);
        let op = sockets
            .receive(socket, &heap, buffer.clone(), 0, 4, callback())
            .unwrap();
        sockets.close(socket);
        sockets.close(socket);
        deliver(&mut sockets, &heap);
        assert_eq!(sockets.take_result(op), Ok(Some(Err(Error::Closed))));
        assert_eq!(
            sockets.receive(socket, &heap, buffer, 0, 0, callback()),
            Err(Error::Closed)
        );
        assert_eq!(peer.read(&mut [0]).unwrap(), 0);
        assert_eq!(sockets.reserved, 0);
        assert_eq!(sockets.poll(&heap).unwrap(), None);
    }

    #[test]
    fn later_ready_socket_bypasses_stalled_read_and_eof_does_not_write() {
        let (_idle_peer, idle) = pair();
        let (peer, stream) = pair();
        let mut sockets = Sockets::default();
        let a = sockets.adopt(idle).unwrap();
        let b = sockets.adopt(stream).unwrap();
        let mut heap = ManagedHeap::default();
        let idle_buffer = array(&mut heap);
        let buffer = array(&mut heap);
        let idle_op = sockets
            .receive(a, &heap, idle_buffer, 0, 4, callback())
            .unwrap();
        let op = sockets
            .receive(b, &heap, buffer.clone(), 0, 4, callback())
            .unwrap();
        drop(peer);
        deliver(&mut sockets, &heap);
        assert_eq!(sockets.take_result(op), Ok(Some(Ok(0))));
        assert_eq!(sockets.take_result(idle_op), Ok(None));
        assert_eq!(bytes(&heap, &buffer), vec![Value::Byte(9); 4]);
    }
    #[test]
    fn open_socket_quota_rejects_and_releases_adopted_resource_then_reuses_capacity() {
        let (mut first_peer, first) = pair();
        let (mut rejected_peer, rejected) = pair();
        let mut sockets = Sockets::default();
        sockets.budget.sockets = 1;
        let id = sockets.adopt(first).unwrap();
        assert_eq!(sockets.adopt(rejected), Err(Error::Limit));
        assert_eq!(rejected_peer.read(&mut [0]).unwrap(), 0);
        sockets.close(id);
        assert_eq!(first_peer.read(&mut [0]).unwrap(), 0);
        let (_peer, stream) = pair();
        assert_ne!(sockets.adopt(stream).unwrap(), id);
    }
}
