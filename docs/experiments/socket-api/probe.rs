//! Test-only transport contract. No guest API, scheduler, or production backend.
#![cfg(all(test, unix))]

use socket2::{Domain, Protocol, Socket as NativeSocket, Type};
use std::{
    io::{self, Read, Write},
    net::{Shutdown, SocketAddr},
    task::Poll,
};

const MAX_TRANSFER: usize = 64 * 1024;

#[derive(Debug, PartialEq, Eq)]
enum Error {
    Closed,
    WrongState,
    InvalidRange,
    LimitExceeded,
    AddressInUse,
    AccessDenied,
    ConnectionRefused,
    ConnectionReset,
    WriteShutdown,
    Io,
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::AddrInUse => Self::AddressInUse,
            io::ErrorKind::PermissionDenied => Self::AccessDenied,
            io::ErrorKind::ConnectionRefused => Self::ConnectionRefused,
            io::ErrorKind::ConnectionReset | io::ErrorKind::BrokenPipe => Self::ConnectionReset,
            _ => Self::Io,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum State {
    Created,
    Bound,
    Listening,
    Connecting,
    Connected,
    Closed,
}

struct Socket {
    native: Option<NativeSocket>,
    state: State,
    write_shutdown: bool,
}

// Pending is a backend observation, never a proposed guest Result error.
fn attempt<T>(result: io::Result<T>) -> Poll<Result<T, Error>> {
    match result {
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
            ) =>
        {
            Poll::Pending
        }
        result => Poll::Ready(result.map_err(Error::from)),
    }
}

impl Socket {
    fn tcp_v4() -> Result<Self, Error> {
        let native = NativeSocket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
        native.set_nonblocking(true)?;
        Ok(Self {
            native: Some(native),
            state: State::Created,
            write_shutdown: false,
        })
    }

    fn require(&self, state: State) -> Result<&NativeSocket, Error> {
        let native = self.native.as_ref().ok_or(Error::Closed)?;
        if self.state != state {
            return Err(Error::WrongState);
        }
        Ok(native)
    }

    fn bind(&mut self, endpoint: SocketAddr) -> Result<(), Error> {
        self.require(State::Created)?.bind(&endpoint.into())?;
        self.state = State::Bound;
        Ok(())
    }

    fn listen(&mut self, backlog: i32) -> Result<(), Error> {
        let native = self.require(State::Bound)?;
        if backlog <= 0 {
            return Err(Error::InvalidRange);
        }
        native.listen(backlog)?;
        self.state = State::Listening;
        Ok(())
    }

    fn local_endpoint(&self) -> Result<SocketAddr, Error> {
        self.native
            .as_ref()
            .ok_or(Error::Closed)?
            .local_addr()?
            .as_socket()
            .ok_or(Error::Io)
    }

    fn remote_endpoint(&self) -> Result<SocketAddr, Error> {
        self.require(State::Connected)?
            .peer_addr()?
            .as_socket()
            .ok_or(Error::Io)
    }

    fn accept(&self) -> Poll<Result<Self, Error>> {
        let native = match self.require(State::Listening) {
            Ok(native) => native,
            Err(error) => return Poll::Ready(Err(error)),
        };
        attempt(native.accept()).map(|result| {
            let (native, _) = result?;
            // Nonblocking inheritance differs across operating systems.
            native.set_nonblocking(true)?;
            Ok(Self {
                native: Some(native),
                state: State::Connected,
                write_shutdown: false,
            })
        })
    }

    fn connect(&mut self, endpoint: SocketAddr) -> Poll<Result<(), Error>> {
        let native = match self.require(State::Created) {
            Ok(native) => native,
            Err(error) => return Poll::Ready(Err(error)),
        };
        match native.connect(&endpoint.into()) {
            Ok(()) => {
                self.state = State::Connected;
                Poll::Ready(Ok(()))
            }
            Err(error)
                if error.raw_os_error() == Some(libc::EINPROGRESS)
                    || error.kind() == io::ErrorKind::WouldBlock =>
            {
                self.state = State::Connecting;
                Poll::Pending
            }
            Err(error) => {
                self.close();
                Poll::Ready(Err(error.into()))
            }
        }
    }

