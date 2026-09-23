# neoCLR Preview 9 — Async and Tasks

Version: **0.1.0-preview.9** · Intended tag: **v0.1.0-preview.9**.
Release scope selected 2026-09-23. Candidate validation is in progress; publication
has not occurred. The final manifest must record the exact revision and validation
results. Earlier e9bb28a evidence is preparation evidence, not certification of this
versioned candidate.

neoCLR is an experimental application platform. This preview adds Task/Promise,
Raven async/await and isolated workers, with small runnable examples showing
completion, composition, expected errors and cancellation. APIs remain experimental.

## Try the async samples

Use the matching macOS arm64 runtime bundle, experimental Raven SDK and VSIX.
The compiler, importer, MSBuild and language server require .NET SDK
11.0.100-rc.1.26425.128; the neoCLR runtime and guest programs do not require .NET.
Configuration and validation use Python 3.9 or later.

Extract the runtime and SDK separately. From the runtime bundle:

```sh
python3 configure.py --sdk /absolute/path/to/extracted/raven-sdk
python3 tools/verify_async_workbench.py --bundle . --sdk /absolute/path/to/extracted/raven-sdk --report async-workbench.json
code msbuild-demo
```

Install the matching VSIX in VS Code. Copy a bundled tools/samples/library-async*.rvn
or library-task*.rvn sample over msbuild-demo/Main.rvn, save it, and use the
neoCLR: Run (MSBuild) task. The ordinary Raven toolbar is not the neoCLR execution
path. The bundled README includes direct neoIL and application/library project demos.

## Included behavior

- System.Tasks.Task<T> represents completion and a result. Promise<T> owns completion
  and cancellation; the first terminal transition wins. Task.State and Task.Outcome
  distinguish completion from cancellation. Expected errors remain Result values;
  Task has no exception-based fault state.
- Named Raven async functions and await use compiler-generated heap state machines.
  Cancellation propagates through await. Map, Then and MapResult compose operations;
  `(await input)?` propagates Result errors with the current Raven precedence rules.
- The invocation default TaskQueue dispatches callbacks before return. Ordinary
  samples need no manual pumping. Explicit queues remain available; a pending
  Promise alone does not keep an invocation alive.
- System.Threading.Thread.Start and ThreadPool.Queue run static string-to-string
  callbacks in isolated guest heaps. They return Task<string>. The pool has two
  threads, submissions are bounded to 64 per invocation, and nested workers are
  rejected. Ordinary completion uses queued blocking joins in submission order.
- Worker result text and captured output share a default 1 MiB quota per worker.
  Host invocation cancellation is cooperative; output already delivered is retained.
  Guest operation cancellation tokens are not provided.
- The website adds an API overview and DocFX reference under /docs/, with descriptions
  for the main async APIs. It is an initial reference, not exhaustive member coverage.

Additional Iterable operators include Any, All, Count, seeded Fold, Take, Skip,
Concat and FlatMap. Option/Result gain transformations, recovery, branch actions,
conversions and nested Option flattening. These use ordinary library contracts;
iterator disposal on normal paths does not imply fault-unwind cleanup.

## Other changes and migration from Preview 8

Rebuild applications and dependent libraries with the matching compiler, reference
assembly, importer and runtime library. Do not mix old generated artifacts with
this preview's toolchain.

- Queries use Filter and Map instead of Where and Select, without legacy aliases.
- File and Path APIs and file error types move from System.IO to System.Storage.
  ConsoleReadError moves to System. Local file operations remain synchronous.
- System.Error, Error.FromMessage and its Message property are removed. Use strings
  or domain-specific Result error types. Result.Error remains a union case. Low-level
  artifacts using the error instruction or retired Error intrinsic must be rebuilt.
- Rust embedders must account for TaskDispatch and IsolatedWorkers service variants
  and Limits.worker_result_bytes. Removed Error variants no longer exist in the host
  value, type, instruction and service APIs.

For intermediate development builds, TaskCompletionSource becomes Promise and
TrySetResult becomes Complete; the Task namespace is System.Tasks. These changes
are not migrations from a previously published Task API. Raven unit still maps to
System.Void. See the candidate's async-preview-migration.md for the detailed inventory.

## Limits and next direction

Workers do not share guest objects or captured closures. They cannot use console
input or native P/Invoke. Worker faults terminate execution; cancelling a Promise
does not stop its worker. Teardown joins workers and may wait for blocking host work.
Await inside for loops is rejected pending suspension-aware iterator cleanup.

The nonblocking worker notification adapter, Byte Copy, UTF-8 chunk decoder and
JSON examples are experiments, not new supported Streams, Encoding or JSON APIs.
Networking, general asynchronous I/O, preemption and runtime suspension are not
included. Raven source debugging on neoCLR is not claimed.

After release, Streams, Storage and Encoding precede networking. System.Concurrency
is the selected future namespace for concurrency, including explicit optional Thread
support. A Task.Run-style API will provide general concurrent submission through
platform-specific execution, potentially Web Workers on WebAssembly. These future
APIs are not available in this candidate.

## Validation and assets

Pending: exact-candidate six-job Linux/macOS/Windows source matrix, full local source
archives, final artifact manifest/hashes. All six extracted Task contract probes pass.
The candidate passes the interactive VSIX build/run and Task type-hover check in a
separate profile, and all eight Async Workbench cases.
Binary validation targets macOS arm64 only. Final release files and checksums must be recorded in the manifest. The changelog continues after release with a fresh Unreleased section.
