//! Invocation-owned terminal modes. All operations run outside managed graph access.
//! Private scalar guest bridge shares the native provider status contract.
use crate::{CancellationToken, Console, TerminalInput, TerminalOptions, TerminalRead};
use std::{
    io,
    sync::{
        Condvar, Mutex,
        atomic::{AtomicBool, AtomicI32, Ordering},
    },
    time::{Duration, Instant},
};

static NEXT_HANDLE: AtomicI32 = AtomicI32::new(1);
const WAIT_QUANTUM: Duration = Duration::from_millis(20);

#[derive(Default)]
struct State {
    handle: i32,
    input: Option<Box<dyn TerminalInput>>,
    reading: bool,
}

pub(crate) struct Service {
    state: Mutex<State>,
    finished: Condvar,
    stopped: AtomicBool,
    cancellation: CancellationToken,
}

fn closed() -> io::Error {
    io::Error::new(io::ErrorKind::NotConnected, "terminal session closed")
}
fn busy() -> io::Error {
    io::Error::new(io::ErrorKind::WouldBlock, "terminal input already owned")
}

impl Service {
    pub(crate) fn new(cancellation: CancellationToken) -> Self {
        Self {
            state: Mutex::new(State::default()),
            finished: Condvar::new(),
            stopped: AtomicBool::new(false),
            cancellation,
        }
    }

    fn check(&self) -> io::Result<()> {
        if self.stopped.load(Ordering::Acquire) {
            return Err(closed());
        }
        if self.cancellation.is_cancelled() {
            return Err(io::Error::from(io::ErrorKind::Interrupted));
        }
        Ok(())
    }

    // Private transport handles, never OS descriptors. No reuse across invocations.
    pub(crate) fn open(&self, console: &dyn Console, options: TerminalOptions) -> io::Result<i32> {
        let mut state = self.state.lock().unwrap();
        self.check()?;
        if state.input.is_some() || state.reading {
            return Err(busy());
        }
        // try_update is newer than the supported Rust 1.85 minimum.
        #[allow(deprecated)]
        let handle = NEXT_HANDLE
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .map_err(|_| io::Error::other("terminal handle capacity exhausted"))?;
        let mut input = console.open_terminal(options)?;
        if let Err(error) = self.check() {
            if input.close().is_err() {
                // Acquisition raced cancellation. Keep failed restoration owned by
                // the invocation even though no handle escapes to the caller.
                state.handle = handle;
                state.input = Some(input);
            }
            return Err(error);
        }
        state.handle = handle;
        state.input = Some(input);
        Ok(handle)
    }

    pub(crate) fn read(
        &self,
        handle: i32,
        destination: &mut [u8],
        timeout: Option<Duration>,
    ) -> io::Result<TerminalRead> {
        self.check()?;
        if destination.is_empty() {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }
        let deadline = match timeout {
            Some(duration) => Some(
                Instant::now()
                    .checked_add(duration)
                    .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?,
            ),
            None => None,
        };
        let mut state = self.state.lock().unwrap();
        self.check()?;
        if handle <= 0 || state.handle != handle {
            return Err(closed());
        }
        if state.reading {
            return Err(busy());
        }
        let input = state.input.take().ok_or_else(closed)?;
        state.reading = true;
        drop(state);
        let mut active = ActiveRead {
            service: self,
            input: Some(input),
        };
        loop {
            self.check()?;
            let remaining =
                deadline.map(|deadline| deadline.saturating_duration_since(Instant::now()));
            let result = if timeout == Some(Duration::ZERO) {
                active.input.as_mut().unwrap().try_read(destination)?
            } else {
                let wait = remaining.unwrap_or(WAIT_QUANTUM).min(WAIT_QUANTUM);
                active
                    .input
                    .as_mut()
                    .unwrap()
                    .read(destination, Some(wait))?
            };
            if matches!(result, TerminalRead::Data(count) if count == 0 || count > destination.len())
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid terminal byte count",
                ));
            }
            if !matches!(result, TerminalRead::Pending) {
                // Bytes already transferred stay visible even if cancellation races this read.
                return Ok(result);
            }
            self.check()?;
            if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                return Ok(TerminalRead::Pending);
            }
        }
    }

    pub(crate) fn close(&self, handle: i32) -> io::Result<()> {
        let mut state = self.state.lock().unwrap();
        if handle <= 0 || state.handle != handle {
            return Err(closed());
        }
        if state.reading {
            return Err(busy());
        }
        if let Some(input) = &mut state.input {
            input.close()?; // Retain ownership on failure so the caller may retry.
            state.input.take();
        }
        Ok(())
    }

    pub(crate) fn invoke(
        &self,
        binding: &crate::native::Binding,
        args: &[crate::Value],
        console: Option<&dyn Console>,
    ) -> Result<crate::Value, crate::Fault> {
        use crate::{Value, native::Binding};
        let result = match (binding, args) {
            (Binding::ConsoleTerminalOpen, [Value::Boolean(capture)]) => match console {
                Some(console) => self.open(
                    console,
                    TerminalOptions {
                        capture_interrupts: *capture,
                    },
                ),
                None => Err(io::ErrorKind::Unsupported.into()),
            },
            (Binding::ConsoleTerminalReadByte, [Value::Int32(handle), Value::Int32(timeout)])
                if *timeout >= -1 =>
            {
                let mut byte = [0u8];
                self.read(
                    *handle,
                    &mut byte,
                    if *timeout == -1 {
                        None
                    } else {
                        Some(Duration::from_millis(*timeout as u64))
                    },
                )
                .map(|result| match result {
                    TerminalRead::Data(_) => i32::from(byte[0]),
                    TerminalRead::Pending => -1,
                    TerminalRead::End => -2,
                })
            }
            (Binding::ConsoleTerminalClose, [Value::Int32(handle)]) => {
                self.close(*handle).map(|()| 0)
            }
            _ => return Err(crate::Fault::new("invalid terminal bridge arguments")),
        };
        Ok(Value::Int32(result.unwrap_or_else(
            |error| match error.kind() {
                io::ErrorKind::Unsupported => -3,
                io::ErrorKind::WouldBlock => -4,
                io::ErrorKind::NotConnected => -5,
                io::ErrorKind::InvalidInput => -6,
                _ => -7,
            },
        )))
    }

    pub(crate) fn request_stop(&self) {
        self.stopped.store(true, Ordering::Release);
    }

    pub(crate) fn shutdown(&self) -> io::Result<()> {
        self.request_stop();
        let mut state = self.state.lock().unwrap();
        while state.reading {
            state = self.finished.wait(state).unwrap();
        }
        if let Some(input) = &mut state.input {
            input.close()?;
            state.input.take();
        }
        Ok(())
    }
}

