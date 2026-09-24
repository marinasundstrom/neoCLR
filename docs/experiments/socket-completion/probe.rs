//! Reduced receive-completion boundary. No guest API or production event loop.
use std::{
    cell::RefCell,
    collections::BTreeMap,
    io::{self, Read},
    net::TcpStream,
    rc::Rc,
};

type Destination = Rc<RefCell<Vec<u8>>>;

#[derive(Debug, PartialEq, Eq)]
enum Error {
    InvalidRange,
    Limit,
    Busy,
    Closed,
    Cancelled,
    Io,
}

struct Receive {
    socket: TcpStream,
    destination: Destination,
    offset: usize,
    bytes: Vec<u8>,
}

struct Receives {
    pending: BTreeMap<u64, Receive>,
    next: u64,
    bytes: usize,
    max_operations: usize,
    max_bytes: usize,
}

impl Receives {
    fn new(max_operations: usize, max_bytes: usize) -> Self {
        Self {
            pending: BTreeMap::new(),
            next: 0,
            bytes: 0,
            max_operations,
            max_bytes,
        }
    }

    // One owned stream per pending operation in this reduced boundary. A real
    // socket registry must enforce directional exclusion by stable socket identity.
    fn start(
        &mut self,
        socket: TcpStream,
        destination: Destination,
        offset: usize,
        count: usize,
    ) -> Result<u64, Error> {
        if offset
            .checked_add(count)
            .filter(|end| *end <= destination.borrow().len())
            .is_none()
        {
            return Err(Error::InvalidRange);
        }
        if self
            .pending
            .values()
            .any(|op| Rc::ptr_eq(&op.destination, &destination))
        {
            return Err(Error::Busy);
        }
        let bytes = self
            .bytes
            .checked_add(count)
            .filter(|n| *n <= self.max_bytes)
            .ok_or(Error::Limit)?;
        if self.pending.len() >= self.max_operations {
            return Err(Error::Limit);
        }
        let next = self.next.checked_add(1).ok_or(Error::Limit)?;
        socket.set_nonblocking(true).map_err(|_| Error::Io)?;
        let mut buffer = Vec::new();
        buffer.try_reserve_exact(count).map_err(|_| Error::Limit)?;
        buffer.resize(count, 0);
        let id = self.next;
        self.next = next;
        self.bytes = bytes;
        self.pending.insert(
            id,
            Receive {
                socket,
                destination,
                offset,
                bytes: buffer,
            },
        );
        Ok(id)
    }

    fn remove(&mut self, id: u64) -> Option<Receive> {
        let op = self.pending.remove(&id)?;
        self.bytes -= op.bytes.len();
        Some(op)
    }

