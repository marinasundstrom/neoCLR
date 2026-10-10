# Retained native HTTP session — development experiment

One Raven `HttpSession` owns a listener and mutable request counter. The native host
boots it once, adopts its exported callback, invokes it three times and collects
between exchanges. Each response uses the same guest counter: `Request 1`,
`Request 2`, `Request 3`. The interpreter runs the same exchange method through an
ordinary async Main. This demonstrates retained state, not concurrent requests,
hot reload, a public host API or a throughput result.

Build with a matching development class-library bundle (including the Task factories):

```sh
python3 scripts/build-native-project.py --profile http \
  --bootstrap-root Bootstrap \
  --project docs/experiments/retained-http/Native.rvnproj \
  --bundle /absolute/path/to/development/bundle \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --output target/retained-http-build
```

On Windows x64 use `--profile windows-http` and the `.exe` backend from a configured
MSVC shell. The executable prints one loopback listening port, serves three requests
on that listener, then exits. `GET /count` returns the count; `GET /fault` raises a
terminal guest fault. The host does not admit another exchange after that fault.

`--bootstrap-root` is a private HTTP host option. The build tool resolves exactly
one assembly-level function from the actual AOT inventory, accepting the native
name or the existing Raven CLI bridge `F_<UTF8 hex>` spelling. It records the
resolved name rather than guessing the assembly hash prefix. Selection and exact
`fn<Void>` admission remain the backend's responsibility. This temporary lookup
belongs to the project build tool; a native export manifest should replace it.
Raven semantics, Runtime Contract configuration and CLI emission are unchanged.
The ordinary project entry and all builds without this option retain their behavior.

The exchange uses `await`. `Dispatch` is the one deliberate synchronous adapter: the
current C callback ABI returns unit, so it starts `Exchange` and the host drives the
existing queue/socket completion loop. Idle queue plus no pending socket operations
is sufficient for this closed consumer; it is **not** a general task-completion
contract. Exporting/observing an async result is still needed for arbitrary tasks,
external completion sources and scheduling integration.

The socket and task scopes live for the whole session. Bootstrap creates no task
queue or pending operations, allowing exclusive callback adoption. On success or
fault, the host renders diagnostics, tears down services and then closes the session,
requiring no roots/frames and an empty heap before releasing its backing memory.
Native shutdown closes sockets through the host scope; interpreter Main calls
`Close` normally. Neither path promises guest finalizers on a terminal fault.
Service callback faults are terminal in this host even though those callbacks enter
through the existing service dispatcher rather than the session-owned callback API.

This reuses the [.NET/GCHandle ownership comparison](../../native-retained-sessions.md)
and [HTTP platform comparison](../../native-http-parity.md). It closes a neoCLR host
integration gap: application state outlives an individual HTTP exchange. Compared
with stateless entry reset, it preserves identity and avoids reconstructing that
state, but requires explicit root/resource ownership and discards the entire session
after faults. Three serial requests and a polled single-thread host do not establish
performance or concurrency advantages over .NET hosting.

## Qualification

```sh
python3 scripts/validate-retained-http.py \
  --bundle /absolute/path/to/development/bundle \
  --output target/retained-http-validation
```

The Windows action obtains the pinned compiler bundle and rebuilds current libraries.
Both platforms run the same cases: three requests, fragmented requests and a handler
fault after one success. Native and interpreter responses, completion counts, exit
codes and guest fault frames must match. The interpreter additionally records its
outer DrainEntryTasks/async-entry frames, which do not exist for a quiescent native
host dispatch. The gate checks exactly those two expected frames against the
artifact entry name and retains both complete traces. The native executable runs alone without managed-runtime
search paths. The host checks session cleanup; the test checks listener shutdown.
Reports retain source, toolchain and artifact hashes. See
[qualification evidence](../../native-retained-http-validation.json). All three cases
pass locally on macOS ARM64 and in Windows x64
[Action 37976713202](https://github.com/marinasundstrom/neoCLR/actions/runs/37976713202)
at `84b6e425`; complete native/interpreter results match across OSes. The downloaded
Windows evidence verifies 1,435 artifact hashes and 37 tracked source inputs.

Windows ARM64, general async export completion, broader cancellation/disconnect
recovery, persistent-host supervision and code-generation lifetimes remain open.

Investigation also exposed a [deferred async pattern-local compiler candidate](async-pattern-local.md).
The final Main uses a normal lexical binding; the pattern-hoisting issue is not fixed
by this hosting slice.

## Matching interpreter qualification (2026-10-10)

The retained, ordinary server and client project validators build the current
checkout’s release interpreter, record its hash and check it remains unchanged
through the run. Source-built development libraries can require services absent
from the published bootstrap VM (for example StringHashOrdinal). The bundle still
supplies the selected compiler/libraries and primitive bootstrap; its older VM is
not used as the development behavior reference. This repairs the comparison harness,
not HTTP runtime semantics or the published bundle.

[Matching-interpreter evidence](matching-interpreter-validation.json) records passing
macOS native/interpreter runs: three retained-session cases, five ordinary server
cases and fourteen client/factory/reuse cases.

[Verified Windows matching-interpreter evidence](windows-matching-interpreter-validation.json)
now records the same 3 retained, 5 server and 14 client cases at `8bf89b57`.
All pass; local verification checked 4,729 downloaded files and the tracked source
inputs (allowing checkout line endings). Isolated executable hashes match the build
outputs; the reused-client host was linked directly in its isolation directory.
The native executables depend on KERNEL32/WS2_32. These runs predate the init-accessor
compiler update and qualify the matching-interpreter repair specifically.
