//! Host-owned terminal transport. Key/text decoding belongs above this byte boundary.
use std::io::{self, IsTerminal};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
#[cfg(unix)]
use std::time::Instant;

/// Independently detected process streams; terminal input support is platform-specific.
/// Terminal detection does not imply UTF-8, ANSI support, or any particular font.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ConsoleCapabilities {
    /// Standard input is connected to a terminal, independently of adapter support.
    pub input_is_terminal: bool,
    /// Standard output is connected to a terminal.
    pub output_is_terminal: bool,
    /// Standard error is connected to a terminal.
    pub error_is_terminal: bool,
    /// This host has an input adapter for the detected terminal; acquisition may still fail.
    pub terminal_input: bool,
}

/// Options apply only for the lifetime of the acquired input session.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TerminalOptions {
    /// Deliver terminal signal characters as bytes rather than generating signals.
    /// On POSIX this disables ISIG (including interrupt, quit and suspend), not just Ctrl+C.
    pub capture_interrupts: bool,
}

/// No input ready is different from a permanently closed input channel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerminalRead {
    /// No bytes currently ready; the input remains open.
    Pending,
    /// Positive number of bytes copied into the caller's buffer.
    Data(usize),
    /// Input has permanently ended.
    End,
}

/// Exclusive unbuffered transport, intended for an invocation-owned terminal session.
/// `try_read` never waits for input. A scheduler must arrange readiness notification before retrying;
/// polling this interface in a tight loop is not an asynchronous implementation.
/// Input bytes preserve the host encoding; no decoding, normalization or key inference occurs.
/// Restore modes explicitly with close (which reports errors), or best-effort on Drop.
pub trait TerminalInput: std::fmt::Debug + Send {
    /// Read into caller-owned storage. An empty destination is InvalidInput.
    fn try_read(&mut self, destination: &mut [u8]) -> io::Result<TerminalRead>;
    /// Synchronously wait for bytes, end, or timeout using caller-owned storage.
    /// None waits indefinitely; zero is equivalent to try_read. Pending means timeout.
    /// Hosts without a blocking adapter return Unsupported rather than polling.
    fn read(
        &mut self,
        destination: &mut [u8],
        timeout: Option<Duration>,
    ) -> io::Result<TerminalRead> {
        if timeout == Some(Duration::ZERO) {
            return self.try_read(destination);
        }
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "blocking terminal input unavailable",
        ))
    }
    /// Idempotent after success. On failure the session retains ownership for a retry.
    fn close(&mut self) -> io::Result<()>;
}

static STANDARD_INPUT_OWNED: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
pub(super) struct InputLease(&'static AtomicBool);
impl InputLease {
    fn acquire(owner: &'static AtomicBool) -> io::Result<Self> {
        owner
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .map(|_| Self(owner))
            .map_err(|_| io::Error::new(io::ErrorKind::WouldBlock, "console input already owned"))
    }
}
impl Drop for InputLease {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

pub(super) fn acquire_standard_input() -> io::Result<InputLease> {
    InputLease::acquire(&STANDARD_INPUT_OWNED)
}

pub(super) fn capabilities() -> ConsoleCapabilities {
    let input_is_terminal = io::stdin().is_terminal();
    ConsoleCapabilities {
        input_is_terminal,
        output_is_terminal: io::stdout().is_terminal(),
        error_is_terminal: io::stderr().is_terminal(),
        terminal_input: cfg!(unix) && input_is_terminal,
    }
}

pub(super) fn open(options: TerminalOptions) -> io::Result<Box<dyn TerminalInput>> {
    #[cfg(unix)]
    {
        let lease = acquire_standard_input()?;
        posix::Session::open(libc::STDIN_FILENO, options, lease)
            .map(|session| Box::new(session) as Box<dyn TerminalInput>)
    }
    #[cfg(not(unix))]
    {
        let _ = options;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "native terminal adapter unavailable",
        ))
    }
}