// Put the session back even if a misbehaving host panics during a read. Shutdown
// can then restore modes; no mutex is held across host reads or cancellation waits.
struct ActiveRead<'a> {
    service: &'a Service,
    input: Option<Box<dyn TerminalInput>>,
}
impl Drop for ActiveRead<'_> {
    fn drop(&mut self) {
        let mut state = self.service.state.lock().unwrap();
        state.input = self.input.take();
        state.reading = false;
        self.service.finished.notify_all();
    }
}
impl Drop for Service {
    fn drop(&mut self) {
        if let Some(input) = &mut self.state.get_mut().unwrap().input {
            let _ = input.close();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, atomic::AtomicUsize};

    #[derive(Debug, Default)]
    struct Observed {
        started: AtomicBool,
        closes: AtomicUsize,
        drops: AtomicUsize,
        fail_close: AtomicBool,
    }
    #[derive(Clone, Copy, Debug)]
    enum Mode {
        Idle,
        Data,
        Panic,
    }
    #[derive(Debug)]
    struct Input {
        observed: Arc<Observed>,
        mode: Mode,
    }
    impl TerminalInput for Input {
        fn try_read(&mut self, destination: &mut [u8]) -> io::Result<TerminalRead> {
            self.observed.started.store(true, Ordering::Release);
            match self.mode {
                Mode::Idle => Ok(TerminalRead::Pending),
                Mode::Data => {
                    destination[0] = 27;
                    Ok(TerminalRead::Data(1))
                }
                Mode::Panic => panic!("synthetic host panic"),
            }
        }
        fn read(
            &mut self,
            destination: &mut [u8],
            timeout: Option<Duration>,
        ) -> io::Result<TerminalRead> {
            self.observed.started.store(true, Ordering::Release);
            if matches!(self.mode, Mode::Idle) {
                let wait = timeout.expect("service must bound waits");
                assert!(wait <= WAIT_QUANTUM);
                std::thread::sleep(wait);
            }
            self.try_read(destination)
        }
        fn close(&mut self) -> io::Result<()> {
            self.observed.closes.fetch_add(1, Ordering::Relaxed);
            if self.observed.fail_close.swap(false, Ordering::Relaxed) {
                Err(io::Error::other("synthetic restoration failure"))
            } else {
                Ok(())
            }
        }
    }
    impl Drop for Input {
        fn drop(&mut self) {
            self.observed.drops.fetch_add(1, Ordering::Relaxed);
        }
    }
    #[derive(Debug)]
    struct Host {
        observed: Arc<Observed>,
        mode: Mode,
    }
    impl Console for Host {
        fn open_terminal(&self, _: TerminalOptions) -> io::Result<Box<dyn TerminalInput>> {
            Ok(Box::new(Input {
                observed: self.observed.clone(),
                mode: self.mode,
            }))
        }
        fn read_byte(&self) -> io::Result<Option<u8>> {
            unreachable!()
        }
        fn write_line(&self, _: &str) -> io::Result<()> {
            unreachable!()
        }
    }
    fn host(mode: Mode) -> Host {
        Host {
            observed: Default::default(),
            mode,
        }
    }
    fn started(host: &Host) {
        let deadline = Instant::now() + Duration::from_secs(2);
        while !host.observed.started.load(Ordering::Acquire) {
            assert!(Instant::now() < deadline, "reader did not start");
            std::thread::yield_now();
        }
    }

    #[test]
    fn scope_restores_terminal_while_invocation_is_still_retained() {
        let invocation = crate::invocation::Invocation::new(crate::Limits::default());
        let host = host(Mode::Data);
        let handle = invocation
            .terminal
            .open(&host, TerminalOptions::default())
            .unwrap();
        let mut bytes = [0; 4];
        assert_eq!(
            invocation.terminal.read(handle, &mut bytes, None).unwrap(),
            TerminalRead::Data(1)
        );
        assert_eq!(bytes, [27, 0, 0, 0]);
        drop(crate::invocation_work::Scope(invocation.clone()));
        assert_eq!(host.observed.closes.load(Ordering::Relaxed), 1);
        assert_eq!(host.observed.drops.load(Ordering::Relaxed), 1);
        assert_eq!(
            invocation
                .terminal
                .open(&host, TerminalOptions::default())
                .unwrap_err()
                .kind(),
            io::ErrorKind::NotConnected
        );
    }

    #[test]
    fn scope_restores_on_unwind() {
        let invocation = crate::invocation::Invocation::new(crate::Limits::default());
        let host = host(Mode::Data);
        invocation
            .terminal
            .open(&host, TerminalOptions::default())
            .unwrap();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _scope = crate::invocation_work::Scope(invocation.clone());
            panic!("root failure");
        }));
        assert!(result.is_err());
        assert_eq!(host.observed.closes.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn cancellation_interrupts_idle_wait_and_scope_restores() {
        let token = CancellationToken::new();
        let invocation = crate::invocation::Invocation::with_cancellation(
            crate::Limits::default(),
            token.clone(),
        );
        let host = host(Mode::Idle);
        let handle = invocation
            .terminal
            .open(&host, TerminalOptions::default())
            .unwrap();
        let reader = invocation.clone();
        let thread = std::thread::spawn(move || reader.terminal.read(handle, &mut [0; 1], None));
        started(&host);
        token.cancel();
        assert_eq!(
            thread.join().unwrap().unwrap_err().kind(),
            io::ErrorKind::Interrupted
        );
        invocation.shutdown().unwrap();
        assert_eq!(host.observed.closes.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn shutdown_releases_idle_reader_and_rejects_overlap() {
        let service = Arc::new(Service::new(CancellationToken::new()));
        let host = host(Mode::Idle);
        let handle = service.open(&host, TerminalOptions::default()).unwrap();
        let reader = service.clone();
        let thread = std::thread::spawn(move || reader.read(handle, &mut [0; 1], None));
        started(&host);
        assert_eq!(
            service
                .read(handle, &mut [0; 1], Some(Duration::ZERO))
                .unwrap_err()
                .kind(),
            io::ErrorKind::WouldBlock
        );
        assert_eq!(
            service.close(handle).unwrap_err().kind(),
            io::ErrorKind::WouldBlock
        );
        service.shutdown().unwrap();
        assert_eq!(
            thread.join().unwrap().unwrap_err().kind(),
            io::ErrorKind::NotConnected
        );
        service.shutdown().unwrap();
        assert_eq!(host.observed.closes.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn restoration_failure_is_retryable_and_stale_handles_are_rejected() {
        let service = Service::new(CancellationToken::new());
        let other = Service::new(CancellationToken::new());
        let host = host(Mode::Data);
        let handle = service.open(&host, TerminalOptions::default()).unwrap();
        let other_handle = other.open(&host, TerminalOptions::default()).unwrap();
        assert_eq!(
            service
                .read(other_handle, &mut [0; 1], None)
                .unwrap_err()
                .kind(),
            io::ErrorKind::NotConnected
        );
        assert_eq!(
            service
                .open(&host, TerminalOptions::default())
                .unwrap_err()
                .kind(),
            io::ErrorKind::WouldBlock
        );
        host.observed.fail_close.store(true, Ordering::Relaxed);
        assert!(service.close(handle).is_err());
        assert_eq!(
            service.read(handle, &mut [0; 1], None).unwrap(),
            TerminalRead::Data(1)
        );
        service.close(handle).unwrap();
        service.close(handle).unwrap();
        let replacement = service.open(&host, TerminalOptions::default()).unwrap();
        assert_ne!(replacement, handle);
        assert_eq!(
            service.close(handle).unwrap_err().kind(),
            io::ErrorKind::NotConnected
        );
    }

    #[test]
    fn cancelled_acquisition_retains_failed_restoration_for_shutdown() {
        #[derive(Debug)]
        struct CancellingHost {
            token: CancellationToken,
            observed: Arc<Observed>,
        }
        impl Console for CancellingHost {
            fn open_terminal(&self, _: TerminalOptions) -> io::Result<Box<dyn TerminalInput>> {
                self.token.cancel();
                self.observed.fail_close.store(true, Ordering::Relaxed);
                Ok(Box::new(Input {
                    observed: self.observed.clone(),
                    mode: Mode::Data,
                }))
            }
            fn read_byte(&self) -> io::Result<Option<u8>> {
                unreachable!()
            }
            fn write_line(&self, _: &str) -> io::Result<()> {
                unreachable!()
            }
        }
        let token = CancellationToken::new();
        let service = Service::new(token.clone());
        let host = CancellingHost {
            token,
            observed: Default::default(),
        };
        assert_eq!(
            service
                .open(&host, TerminalOptions::default())
                .unwrap_err()
                .kind(),
            io::ErrorKind::Interrupted
        );
        assert_eq!(host.observed.drops.load(Ordering::Relaxed), 0);
        service.shutdown().unwrap();
        assert_eq!(host.observed.closes.load(Ordering::Relaxed), 2);
        assert_eq!(host.observed.drops.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn shutdown_reports_restoration_failure_and_allows_retry() {
        let invocation = crate::invocation::Invocation::new(crate::Limits::default());
        let host = host(Mode::Data);
        invocation
            .terminal
            .open(&host, TerminalOptions::default())
            .unwrap();
        host.observed.fail_close.store(true, Ordering::Relaxed);
        assert!(
            invocation
                .shutdown()
                .unwrap_err()
                .message
                .contains("terminal restoration failed")
        );
        assert_eq!(host.observed.drops.load(Ordering::Relaxed), 0);
        invocation.shutdown().unwrap();
        assert_eq!(host.observed.closes.load(Ordering::Relaxed), 2);
        assert_eq!(host.observed.drops.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn host_read_panic_returns_session_for_cleanup() {
        let service = Service::new(CancellationToken::new());
        let host = host(Mode::Panic);
        let handle = service.open(&host, TerminalOptions::default()).unwrap();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            service.read(handle, &mut [0; 1], None)
        }));
        assert!(result.is_err());
        service.shutdown().unwrap();
        assert_eq!(host.observed.closes.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn finite_wait_keeps_deadline_and_empty_reads_do_not_consume() {
        let service = Service::new(CancellationToken::new());
        let host = host(Mode::Idle);
        let handle = service.open(&host, TerminalOptions::default()).unwrap();
        assert_eq!(
            service.read(handle, &mut [], None).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        assert!(!host.observed.started.load(Ordering::Acquire));
        let start = Instant::now();
        assert_eq!(
            service
                .read(handle, &mut [0; 1], Some(Duration::from_millis(35)))
                .unwrap(),
            TerminalRead::Pending
        );
        assert!(start.elapsed() >= Duration::from_millis(35));
    }
    #[test]
    fn scalar_guest_bridge_reports_bytes_and_lifecycle_status() {
        use crate::{Value, native::Binding};
        let service = Service::new(CancellationToken::new());
        let host = host(Mode::Data);
        let Value::Int32(id) = service
            .invoke(
                &Binding::ConsoleTerminalOpen,
                &[Value::Boolean(true)],
                Some(&host),
            )
            .unwrap()
        else {
            panic!("handle")
        };
        assert!(id > 0);
        assert!(matches!(
            service
                .invoke(
                    &Binding::ConsoleTerminalOpen,
                    &[Value::Boolean(false)],
                    Some(&host)
                )
                .unwrap(),
            Value::Int32(-4)
        ));
        assert!(matches!(
            service
                .invoke(
                    &Binding::ConsoleTerminalReadByte,
                    &[Value::Int32(id), Value::Int32(0)],
                    Some(&host)
                )
                .unwrap(),
            Value::Int32(0..=255)
        ));
        assert!(matches!(
            service
                .invoke(
                    &Binding::ConsoleTerminalClose,
                    &[Value::Int32(id)],
                    Some(&host)
                )
                .unwrap(),
            Value::Int32(0)
        ));
        assert!(matches!(
            service
                .invoke(
                    &Binding::ConsoleTerminalReadByte,
                    &[Value::Int32(id), Value::Int32(0)],
                    Some(&host)
                )
                .unwrap(),
            Value::Int32(-5)
        ));
        assert!(matches!(
            service
                .invoke(
                    &Binding::ConsoleTerminalOpen,
                    &[Value::Boolean(false)],
                    None
                )
                .unwrap(),
            Value::Int32(-3)
        ));
    }
}
