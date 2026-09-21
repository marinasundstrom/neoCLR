# Isolated workers: development PoC

This is development work after Preview 8. It starts [System.Tasks](task-contracts.md) work on isolated OS threads; it is not
a shared-object threading model.

| API in System.Threading | Current behavior |
| --- | --- |
| Thread.Start(callback, input) | Start a dedicated OS thread; return Task<string>. |
| ThreadPool.Queue(callback, input) | Submit to an invocation-owned pool of two reusable OS threads; return Task<string>. |

The callback is a static `Func<string, string>` with no captured receiver. Named
functions are the clearest current spelling. Input and result text are owned copies;
guest objects, byrefs, pointers and Task state never cross threads. Every job has
its own interpreter, managed heap, pointer heap and limits, even when an OS pool
thread is reused. Heap isolation is not a sandbox: files and other host resources
remain external shared resources.

The tested sample is [library-workers.rvn](experiments/raven-target/samples/library-workers.rvn).
Both APIs require an active TaskQueue.Run/Drain scope. Start submits work before
returning a pending Task; the caller can use ordinary `await`. Completion is posted
to the caller's queue, which waits for the worker result and completes its local
TaskCompletionSource. Task objects remain entirely in the caller's heap. Pumping
the queue may block on a worker; completions are processed in submission order,
not readiness order. This is an explicit PoC limitation, not a nonblocking scheduler.
There is no context capture, affinity, thread naming, priority or guest cancellation
API. A worker may create its own TaskQueue for local async work, but cannot transfer
its Task back across the isolation boundary.

At most 64 jobs may be submitted in one invocation. Worker creation inside a worker
is rejected. A callback Fault becomes a terminal Fault when the queue processes completion; it is not a Task
failure state or a Result conversion. Captured callbacks fault. Bootstrap handles are invocation-scoped and single-use. Unjoined work is cancelled cooperatively and all OS threads are joined
when the owning invocation exits. Blocking host operations cannot be interrupted,
so cleanup may wait for them. Parent cancellation is observed while waiting during completion waits.

Worker console input is unavailable. Output is buffered and forwarded during completion, in
submission order, rather than interleaved with the parent's console. Output from unjoined
work is discarded. The parent debugger does not attach to worker interpreters.
An immutable metadata snapshot is copied per job; this PoC makes no throughput or
allocation-performance claim. Native P/Invoke remains disabled in workers.

## Comparison, decision and open design

Primary sources reviewed 2026-09-21: .NET's
[ThreadPool.QueueUserWorkItem](https://learn.microsoft.com/en-us/dotnet/api/system.threading.threadpool.queueuserworkitem?view=net-10.0)
queues callbacks with optional state, and Rust's
[thread Builder](https://doc.rust-lang.org/std/thread/struct.Builder.html)
creates OS threads with owned, transferable inputs. These are distinct library and
host-runtime contracts; neither requires neoCLR to copy .NET's object-sharing model.

The author selected isolated workers for the first PoC. Shared-object callbacks
would be more familiar to .NET developers but require synchronized slots, GC and
completion. Isolation provides actual parallel work while retaining the current
single-threaded object model. Its cost is a narrower callback/data contract and
metadata copying. Strings are a deliberately small exchange format, not a decision
to serialize every future API as text. Typed transferable values, Result payloads,
nonblocking completion dispatch, configurable pooling, resource accounting and better worker
debugging remain open. OS threads and the provisional compiler async state machine
are independent mechanisms; neither defines a permanent suspension ABI.

## Validation

Run `cargo test --locked --test workers --test tasks --test runtime_services` for
runtime dispatch, handle boundaries and Task regressions. Run the source scenarios
with matching development artifacts:

```sh
python3 docs/experiments/task-contract/verify_workers.py /path/to/Demo.rvnproj \
  --bridge /path/to/Probe.dll --system /path/to/System.neoil \
  --runtime /path/to/neoclr
```

These cover awaiting dedicated and pooled results, multiple queued jobs, captured
callback rejection and the required TaskQueue scope.

The author intends to shape this API further with Task integrated from the outset.
Potential public synchronous counterparts remain a future design question; the
internal blocking completion wait is not a settled synchronous API design.
