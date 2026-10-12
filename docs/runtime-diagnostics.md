# Attachable runtime diagnostics

Author direction, 2026-10-12: debugger and profiler are separate external tools that
can attach to a running runtime. This is the intended architecture, not implemented
attach support. Tool names and protocol encodings remain provisional.

## Current implementation and next boundary

The [terminal debugger](debugger.md) runs inside the interpreter command. The
[native profiler](experiments/runtime-async/native-profiling.md) rebuilds a native
benchmark host and samples it using macOS tools. Neither supplies a general runtime
attach service. Managed StackTrace captures current guest frames; it is not a
remote debugger or a causal async stack.

Provide an opt-in runtime diagnostics service and two independently usable clients:

| Component | Responsibility |
| --- | --- |
| Runtime service | Process identity, capability negotiation, safe snapshots and execution-control hooks |
| Profiler client | Counters, allocation/GC measurements, sampled stacks and bounded event capture |
| Debugger client | Breakpoints, pause/resume, stepping, frame and variable inspection |
| Shared client library | Discovery, transport, versioning, method/module identities and symbol resolution |

Profiling must work without a debugger session. Debugger control must not depend on
starting a profiling capture. Shared transport does not imply identical commands or
synchronization: live profiling and stopped-state inspection have different needs.
Runtime internals own safepoints, roots and suspension; clients do not interpret
scheduler queues or retain raw pointers into collectable objects.

Interpreter, JIT and AOT hosts should expose the same conceptual contracts and
explicitly report supported capabilities. AOT instrumentation and symbol availability
are build capabilities; attaching cannot enable code that was compiled out. This
adds no managed Task/Scheduler dependency and requires no language-specific API.

## Baseline and alternatives

Primary documentation reviewed 2026-10-12:

- [.NET diagnostic ports](https://learn.microsoft.com/en-us/dotnet/core/diagnostics/diagnostic-port)
  provide an IPC service used by separate diagnostics tools. Desktop transports are
  Unix domain sockets and Windows named pipes. This is the baseline for external
  profiling clients, not evidence that every debugger operation uses that protocol.
- [Node inspector](https://nodejs.org/api/inspector.html) exposes debugger and profiler
  functionality through inspector protocol domains. It demonstrates that separate
  clients can share runtime infrastructure without merging tool responsibilities.
- [Go net/http/pprof](https://pkg.go.dev/net/http/pprof) exposes profiling handlers for
  external pprof tools. It is an alternative transport model; an application HTTP
  server should not be a dependency of neoCLR diagnostics.

Keeping only the current in-process debugger and benchmark harness is simpler but
cannot diagnose an already-running HTTP service. OS sampling remains useful, but
alone cannot expose managed ownership, runtime suspension or precise heap counters.
The selected direction follows .NET's separation of runtime instrumentation and
external tools. An opt-in endpoint differs from CoreCLR's default-on desktop port:
this fits neoCLR's optional runtime services, at the cost of requiring diagnostics
to be enabled before attaching. No speed or tooling-parity improvement is claimed.

Provisional transport: local Unix sockets on macOS/Linux and named pipes on Windows,
restricted to the runtime's user. Remote networking, stable wire format and universal
startup-suspend support are not selected in this slice. Transport and protocol
compatibility, synchronization, buffering and instrumentation all have measurable costs.

## First attach milestone

Build a read-only profiler attachment to a long-running native HTTP showcase:

1. Enable diagnostics in the host, identify the process instance (not PID alone),
   negotiate protocol version and available counters, then attach after startup.
2. Read coherent GC/allocated-payload snapshots while requests continue; detach and
   reconnect without restarting the application or resetting its counters.
3. Reject malformed requests, incompatible versions and unsupported capabilities;
   bound message sizes and buffering. A slow or disconnected reader must not block
   request handling or keep guest objects alive.
4. Retain an executable client/host test, HTTP response checks and disabled/enabled
   overhead measurements. Qualify each host and execution mode explicitly rather
   than infer portability from a macOS proof.

Existing native GC diagnostics are thread-local. The service must obtain owner-thread
snapshots or safely published aggregates; reading its own thread-local storage is
wrong and reading another thread's mutable counters directly races. Define epoch,
thread/context scope and stale-snapshot behavior before implementing aggregation.
Clients use session baselines; the private benchmark reset is not an attach command.
Live payload after collection is not instantaneous retained heap size, and histogram
quantiles are bounds rather than exact pause durations.

Subsequent debugger attachment needs its own gate: cooperative stop/resume across
threads, suspended logical frames, frame/IL/source mapping, breakpoint ownership and
safe detach while paused. A disconnect must not strand a runtime that the session
paused; explicit startup wait remains a separate host policy. Multiple readers,
exclusive debugger control, process exit and reattachment require lifecycle tests.
Retention paths and causal async stacks remain further work. The current measurement
slice does not implement any of these attach capabilities.
