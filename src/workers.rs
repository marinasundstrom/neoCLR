//! Isolated, invocation-owned workers. Guest references never cross OS threads.
use crate::{CancellationToken, ExecutionOptions, Fault, Module, Value};
use std::sync::{Arc, Mutex, mpsc};
use std::thread::JoinHandle;

type Outcome = Result<(String, Vec<String>), Fault>;
struct Job {
    module: Arc<Module>,
    function: crate::metadata::Function,
    input: String,
    options: ExecutionOptions,
    reply: mpsc::Sender<Outcome>,
}
thread_local! { static IN_WORKER: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
fn execute(job: Job) {
    IN_WORKER.with(|flag| flag.set(true));
    let result = crate::vm::interpret_function(
        &job.module,
        job.function,
        vec![Value::String(job.input)],
        job.options,
        None,
    )
    .and_then(|execution| match execution.value {
        Value::String(value) => Ok((value, execution.output)),
        _ => Err(Fault::new("Worker must return String")),
    });
    let _ = job.reply.send(result);
}
struct WorkerResult {
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
    cancellation: CancellationToken,
}
impl Workers {
    pub(crate) fn start(
        &mut self,
        module: &Module,
        args: Vec<Value>,
        options: &ExecutionOptions,
        pooled: bool,
    ) -> Result<Value, Fault> {
        if IN_WORKER.with(|flag| flag.get()) {
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
        let job = Job {
            module: Arc::new(module.clone()),
            function,
            input: input.clone(),
            options: ExecutionOptions {
                limits: options.limits,
                arguments: options.arguments.clone(),
                cancellation: Some(self.cancellation.clone()),
                ..Default::default()
            },
            reply,
        };
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
            self.threads.push(
                std::thread::Builder::new()
                    .name("neoclr-thread".into())
                    .spawn(move || execute(job))
                    .map_err(|e| Fault::new(format!("Cannot create worker: {e}")))?,
            );
        }
        self.results.push(WorkerResult {
            receive: Some(receive),
            ready: None,
            notification: None,
            registered: false,
        });
        Ok(Value::Int32((self.results.len() - 1) as i32))
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

    pub(crate) fn wait_notification(
        &mut self,
        options: &ExecutionOptions,
    ) -> Result<Option<Value>, Fault> {
        loop {
            options.check_cancellation("Worker.Completion", 0)?;
            let mut waiting = None;
            for (index, result) in self.results.iter_mut().enumerate() {
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
                            result.ready =
                                Some(Err(Fault::new("Worker terminated without a result")));
                            result.receive = None;
                        }
                        Err(mpsc::TryRecvError::Empty) => {
                            waiting.get_or_insert(index);
                        }
                    }
                }
                if result.ready.is_some() {
                    return Ok(result.notification.take());
                }
            }
            let Some(index) = waiting else {
                return Ok(None);
            };
            let result = &mut self.results[index];
            // Bounded wait, then poll every registered result again: an earlier
            // unfinished worker cannot indefinitely block a later ready worker.
            match result
                .receive
                .as_ref()
                .unwrap()
                .recv_timeout(std::time::Duration::from_millis(10))
            {
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
    }

    pub(crate) fn join(
        &mut self,
        args: Vec<Value>,
        output: &mut Vec<String>,
        options: &ExecutionOptions,
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
            match next {
                Ok(outcome) => {
                    let (value, lines) = outcome?;
                    // Keep worker output isolated until explicitly joined.
                    if let Some(console) = &options.console {
                        for line in lines {
                            console
                                .write_line(&line)
                                .map_err(|_| Fault::new("console output failed"))?;
                        }
                    } else {
                        output.extend(lines);
                    }
                    return Ok(Value::String(value));
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(_) => return Err(Fault::new("Worker terminated without a result")),
            }
        }
    }
}
impl Drop for Workers {
    fn drop(&mut self) {
        self.cancellation.cancel();
        self.pool.take();
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
            receive: Some(receive),
            ready: None,
            notification: Some(Value::Int32(marker)),
            registered: true,
        }
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
}
