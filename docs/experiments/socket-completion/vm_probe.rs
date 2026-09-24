//! Test-only native adapter exercising the real interpreter, GC and TaskQueue.
use crate::{Fault, ManagedHeap, Value, metadata::Type};
use std::{
    cell::RefCell,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, mpsc},
    time::Duration,
};

thread_local! { static INPUT: RefCell<Option<TcpStream>> = const { RefCell::new(None) }; }
struct Pending {
    socket: TcpStream,
    destination: Value,
    callback: Value,
    offset: usize,
    bytes: Vec<u8>,
}
#[derive(Default)]
pub(crate) struct Receives {
    pending: Option<Pending>,
}
impl Receives {
    pub(crate) fn begin(&mut self, args: Vec<Value>, heap: &ManagedHeap) -> Result<(), Fault> {
        if self.pending.is_some() {
            return Err(Fault::new("Socket probe operation limit"));
        }
        let [
            destination @ Value::ObjectReference(object),
            Value::Int32(offset),
            Value::Int32(count),
            callback @ Value::Delegate(_),
        ] = args.as_slice()
        else {
            return Err(Fault::new("Invalid socket probe arguments"));
        };
        let Value::Array {
            element: Type::Byte,
            elements,
        } = heap.read_reference(&object.reference)?
        else {
            return Err(Fault::new("Expected byte array"));
        };
        if callback.ty() != crate::assembler::parse_type("System.Func<Void>")? {
            return Err(Fault::new("Expected completion callback"));
        }
        let offset = usize::try_from(*offset).map_err(|_| Fault::new("Invalid socket range"))?;
        let count = usize::try_from(*count).map_err(|_| Fault::new("Invalid socket range"))?;
        if offset > elements.len() || count > elements.len() - offset || count > 8 {
            return Err(Fault::new("Invalid socket range"));
        }
        let socket = INPUT
            .with(|input| input.borrow_mut().take())
            .ok_or_else(|| Fault::new("No injected socket"))?;
        socket
            .set_nonblocking(true)
            .map_err(|e| Fault::new(e.to_string()))?;
        self.pending = Some(Pending {
            socket,
            destination: destination.clone(),
            callback: callback.clone(),
            offset,
            bytes: vec![0; count],
        });
        Ok(())
    }
    pub(crate) fn trace_roots(&self, roots: &mut Vec<usize>) {
        if let Some(pending) = &self.pending {
            crate::gc::trace(&pending.destination, roots);
            crate::gc::trace(&pending.callback, roots);
        }
    }
    pub(crate) fn is_pending(&self) -> bool {
        self.pending.is_some()
    }
    pub(crate) fn poll(&mut self, heap: &ManagedHeap) -> Result<Option<Value>, Fault> {
        let Some(pending) = &mut self.pending else {
            return Ok(None);
        };
        let count = match pending.socket.read(&mut pending.bytes) {
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                ) =>
            {
                return Ok(None);
            }
            result => result.map_err(|e| Fault::new(e.to_string()))?,
        };
        let Value::ObjectReference(object) = &pending.destination else {
            unreachable!()
        };
        let mut replacement = heap.read_reference(&object.reference)?;
        let Value::Array { elements, .. } = &mut replacement else {
            unreachable!()
        };
        for (slot, byte) in elements[pending.offset..pending.offset + count]
            .iter_mut()
            .zip(&pending.bytes)
        {
            *slot = Value::Byte(*byte);
        }
        object.reference.write(replacement)?;
        Ok(Some(self.pending.take().unwrap().callback))
    }
}