    // All progress and cancellation occur on the same owner thread. No callback
    // can run between read and commit. Producer-thread cancellation is not modeled.
    fn poll(&mut self, id: u64) -> Option<Result<usize, Error>> {
        let op = self.pending.get_mut(&id)?;
        let outcome = if op.bytes.is_empty() {
            Ok(0)
        } else {
            op.socket.read(&mut op.bytes)
        };
        let result = match outcome {
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                ) =>
            {
                return None;
            }
            Err(_) => Err(Error::Io),
            Ok(count) => Ok(count),
        };
        let op = self.remove(id).unwrap();
        if let Ok(count) = result {
            // Recheck the destination in this Vec model; real managed arrays have
            // fixed length. Native I/O never receives a pointer into this storage.
            let mut destination = op.destination.borrow_mut();
            let Some(end) = op
                .offset
                .checked_add(count)
                .filter(|end| *end <= destination.len())
            else {
                return Some(Err(Error::InvalidRange));
            };
            destination[op.offset..end].copy_from_slice(&op.bytes[..count]);
        }
        Some(result)
    }

    fn terminate(&mut self, id: u64, reason: Error) -> Option<Result<usize, Error>> {
        self.remove(id).map(|_| Err(reason))
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
        let sender = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        sender
            .set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        (sender, listener.accept().unwrap().0)
    }
    fn destination() -> Destination {
        Rc::new(RefCell::new(vec![9; 6]))
    }
    fn finish(receives: &mut Receives, id: u64) -> Result<usize, Error> {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(result) = receives.poll(id) {
                return result;
            }
            assert!(Instant::now() < deadline, "completion watchdog");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn pending_retains_destination_and_commits_only_the_received_range_once() {
        let (mut sender, receiver) = pair();
        let mut receives = Receives::new(2, 8);
        let buffer = destination();
        let weak = Rc::downgrade(&buffer);
        let id = receives.start(receiver, buffer.clone(), 2, 3).unwrap();
        assert_eq!(receives.poll(id), None);
        assert_eq!(*buffer.borrow(), [9; 6]);
        drop(buffer);
        assert!(weak.upgrade().is_some());
        sender.write_all(b"A").unwrap();
        let retained = weak.upgrade().unwrap();
        assert_eq!(finish(&mut receives, id), Ok(1));
        assert_eq!(*retained.borrow(), [9, 9, b'A', 9, 9, 9]);
        assert_eq!(receives.poll(id), None);
        assert_eq!(receives.terminate(id, Error::Cancelled), None);
        assert_eq!(receives.bytes, 0);
        drop(retained);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn cancellation_and_close_before_delivery_never_modify_destination() {
        for reason in [Error::Cancelled, Error::Closed] {
            let (mut sender, receiver) = pair();
            let buffer = destination();
            let mut receives = Receives::new(1, 8);
            let id = receives.start(receiver, buffer.clone(), 0, 4).unwrap();
            sender.write_all(b"data").unwrap();
            let expected = if reason == Error::Cancelled {
                Error::Cancelled
            } else {
                Error::Closed
            };
            assert_eq!(receives.terminate(id, reason), Some(Err(expected)));
            assert_eq!(*buffer.borrow(), [9; 6]);
            assert_eq!(receives.poll(id), None);
            assert_eq!(receives.bytes, 0);
        }
    }

    #[test]
    fn idle_receive_does_not_block_a_later_ready_receive() {
        let (_idle_sender, idle_receiver) = pair();
        let (mut sender, receiver) = pair();
        let mut receives = Receives::new(2, 8);
        let idle = receives.start(idle_receiver, destination(), 0, 4).unwrap();
        let ready = receives.start(receiver, destination(), 0, 4).unwrap();
        assert_eq!(receives.poll(idle), None);
        sender.write_all(b"ok").unwrap();
        assert_eq!(finish(&mut receives, ready), Ok(2));
        assert!(receives.pending.contains_key(&idle));
    }

    #[test]
    fn validation_and_quotas_are_transactional_and_ids_are_not_reused() {
        let (_sender, receiver) = pair();
        let buffer = destination();
        let mut receives = Receives::new(1, 3);
        assert_eq!(
            receives.start(receiver.try_clone().unwrap(), buffer.clone(), usize::MAX, 2),
            Err(Error::InvalidRange)
        );
        assert_eq!(
            receives.start(receiver.try_clone().unwrap(), buffer.clone(), 0, 4),
            Err(Error::Limit)
        );
        assert_eq!((receives.bytes, receives.next), (0, 0));
        let first = receives
            .start(receiver.try_clone().unwrap(), buffer.clone(), 0, 3)
            .unwrap();
        assert_eq!(
            receives.start(receiver.try_clone().unwrap(), buffer, 0, 1),
            Err(Error::Busy)
        );
        assert_eq!(
            receives.start(receiver.try_clone().unwrap(), destination(), 0, 0),
            Err(Error::Limit)
        );
        assert_eq!(
            receives.terminate(first, Error::Cancelled),
            Some(Err(Error::Cancelled))
        );
        let second = receives.start(receiver, destination(), 0, 3).unwrap();
        assert!(second > first);
        assert_eq!(receives.terminate(first, Error::Closed), None);
        assert!(receives.pending.contains_key(&second));
    }

    #[test]
    fn empty_receive_and_peer_eof_complete_without_writing() {
        for count in [0, 4] {
            let (sender, receiver) = pair();
            let buffer = destination();
            let mut receives = Receives::new(1, 4);
            let id = receives.start(receiver, buffer.clone(), 0, count).unwrap();
            if count != 0 {
                drop(sender);
            }
            assert_eq!(finish(&mut receives, id), Ok(0));
            assert_eq!(*buffer.borrow(), [9; 6]);
        }
    }

    #[test]
    fn dropping_registry_releases_destination_and_socket() {
        let (mut sender, receiver) = pair();
        sender
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let buffer = destination();
        let weak = Rc::downgrade(&buffer);
        let mut receives = Receives::new(1, 4);
        receives.start(receiver, buffer, 0, 4).unwrap();
        drop(receives);
        assert!(weak.upgrade().is_none());
        assert_eq!(sender.read(&mut [0; 1]).unwrap(), 0);
    }
}
