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
#[derive(Default)]
pub(crate) struct Workers {
    results: Vec<Option<mpsc::Receiver<Outcome>>>,
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
        self.results.push(Some(receive));
        Ok(Value::Int32((self.results.len() - 1) as i32))
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
            .and_then(Option::take)
            .ok_or_else(|| Fault::new("Unknown or already joined worker"))?;
        loop {
            options.check_cancellation("Worker.Join", 0)?;
            match result.recv_timeout(std::time::Duration::from_millis(10)) {
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