    fn finish_connect(&mut self) -> Poll<Result<(), Error>> {
        let result = (|| {
            let native = self.require(State::Connecting)?;
            if let Some(error) = native.take_error()? {
                return Err(error.into());
            }
            match native.peer_addr() {
                Ok(_) => Ok(true),
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::NotConnected | io::ErrorKind::WouldBlock
                    ) =>
                {
                    Ok(false)
                }
                Err(error) => Err(error.into()),
            }
        })();
        match result {
            Ok(true) => {
                self.state = State::Connected;
                Poll::Ready(Ok(()))
            }
            Ok(false) => Poll::Pending,
            Err(error) => {
                // A failed connection is terminal; retries use a new socket.
                if self.state == State::Connecting {
                    self.close();
                }
                Poll::Ready(Err(error))
            }
        }
    }

    fn receive(&self, bytes: &mut [u8]) -> Poll<Result<usize, Error>> {
        let mut native = match self.require(State::Connected) {
            Ok(native) => native,
            Err(error) => return Poll::Ready(Err(error)),
        };
        if bytes.len() > MAX_TRANSFER {
            return Poll::Ready(Err(Error::LimitExceeded));
        }
        if bytes.is_empty() {
            return Poll::Ready(Ok(0));
        }
        attempt(native.read(bytes))
    }

    fn send(&self, bytes: &[u8]) -> Poll<Result<usize, Error>> {
        let mut native = match self.require(State::Connected) {
            Ok(native) => native,
            Err(error) => return Poll::Ready(Err(error)),
        };
        if self.write_shutdown {
            return Poll::Ready(Err(Error::WriteShutdown));
        }
        if bytes.len() > MAX_TRANSFER {
            return Poll::Ready(Err(Error::LimitExceeded));
        }
        if bytes.is_empty() {
            return Poll::Ready(Ok(0));
        }
        attempt(native.write(bytes))
    }

    fn shutdown_send(&mut self) -> Result<(), Error> {
        let native = self.require(State::Connected)?;
        if !self.write_shutdown {
            native.shutdown(Shutdown::Write)?;
            self.write_shutdown = true;
        }
        Ok(())
    }

    fn close(&mut self) {
        self.native.take();
        self.state = State::Closed;
    }
}

// A test driver, NOT a proposed runtime polling loop or timeout API.
fn complete<T>(mut operation: impl FnMut() -> Poll<Result<T, Error>>) -> Result<T, Error> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if let Poll::Ready(result) = operation() {
            return result;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "socket test watchdog expired"
        );
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

fn listener() -> Socket {
    let mut socket = Socket::tcp_v4().unwrap();
    socket.bind("127.0.0.1:0".parse().unwrap()).unwrap();
    socket.listen(4).unwrap();
    socket
}

fn pair() -> (Socket, Socket) {
    let listener = listener();
    let mut client = Socket::tcp_v4().unwrap();
    match client.connect(listener.local_endpoint().unwrap()) {
        Poll::Ready(result) => result.unwrap(),
        Poll::Pending => complete(|| client.finish_connect()).unwrap(),
    }
    let server = complete(|| listener.accept()).unwrap();
    (client, server)
}

#[test]
fn echo_short_reads_endpoints_and_half_close() {
    let (mut client, server) = pair();
    assert_eq!(client.remote_endpoint(), server.local_endpoint());
    assert_eq!(server.remote_endpoint(), client.local_endpoint());
    let payload = "Hello, e\u{301} 🦀".as_bytes();
    let mut sent = 0;
    while sent < payload.len() {
        let count = complete(|| client.send(&payload[sent..])).unwrap();
        assert!(count > 0);
        sent += count;
    }
    client.shutdown_send().unwrap();
    client.shutdown_send().unwrap();
    assert_eq!(client.send(b"x"), Poll::Ready(Err(Error::WriteShutdown)));
    let mut received = Vec::new();
    let mut chunk = [0; 2];
    loop {
        let count = complete(|| server.receive(&mut chunk)).unwrap();
        if count == 0 {
            break;
        }
        received.extend_from_slice(&chunk[..count]);
    }
    assert_eq!(received, payload);
    // Peer EOF does not close the reverse direction.
    assert_eq!(complete(|| server.send(b"!")), Ok(1));
    assert_eq!(complete(|| client.receive(&mut chunk)), Ok(1));
    assert_eq!(chunk[0], b'!');
}

