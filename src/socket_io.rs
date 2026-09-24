//! Private owned TCP client backend behind the library Task/Result bridge.
use crate::{Fault, ManagedHeap, Value, metadata::Type};
use std::{
    collections::BTreeMap,
    io::{self, Read},
    net::{Ipv4Addr, SocketAddrV4, TcpStream},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
fn next_id() -> Result<u64, Error> {
    NEXT_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| {
            id.checked_add(1).filter(|next| *next <= i64::MAX as u64)
        })
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
    InvalidAddress,
    ConnectionRefused,
    ConnectionReset,
    AccessDenied,
    TimedOut,
}
impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::ConnectionRefused => Self::ConnectionRefused,
            io::ErrorKind::ConnectionReset | io::ErrorKind::BrokenPipe => Self::ConnectionReset,
            io::ErrorKind::PermissionDenied => Self::AccessDenied,
            io::ErrorKind::TimedOut => Self::TimedOut,
            _ => Self::Io,
        }
    }
}
// Platform-specific pending-connect errno stays below the guest contract.
fn connecting(error: &io::Error) -> bool {
    if error.kind() == io::ErrorKind::WouldBlock {
        return true;
    }
    #[cfg(unix)]
    if error.raw_os_error() == Some(libc::EINPROGRESS) {
        return true;
    }
    #[cfg(windows)]
    if error.raw_os_error() == Some(10036) {
        return true;
    }
    false
}
struct Connect {
    stream: Option<TcpStream>,
    callback: Option<Value>,
    outcome: Option<Result<SocketId, Error>>,
}
#[derive(Clone, Copy)]
pub(crate) enum Operation {
    Connect,
    ConnectResult,
    Receive,
    ReceiveResult,
    Close,
}
fn payload(result: Result<Value, Error>) -> Value {
    Value::Erased(Box::new(result.unwrap_or_else(|error| {
        Value::Byte(match error {
            Error::Closed => 1,
            Error::Busy => 2,
            Error::InvalidBuffer | Error::InvalidRange => 3,
            Error::Limit => 4,
            Error::Cancelled => 5,
            Error::InvalidAddress => 6,
            Error::ConnectionRefused => 7,
            Error::ConnectionReset => 8,
            Error::AccessDenied => 9,
            Error::TimedOut => 10,
            _ => 11,
        })
    })))
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
    connects: BTreeMap<OperationId, Connect>,
    reserved: usize,
    budget: Budget,
    receive_first: bool,
}
impl Default for Sockets {
    fn default() -> Self {
        Self {
            sockets: BTreeMap::new(),
            operations: BTreeMap::new(),
            connects: BTreeMap::new(),
            reserved: 0,
            budget: Budget::default(),
            receive_first: false,
        }
    }
}
// Adoption and individual cancellation remain private fixture/prospective-provider
// entry points; application code uses invoke through the library bridge.
#[allow(dead_code)]
impl Sockets {
    fn connect(&mut self, address: &str, port: i32, callback: Value) -> Result<OperationId, Error> {
        let address = address
            .parse::<Ipv4Addr>()
            .map_err(|_| Error::InvalidAddress)?;
        let port = u16::try_from(port)
            .ok()
            .filter(|port| *port != 0)
            .ok_or(Error::InvalidRange)?;
        if !matches!(&callback, Value::Delegate(_))
            || callback.ty() != crate::assembler::parse_type("System.Func<Void>").unwrap()
        {
            return Err(Error::InvalidCallback);
        }
        if self.operations.len() + self.connects.len() >= self.budget.operations
            || self.sockets.len()
                + self
                    .connects
                    .values()
                    .filter(|op| op.stream.is_some())
                    .count()
                >= self.budget.sockets
        {
            return Err(Error::Limit);
        }
        let id = OperationId(next_id()?);
        let socket = socket2::Socket::new(
            socket2::Domain::IPV4,
            socket2::Type::STREAM,
            Some(socket2::Protocol::TCP),
        )
        .map_err(Error::from)?;
        socket.set_nonblocking(true).map_err(Error::from)?;
        let outcome = match socket.connect(&SocketAddrV4::new(address, port).into()) {
            Ok(()) => {
                let socket_id = SocketId(next_id()?);
                self.sockets.insert(socket_id, socket.into());
                self.connects.insert(
                    id,
                    Connect {
                        stream: None,
                        callback: Some(callback),
                        outcome: Some(Ok(socket_id)),
                    },
                );
                return Ok(id);
            }
            Err(error) if connecting(&error) => None,
            Err(error) => Some(Err(Error::from(error))),
        };
        let stream = if outcome.is_none() {
            Some(socket.into())
        } else {
            None
        };
        self.connects.insert(
            id,
            Connect {
                stream,
                callback: Some(callback),
                outcome,
            },
        );
        Ok(id)
    }
    pub(crate) fn invoke(
        &mut self,
        operation: Operation,
        args: &[Value],
        heap: &ManagedHeap,
    ) -> Result<Value, Fault> {
        let result = match (operation, args) {
            (Operation::Connect, [Value::String(address), Value::Int32(port), callback]) => self
                .connect(address, *port, callback.clone())
                .map(|id| Value::Int64(id.0 as i64)),
            (
                Operation::Receive,
                [
                    Value::Int64(socket),
                    destination,
                    Value::Int32(offset),
                    Value::Int32(count),
                    callback,
                ],
            ) => self
                .receive(
                    SocketId(*socket as u64),
                    heap,
                    destination.clone(),
                    *offset,
                    *count,
                    callback.clone(),
                )
                .map(|id| Value::Int64(id.0 as i64)),
            (Operation::Close, [Value::Int64(socket)]) => {
                self.close(SocketId(*socket as u64));
                Ok(Value::Void)
            }
            (Operation::ConnectResult, [Value::Int64(id)]) => {
                let id = OperationId(*id as u64);
                let op = self
                    .connects
                    .get(&id)
                    .ok_or_else(|| Fault::new("Unknown socket connect operation"))?;
                if op.callback.is_some() || op.outcome.is_none() {
                    return Err(Fault::new("Socket connect result is not ready"));
                }
                self.connects
                    .remove(&id)
                    .unwrap()
                    .outcome
                    .unwrap()
                    .map(|socket| Value::Int64(socket.0 as i64))
            }
            (Operation::ReceiveResult, [Value::Int64(id)]) => self
                .take_result(OperationId(*id as u64))
                .map_err(|_| Fault::new("Unknown socket receive operation"))?
                .ok_or_else(|| Fault::new("Socket receive result is not ready"))?
                .map(|count| Value::Int32(count as i32)),
            _ => return Err(Fault::new("Invalid socket runtime service arguments")),
        };
        Ok(payload(result))
    }
    pub(crate) fn adopt(&mut self, socket: TcpStream) -> Result<SocketId, Error> {
        if self.sockets.len()
            + self
                .connects
                .values()
                .filter(|op| op.stream.is_some())
                .count()
            >= self.budget.sockets
        {
            return Err(Error::Limit);
        }
        socket.set_nonblocking(true).map_err(Error::from)?;
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
        if self.operations.len() + self.connects.len() >= self.budget.operations {
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
        self.connects.values().any(|op| op.callback.is_some())
            || self.operations.values().any(|op| op.callback.is_some())
    }
    pub(crate) fn trace_roots(&self, roots: &mut Vec<usize>) {
        for op in self.connects.values() {
            if let Some(callback) = &op.callback {
                crate::gc::trace(callback, roots);
            }
        }
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
        let ready = if self.receive_first {
            match self.poll_receive(heap)? {
                Some(value) => Some(value),
                None => self.poll_connect()?,
            }
        } else {
            match self.poll_connect()? {
                Some(value) => Some(value),
                None => self.poll_receive(heap)?,
            }
        };
        if ready.is_some() {
            self.receive_first = !self.receive_first;
        }
        Ok(ready)
    }
    fn poll_connect(&mut self) -> Result<Option<Value>, Fault> {
        for op in self.connects.values_mut() {
            if op.callback.is_none() {
                continue;
            }
            if op.outcome.is_none() {
                let stream = op.stream.as_ref().expect("pending connection owns stream");
                let status = match stream.take_error() {
                    Ok(Some(error)) | Err(error) => Err(Error::from(error)),
                    Ok(None) => match stream.peer_addr() {
                        Ok(_) => Ok(()),
                        Err(error)
                            if matches!(
                                error.kind(),
                                io::ErrorKind::NotConnected | io::ErrorKind::WouldBlock
                            ) =>
                        {
                            continue;
                        }
                        Err(error) => Err(Error::from(error)),
                    },
                };
                op.outcome = Some(status.and_then(|()| {
                    let id = SocketId(next_id()?);
                    self.sockets.insert(id, op.stream.take().unwrap());
                    Ok(id)
                }));
                op.stream = None;
            }
            return Ok(op.callback.take());
        }
        Ok(None)
    }
    fn poll_receive(&mut self, heap: &ManagedHeap) -> Result<Option<Value>, Fault> {
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
                    Err(error) => Err(Error::from(error)),
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

    fn connect_result(sockets: &mut Sockets, heap: &ManagedHeap, id: OperationId) -> Value {
        sockets
            .invoke(Operation::ConnectResult, &[Value::Int64(id.0 as i64)], heap)
            .unwrap()
    }
    #[test]
    fn ready_connects_do_not_starve_ready_receives() {
        let (_, stream) = pair();
        let mut sockets = Sockets::default();
        let mut heap = ManagedHeap::default();
        let socket = sockets.adopt(stream).unwrap();
        let buffer = array(&mut heap);
        let read = sockets
            .receive(socket, &heap, buffer, 0, 0, callback())
            .unwrap();
        for _ in 0..2 {
            sockets.connects.insert(
                OperationId(next_id().unwrap()),
                Connect {
                    stream: None,
                    callback: Some(callback()),
                    outcome: Some(Err(Error::Io)),
                },
            );
        }
        assert!(sockets.poll(&heap).unwrap().is_some());
        assert!(sockets.operations[&read].callback.is_some());
        assert!(sockets.poll(&heap).unwrap().is_some());
        assert!(sockets.operations[&read].callback.is_none());
        assert_eq!(
            sockets
                .connects
                .values()
                .filter(|op| op.callback.is_some())
                .count(),
            1
        );
    }

    #[test]
    fn public_connect_receive_bridge_consumes_results_and_closes() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut sockets = Sockets::default();
        let mut heap = ManagedHeap::default();
        let id = sockets
            .connect(
                "127.0.0.1",
                listener.local_addr().unwrap().port().into(),
                callback(),
            )
            .unwrap();
        assert!(
            sockets
                .invoke(
                    Operation::ConnectResult,
                    &[Value::Int64(id.0 as i64)],
                    &heap
                )
                .is_err()
        );
        deliver(&mut sockets, &heap);
        let Value::Erased(result) = connect_result(&mut sockets, &heap, id) else {
            panic!()
        };
        let Value::Int64(handle) = *result else {
            panic!("{result:?}")
        };
        assert!(
            sockets
                .invoke(
                    Operation::ConnectResult,
                    &[Value::Int64(id.0 as i64)],
                    &heap
                )
                .is_err()
        );
        let mut peer = listener.accept().unwrap().0;
        peer.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let buffer = array(&mut heap);
        let operation = sockets
            .receive(
                SocketId(handle as u64),
                &heap,
                buffer.clone(),
                1,
                2,
                callback(),
            )
            .unwrap();
        peer.write_all(b"Hi").unwrap();
        deliver(&mut sockets, &heap);
        assert_eq!(
            sockets
                .invoke(
                    Operation::ReceiveResult,
                    &[Value::Int64(operation.0 as i64)],
                    &heap
                )
                .unwrap(),
            Value::Erased(Box::new(Value::Int32(2)))
        );
        assert_eq!(
            bytes(&heap, &buffer),
            vec![
                Value::Byte(9),
                Value::Byte(72),
                Value::Byte(105),
                Value::Byte(9)
            ]
        );
        sockets
            .invoke(Operation::Close, &[Value::Int64(handle)], &heap)
            .unwrap();
        assert_eq!(peer.read(&mut [0]).unwrap(), 0);
        assert!(
            sockets.operations.is_empty()
                && sockets.connects.is_empty()
                && sockets.sockets.is_empty()
        );
    }
    #[test]
    fn connect_validation_and_shared_quotas_are_transactional() {
        let mut sockets = Sockets::default();
        let heap = ManagedHeap::default();
        assert_eq!(
            sockets.connect("localhost", 80, callback()),
            Err(Error::InvalidAddress)
        );
        assert_eq!(
            sockets.connect("::1", 80, callback()),
            Err(Error::InvalidAddress)
        );
        for port in [-1, 0, 65536] {
            assert_eq!(
                sockets.connect("127.0.0.1", port, callback()),
                Err(Error::InvalidRange)
            );
        }
        assert_eq!(
            sockets.connect("127.0.0.1", 80, Value::Void),
            Err(Error::InvalidCallback)
        );
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port().into();
        sockets.budget.sockets = 1;
        sockets.budget.operations = 1;
        let first = sockets.connect("127.0.0.1", port, callback()).unwrap();
        assert_eq!(
            sockets.connect("127.0.0.1", port, callback()),
            Err(Error::Limit)
        );
        deliver(&mut sockets, &heap);
        assert_eq!(
            sockets.connect("127.0.0.1", port, callback()),
            Err(Error::Limit)
        );
        assert!(sockets.poll(&heap).unwrap().is_none());
        let Value::Erased(result) = connect_result(&mut sockets, &heap, first) else {
            panic!()
        };
        let Value::Int64(socket) = *result else {
            panic!()
        };
        sockets.close(SocketId(socket as u64));
        assert!(sockets.connect("127.0.0.1", port, callback()).is_ok());
    }
    #[test]
    fn refused_connect_releases_socket_slot_but_retains_result_until_consumed() {
        // Select an ephemeral loopback port, then release it so the kernel refuses connect.
        // Keeping it bound without listen can silently drop SYN on macOS.
        let reserved = socket2::Socket::new(
            socket2::Domain::IPV4,
            socket2::Type::STREAM,
            Some(socket2::Protocol::TCP),
        )
        .unwrap();
        reserved
            .bind(
                &"127.0.0.1:0"
                    .parse::<std::net::SocketAddr>()
                    .unwrap()
                    .into(),
            )
            .unwrap();
        let port = reserved
            .local_addr()
            .unwrap()
            .as_socket()
            .unwrap()
            .port()
            .into();
        drop(reserved);
        let mut sockets = Sockets::default();
        let heap = ManagedHeap::default();
        let id = sockets.connect("127.0.0.1", port, callback()).unwrap();
        deliver(&mut sockets, &heap);
        assert!(sockets.sockets.is_empty());
        assert!(sockets.connects[&id].stream.is_none());
        assert_eq!(
            connect_result(&mut sockets, &heap, id),
            Value::Erased(Box::new(Value::Byte(7)))
        );
        assert!(sockets.connects.is_empty());
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