fn library() -> &'static crate::Module {
    static LIBRARY: std::sync::OnceLock<crate::Module> = std::sync::OnceLock::new();
    LIBRARY.get_or_init(|| {
        let output = std::process::Command::new("python3")
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .args(["-c", "import runpy; m=runpy.run_path('docs/experiments/raven-target/collection_library.py'); print(m['build'](m['ROOT'] / 'runtime/System.neoil'))"])
            .output().unwrap();
        assert!(output.status.success());
        let source = String::from_utf8(output.stdout).unwrap() + "\n.function neoCLR.Runtime.TestSocketReceive(arrayref<Byte> destination,Int32 offset,Int32 count,System.Func<Void> callback) -> Void\n.methodimpl InternalCall\n.end\n";
        crate::assemble(&source).unwrap()
    })
}
const TYPES: &str = r#"
.type class SocketCompletion
.field Buffer arrayref<Byte>
.method instance Complete() -> Void
ldarg this
ldfld SocketCompletion::Buffer
ldc.i4 1
ldelem Byte
conv.i4
ldc.i4 72
beq Copied
fault "Socket destination lost or not copied"
Copied:
ldarg this
ldfld SocketCompletion::Buffer
ldc.i4 0
ldelem Byte
conv.i4
ldc.i4 9
beq Untouched
fault "Socket range overwritten"
Untouched:
ldstr "copied"
call neoCLR.Runtime.WriteLine(String)
pop
ldvoid
ret
.end
.method instance Busy() -> Void
ldarg this
ldfld SocketCompletion::Buffer
ldc.i4 1
ldelem Byte
conv.i4
ldc.i4 72
beq Done
call System.Tasks.TaskQueue::get_Default()
ldarg this
delegate.bind System.Func<Void> = instance SocketCompletion::Busy()
call instance System.Tasks.TaskQueue::Post(System.Func<Void>)
Done:
ldvoid
ret
.end
.end
.function Ready() -> Void
ldstr "ready"
call neoCLR.Runtime.WriteLine(String)
pop
ldvoid
ret
.end
.function Launch() -> Void
.local arrayref<Byte> buffer
call System.Tasks.TaskQueue::get_Default()
pop
ldc.i4 3
newarr Byte
stloc buffer
ldloc buffer
ldc.i4 0
ldc.i4 9
conv.u1
stelem Byte
ldloc buffer
ldc.i4 1
ldc.i4 1
ldloc buffer
newobj SocketCompletion
delegate.bind System.Func<Void> = instance SocketCompletion::Complete()
call neoCLR.Runtime.TestSocketReceive(arrayref<Byte>,Int32,Int32,System.Func<Void>)
pop
ldvoid
ret
.end
"#;
const PRESSURE: &str = r#"
ldc.i4 0
stloc index
Again:
ldc.i4 16
newarr Byte
pop
ldloc index
ldc.i4 1
add
stloc index
ldloc index
ldc.i4 100
blt Again
"#;
fn execute(body: &str, options: crate::ExecutionOptions) -> Result<crate::Execution, Fault> {
    execute_types(body, options, TYPES)
}
fn execute_types(
    body: &str,
    options: crate::ExecutionOptions,
    types: &str,
) -> Result<crate::Execution, Fault> {
    let source = format!(
        ".module SocketProbe\n.entry Main\n{types}\n.function Main() -> String\n{body}\nret\n.end\n"
    );
    let app = crate::assembler::read_modules(
        &[crate::assembler::ModuleInput::Source(&source)],
        library(),
    )?
    .remove(0);
    crate::LoadedProgram::with_library(&app, library())?.run(options)
}
#[derive(Debug)]
struct SignalConsole {
    lines: Arc<std::sync::Mutex<Vec<String>>>,
    signal: mpsc::SyncSender<()>,
    cancel: Option<crate::CancellationToken>,
}
impl crate::Console for SignalConsole {
    fn read_byte(&self) -> io::Result<Option<u8>> {
        Ok(None)
    }
    fn write_line(&self, text: &str) -> io::Result<()> {
        self.lines.lock().unwrap().push(text.to_owned());
        if text == "ready" {
            self.signal.send(()).unwrap();
            if let Some(token) = &self.cancel {
                token.cancel();
            }
        }
        Ok(())
    }
}
fn pair() -> TcpStream {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let peer = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    peer.set_write_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    INPUT.with(|input| *input.borrow_mut() = Some(listener.accept().unwrap().0));
    peer
}

#[test]
fn tcp_completion_survives_both_vm_collection_paths_and_empty_queue_wait() {
    for limits in [
        crate::Limits {
            heap_objects: 32,
            ..Default::default()
        },
        crate::Limits {
            array_elements: 128,
            ..Default::default()
        },
    ] {
        let mut peer = pair();
        let (signal, begin) = mpsc::sync_channel(1);
        let lines = Arc::new(std::sync::Mutex::new(Vec::new()));
        let producer = std::thread::spawn(move || {
            begin.recv().unwrap();
            peer.write_all(b"H").unwrap();
            assert_eq!(peer.read(&mut [0]).unwrap(), 0);
        });
        let body = format!(
            ".local Int32 index\ncall Launch()\npop\n{PRESSURE}\ncall System.Tasks.TaskQueue::get_Default()\ndelegate.bind System.Func<Void> = Ready()\ncall instance System.Tasks.TaskQueue::Post(System.Func<Void>)\nldstr \"entry\""
        );
        let result = execute(
            &body,
            crate::ExecutionOptions {
                limits,
                console: Some(Arc::new(SignalConsole {
                    signal,
                    lines: lines.clone(),
                    cancel: None,
                })),
                ..Default::default()
            },
        );
        // Always release an unconsumed injection on an early validation failure.
        INPUT.with(|input| input.borrow_mut().take());
        let result = result.unwrap();
        producer.join().unwrap();
        assert_eq!(*lines.lock().unwrap(), ["ready", "copied"]);
        assert!(result.heap.statistics().reclaimed_objects >= 100);
        assert_eq!(result.heap.statistics().live_objects, 0);
    }
}