#[test]
fn pending_and_empty_receive_do_not_consume_or_mean_eof() {
    let (client, server) = pair();
    let mut buffer = [99; 4];
    assert_eq!(server.receive(&mut buffer), Poll::Pending);
    assert_eq!(buffer, [99; 4]);
    assert_eq!(server.receive(&mut []), Poll::Ready(Ok(0)));
    assert_eq!(client.send(&[]), Poll::Ready(Ok(0)));
    assert_eq!(complete(|| client.send(b"ok")), Ok(2));
    let mut received = Vec::new();
    while received.len() < 2 {
        let count = complete(|| server.receive(&mut buffer)).unwrap();
        assert!(count > 0);
        received.extend_from_slice(&buffer[..count]);
    }
    assert_eq!(received, b"ok");
}

#[test]
fn stalled_connection_does_not_prevent_other_socket_progress() {
    let (_idle_client, idle_server) = pair();
    let mut byte = [0];
    assert_eq!(idle_server.receive(&mut byte), Poll::Pending);
    let (client, server) = pair();
    assert_eq!(complete(|| client.send(b"x")), Ok(1));
    assert_eq!(complete(|| server.receive(&mut byte)), Ok(1));
    assert_eq!(idle_server.receive(&mut byte), Poll::Pending);
}

#[test]
fn close_pending_accept_and_receive_is_terminal_and_idempotent() {
    let mut listener = listener();
    assert!(matches!(listener.accept(), Poll::Pending));
    listener.close();
    listener.close();
    assert!(matches!(listener.accept(), Poll::Ready(Err(Error::Closed))));
    let (_client, mut server) = pair();
    assert_eq!(server.receive(&mut [0]), Poll::Pending);
    server.close();
    assert_eq!(server.receive(&mut [0]), Poll::Ready(Err(Error::Closed)));
    assert_eq!(server.send(b""), Poll::Ready(Err(Error::Closed)));
    assert_eq!(server.local_endpoint(), Err(Error::Closed));
}

#[test]
fn invalid_state_backlog_and_transfer_limits() {
    let mut socket = Socket::tcp_v4().unwrap();
    assert_eq!(socket.listen(4), Err(Error::WrongState));
    assert_eq!(socket.send(b"x"), Poll::Ready(Err(Error::WrongState)));
    socket.bind("127.0.0.1:0".parse().unwrap()).unwrap();
    assert_eq!(socket.listen(0), Err(Error::InvalidRange));
    socket.listen(4).unwrap();
    assert_eq!(socket.listen(4), Err(Error::WrongState));
    let (client, server) = pair();
    assert_eq!(
        client.send(&vec![0; MAX_TRANSFER + 1]),
        Poll::Ready(Err(Error::LimitExceeded))
    );
    assert_eq!(
        server.receive(&mut vec![0; MAX_TRANSFER + 1]),
        Poll::Ready(Err(Error::LimitExceeded))
    );
    assert_eq!(complete(|| client.send(b"x")), Ok(1));
}

#[test]
fn duplicate_bind_is_recoverable_and_close_releases_listener() {
    let mut first = listener();
    let endpoint = first.local_endpoint().unwrap();
    let mut second = Socket::tcp_v4().unwrap();
    assert_eq!(second.bind(endpoint), Err(Error::AddressInUse));
    first.close();
    second.bind(endpoint).unwrap();
    second.listen(4).unwrap();
}

#[test]
fn refused_connect_is_recoverable_and_terminal() {
    // Close an unused listener. A bound non-listening port can silently drop
    // SYNs on macOS rather than refuse them. An external process could claim
    // this port between close/connect; this is a local test, not a reservation.
    let mut reserved = listener();
    let endpoint = reserved.local_endpoint().unwrap();
    reserved.close();
    let mut client = Socket::tcp_v4().unwrap();
    let result = match client.connect(endpoint) {
        Poll::Ready(result) => result,
        Poll::Pending => complete(|| client.finish_connect()),
    };
    assert_eq!(result, Err(Error::ConnectionRefused));
    assert_eq!(client.state, State::Closed);
}

#[test]
fn dropping_connection_releases_resource_and_peer_observes_eof() {
    for _ in 0..16 {
        let (client, server) = pair();
        drop(client);
        assert_eq!(complete(|| server.receive(&mut [0])), Ok(0));
    }
}
