//! Isolated, invocation-owned workers. Guest references never cross OS threads.
use crate::{CancellationToken, ExecutionOptions, Fault, Module, Value};
use std::sync::{Arc, Mutex, mpsc};
use std::thread::JoinHandle;

// Capturing through Console checks the quota before copying each line, rather
// than letting an interpreter build an unlimited output Vec before returning.
#[derive(Debug)]
struct WorkerOutput {
    capture: Mutex<CapturedOutput>,
}
#[derive(Debug)]
struct CapturedOutput {
    remaining: usize,
    lines: Vec<String>,
    exceeded: bool,
}
impl crate::Console for WorkerOutput {
    fn read_byte(&self) -> std::io::Result<Option<u8>> {
        Err(std::io::Error::other("Worker console input is unavailable"))
    }

    fn write_line(&self, text: &str) -> std::io::Result<()> {
        let mut capture = self.capture.lock().unwrap();
        let Some(remaining) = text
            .len()
            .checked_add(1)
            .and_then(|cost| capture.remaining.checked_sub(cost))
        else {
            capture.exceeded = true;
            return Err(std::io::Error::other("Worker result byte limit exceeded"));
        };
        capture.lines.push(text.to_owned());
        capture.remaining = remaining;
        Ok(())
    }
}

type Outcome = Result<(String, Vec<String>), Fault>;
struct Job {
    module: Arc<Module>,
    function: crate::metadata::Function,
    input: String,
    options: ExecutionOptions,
    reply: mpsc::Sender<Outcome>,
}
thread_local! { static IN_WORKER: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
pub(crate) fn is_worker() -> bool {
    IN_WORKER.with(|flag| flag.get())
}
fn execute(mut job: Job) {
    IN_WORKER.with(|flag| flag.set(true));
    let output = Arc::new(WorkerOutput {
        capture: Mutex::new(CapturedOutput {
            remaining: job.options.limits.worker_result_bytes,
            lines: Vec::new(),
            exceeded: false,
        }),
    });
    job.options.console = Some(output.clone());
    let result = crate::vm::interpret_function(
        &job.module,
        job.function,
        vec![Value::String(job.input.into())],
        job.options,
        None,
    )
    .and_then(|execution| match execution.value {
        Value::String(value) => {
            let mut capture = output.capture.lock().unwrap();
            if value.len() > capture.remaining {
                return Err(Fault::new("Worker result byte limit exceeded"));
            }
            Ok((value.into_owned(), std::mem::take(&mut capture.lines)))
        }
        _ => Err(Fault::new("Worker must return String")),
    });
    let result = result.map_err(|mut fault| {
        if output.capture.lock().unwrap().exceeded {
            fault.message = "Worker result byte limit exceeded".into();
        }
        fault
    });
    // Publish only after per-job producer state is released, including on pooled workers.
    drop(output);
    drop(job.module);
    let _ = job.reply.send(result);
}
struct WorkerResult {
    cancellation: CancellationToken,
    thread: Option<JoinHandle<()>>,
    receive: Option<mpsc::Receiver<Outcome>>,
    ready: Option<Outcome>,
    notification: Option<Value>,
    registered: bool,
}
#[derive(Default)]
pub(crate) struct Workers {
    results: Vec<WorkerResult>,
    threads: Vec<JoinHandle<()>>,
    pool: Option<mpsc::SyncSender<Job>>,
}
impl Workers {
    pub(crate) fn start(
        &mut self,
        module: &Module,
        args: Vec<Value>,
        options: &ExecutionOptions,
        pooled: bool,
    ) -> Result<Value, Fault> {
        if is_worker() {
            return Err(Fault::new("Nested worker creation is not supported"));
        }
        if self.results.len() >= 64 {
            return Err(Fault::new(
                "Worker submission limit exceeded (64 per invocation)",
            ));
        }
        let [Value::Delegate(callback), Value::String(input)] = args.as_slice() else {
            return Err(Fault::new(
                "Worker requires a static String-to-String callback and input",
            ));
        };
        if callback.receiver.is_some() {
            return Err(Fault::new("Worker callbacks cannot capture guest state"));
        }
        let function = crate::vm::resolve(module, &callback.target)?;
        if function.instance
            || function.parameters != [crate::metadata::Type::String]
            || function.returns != crate::metadata::Type::String
            || function.no_result
            || function.is_internal_call()
            || function.pinvoke.is_some()
        {
            return Err(Fault::new(
                "Worker requires a managed static String-to-String callback",
            ));
        }
        let (reply, receive) = mpsc::channel();
        let cancellation = CancellationToken::new();
        let job = Job {
            module: Arc::new(module.clone()),
            function,
            input: input.as_str().to_owned(),
            options: ExecutionOptions {
                limits: options.limits,
                arguments: options.arguments.clone(),
                cancellation: Some(cancellation.clone()),
                ..Default::default()
            },
            reply,
        };
        let mut dedicated = None;
        if pooled {
            if self.pool.is_none() {
                let (send, receive) = mpsc::sync_channel::<Job>(64);
                let receive = Arc::new(Mutex::new(receive));
                for _ in 0..2 {
                    let receive = receive.clone();
                    self.threads.push(
                        std::thread::Builder::new()
                            .name("neoclr-pool".into())
                            .spawn(move || {
                                loop {
                                    let job = receive.lock().unwrap().recv();
                                    match job {
                                        Ok(job) => execute(job),
                                        Err(_) => break,
                                    }
                                }
                            })
                            .map_err(|e| Fault::new(format!("Cannot create worker: {e}")))?,
                    );
                }
                self.pool = Some(send);
            }
            self.pool
                .as_ref()
                .unwrap()
                .send(job)
                .map_err(|_| Fault::new("Worker pool unavailable"))?;
        } else {
            dedicated = Some(
                std::thread::Builder::new()
                    .name("neoclr-thread".into())
                    .spawn(move || execute(job))
                    .map_err(|e| Fault::new(format!("Cannot create worker: {e}")))?,
            );
        }
        self.results.push(WorkerResult {
            cancellation,
            thread: dedicated,
            receive: Some(receive),
            ready: None,
            notification: None,
            registered: false,
        });
        Ok(Value::Int32((self.results.len() - 1) as i32))
    }
    // A request changes no outcome and releases no roots or producer resources.
    pub(crate) fn request_cancellation(&mut self, args: Vec<Value>) -> Result<Value, Fault> {
        let [Value::Int32(id)] = args.as_slice() else {
            return Err(Fault::new("Invalid worker handle"));
        };
        let result = usize::try_from(*id)
            .ok()
            .and_then(|i| self.results.get_mut(i))
            .ok_or_else(|| Fault::new("Unknown worker"))?;
        if result.receive.is_none() && result.ready.is_none() || result.cancellation.is_cancelled()
        {
            return Ok(Value::Boolean(false));
        }
        result.cancellation.cancel();
        Ok(Value::Boolean(true))
    }

    // Experimental notification path. It retains the callback until the VM
    // transfers it into a TaskQueue.Post frame; no guest Value enters a worker.
    pub(crate) fn notify(&mut self, args: Vec<Value>) -> Result<Value, Fault> {
        let [Value::Int32(id), callback @ Value::Delegate(_)] = args.as_slice() else {
            return Err(Fault::new(
                "Worker notification requires a handle and callback",
            ));
        };
        if callback.ty() != crate::assembler::parse_type("System.Func<Void>")? {
            return Err(Fault::new(
                "Worker notification callback must be Func<Void>",
            ));
        }
        let result = usize::try_from(*id)
            .ok()
            .and_then(|i| self.results.get_mut(i))
            .filter(|r| !r.registered && r.receive.is_some())
            .ok_or_else(|| Fault::new("Unknown, joined or already registered worker"))?;
        result.notification = Some(callback.clone());
        result.registered = true;
        Ok(Value::Void)
    }

    pub(crate) fn trace_roots(&self, roots: &mut Vec<usize>) {
        for result in &self.results {
            if let Some(callback) = &result.notification {
                crate::gc::trace(callback, roots);
            }
        }
    }

    // Nonblocking poll for a safe guest dispatch boundary. A receiver remains
    // rooted here until its callback moves into the VM's Post frame.
    pub(crate) fn poll_notification(&mut self) -> Option<Value> {
        for result in &mut self.results {
            if result.notification.is_none() {
                continue;
            }
            if result.ready.is_none() {
                match result.receive.as_ref().unwrap().try_recv() {
                    Ok(outcome) => {
                        result.ready = Some(outcome);
                        result.receive = None;
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        result.ready = Some(Err(Fault::new("Worker terminated without a result")));
                        result.receive = None;
                    }
                    Err(mpsc::TryRecvError::Empty) => {}
                }
            }
            if result.ready.is_some() {
                return result.notification.take();
            }
        }
        None
    }

    pub(crate) fn has_notifications(&self) -> bool {
        self.results.iter().any(|result| result.notification.is_some())
    }

    // One bounded wait only. The invocation must poll other completion sources
    // between waits rather than letting a pending worker monopolize quiescence.
    pub(crate) fn wait_notification_tick(&mut self) {
        let Some(result) = self.results.iter_mut().find(|r| r.notification.is_some()) else {
            return;
        };
        let Some(receive) = &result.receive else {
            return;
        };
        match receive.recv_timeout(std::time::Duration::from_millis(10)) {
            Ok(outcome) => {
                result.ready = Some(outcome);
                result.receive = None;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                result.ready = Some(Err(Fault::new("Worker terminated without a result")));
                result.receive = None;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
    }

    #[cfg(test)]
    pub(crate) fn wait_notification(
        &mut self,
        options: &ExecutionOptions,
    ) -> Result<Option<Value>, Fault> {
        loop {
            options.check_cancellation("Worker.Completion", 0)?;
            if let Some(callback) = self.poll_notification() {
                return Ok(Some(callback));
            }
            if !self.has_notifications() {
                return Ok(None);
            }
            self.wait_notification_tick();
        }
    }

    pub(crate) fn join(
        &mut self,
        args: Vec<Value>,
        output: &mut Vec<String>,
        options: &ExecutionOptions,
    ) -> Result<Value, Fault> {
        self.join_result(args, output, options, false)
    }

    pub(crate) fn join_result(
        &mut self,
        args: Vec<Value>,
        output: &mut Vec<String>,
        options: &ExecutionOptions,
        cancellation_as_value: bool,
    ) -> Result<Value, Fault> {
        let [Value::Int32(id)] = args.as_slice() else {
            return Err(Fault::new("Invalid worker handle"));
        };
        let result = usize::try_from(*id)
            .ok()
            .and_then(|i| self.results.get_mut(i))
            .filter(|r| r.notification.is_none())
            .ok_or_else(|| Fault::new("Unknown worker or completion not yet dispatched"))?;
        let cached = result.ready.take();
        let receive = result.receive.take();
        if cached.is_none() && receive.is_none() {
            return Err(Fault::new("Unknown or already joined worker"));
        }
        let mut cached = cached;
        loop {
            options.check_cancellation("Worker.Join", 0)?;
            let next = if let Some(outcome) = cached.take() {
                Ok(outcome)
            } else {
                receive
                    .as_ref()
                    .unwrap()
                    .recv_timeout(std::time::Duration::from_millis(10))
            };
            // Cancellation may have arrived during recv_timeout. Observe it
            // before accepting success/failure or forwarding producer output.
            options.check_cancellation("Worker.Join", 0)?;
            match next {
                Ok(outcome) => {
                    // A dedicated thread is finished, including host cleanup, before
                    // its Task can be completed on the invoking VM.
                    if let Some(thread) = result.thread.take() {
                        thread
                            .join()
                            .map_err(|_| Fault::new("Worker thread panicked"))?;
                    }
                    let (value, lines) = match outcome {
                        Err(fault)
                            if cancellation_as_value
                                && result.cancellation.is_cancelled()
                                && fault.code == crate::FaultCode::ExecutionCancelled =>
                        {
                            options.check_cancellation("Worker.Join", 0)?;
                            return Ok(Value::Void);
                        }
                        other => other?,
                    };
                    // Keep worker output isolated until explicitly joined.
                    if let Some(console) = &options.console {
                        for line in lines {
                            options.check_cancellation("Worker.Join", 0)?;
                            console
                                .write_line(&line)
                                .map_err(|_| Fault::new("console output failed"))?;
                        }
                    } else {
                        output.extend(lines);
                    }
                    // The last host write may itself request cancellation.
                    options.check_cancellation("Worker.Join", 0)?;
                    return Ok(Value::String(value.into()));
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(_) => return Err(Fault::new("Worker terminated without a result")),
            }
        }
    }
}
impl Drop for Workers {
    fn drop(&mut self) {
        for result in &self.results {
            result.cancellation.cancel();
        }
        self.pool.take();
        for result in &mut self.results {
            if let Some(thread) = result.thread.take() {
                let _ = thread.join();
            }
        }
        for thread in self.threads.drain(..) {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pending(receive: mpsc::Receiver<Outcome>, marker: i32) -> WorkerResult {
        WorkerResult {
            cancellation: CancellationToken::new(),
            thread: None,
            receive: Some(receive),
            ready: None,
            notification: Some(Value::Int32(marker)),
            registered: true,
        }
    }

    #[test]
    fn operation_cancel_waits_for_acknowledgement_and_leaves_sibling_live() {
        let (send, receive) = mpsc::channel();
        let (_sibling_send, sibling_receive) = mpsc::channel();
        let mut workers = Workers::default();
        workers.results = vec![pending(receive, 1), pending(sibling_receive, 2)];
        let token = workers.results[0].cancellation.clone();
        let cleaned = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let finished = cleaned.clone();
        workers.results[0].thread = Some(std::thread::spawn(move || {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            while !token.is_cancelled() {
                assert!(std::time::Instant::now() < deadline);
                std::thread::yield_now();
            }
            send.send(Err(Fault::coded(
                crate::FaultCode::ExecutionCancelled,
                "execution cancelled",
            )))
            .unwrap();
            finished.store(true, std::sync::atomic::Ordering::Release);
        }));
        assert_eq!(
            workers.request_cancellation(vec![Value::Int32(0)]).unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            workers.request_cancellation(vec![Value::Int32(0)]).unwrap(),
            Value::Boolean(false)
        );
        assert!(!workers.results[1].cancellation.is_cancelled());
        assert_eq!(workers.results[0].notification, Some(Value::Int32(1)));
        let options = ExecutionOptions::default();
        assert_eq!(
            workers.wait_notification(&options).unwrap(),
            Some(Value::Int32(1))
        );
        assert_eq!(
            workers
                .join_result(vec![Value::Int32(0)], &mut vec![], &options, true)
                .unwrap(),
            Value::Void
        );
        assert!(cleaned.load(std::sync::atomic::Ordering::Acquire));
        assert_eq!(
            workers.request_cancellation(vec![Value::Int32(0)]).unwrap(),
            Value::Boolean(false)
        );
        assert!(
            workers
                .join_result(vec![Value::Int32(0)], &mut vec![], &options, true)
                .is_err()
        );
    }

    #[test]
    fn request_does_not_override_ready_success_or_forge_cancellation() {
        for outcome in [
            Ok(("done".into(), vec![])),
            Err(Fault::coded(
                crate::FaultCode::UserFault,
                "execution cancelled",
            )),
        ] {
            let success = outcome.is_ok();
            let (send, receive) = mpsc::channel();
            send.send(outcome).unwrap();
            let mut workers = Workers::default();
            workers.results.push(pending(receive, 1));
            assert_eq!(workers.poll_notification(), Some(Value::Int32(1)));
            assert_eq!(
                workers.request_cancellation(vec![Value::Int32(0)]).unwrap(),
                Value::Boolean(true)
            );
            let result = workers.join_result(
                vec![Value::Int32(0)],
                &mut vec![],
                &ExecutionOptions::default(),
                true,
            );
            if success {
                assert_eq!(result.unwrap(), Value::String("done".into()));
            } else {
                assert_eq!(result.unwrap_err().code, crate::FaultCode::UserFault);
            }
        }
    }

    #[test]
    fn legacy_join_and_invocation_cancellation_still_fault() {
        for invocation_cancelled in [false, true] {
            let (send, receive) = mpsc::channel();
            send.send(Err(Fault::coded(
                crate::FaultCode::ExecutionCancelled,
                "execution cancelled",
            )))
            .unwrap();
            let mut workers = Workers::default();
            workers.results.push(pending(receive, 1));
            workers.request_cancellation(vec![Value::Int32(0)]).unwrap();
            assert_eq!(workers.poll_notification(), Some(Value::Int32(1)));
            let token = CancellationToken::new();
            if invocation_cancelled {
                token.cancel();
            }
            let options = ExecutionOptions {
                cancellation: Some(token),
                ..Default::default()
            };
            // Legacy join always faults; the new join must not swallow host cancellation.
            let result = workers.join_result(
                vec![Value::Int32(0)],
                &mut vec![],
                &options,
                invocation_cancelled,
            );
            assert_eq!(
                result.unwrap_err().code,
                crate::FaultCode::ExecutionCancelled
            );
        }
    }

    #[test]
    fn dedicated_join_waits_for_thread_cleanup() {
        let (send, receive) = mpsc::channel();
        let cleaned = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let finished = cleaned.clone();
        let thread = std::thread::spawn(move || {
            send.send(Ok(("done".into(), vec![]))).unwrap();
            finished.store(true, std::sync::atomic::Ordering::Release);
        });
        let mut workers = Workers::default();
        workers.results.push(WorkerResult {
            cancellation: CancellationToken::new(),
            thread: Some(thread),
            receive: Some(receive),
            ready: None,
            notification: None,
            registered: false,
        });
        let result = workers.join(
            vec![Value::Int32(0)],
            &mut vec![],
            &ExecutionOptions::default(),
        );
        assert_eq!(result.unwrap(), Value::String("done".into()));
        assert!(cleaned.load(std::sync::atomic::Ordering::Acquire));
        assert!(workers.results[0].thread.is_none());
    }

    #[test]
    fn ready_result_bypasses_pending_result_and_delivers_once() {
        let (_first, first) = mpsc::channel();
        let (second, receive) = mpsc::channel();
        second
            .send(Ok(("done".into(), vec!["worker output".into()])))
            .unwrap();
        let mut workers = Workers::default();
        workers.results = vec![pending(first, 1), pending(receive, 2)];
        let options = ExecutionOptions::default();
        assert_eq!(
            workers.wait_notification(&options).unwrap(),
            Some(Value::Int32(2))
        );
        let mut output = vec![];
        assert_eq!(
            workers
                .join(vec![Value::Int32(1)], &mut output, &options)
                .unwrap(),
            Value::String("done".into())
        );
        assert_eq!(output, ["worker output"]);
        assert!(
            workers
                .join(vec![Value::Int32(1)], &mut output, &options)
                .is_err()
        );
        assert!(
            workers
                .join(vec![Value::Int32(0)], &mut output, &options)
                .is_err()
        );
        assert_eq!(workers.poll_notification(), None);
        let cancellation = CancellationToken::default();
        cancellation.cancel();
        assert!(
            workers
                .wait_notification(&ExecutionOptions {
                    cancellation: Some(cancellation),
                    ..Default::default()
                })
                .is_err()
        );
    }

    #[test]
    fn disconnected_producer_notifies_then_join_reports_failure() {
        let (send, receive) = mpsc::channel();
        drop(send);
        let mut workers = Workers::default();
        workers.results = vec![pending(receive, 1)];
        let options = ExecutionOptions::default();
        assert_eq!(
            workers.wait_notification(&options).unwrap(),
            Some(Value::Int32(1))
        );
        assert!(
            workers
                .join(vec![Value::Int32(0)], &mut vec![], &options)
                .unwrap_err()
                .message
                .contains("without a result")
        );
        assert_eq!(workers.wait_notification(&options).unwrap(), None);
    }

    #[test]
    fn cancellation_before_dispatch_retains_registration_for_teardown() {
        // Both producer outcomes can be ready before cancellation is observed.
        for outcome in [
            Ok(("done".into(), vec!["hidden".into()])),
            Err(Fault::new("producer failed")),
        ] {
            let (send, receive) = mpsc::channel();
            send.send(outcome).unwrap();
            let mut workers = Workers::default();
            workers.results.push(pending(receive, 1));
            let token = CancellationToken::new();
            token.cancel();
            let options = ExecutionOptions {
                cancellation: Some(token),
                ..Default::default()
            };
            assert_eq!(
                workers.wait_notification(&options).unwrap_err().message,
                "execution cancelled"
            );
            assert_eq!(workers.results[0].notification, Some(Value::Int32(1)));
            assert!(workers.results[0].receive.is_some());
        }
    }

    #[test]
    fn cancellation_after_dispatch_discards_cached_success_or_failure() {
        for outcome in [
            Ok(("done".into(), vec!["hidden".into()])),
            Err(Fault::new("producer failed")),
        ] {
            let (send, receive) = mpsc::channel();
            send.send(outcome).unwrap();
            let mut workers = Workers::default();
            workers.results.push(pending(receive, 1));
            assert_eq!(workers.poll_notification(), Some(Value::Int32(1)));
            let token = CancellationToken::new();
            token.cancel();
            let options = ExecutionOptions {
                cancellation: Some(token),
                ..Default::default()
            };
            let mut output = vec![];
            assert_eq!(
                workers
                    .join(vec![Value::Int32(0)], &mut output, &options)
                    .unwrap_err()
                    .message,
                "execution cancelled"
            );
            assert!(output.is_empty());
            assert!(workers.results[0].ready.is_none());
            assert!(workers.results[0].receive.is_none());
            assert_eq!(workers.poll_notification(), None);
        }
    }

    #[test]
    fn teardown_waits_for_producer_cancellation_acknowledgement() {
        let mut workers = Workers::default();
        let (send, receive) = mpsc::channel();
        workers.results.push(pending(receive, 1));
        let cancellation = workers.results[0].cancellation.clone();
        let acknowledged = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let finished = acknowledged.clone();
        workers.threads.push(std::thread::spawn(move || {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            while !cancellation.is_cancelled() {
                assert!(
                    std::time::Instant::now() < deadline,
                    "teardown never requested cancellation"
                );
                std::thread::yield_now();
            }
            // Teardown must retain the receiver until the producer has stopped.
            send.send(Err(Fault::new("cancelled producer"))).unwrap();
            finished.store(true, std::sync::atomic::Ordering::SeqCst);
        }));
        drop(workers);
        assert!(acknowledged.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn last_output_write_observes_cancellation_without_overriding_write_failure() {
        #[derive(Debug)]
        struct CancelOnWrite {
            token: CancellationToken,
            fail: bool,
        }
        impl crate::Console for CancelOnWrite {
            fn read_byte(&self) -> std::io::Result<Option<u8>> {
                Ok(None)
            }
            fn write_line(&self, _: &str) -> std::io::Result<()> {
                self.token.cancel();
                if self.fail {
                    Err(std::io::Error::other("write failed"))
                } else {
                    Ok(())
                }
            }
        }
        for fail in [false, true] {
            let console = Arc::new(CancelOnWrite {
                token: CancellationToken::new(),
                fail,
            });
            let options = ExecutionOptions {
                cancellation: Some(console.token.clone()),
                console: Some(console),
                ..Default::default()
            };
            let (send, receive) = mpsc::channel();
            send.send(Ok(("done".into(), vec!["last line".into()])))
                .unwrap();
            let mut workers = Workers::default();
            workers.results.push(pending(receive, 1));
            workers.poll_notification();
            let fault = workers
                .join(vec![Value::Int32(0)], &mut vec![], &options)
                .unwrap_err();
            assert_eq!(
                fault.message,
                if fail {
                    "console output failed"
                } else {
                    "execution cancelled"
                }
            );
        }
    }
}