#[test]
fn pending_socket_is_closed_on_guest_fault_and_host_cancellation() {
    for cancel in [false, true] {
        let mut peer = pair();
        let (signal, _begin) = mpsc::sync_channel(1);
        let token = crate::CancellationToken::new();
        let body = if cancel {
            "call Launch()\npop\ncall Ready()\npop\nldstr \"entry\""
        } else {
            "call Launch()\npop\nfault \"stop\""
        };
        let fault = execute(
            body,
            crate::ExecutionOptions {
                cancellation: Some(token.clone()),
                console: Some(Arc::new(SignalConsole {
                    signal,
                    lines: Default::default(),
                    cancel: Some(token),
                })),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert_eq!(
            fault.code,
            if cancel {
                crate::FaultCode::ExecutionCancelled
            } else {
                crate::FaultCode::UserFault
            }
        );
        assert_eq!(peer.read(&mut [0]).unwrap(), 0);
    }
}

#[test]
fn socket_completes_while_default_queue_remains_busy() {
    let mut peer = pair();
    let (signal, begin) = mpsc::sync_channel(1);
    let lines = Arc::new(std::sync::Mutex::new(Vec::new()));
    let producer = std::thread::spawn(move || {
        begin.recv().unwrap();
        peer.write_all(b"H").unwrap();
    });
    let launch =
        "call neoCLR.Runtime.TestSocketReceive(arrayref<Byte>,Int32,Int32,System.Func<Void>)\npop";
    let types = TYPES.replace(launch, &(launch.to_string() + "\ncall System.Tasks.TaskQueue::get_Default()\nldloc buffer\nnewobj SocketCompletion\ndelegate.bind System.Func<Void> = instance SocketCompletion::Busy()\ncall instance System.Tasks.TaskQueue::Post(System.Func<Void>)"));
    let result = execute_types(
        "call Launch()\npop\ncall Ready()\npop\nldstr \"entry\"",
        crate::ExecutionOptions {
            console: Some(Arc::new(SignalConsole {
                signal,
                lines: lines.clone(),
                cancel: None,
            })),
            ..Default::default()
        },
        &types,
    )
    .unwrap();
    producer.join().unwrap();
    assert_eq!(*lines.lock().unwrap(), ["ready", "copied"]);
    assert_eq!(result.heap.statistics().live_objects, 0);
}

#[test]
fn vm_rejects_invalid_receive_ranges_and_missing_dispatcher_before_consuming_socket() {
    for (offset, count, dispatcher) in [(-1, 1, true), (1, -1, true), (2, 2, true), (0, 1, false)] {
        let _peer = pair();
        let mut types = TYPES.replace(
            "ldc.i4 1\nldc.i4 1\nldloc buffer",
            &format!("ldc.i4 {offset}\nldc.i4 {count}\nldloc buffer"),
        );
        if !dispatcher {
            types = types.replace("call System.Tasks.TaskQueue::get_Default()\npop", "");
        }
        let fault = execute_types(
            "call Launch()\npop\nldstr \"entry\"",
            Default::default(),
            &types,
        )
        .unwrap_err();
        assert!(
            fault.message.contains(if dispatcher {
                "Invalid socket range"
            } else {
                "default TaskQueue"
            }),
            "{fault}"
        );
        assert!(INPUT.with(|input| input.borrow_mut().take()).is_some());
    }
}

#[test]
fn pending_worker_does_not_prevent_socket_dispatch() {
    #[derive(Debug)]
    struct Signals {
        ready: mpsc::SyncSender<()>,
        copied: mpsc::SyncSender<()>,
    }
    impl crate::Console for Signals {
        fn read_byte(&self) -> io::Result<Option<u8>> {
            Ok(None)
        }
        fn write_line(&self, text: &str) -> io::Result<()> {
            if text == "ready" {
                self.ready.send(()).unwrap();
            }
            if text == "copied" {
                self.copied.send(()).unwrap();
            }
            Ok(())
        }
    }
    let mut peer = pair();
    let (ready, start) = mpsc::sync_channel(1);
    let (copied, delivered) = mpsc::sync_channel(1);
    let token = crate::CancellationToken::new();
    let stop = token.clone();
    let producer = std::thread::spawn(move || {
        start.recv().unwrap();
        peer.write_all(b"H").unwrap();
        let completed = delivered.recv_timeout(Duration::from_secs(5)).is_ok();
        stop.cancel(); // Also bounds a broken arbitration loop.
        completed
    });
    let types = TYPES.to_string()
        + "\n.function Spin(String input) -> String\nAgain:\nbr Again\n.end\n.function WorkerDone() -> Void\nldvoid\nret\n.end\n";
    let body = "call Launch()\npop\ndelegate.bind System.Func<String,String> = Spin(String)\nldstr \"\"\ncall neoCLR.Runtime.StartWorker(System.Func<String,String>,String)\ndelegate.bind System.Func<Void> = WorkerDone()\ncall neoCLR.Runtime.NotifyWorker(Int32,System.Func<Void>)\npop\ncall Ready()\npop\nldstr \"entry\"";
    let fault = execute_types(
        body,
        crate::ExecutionOptions {
            limits: crate::Limits {
                instructions: usize::MAX,
                ..Default::default()
            },
            cancellation: Some(token),
            console: Some(Arc::new(Signals { ready, copied })),
            ..Default::default()
        },
        &types,
    )
    .unwrap_err();
    assert!(
        producer.join().unwrap(),
        "socket completion starved behind worker"
    );
    assert_eq!(fault.code, crate::FaultCode::ExecutionCancelled);
}
