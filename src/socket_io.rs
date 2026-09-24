//! Private owned TCP client backend behind the library Task/Result bridge.
use crate::{Fault, ManagedHeap, Value, metadata::Type};
use std::{
    collections::{BTreeMap, VecDeque},
    io::{self, Read, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

// Provisional shared connection budget; DNS remains a separate operation.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
// Provisional per-transfer budget; not a connection idle or whole-request timeout.
const TRANSFER_TIMEOUT: Duration = Duration::from_secs(5);
const ADDRESS_TIMEOUT: Duration = Duration::from_secs(1);
const MAX_ADDRESSES: usize = 16;

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
    InvalidOperation,
    AddressInUse,
}
impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::ConnectionRefused => Self::ConnectionRefused,
            io::ErrorKind::ConnectionReset | io::ErrorKind::BrokenPipe => Self::ConnectionReset,
            io::ErrorKind::PermissionDenied => Self::AccessDenied,
            io::ErrorKind::TimedOut => Self::TimedOut,
            io::ErrorKind::AddrInUse => Self::AddressInUse,
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
    deadline: Option<Instant>,
    attempt_deadline: Option<Instant>,
    remaining: VecDeque<SocketAddrV4>,
    listener: Option<SocketId>,
    stream: Option<TcpStream>,
    callback: Option<Value>,
    outcome: Option<Result<SocketId, Error>>,
}
#[derive(Clone, Copy)]
pub(crate) enum Operation {
    DeadlineAfter,
    DeadlineExpired,
    ConnectAddressesUntil,
    ReceiveUntil,
    SendUntil,
    Listen,
    Accept,
    LocalPort,
    Connect,
    ConnectAddresses,
    ConnectResult,
    Receive,
    Send,
    TransferResult,
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
            Error::InvalidOperation => 12,
            Error::AddressInUse => 13,
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
struct Transfer {
    deadline: Instant,
    sending: bool,
    socket: SocketId,
    destination: Option<Value>,
    callback: Option<Value>,
    offset: usize,
    buffer: Vec<u8>,
    outcome: Option<Result<usize, Error>>,
}
pub(crate) struct Sockets {
    sockets: BTreeMap<SocketId, TcpStream>,
    listeners: BTreeMap<SocketId, TcpListener>,
    operations: BTreeMap<OperationId, Transfer>,
    connects: BTreeMap<OperationId, Connect>,
    reserved: usize,
    budget: Budget,
    transfer_first: bool,
}
impl Default for Sockets {
    fn default() -> Self {
        Self {
            sockets: BTreeMap::new(),
            listeners: BTreeMap::new(),
            operations: BTreeMap::new(),
            connects: BTreeMap::new(),
            reserved: 0,
            budget: Budget::default(),
            transfer_first: false,
        }
    }
}
// Adoption and individual cancellation remain private fixture/prospective-provider
// entry points; application code uses invoke through the library bridge.
#[allow(dead_code)]
impl Sockets {
    fn resource_count(&self) -> usize {
        self.sockets.len()
            + self.listeners.len()
            + self
                .connects
                .values()
                .filter(|op| op.stream.is_some() || op.listener.is_some() && op.outcome.is_none())
                .count()
    }
    fn listen(&mut self, address: &str, port: i32, backlog: i32) -> Result<SocketId, Error> {
        let address = address
            .parse::<Ipv4Addr>()
            .map_err(|_| Error::InvalidAddress)?;
        let port = u16::try_from(port).map_err(|_| Error::InvalidRange)?;
        if !(1..=128).contains(&backlog) {
            return Err(Error::InvalidRange);
        }
        if self.resource_count() >= self.budget.sockets {
            return Err(Error::Limit);
        }
        let id = SocketId(next_id()?);
        let socket = socket2::Socket::new(
            socket2::Domain::IPV4,
            socket2::Type::STREAM,
            Some(socket2::Protocol::TCP),
        )
        .map_err(Error::from)?;
        socket.set_nonblocking(true).map_err(Error::from)?;
        socket
            .bind(&SocketAddrV4::new(address, port).into())
            .map_err(Error::from)?;
        socket.listen(backlog).map_err(Error::from)?;
        self.listeners.insert(id, socket.into());
        Ok(id)
    }
    fn local_port(&self, socket: SocketId) -> Result<i32, Error> {
        let address = if let Some(listener) = self.listeners.get(&socket) {
            listener.local_addr()
        } else if let Some(stream) = self.sockets.get(&socket) {
            stream.local_addr()
        } else {
            return Err(Error::Closed);
        };
        address
            .map(|address| i32::from(address.port()))
            .map_err(Error::from)
    }
    fn accept(&mut self, listener: SocketId, callback: Value) -> Result<OperationId, Error> {
        if self.sockets.contains_key(&listener) {
            return Err(Error::InvalidOperation);
        }
        if !self.listeners.contains_key(&listener) {
            return Err(Error::Closed);
        }
        if !matches!(&callback, Value::Delegate(_))
            || callback.ty() != crate::assembler::parse_type("System.Func<Void>").unwrap()
        {
            return Err(Error::InvalidCallback);
        }
        if self
            .connects
            .values()
            .any(|op| op.listener == Some(listener) && op.outcome.is_none())
        {
            return Err(Error::Busy);
        }
        if self.resource_count() >= self.budget.sockets
            || self.connects.len() + self.operations.len() >= self.budget.operations
        {
            return Err(Error::Limit);
        }
        let id = OperationId(next_id()?);
        self.connects.insert(
            id,
            Connect {
                attempt_deadline: None,
                remaining: VecDeque::new(),
                deadline: None,
                listener: Some(listener),
                stream: None,
                callback: Some(callback),
                outcome: None,
            },
        );
        Ok(id)
    }
    fn connect(&mut self, address: &str, port: i32, callback: Value) -> Result<OperationId, Error> {
        self.connect_addresses(&[address], port, callback)
    }
    fn connect_addresses(
        &mut self,
        addresses: &[&str],
        port: i32,
        callback: Value,
    ) -> Result<OperationId, Error> {
        self.connect_addresses_until(addresses, port, callback, None)
    }
    // A provider can share one absolute budget across lookup, connect and I/O.
    // Existing guest entry points retain their phase-local policies.
    pub(crate) fn connect_addresses_until(
        &mut self,
        addresses: &[&str],
        port: i32,
        callback: Value,
        until: Option<Instant>,
    ) -> Result<OperationId, Error> {
        if addresses.is_empty() {
            return Err(Error::InvalidRange);
        }
        if addresses.len() > MAX_ADDRESSES {
            return Err(Error::Limit);
        }
        let port = u16::try_from(port)
            .ok()
            .filter(|port| *port != 0)
            .ok_or(Error::InvalidRange)?;
        let mut remaining = VecDeque::new();
        for address in addresses {
            let address = address
                .parse::<Ipv4Addr>()
                .map_err(|_| Error::InvalidAddress)?;
            let endpoint = SocketAddrV4::new(address, port);
            if !remaining.contains(&endpoint) {
                remaining.push_back(endpoint);
            }
        }
        if !matches!(&callback, Value::Delegate(_))
            || callback.ty() != crate::assembler::parse_type("System.Func<Void>").unwrap()
        {
            return Err(Error::InvalidCallback);
        }
        if self.operations.len() + self.connects.len() >= self.budget.operations
            || self.resource_count() >= self.budget.sockets
        {
            return Err(Error::Limit);
        }
        let now = Instant::now();
        let deadline = until.map_or(now + CONNECT_TIMEOUT, |end| end.min(now + CONNECT_TIMEOUT));
        if now >= deadline {
            return Err(Error::TimedOut);
        }
        let id = OperationId(next_id()?);
        let mut op = Connect {
            deadline: Some(deadline),
            attempt_deadline: None,
            remaining,
            listener: None,
            stream: None,
            callback: Some(callback),
            outcome: None,
        };
        Self::start_next_address(&mut op, &mut self.sockets, now);
        self.connects.insert(id, op);
        Ok(id)
    }
    fn start_next_address(
        op: &mut Connect,
        sockets: &mut BTreeMap<SocketId, TcpStream>,
        now: Instant,
    ) {
        // Every retry drops the previous native owner before opening another one.
        op.stream = None;
        if op.deadline.is_some_and(|deadline| now >= deadline) {
            op.remaining.clear();
            op.outcome = Some(Err(Error::TimedOut));
            return;
        }
        while let Some(address) = op.remaining.pop_front() {
            let attempt = (|| {
                let socket = socket2::Socket::new(
                    socket2::Domain::IPV4,
                    socket2::Type::STREAM,
                    Some(socket2::Protocol::TCP),
                )?;
                socket.set_nonblocking(true)?;
                match socket.connect(&address.into()) {
                    Ok(()) => Ok((socket.into(), true)),
                    Err(error) if connecting(&error) => Ok((socket.into(), false)),
                    Err(error) => Err(error),
                }
            })();
            match attempt {
                Ok((stream, true)) => {
                    op.outcome = Some(next_id().map(|id| {
                        let id = SocketId(id);
                        sockets.insert(id, stream);
                        id
                    }));
                    op.remaining.clear();
                    return;
                }
                Ok((stream, false)) => {
                    op.stream = Some(stream);
                    let deadline = op.deadline.expect("connect has deadline");
                    op.attempt_deadline = Some(if op.remaining.is_empty() {
                        deadline
                    } else {
                        deadline.min(now + ADDRESS_TIMEOUT)
                    });
                    return;
                }
                Err(error) if op.remaining.is_empty() => {
                    op.outcome = Some(Err(Error::from(error)));
                    return;
                }
                Err(_) => {}
            }
        }
    }
    fn connect_array(
        &mut self,
        addresses: &Value,
        port: i32,
        callback: Value,
        heap: &ManagedHeap,
        until: Option<Instant>,
    ) -> Result<OperationId, Error> {
        let Value::ObjectReference(object) = addresses else {
            return Err(Error::InvalidBuffer);
        };
        let Value::Array {
            element: Type::String,
            elements,
        } = heap
            .read_reference(&object.reference)
            .map_err(|_| Error::InvalidBuffer)?
        else {
            return Err(Error::InvalidBuffer);
        };
        if elements.len() > MAX_ADDRESSES {
            return Err(Error::Limit);
        }
        let addresses = elements
            .iter()
            .map(|value| match value {
                Value::String(address) => Ok(address.as_str()),
                _ => Err(Error::InvalidAddress),
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.connect_addresses_until(&addresses, port, callback, until)
    }

    pub(crate) fn invoke(
        &mut self,
        operation: Operation,
        args: &[Value],
        heap: &ManagedHeap,
    ) -> Result<Value, Fault> {
        match (operation, args) {
            (Operation::DeadlineAfter, [Value::Int32(milliseconds)]) => {
                return crate::clock::network_deadline_after(*milliseconds).map(Value::Int64);
            }
            (Operation::DeadlineExpired, [Value::Int64(stamp)]) => {
                let deadline = crate::clock::network_deadline(*stamp)?;
                return Ok(Value::Boolean(Instant::now() >= deadline));
            }
            _ => {}
        }
        let result = match (operation, args) {
            (
                Operation::ConnectAddressesUntil,
                [addresses, Value::Int32(port), Value::Int64(stamp), callback],
            ) => {
                let until = crate::clock::network_deadline(*stamp)?;
                self.connect_array(addresses, *port, callback.clone(), heap, Some(until))
                    .map(|id| Value::Int64(id.0 as i64))
            }
            (
                Operation::ReceiveUntil | Operation::SendUntil,
                [
                    Value::Int64(socket),
                    destination,
                    Value::Int32(offset),
                    Value::Int32(count),
                    Value::Int64(stamp),
                    callback,
                ],
            ) => {
                let until = crate::clock::network_deadline(*stamp)?;
                self.transfer_until(
                    matches!(operation, Operation::SendUntil),
                    SocketId(*socket as u64),
                    heap,
                    destination.clone(),
                    *offset,
                    *count,
                    callback.clone(),
                    Some(until),
                )
                .map(|id| Value::Int64(id.0 as i64))
            }
            (
                Operation::Listen,
                [
                    Value::String(address),
                    Value::Int32(port),
                    Value::Int32(backlog),
                ],
            ) => self
                .listen(address, *port, *backlog)
                .map(|id| Value::Int64(id.0 as i64)),
            (Operation::Accept, [Value::Int64(listener), callback]) => self
                .accept(SocketId(*listener as u64), callback.clone())
                .map(|id| Value::Int64(id.0 as i64)),
            (Operation::LocalPort, [Value::Int64(socket)]) => {
                self.local_port(SocketId(*socket as u64)).map(Value::Int32)
            }
            (Operation::ConnectAddresses, [addresses, Value::Int32(port), callback]) => self
                .connect_array(addresses, *port, callback.clone(), heap, None)
                .map(|id| Value::Int64(id.0 as i64)),
            (Operation::Connect, [Value::String(address), Value::Int32(port), callback]) => self
                .connect(address, *port, callback.clone())
                .map(|id| Value::Int64(id.0 as i64)),
            (
                Operation::Receive | Operation::Send,
                [
                    Value::Int64(socket),
                    destination,
                    Value::Int32(offset),
                    Value::Int32(count),
                    callback,
                ],
            ) => self
                .transfer(
                    matches!(operation, Operation::Send),
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
            (Operation::TransferResult, [Value::Int64(id)]) => self
                .take_result(OperationId(*id as u64))
                .map_err(|_| Fault::new("Unknown socket transfer operation"))?
                .ok_or_else(|| Fault::new("Socket transfer result is not ready"))?
                .map(|count| Value::Int32(count as i32)),
            _ => return Err(Fault::new("Invalid socket runtime service arguments")),
        };
        Ok(payload(result))
    }
    pub(crate) fn adopt(&mut self, socket: TcpStream) -> Result<SocketId, Error> {
        if self.resource_count() >= self.budget.sockets {
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
        self.transfer(false, socket, heap, destination, offset, count, callback)
    }
    fn send(
        &mut self,
        socket: SocketId,
        heap: &ManagedHeap,
        source: Value,
        offset: i32,
        count: i32,
        callback: Value,
    ) -> Result<OperationId, Error> {
        self.transfer(true, socket, heap, source, offset, count, callback)
    }
    fn transfer(
        &mut self,
        sending: bool,
        socket: SocketId,
        heap: &ManagedHeap,
        destination: Value,
        offset: i32,
        count: i32,
        callback: Value,
    ) -> Result<OperationId, Error> {
        self.transfer_until(
            sending,
            socket,
            heap,
            destination,
            offset,
            count,
            callback,
            None,
        )
    }
    pub(crate) fn transfer_until(
        &mut self,
        sending: bool,
        socket: SocketId,
        heap: &ManagedHeap,
        destination: Value,
        offset: i32,
        count: i32,
        callback: Value,
        until: Option<Instant>,
    ) -> Result<OperationId, Error> {
        if self.listeners.contains_key(&socket) {
            return Err(Error::InvalidOperation);
        }
        if !self.sockets.contains_key(&socket) {
            return Err(Error::Closed);
        }
        if self
            .operations
            .values()
            .any(|op| op.socket == socket && op.sending == sending && op.outcome.is_none())
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
        let now = Instant::now();
        let deadline = until.map_or(now + TRANSFER_TIMEOUT, |end| {
            end.min(now + TRANSFER_TIMEOUT)
        });
        // A request budget also forbids starting an empty operation after expiry;
        // ordinary empty Send/Receive retain their successful-zero behavior.
        if until.is_some_and(|end| now >= end) {
            return Err(Error::TimedOut);
        }
        let mut buffer = Vec::new();
        buffer.try_reserve_exact(count).map_err(|_| Error::Limit)?;
        if sending {
            // Snapshot only after admission checks. No borrowed guest storage crosses
            // a pending send; later mutations cannot change this operation's bytes.
            for value in &elements[offset..offset + count] {
                let Value::Byte(byte) = value else {
                    return Err(Error::InvalidBuffer);
                };
                buffer.push(*byte);
            }
        } else {
            buffer.resize(count, 0);
        }
        let id = OperationId(next_id()?);
        self.operations.insert(
            id,
            Transfer {
                deadline,
                sending,
                socket,
                destination: if sending { None } else { Some(destination) },
                callback: Some(callback),
                offset,
                buffer,
                outcome: None,
            },
        );
        self.reserved = reserved;
        Ok(id)
    }
    fn settle(op: &mut Transfer, reserved: &mut usize, outcome: Result<usize, Error>) {
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
        // No native transfer is in flight concurrently: all calls run on the owner.
        Self::settle(op, &mut self.reserved, Err(Error::Cancelled));
        Ok(true)
    }
    pub(crate) fn close(&mut self, socket: SocketId) {
        self.sockets.remove(&socket);
        self.listeners.remove(&socket);
        for op in self
            .connects
            .values_mut()
            .filter(|op| op.listener == Some(socket) && op.outcome.is_none())
        {
            op.outcome = Some(Err(Error::Closed));
        }
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
        let ready = if self.transfer_first {
            match self.poll_transfer(heap)? {
                Some(value) => Some(value),
                None => self.poll_connect()?,
            }
        } else {
            match self.poll_connect()? {
                Some(value) => Some(value),
                None => self.poll_transfer(heap)?,
            }
        };
        if ready.is_some() {
            self.transfer_first = !self.transfer_first;
        }
        Ok(ready)
    }
    fn poll_connect(&mut self) -> Result<Option<Value>, Fault> {
        self.poll_connect_at(Instant::now())
    }
    fn poll_connect_at(&mut self, now: Instant) -> Result<Option<Value>, Fault> {
        for op in self.connects.values_mut() {
            if op.callback.is_none() {
                continue;
            }
            if op.outcome.is_none() {
                // Owner-observed completion wins only before expiry. Once committed,
                // an outcome survives later polls and delayed guest delivery.
                if op.deadline.is_some_and(|deadline| now >= deadline) {
                    op.stream = None;
                    op.remaining.clear();
                    op.outcome = Some(Err(Error::TimedOut));
                    return Ok(op.callback.take());
                }
                if let Some(listener) = op.listener {
                    let listener = self
                        .listeners
                        .get(&listener)
                        .expect("close settles pending accept");
                    let accepted = match listener.accept() {
                        Err(error)
                            if matches!(
                                error.kind(),
                                io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                            ) =>
                        {
                            continue;
                        }
                        result => result.map_err(Error::from),
                    };
                    op.outcome = Some(accepted.and_then(|(stream, _)| {
                        stream.set_nonblocking(true).map_err(Error::from)?;
                        let id = SocketId(next_id()?);
                        self.sockets.insert(id, stream);
                        Ok(id)
                    }));
                    return Ok(op.callback.take());
                }
                if !op.remaining.is_empty()
                    && op.attempt_deadline.is_some_and(|deadline| now >= deadline)
                {
                    Self::start_next_address(op, &mut self.sockets, now);
                    if op.outcome.is_some() {
                        return Ok(op.callback.take());
                    }
                    continue;
                }
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
                if status.is_err() && !op.remaining.is_empty() {
                    Self::start_next_address(op, &mut self.sockets, now);
                    if op.outcome.is_some() {
                        return Ok(op.callback.take());
                    }
                    continue;
                }
                op.remaining.clear();
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
    fn poll_transfer(&mut self, heap: &ManagedHeap) -> Result<Option<Value>, Fault> {
        self.poll_transfer_at(heap, Instant::now())
    }
    fn poll_transfer_at(
        &mut self,
        heap: &ManagedHeap,
        now: Instant,
    ) -> Result<Option<Value>, Fault> {
        for op in self.operations.values_mut() {
            if op.callback.is_none() {
                continue;
            }
            if op.outcome.is_none() {
                // Expiry wins over readiness not yet observed by the owner. No
                // native I/O is in flight elsewhere, so no bytes can arrive late
                // into guest storage after this operation has settled. Empty
                // transfers perform no I/O and retain their successful-zero rule.
                if !op.buffer.is_empty() && now >= op.deadline {
                    Self::settle(op, &mut self.reserved, Err(Error::TimedOut));
                    return Ok(op.callback.take());
                }
                let socket = self
                    .sockets
                    .get_mut(&op.socket)
                    .expect("close settles pending operations");
                let transfer = if op.buffer.is_empty() {
                    Ok(0)
                } else if op.sending {
                    socket.write(&op.buffer)
                } else {
                    socket.read(&mut op.buffer)
                };
                let outcome = match transfer {
                    Err(e)
                        if matches!(
                            e.kind(),
                            io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                        ) =>
                    {
                        continue;
                    }
                    Err(error) => Err(Error::from(error)),
                    Ok(0) if op.sending && !op.buffer.is_empty() => Err(Error::Io),
                    Ok(count) if op.sending => Ok(count),
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
    fn transfer_expiry_releases_storage_without_transferring_or_closing_socket() {
        for sending in [false, true] {
            let (mut peer, stream) = pair();
            let mut sockets = Sockets::default();
            let socket = sockets.adopt(stream).unwrap();
            let mut heap = ManagedHeap::default();
            let buffer = array(&mut heap);
            let id = sockets
                .transfer(sending, socket, &heap, buffer.clone(), 0, 4, callback())
                .unwrap();
            let deadline = sockets.operations[&id].deadline;
            // Even ready bytes must not beat expiry at the owner observation point.
            peer.write_all(b"H").unwrap();
            assert_eq!(sockets.reserved, 4);
            assert!(sockets.poll_transfer_at(&heap, deadline).unwrap().is_some());
            assert_eq!(sockets.reserved, 0);
            assert!(sockets.operations[&id].destination.is_none());
            assert!(!sockets.cancel(id).unwrap());
            assert!(sockets.poll_transfer_at(&heap, deadline).unwrap().is_none());
            assert_eq!(sockets.take_result(id), Ok(Some(Err(Error::TimedOut))));
            assert_eq!(sockets.take_result(id), Err(Error::UnknownOperation));
            assert_eq!(bytes(&heap, &buffer), vec![Value::Byte(9); 4]);
            peer.set_nonblocking(true).unwrap();
            assert_eq!(
                peer.read(&mut [0]).unwrap_err().kind(),
                io::ErrorKind::WouldBlock
            );
            let next = sockets
                .receive(socket, &heap, buffer.clone(), 1, 1, callback())
                .unwrap();
            deliver(&mut sockets, &heap);
            assert_eq!(sockets.take_result(next), Ok(Some(Ok(1))));
            assert_eq!(bytes(&heap, &buffer)[1], Value::Byte(b'H'));
            let mut roots = vec![];
            sockets.trace_roots(&mut roots);
            assert!(roots.is_empty(), "completed transfers release guest roots");
        }
    }

    #[test]
    fn pending_transfer_expires_at_boundary_and_does_not_renew_while_polled() {
        let (_peer, stream) = pair();
        let mut sockets = Sockets::default();
        sockets.budget.operations = 1;
        let socket = sockets.adopt(stream).unwrap();
        let mut heap = ManagedHeap::default();
        let buffer = array(&mut heap);
        let id = sockets
            .receive(socket, &heap, buffer.clone(), 0, 1, callback())
            .unwrap();
        let deadline = sockets.operations[&id].deadline;
        assert!(
            sockets
                .poll_transfer_at(&heap, deadline - Duration::from_nanos(1))
                .unwrap()
                .is_none()
        );
        assert_eq!(sockets.operations[&id].deadline, deadline);
        assert!(sockets.poll_transfer_at(&heap, deadline).unwrap().is_some());
        assert_eq!(
            sockets.send(socket, &heap, buffer.clone(), 0, 1, callback()),
            Err(Error::Limit)
        );
        assert_eq!(sockets.take_result(id), Ok(Some(Err(Error::TimedOut))));
        let next = sockets
            .send(socket, &heap, buffer, 0, 1, callback())
            .unwrap();
        deliver(&mut sockets, &heap);
        assert_eq!(sockets.take_result(next), Ok(Some(Ok(1))));
    }

    #[test]
    fn committed_outcomes_and_empty_transfers_survive_late_poll() {
        for mode in 0..4 {
            let (_peer, stream) = pair();
            let mut sockets = Sockets::default();
            let socket = sockets.adopt(stream).unwrap();
            let mut heap = ManagedHeap::default();
            let buffer = array(&mut heap);
            let id = sockets
                .send(
                    socket,
                    &heap,
                    buffer,
                    0,
                    if mode == 3 { 0 } else { 1 },
                    callback(),
                )
                .unwrap();
            let deadline = sockets.operations[&id].deadline;
            let expected = match mode {
                0 => {
                    sockets.cancel(id).unwrap();
                    Err(Error::Cancelled)
                }
                1 => {
                    sockets.close(socket);
                    Err(Error::Closed)
                }
                2 => {
                    deliver(&mut sockets, &heap);
                    Ok(1)
                }
                _ => Ok(0),
            };
            let delivered = sockets
                .poll_transfer_at(&heap, deadline + Duration::from_secs(1))
                .unwrap();
            assert_eq!(delivered.is_some(), mode != 2);
            assert_eq!(sockets.take_result(id), Ok(Some(expected)));
            assert_eq!(sockets.reserved, 0);
        }
    }

    #[test]
    fn shared_request_deadline_is_not_renewed_by_successful_short_transfers() {
        let (mut peer, stream) = pair();
        let mut sockets = Sockets::default();
        let socket = sockets.adopt(stream).unwrap();
        let mut heap = ManagedHeap::default();
        let buffer = array(&mut heap);
        let end = Instant::now() + Duration::from_secs(1);
        // Same absolute budget, progressing through two one-byte body fragments.
        for value in [b'A', b'B'] {
            peer.write_all(&[value]).unwrap();
            let id = sockets
                .transfer_until(
                    false,
                    socket,
                    &heap,
                    buffer.clone(),
                    0,
                    4,
                    callback(),
                    Some(end),
                )
                .unwrap();
            assert_eq!(sockets.operations[&id].deadline, end);
            deliver(&mut sockets, &heap);
            assert_eq!(sockets.take_result(id), Ok(Some(Ok(1))));
            assert_eq!(bytes(&heap, &buffer)[0], Value::Byte(value));
        }
        let id = sockets
            .transfer_until(
                false,
                socket,
                &heap,
                buffer.clone(),
                0,
                4,
                callback(),
                Some(end),
            )
            .unwrap();
        assert_eq!(sockets.operations[&id].deadline, end);
        peer.write_all(b"C").unwrap();
        assert!(sockets.poll_transfer_at(&heap, end).unwrap().is_some());
        assert_eq!(sockets.take_result(id), Ok(Some(Err(Error::TimedOut))));
        assert_eq!(bytes(&heap, &buffer)[0], Value::Byte(b'B'));
        assert_eq!(sockets.reserved, 0);
        sockets.close(socket);
    }

    #[test]
    fn lookup_connect_and_body_progress_share_one_absolute_budget() {
        let end = Instant::now() + Duration::from_secs(4);
        let mut resolver = crate::name_resolution::Resolver::with_wake(std::sync::Arc::new(
            crate::scheduler::Wake::default(),
        ));
        let lookup = resolver
            .submit_until("127.0.0.1", callback(), Duration::from_secs(5), Some(end))
            .unwrap();
        while resolver.poll().is_none() {
            assert!(Instant::now() < end, "local resolver watchdog");
            std::thread::yield_now();
        }
        let addresses = resolver.take_result(lookup).unwrap().unwrap().unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut sockets = Sockets::default();
        let mut heap = ManagedHeap::default();
        let address = addresses[0].to_string();
        let connect = sockets
            .connect_addresses_until(
                &[&address],
                listener.local_addr().unwrap().port().into(),
                callback(),
                Some(end),
            )
            .unwrap();
        assert_eq!(sockets.connects[&connect].deadline, Some(end));
        deliver(&mut sockets, &heap);
        let socket = accepted(&mut sockets, &heap, connect);
        let (mut peer, _) = listener.accept().unwrap();
        let buffer = array(&mut heap);
        peer.write_all(b"A").unwrap();
        let first = sockets
            .transfer_until(
                false,
                socket,
                &heap,
                buffer.clone(),
                0,
                4,
                callback(),
                Some(end),
            )
            .unwrap();
        assert_eq!(sockets.operations[&first].deadline, end);
        deliver(&mut sockets, &heap);
        assert_eq!(sockets.take_result(first), Ok(Some(Ok(1))));
        let next = sockets
            .transfer_until(
                false,
                socket,
                &heap,
                buffer.clone(),
                1,
                3,
                callback(),
                Some(end),
            )
            .unwrap();
        assert_eq!(sockets.operations[&next].deadline, end);
        assert!(sockets.poll_transfer_at(&heap, end).unwrap().is_some());
        assert_eq!(sockets.take_result(next), Ok(Some(Err(Error::TimedOut))));
        assert_eq!(
            bytes(&heap, &buffer),
            vec![
                Value::Byte(b'A'),
                Value::Byte(9),
                Value::Byte(9),
                Value::Byte(9)
            ]
        );
        sockets.close(socket);
        assert!(sockets.operations.is_empty() && sockets.connects.is_empty());
        assert!(!resolver.pending());
        assert_eq!(sockets.reserved, 0);
    }

    #[test]
    fn expired_request_does_not_admit_connect_or_transfer_and_long_budget_keeps_phase_bound() {
        let (mut peer, stream) = pair();
        let mut sockets = Sockets::default();
        let socket = sockets.adopt(stream).unwrap();
        let mut heap = ManagedHeap::default();
        let buffer = array(&mut heap);
        let expired = Instant::now();
        assert_eq!(
            sockets.connect_addresses_until(&["127.0.0.1"], 80, callback(), Some(expired)),
            Err(Error::TimedOut)
        );
        for sending in [false, true] {
            assert_eq!(
                sockets.transfer_until(
                    sending,
                    socket,
                    &heap,
                    buffer.clone(),
                    0,
                    1,
                    callback(),
                    Some(expired)
                ),
                Err(Error::TimedOut)
            );
        }
        assert!(sockets.operations.is_empty() && sockets.connects.is_empty());
        assert_eq!(sockets.reserved, 0);
        peer.set_nonblocking(true).unwrap();
        assert_eq!(
            peer.read(&mut [0]).unwrap_err().kind(),
            io::ErrorKind::WouldBlock
        );
        let now = Instant::now();
        let id = sockets
            .transfer_until(
                false,
                socket,
                &heap,
                buffer,
                0,
                1,
                callback(),
                Some(now + Duration::from_secs(60)),
            )
            .unwrap();
        assert!(sockets.operations[&id].deadline <= Instant::now() + TRANSFER_TIMEOUT);
        assert!(sockets.operations[&id].deadline < now + Duration::from_secs(60));
        sockets.close(socket);
    }

    #[test]
    fn shared_connect_budget_survives_fallback_and_stops_next_attempt() {
        let (_peer, stream) = pair();
        let mut sockets = Sockets::default();
        let end = Instant::now() + Duration::from_secs(2);
        let id = staged_connect(&mut sockets, stream, end);
        let op = sockets.connects.get_mut(&id).unwrap();
        op.remaining
            .push_back(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 80));
        // Even a prepared fallback must not open another native socket at expiry.
        Sockets::start_next_address(op, &mut sockets.sockets, end);
        assert_eq!(op.outcome, Some(Err(Error::TimedOut)));
        assert_eq!(op.deadline, Some(end));
        assert!(op.stream.is_none() && op.remaining.is_empty());
        assert!(sockets.sockets.is_empty());
    }

    fn connect_result(sockets: &mut Sockets, heap: &ManagedHeap, id: OperationId) -> Value {
        sockets
            .invoke(Operation::ConnectResult, &[Value::Int64(id.0 as i64)], heap)
            .unwrap()
    }
    fn accepted(sockets: &mut Sockets, heap: &ManagedHeap, id: OperationId) -> SocketId {
        let Value::Erased(value) = connect_result(sockets, heap, id) else {
            panic!()
        };
        let Value::Int64(id) = *value else {
            panic!("accept failed: {value:?}")
        };
        SocketId(id as u64)
    }
    // A native stream plus an owner-controlled clock makes expiry deterministic:
    // no external black-hole address, routing assumption or five-second sleep.
    fn staged_connect(sockets: &mut Sockets, stream: TcpStream, deadline: Instant) -> OperationId {
        stream.set_nonblocking(true).unwrap();
        let id = OperationId(next_id().unwrap());
        sockets.connects.insert(
            id,
            Connect {
                attempt_deadline: None,
                remaining: VecDeque::new(),
                deadline: Some(deadline),
                listener: None,
                stream: Some(stream),
                callback: Some(callback()),
                outcome: None,
            },
        );
        id
    }

    #[test]
    fn address_admission_validates_every_candidate_before_opening_a_socket() {
        let mut sockets = Sockets::default();
        assert_eq!(
            sockets.connect_addresses(&[], 80, callback()),
            Err(Error::InvalidRange)
        );
        assert_eq!(
            sockets.connect_addresses(&["127.0.0.1"; 17], 80, callback()),
            Err(Error::Limit)
        );
        assert_eq!(
            sockets.connect_addresses(&["127.0.0.1", "not an address"], 80, callback()),
            Err(Error::InvalidAddress)
        );
        assert_eq!(
            sockets.connect_addresses(&["127.0.0.1"], 0, callback()),
            Err(Error::InvalidRange)
        );
        assert_eq!(sockets.resource_count(), 0);
        assert!(sockets.connects.is_empty());
    }

    #[test]
    fn address_slice_expiry_falls_back_without_extending_overall_deadline() {
        let (mut first_peer, stream) = pair();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = match listener.local_addr().unwrap() {
            std::net::SocketAddr::V4(address) => address,
            _ => unreachable!(),
        };
        let mut sockets = Sockets::default();
        let heap = ManagedHeap::default();
        let now = Instant::now();
        let deadline = now + CONNECT_TIMEOUT;
        let id = staged_connect(&mut sockets, stream, deadline);
        let op = sockets.connects.get_mut(&id).unwrap();
        op.attempt_deadline = Some(now);
        op.remaining.push_back(address);
        let _ = sockets.poll_connect_at(now).unwrap();
        assert_eq!(first_peer.read(&mut [0; 1]).unwrap(), 0);
        assert_eq!(sockets.connects[&id].deadline, Some(deadline));
        assert!(sockets.resource_count() <= 1);
        if sockets.connects[&id].callback.is_some() {
            deliver(&mut sockets, &heap);
        }
        let connected = accepted(&mut sockets, &heap, id);
        assert!(sockets.local_port(connected).is_ok());
        sockets.close(connected);
    }

    #[test]
    fn overall_expiry_does_not_start_remaining_address() {
        let (_peer, stream) = pair();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = match listener.local_addr().unwrap() {
            std::net::SocketAddr::V4(address) => address,
            _ => unreachable!(),
        };
        let mut sockets = Sockets::default();
        let now = Instant::now();
        let id = staged_connect(&mut sockets, stream, now);
        sockets
            .connects
            .get_mut(&id)
            .unwrap()
            .remaining
            .push_back(address);
        assert!(sockets.poll_connect_at(now).unwrap().is_some());
        assert_eq!(sockets.connects[&id].outcome, Some(Err(Error::TimedOut)));
        assert!(sockets.connects[&id].remaining.is_empty());
        assert_eq!(
            listener.accept().unwrap_err().kind(),
            io::ErrorKind::WouldBlock
        );
        assert_eq!(sockets.resource_count(), 0);
    }

    #[test]
    fn duplicate_addresses_are_skipped_without_reserving_more_sockets() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = i32::from(listener.local_addr().unwrap().port());
        let mut sockets = Sockets::default();
        sockets.budget.sockets = 1;
        let heap = ManagedHeap::default();
        let id = sockets
            .connect_addresses(&["127.0.0.1", "127.0.0.1"], port, callback())
            .unwrap();
        assert!(sockets.connects[&id].remaining.is_empty());
        deliver(&mut sockets, &heap);
        let connected = accepted(&mut sockets, &heap, id);
        sockets.close(connected);
        assert_eq!(sockets.resource_count(), 0);
    }

    #[test]
    fn expired_connect_drops_native_owner_and_retains_result_until_consumed() {
        let (mut peer, stream) = pair();
        let mut sockets = Sockets::default();
        sockets.budget.sockets = 1;
        sockets.budget.operations = 1;
        let heap = ManagedHeap::default();
        let deadline = Instant::now();
        let id = staged_connect(&mut sockets, stream, deadline);
        assert_eq!(sockets.resource_count(), 1);
        assert!(
            sockets
                .invoke(
                    Operation::ConnectResult,
                    &[Value::Int64(id.0 as i64)],
                    &heap
                )
                .is_err()
        );
        // Even a native connection already ready loses if not observed before expiry.
        assert!(sockets.poll_connect_at(deadline).unwrap().is_some());
        assert_eq!(sockets.connects[&id].outcome, Some(Err(Error::TimedOut)));
        assert_eq!(sockets.resource_count(), 0);
        assert_eq!(
            peer.read(&mut [0; 1]).unwrap(),
            0,
            "timeout must close transport"
        );
        assert!(!sockets.pending());
        assert!(
            sockets
                .poll_connect_at(deadline + Duration::from_secs(1))
                .unwrap()
                .is_none()
        );
        assert_eq!(
            sockets.connect("127.0.0.1", 1, callback()),
            Err(Error::Limit)
        );
        assert_eq!(
            connect_result(&mut sockets, &heap, id),
            payload(Err(Error::TimedOut))
        );
        assert!(
            sockets
                .invoke(
                    Operation::ConnectResult,
                    &[Value::Int64(id.0 as i64)],
                    &heap
                )
                .is_err()
        );
        // Both capacity reservations are reusable after consuming the outcome.
        assert!(sockets.connect("127.0.0.1", 1, callback()).is_ok());
    }

    #[test]
    fn connect_committed_before_expiry_survives_delayed_consumption() {
        let (_peer, stream) = pair();
        let mut sockets = Sockets::default();
        let heap = ManagedHeap::default();
        let deadline = Instant::now() + CONNECT_TIMEOUT;
        let id = staged_connect(&mut sockets, stream, deadline);
        assert!(
            sockets
                .poll_connect_at(deadline - Duration::from_nanos(1))
                .unwrap()
                .is_some()
        );
        assert!(sockets.poll_connect_at(deadline).unwrap().is_none());
        let socket = accepted(&mut sockets, &heap, id);
        assert!(sockets.local_port(socket).is_ok());
        sockets.close(socket);
        assert_eq!(sockets.resource_count(), 0);
    }

    #[test]
    fn connect_deadline_does_not_expire_accept_or_committed_failure() {
        let mut sockets = Sockets::default();
        let listener = sockets.listen("127.0.0.1", 0, 8).unwrap();
        let accept = sockets.accept(listener, callback()).unwrap();
        let now = Instant::now();
        assert!(
            sockets
                .poll_connect_at(now + CONNECT_TIMEOUT * 2)
                .unwrap()
                .is_none()
        );
        assert!(sockets.connects[&accept].outcome.is_none());
        let id = OperationId(next_id().unwrap());
        sockets.connects.insert(
            id,
            Connect {
                attempt_deadline: None,
                remaining: VecDeque::new(),
                deadline: Some(now),
                listener: None,
                stream: None,
                callback: Some(callback()),
                outcome: Some(Err(Error::ConnectionRefused)),
            },
        );
        assert!(
            sockets
                .poll_connect_at(now + CONNECT_TIMEOUT)
                .unwrap()
                .is_some()
        );
        assert_eq!(
            sockets.connects[&id].outcome,
            Some(Err(Error::ConnectionRefused))
        );
        sockets.close(listener);
    }

    #[test]
    fn listener_accept_echo_and_close_leave_accepted_connection_independent() {
        let mut sockets = Sockets::default();
        let mut heap = ManagedHeap::default();
        let listener = sockets.listen("127.0.0.1", 0, 8).unwrap();
        let port = sockets.local_port(listener).unwrap();
        assert!(port > 0);
        let id = sockets.accept(listener, callback()).unwrap();
        assert!(sockets.poll(&heap).unwrap().is_none());
        assert_eq!(sockets.accept(listener, callback()), Err(Error::Busy));
        let mut peer = TcpStream::connect(("127.0.0.1", port as u16)).unwrap();
        peer.set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();
        deliver(&mut sockets, &heap);
        sockets.close(listener); // committed accept must survive even before consumption
        let connected = accepted(&mut sockets, &heap, id);
        assert_eq!(sockets.local_port(connected).unwrap(), port);
        assert!(
            sockets
                .invoke(
                    Operation::ConnectResult,
                    &[Value::Int64(id.0 as i64)],
                    &heap
                )
                .is_err()
        );
        let buffer = array(&mut heap);
        peer.write_all(&[72, 105]).unwrap();
        let read = sockets
            .receive(connected, &heap, buffer.clone(), 1, 2, callback())
            .unwrap();
        deliver(&mut sockets, &heap);
        let count = sockets.take_result(read).unwrap().unwrap().unwrap();
        let send = sockets
            .send(connected, &heap, buffer, 1, count as i32, callback())
            .unwrap();
        deliver(&mut sockets, &heap);
        let sent = sockets.take_result(send).unwrap().unwrap().unwrap();
        let mut echo = vec![0; sent];
        peer.read_exact(&mut echo).unwrap();
        assert_eq!(echo, [72, 105][..sent]);
        sockets.close(connected);
        assert_eq!(peer.read(&mut [0]).unwrap(), 0);
    }
    #[test]
    fn listener_validation_wrong_kind_and_address_in_use_are_distinct() {
        let mut sockets = Sockets::default();
        let mut heap = ManagedHeap::default();
        for (address, port, backlog, error) in [
            ("localhost", 0, 1, Error::InvalidAddress),
            ("127.0.0.1", -1, 1, Error::InvalidRange),
            ("127.0.0.1", 65536, 1, Error::InvalidRange),
            ("127.0.0.1", 0, 0, Error::InvalidRange),
            ("127.0.0.1", 0, 129, Error::InvalidRange),
        ] {
            assert_eq!(sockets.listen(address, port, backlog), Err(error));
        }
        let listener = sockets.listen("127.0.0.1", 0, 1).unwrap();
        let port = sockets.local_port(listener).unwrap();
        assert_eq!(
            sockets.listen("127.0.0.1", port, 1),
            Err(Error::AddressInUse)
        );
        assert_eq!(
            sockets.accept(listener, Value::Int32(0)),
            Err(Error::InvalidCallback)
        );
        let buffer = array(&mut heap);
        assert_eq!(
            sockets.receive(listener, &heap, buffer.clone(), 0, 1, callback()),
            Err(Error::InvalidOperation)
        );
        assert_eq!(
            sockets.send(listener, &heap, buffer, 0, 1, callback()),
            Err(Error::InvalidOperation)
        );
        let (_peer, stream) = pair();
        let connected = sockets.adopt(stream).unwrap();
        assert_eq!(
            sockets.accept(connected, callback()),
            Err(Error::InvalidOperation)
        );
        let mut other = Sockets::default();
        assert_eq!(other.accept(listener, callback()), Err(Error::Closed));
        sockets.close(listener);
        sockets.close(listener);
        assert_eq!(sockets.local_port(listener), Err(Error::Closed));
        assert_eq!(sockets.accept(listener, callback()), Err(Error::Closed));
    }
    #[test]
    fn pending_accept_reserves_socket_capacity_and_close_settles_once() {
        let mut sockets = Sockets {
            budget: Budget {
                sockets: 2,
                operations: 1,
                bytes: 4,
            },
            ..Default::default()
        };
        let heap = ManagedHeap::default();
        let listener = sockets.listen("127.0.0.1", 0, 1).unwrap();
        let id = sockets.accept(listener, callback()).unwrap();
        assert_eq!(sockets.resource_count(), 2);
        assert_eq!(sockets.listen("127.0.0.1", 0, 1), Err(Error::Limit));
        assert_eq!(
            sockets.connect("127.0.0.1", 1, callback()),
            Err(Error::Limit)
        );
        sockets.close(listener);
        assert_eq!(sockets.resource_count(), 0);
        deliver(&mut sockets, &heap);
        assert_eq!(
            connect_result(&mut sockets, &heap, id),
            payload(Err(Error::Closed))
        );
        assert!(sockets.poll(&heap).unwrap().is_none());
        let listener = sockets.listen("127.0.0.1", 0, 1).unwrap();
        let port = sockets.local_port(listener).unwrap();
        let id = sockets.accept(listener, callback()).unwrap();
        let mut peer = TcpStream::connect(("127.0.0.1", port as u16)).unwrap();
        peer.set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();
        deliver(&mut sockets, &heap);
        assert_eq!(sockets.resource_count(), 2);
        assert_eq!(sockets.accept(listener, callback()), Err(Error::Limit));
        let _ = id; // result intentionally unconsumed at invocation teardown
        drop(sockets);
        assert_eq!(peer.read(&mut [0]).unwrap(), 0);
    }

    #[test]
    fn send_snapshots_only_range_and_releases_guest_source_before_completion() {
        let (mut peer, stream) = pair();
        let mut sockets = Sockets::default();
        let socket = sockets.adopt(stream).unwrap();
        let mut heap = ManagedHeap::default();
        let source = array(&mut heap);
        let id = sockets
            .send(socket, &heap, source.clone(), 1, 2, callback())
            .unwrap();
        let Value::ObjectReference(object) = &source else {
            panic!()
        };
        object
            .reference
            .write(Value::Array {
                element: Type::Byte,
                elements: vec![Value::Byte(42); 4],
            })
            .unwrap();
        let mut roots = vec![];
        sockets.trace_roots(&mut roots);
        heap.collect(roots, crate::CollectionReason::AllocationPressure)
            .unwrap();
        assert_eq!(heap.statistics().live_objects, 0);
        deliver(&mut sockets, &heap);
        assert_eq!(sockets.take_result(id).unwrap(), Some(Ok(2)));
        assert_eq!(sockets.reserved, 0);
        let mut output = [0; 2];
        peer.read_exact(&mut output).unwrap();
        assert_eq!(output, [9, 9]);
        assert!(sockets.poll(&heap).unwrap().is_none());
        assert_eq!(sockets.take_result(id), Err(Error::UnknownOperation));
        sockets.close(socket);
        assert_eq!(peer.read(&mut [0]).unwrap(), 0);
    }

    #[test]
    fn one_send_and_one_receive_can_wait_on_same_connection() {
        let (mut peer, stream) = pair();
        let mut sockets = Sockets::default();
        let socket = sockets.adopt(stream).unwrap();
        let mut heap = ManagedHeap::default();
        let destination = array(&mut heap);
        let source = array(&mut heap);
        let read = sockets
            .receive(socket, &heap, destination.clone(), 0, 1, callback())
            .unwrap();
        let write = sockets
            .send(socket, &heap, source.clone(), 1, 2, callback())
            .unwrap();
        assert_eq!(
            sockets.send(socket, &heap, source, 0, 0, callback()),
            Err(Error::Busy)
        );
        assert_eq!(
            sockets.receive(socket, &heap, destination.clone(), 0, 0, callback()),
            Err(Error::Busy)
        );
        deliver(&mut sockets, &heap);
        assert_eq!(sockets.take_result(write).unwrap(), Some(Ok(2)));
        assert_eq!(sockets.take_result(read).unwrap(), None);
        let mut output = [0; 2];
        peer.read_exact(&mut output).unwrap();
        assert_eq!(output, [9, 9]);
        peer.write_all(b"R").unwrap();
        deliver(&mut sockets, &heap);
        assert_eq!(sockets.take_result(read).unwrap(), Some(Ok(1)));
        assert_eq!(bytes(&heap, &destination)[0], Value::Byte(b'R'));
    }

    #[test]
    fn send_admission_empty_close_and_cancel_preserve_shared_accounting() {
        let (mut peer, stream) = pair();
        let mut sockets = Sockets::default();
        let socket = sockets.adopt(stream).unwrap();
        let mut heap = ManagedHeap::default();
        let source = array(&mut heap);
        for (offset, count) in [(-1, 1), (0, -1), (5, 0), (3, 2)] {
            assert_eq!(
                sockets.send(socket, &heap, source.clone(), offset, count, callback()),
                Err(Error::InvalidRange)
            );
        }
        assert_eq!(
            sockets.send(
                socket,
                &ManagedHeap::default(),
                source.clone(),
                0,
                1,
                callback()
            ),
            Err(Error::InvalidBuffer)
        );
        assert_eq!(
            sockets.send(socket, &heap, source.clone(), 0, 1, Value::Void),
            Err(Error::InvalidCallback)
        );
        let mut foreign = Sockets::default();
        assert_eq!(
            foreign.send(socket, &heap, source.clone(), 0, 1, callback()),
            Err(Error::Closed)
        );
        sockets.budget.bytes = 4;
        let read = sockets
            .receive(socket, &heap, source.clone(), 0, 3, callback())
            .unwrap();
        assert_eq!(
            sockets.send(socket, &heap, source.clone(), 0, 2, callback()),
            Err(Error::Limit)
        );
        let write = sockets
            .send(socket, &heap, source.clone(), 0, 1, callback())
            .unwrap();
        assert_eq!(sockets.reserved, 4);
        assert!(sockets.cancel(write).unwrap());
        assert_eq!(sockets.reserved, 3);
        deliver(&mut sockets, &heap);
        assert_eq!(
            sockets.take_result(write).unwrap(),
            Some(Err(Error::Cancelled))
        );
        let empty = sockets
            .send(socket, &heap, source.clone(), 4, 0, callback())
            .unwrap();
        deliver(&mut sockets, &heap);
        assert_eq!(sockets.take_result(empty).unwrap(), Some(Ok(0)));
        let closed = sockets
            .send(socket, &heap, source.clone(), 0, 1, callback())
            .unwrap();
        sockets.close(socket);
        assert_eq!(sockets.reserved, 0);
        deliver(&mut sockets, &heap);
        deliver(&mut sockets, &heap);
        assert_eq!(sockets.take_result(read).unwrap(), Some(Err(Error::Closed)));
        assert_eq!(
            sockets.take_result(closed).unwrap(),
            Some(Err(Error::Closed))
        );
        assert_eq!(
            sockets.send(socket, &heap, source, 0, 0, callback()),
            Err(Error::Closed)
        );
        assert_eq!(
            peer.read(&mut [0]).unwrap(),
            0,
            "cancelled/closed writes must not reach peer"
        );
    }

    #[test]
    fn tcp_short_sends_and_backpressure_keep_pending_bytes_until_progress() {
        let (mut peer, stream) = pair();
        socket2::SockRef::from(&stream)
            .set_send_buffer_size(4096)
            .unwrap();
        peer.set_nonblocking(true).unwrap();
        let mut sockets = Sockets::default();
        let socket = sockets.adopt(stream).unwrap();
        let mut heap = ManagedHeap::default();
        let id = heap
            .allocate(Value::Array {
                element: Type::Byte,
                elements: vec![Value::Byte(9); 65536],
            })
            .unwrap();
        let source = Value::ObjectReference(crate::value::ObjectReference {
            reference: heap.address(id).unwrap(),
            view: None,
        });
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut total_sent = 0;
        let mut saw_short = false;
        let blocked = loop {
            assert!(
                total_sent < 16 * 1024 * 1024 && Instant::now() < deadline,
                "TCP backpressure watchdog"
            );
            let id = sockets
                .send(socket, &heap, source.clone(), 0, 65536, callback())
                .unwrap();
            if sockets.poll(&heap).unwrap().is_none() {
                break id;
            }
            let count = sockets.take_result(id).unwrap().unwrap().unwrap();
            assert!(count > 0 && count <= 65536);
            saw_short |= count < 65536;
            total_sent += count;
        };
        assert_eq!(sockets.reserved, 65536);
        assert_eq!(sockets.take_result(blocked).unwrap(), None);
        assert_eq!(
            sockets.send(socket, &heap, source.clone(), 0, 1, callback()),
            Err(Error::Busy)
        );
        let mut received = 0;
        let mut chunk = [0; 32768];
        let count = loop {
            assert!(Instant::now() < deadline, "TCP send progress watchdog");
            match peer.read(&mut chunk) {
                Ok(count) if count > 0 => {
                    assert!(chunk[..count].iter().all(|byte| *byte == 9));
                    received += count;
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                other => panic!("Unexpected peer read: {other:?}"),
            }
            if sockets.poll(&heap).unwrap().is_some() {
                break sockets.take_result(blocked).unwrap().unwrap().unwrap();
            }
            std::thread::sleep(Duration::from_millis(1));
        };
        assert!(count > 0 && count <= 65536);
        assert!(
            saw_short || count < 65536,
            "small send buffer should exercise a short send"
        );
        total_sent += count;
        assert_eq!(sockets.reserved, 0);
        assert!(sockets.poll(&heap).unwrap().is_none());
        sockets.close(socket);
        peer.set_nonblocking(false).unwrap();
        loop {
            let count = peer.read(&mut chunk).unwrap();
            if count == 0 {
                break;
            }
            assert!(chunk[..count].iter().all(|byte| *byte == 9));
            received += count;
        }
        assert_eq!(
            received, total_sent,
            "only reported prefixes are sent, exactly once"
        );
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
                    attempt_deadline: None,
                    remaining: VecDeque::new(),
                    deadline: None,
                    listener: None,
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
                    Operation::TransferResult,
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