#[cfg(unix)]
pub(super) fn read_standard_byte() -> io::Result<Option<u8>> {
    let mut byte = 0;
    loop {
        // Use the same unbuffered descriptor as terminal sessions: std::io::Stdin
        // may prefetch input into a private buffer that the session cannot see.
        let count = unsafe { libc::read(libc::STDIN_FILENO, (&mut byte as *mut u8).cast(), 1) };
        if count >= 0 {
            return Ok((count == 1).then_some(byte));
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

#[cfg(not(unix))]
pub(super) fn read_standard_byte() -> io::Result<Option<u8>> {
    use std::io::Read;
    let mut byte = [0];
    loop {
        match io::stdin().lock().read(&mut byte) {
            Ok(0) => return Ok(None),
            Ok(_) => return Ok(Some(byte[0])),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
}

#[cfg(unix)]
mod posix {
    use super::*;
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};

    #[derive(Debug)]
    pub(super) struct Session {
        fd: Option<OwnedFd>,
        original: libc::termios,
        lease: Option<InputLease>,
        ended: bool,
    }
    impl Session {
        pub(super) fn open(
            fd: RawFd,
            options: TerminalOptions,
            lease: InputLease,
        ) -> io::Result<Self> {
            if unsafe { libc::isatty(fd) } != 1 {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "input is not a terminal",
                ));
            }
            let duplicate = unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 0) };
            if duplicate < 0 {
                return Err(io::Error::last_os_error());
            }
            // Own a descriptor so closing/replacing stdin cannot redirect our restoration.
            let fd = unsafe { OwnedFd::from_raw_fd(duplicate) };
            let mut original = unsafe { std::mem::zeroed::<libc::termios>() };
            if unsafe { libc::tcgetattr(fd.as_raw_fd(), &mut original) } != 0 {
                return Err(io::Error::last_os_error());
            }
            let mut mode = original;
            mode.c_lflag &= !(libc::ICANON | libc::ECHO | libc::ECHONL | libc::IEXTEN);
            if options.capture_interrupts {
                mode.c_lflag &= !libc::ISIG;
            } else {
                mode.c_lflag |= libc::ISIG;
            }
            mode.c_iflag &= !(libc::ISTRIP
                | libc::INLCR
                | libc::IGNCR
                | libc::ICRNL
                | libc::IXON
                | libc::PARMRK);
            mode.c_cc[libc::VMIN] = 0;
            mode.c_cc[libc::VTIME] = 0;
            let mut session = Self {
                fd: Some(fd),
                original,
                lease: Some(lease),
                ended: false,
            };
            if unsafe {
                libc::tcsetattr(
                    session.fd.as_ref().unwrap().as_raw_fd(),
                    libc::TCSANOW,
                    &mode,
                )
            } != 0
            {
                let error = io::Error::last_os_error();
                // Drop also attempts restoration in case the host partially applied settings.
                let _ = session.close();
                return Err(error);
            }
            Ok(session)
        }
        fn read_once(
            &mut self,
            destination: &mut [u8],
            timeout_ms: i32,
        ) -> io::Result<TerminalRead> {
            let fd = self
                .fd
                .as_ref()
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotConnected, "terminal session closed")
                })?
                .as_raw_fd();
            if destination.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "empty terminal read buffer",
                ));
            }
            if self.ended {
                return Ok(TerminalRead::End);
            }
            let mut readiness = libc::pollfd {
                fd,
                events: libc::POLLIN,
                revents: 0,
            };
            let ready = unsafe { libc::poll(&mut readiness, 1, timeout_ms) };
            if ready < 0 {
                let error = io::Error::last_os_error();
                return if error.kind() == io::ErrorKind::Interrupted {
                    Ok(TerminalRead::Pending)
                } else {
                    Err(error)
                };
            }
            if ready == 0 {
                return Ok(TerminalRead::Pending);
            }
            if readiness.revents & libc::POLLNVAL != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::NotConnected,
                    "invalid terminal descriptor",
                ));
            }
            let count =
                unsafe { libc::read(fd, destination.as_mut_ptr().cast(), destination.len()) };
            if count > 0 {
                return Ok(TerminalRead::Data(count as usize));
            }
            if count < 0 {
                let error = io::Error::last_os_error();
                if matches!(
                    error.kind(),
                    io::ErrorKind::Interrupted | io::ErrorKind::WouldBlock
                ) {
                    return Ok(TerminalRead::Pending);
                }
                // Some PTYs report EIO instead of zero after the master closes.
                if readiness.revents & libc::POLLHUP == 0 || error.raw_os_error() != Some(libc::EIO)
                {
                    return Err(error);
                }
            }
            if readiness.revents & libc::POLLHUP != 0 {
                self.ended = true;
                Ok(TerminalRead::End)
            } else if readiness.revents & libc::POLLERR != 0 {
                Err(io::Error::other("terminal input failure"))
            } else {
                // VMIN=0 permits a zero-length read even though the terminal is still open.
                Ok(TerminalRead::Pending)
            }
        }
    }
    impl TerminalInput for Session {
        fn try_read(&mut self, destination: &mut [u8]) -> io::Result<TerminalRead> {
            self.read_once(destination, 0)
        }
        fn read(
            &mut self,
            destination: &mut [u8],
            timeout: Option<Duration>,
        ) -> io::Result<TerminalRead> {
            if timeout == Some(Duration::ZERO) {
                return self.try_read(destination);
            }
            let deadline = match timeout {
                Some(duration) => Some(Instant::now().checked_add(duration).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "terminal timeout too large")
                })?),
                None => None,
            };
            loop {
                let timeout_ms = match deadline {
                    None => -1,
                    Some(deadline) => {
                        let remaining = deadline.saturating_duration_since(Instant::now());
                        // Round up: a sub-millisecond remainder must not turn into busy polling.
                        remaining
                            .as_millis()
                            .saturating_add(u128::from(remaining.subsec_nanos() % 1_000_000 != 0))
                            .min(i32::MAX as u128) as i32
                    }
                };
                match self.read_once(destination, timeout_ms)? {
                    TerminalRead::Pending => {
                        if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                            return Ok(TerminalRead::Pending);
                        }
                        // Interrupted waits and spurious readiness retain the original deadline.
                    }
                    result => return Ok(result),
                }
            }
        }
        fn close(&mut self) -> io::Result<()> {
            let Some(fd) = &self.fd else {
                return Ok(());
            };
            if unsafe { libc::tcsetattr(fd.as_raw_fd(), libc::TCSANOW, &self.original) } != 0 {
                return Err(io::Error::last_os_error());
            }
            self.fd.take();
            self.lease.take();
            Ok(())
        }
    }
    impl Drop for Session {
        fn drop(&mut self) {
            let _ = self.close();
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        fn pty() -> (OwnedFd, OwnedFd) {
            let (mut master, mut slave) = (-1, -1);
            assert_eq!(
                unsafe {
                    libc::openpty(
                        &mut master,
                        &mut slave,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                    )
                },
                0
            );
            unsafe { (OwnedFd::from_raw_fd(master), OwnedFd::from_raw_fd(slave)) }
        }
        fn lease() -> InputLease {
            InputLease::acquire(Box::leak(Box::new(AtomicBool::new(false)))).unwrap()
        }
        fn mode(fd: &OwnedFd) -> libc::termios {
            let mut mode = unsafe { std::mem::zeroed() };
            assert_eq!(unsafe { libc::tcgetattr(fd.as_raw_fd(), &mut mode) }, 0);
            mode
        }
        fn assert_mode(expected: libc::termios, actual: libc::termios) {
            // BSD marks pending canonical queue processing independently of tcsetattr.
            assert_eq!(
                (
                    expected.c_iflag,
                    expected.c_oflag,
                    expected.c_cflag,
                    expected.c_lflag & !libc::PENDIN,
                    expected.c_cc
                ),
                (
                    actual.c_iflag,
                    actual.c_oflag,
                    actual.c_cflag,
                    actual.c_lflag & !libc::PENDIN,
                    actual.c_cc
                )
            );
        }
        #[test]
        fn nonblocking_input_preserves_bytes_and_restores_modes() {
            let (master, slave) = pty();
            let original = mode(&slave);
            let mut session = Session::open(
                slave.as_raw_fd(),
                TerminalOptions {
                    capture_interrupts: true,
                },
                lease(),
            )
            .unwrap();
            let active = mode(&slave);
            assert_eq!(active.c_lflag & (libc::ISIG | libc::ICANON | libc::ECHO), 0);
            assert_eq!(active.c_oflag, original.c_oflag);
            let mut buffer = [0; 64];
            assert_eq!(
                session.try_read(&mut buffer).unwrap(),
                TerminalRead::Pending
            );
            assert_eq!(
                session.try_read(&mut []).unwrap_err().kind(),
                io::ErrorKind::InvalidInput
            );
            let bytes = b"\x03\xc3\xa9\x1b[A\r\n";
            assert_eq!(
                unsafe { libc::write(master.as_raw_fd(), bytes.as_ptr().cast(), bytes.len()) },
                bytes.len() as isize
            );
            let mut received = Vec::new();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
            while received.len() < bytes.len() {
                match session.try_read(&mut buffer).unwrap() {
                    TerminalRead::Data(count) => received.extend_from_slice(&buffer[..count]),
                    TerminalRead::Pending => {
                        assert!(std::time::Instant::now() < deadline);
                        std::thread::yield_now();
                    }
                    TerminalRead::End => panic!("early EOF"),
                }
            }
            assert_eq!(received, bytes);
            session.close().unwrap();
            session.close().unwrap();
            assert_mode(original, mode(&slave));
            assert_eq!(
                session.try_read(&mut buffer).unwrap_err().kind(),
                io::ErrorKind::NotConnected
            );
        }
        #[test]
        fn blocking_read_waits_for_input_and_reuses_destination() {
            let (master, slave) = pty();
            let original = mode(&slave);
            let mut session =
                Session::open(slave.as_raw_fd(), TerminalOptions::default(), lease()).unwrap();
            let writer = std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(25));
                let bytes = b"\xc3\xa9\x1b[A";
                assert_eq!(
                    unsafe { libc::write(master.as_raw_fd(), bytes.as_ptr().cast(), bytes.len()) },
                    bytes.len() as isize
                );
                master
            });
            let mut buffer = [0; 1];
            let mut received = [0; 5];
            for byte in &mut received {
                assert_eq!(
                    session
                        .read(&mut buffer, Some(Duration::from_secs(2)))
                        .unwrap(),
                    TerminalRead::Data(1)
                );
                *byte = buffer[0];
            }
            let _master = writer.join().unwrap();
            assert_eq!(&received, b"\xc3\xa9\x1b[A");
            session.close().unwrap();
            assert_mode(original, mode(&slave));
            assert_eq!(
                session.read(&mut buffer, None).unwrap_err().kind(),
                io::ErrorKind::NotConnected
            );
        }
        #[test]
        fn blocking_timeout_and_validation_preserve_session() {
            let (_master, slave) = pty();
            let mut session =
                Session::open(slave.as_raw_fd(), TerminalOptions::default(), lease()).unwrap();
            let start = Instant::now();
            assert_eq!(
                session
                    .read(&mut [0; 1], Some(Duration::from_millis(15)))
                    .unwrap(),
                TerminalRead::Pending
            );
            assert!(start.elapsed() >= Duration::from_millis(15));
            assert_eq!(
                session.read(&mut [0; 1], Some(Duration::ZERO)).unwrap(),
                TerminalRead::Pending
            );
            assert_eq!(
                session.read(&mut [], None).unwrap_err().kind(),
                io::ErrorKind::InvalidInput
            );
        }
        #[test]
        fn blocking_read_reports_hangup() {
            let (master, slave) = pty();
            let mut session =
                Session::open(slave.as_raw_fd(), TerminalOptions::default(), lease()).unwrap();
            drop(master);
            assert_eq!(session.read(&mut [0; 1], None).unwrap(), TerminalRead::End);
            assert_eq!(session.read(&mut [0; 1], None).unwrap(), TerminalRead::End);
        }
        #[test]
        fn default_signal_mode_and_drop_restore() {
            let (_master, slave) = pty();
            let original = mode(&slave);
            let session =
                Session::open(slave.as_raw_fd(), TerminalOptions::default(), lease()).unwrap();
            assert_ne!(mode(&slave).c_lflag & libc::ISIG, 0);
            drop(session);
            assert_mode(original, mode(&slave));
        }
        #[test]
        fn rejects_redirected_input_and_releases_failed_lease() {
            use std::fs::File;
            let file = File::open("/dev/null").unwrap();
            let owner = Box::leak(Box::new(AtomicBool::new(false)));
            let error = Session::open(
                file.as_raw_fd(),
                TerminalOptions::default(),
                InputLease::acquire(owner).unwrap(),
            )
            .unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::Unsupported);
            assert!(!owner.load(Ordering::Relaxed));
        }
        #[test]
        fn hangup_is_end_not_pending() {
            let (master, slave) = pty();
            let mut session =
                Session::open(slave.as_raw_fd(), TerminalOptions::default(), lease()).unwrap();
            drop(master);
            assert_eq!(session.try_read(&mut [0; 4]).unwrap(), TerminalRead::End);
            assert_eq!(session.try_read(&mut [0; 4]).unwrap(), TerminalRead::End);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[cfg(unix)]
    fn standard_input_lease_rejects_both_legacy_and_terminal_readers() {
        use crate::{Console, StdioConsole};
        let _lease = acquire_standard_input().unwrap();
        assert_eq!(
            StdioConsole.read_byte().unwrap_err().kind(),
            io::ErrorKind::WouldBlock
        );
        assert_eq!(
            StdioConsole
                .open_terminal(TerminalOptions::default())
                .unwrap_err()
                .kind(),
            io::ErrorKind::WouldBlock
        );
    }
    #[test]
    fn input_lease_is_exclusive_and_reusable() {
        let owner = Box::leak(Box::new(AtomicBool::new(false)));
        let first = InputLease::acquire(owner).unwrap();
        assert_eq!(
            InputLease::acquire(owner).unwrap_err().kind(),
            io::ErrorKind::WouldBlock
        );
        drop(first);
        assert!(InputLease::acquire(owner).is_ok());
    }
}
