# Provisional runtime GC attachment

This development tool reads GC snapshots from a running native runtime. It does not
rebuild, restart, suspend or collect the application. The endpoint and client are
experimental and are not included in installed bundles. The goal is useful runtime
integration during development; standalone packaging and the wire protocol are not
settled architecture. See the [design comparison](../../runtime-diagnostics.md).

## Build and use

Compile the native host/collector with `-DNEOCLR_GC_PROFILE -DNEOCLR_DIAGNOSTICS`
and link `attach.c` with pthread support. This proof uses Unix sockets and is
qualified on macOS ARM64 only. Windows transport, interpreter integration, installed
CLI integration and process-wide multi-owner aggregation remain open.

Create a private directory (mode 0700) and launch the host with
`NEOCLR_DIAGNOSTICS_DIRECTORY` set to its absolute path. The host creates
`neoclr-<pid>.sock` there, mode 0600. Without the environment variable, the endpoint
and worker are disabled. Without the compile flags, collector attachment hooks and
profile measurements are absent. An explicitly configured but unusable endpoint
exits with status 78; existing socket paths are never replaced.

From a separate terminal:

```sh
python3 tools/profiler/neoclr-profiler.py \
  --socket /tmp/my-private-diagnostics/neoclr-1234.sock --count 10 --interval 0.2
```

The client emits one JSON object per snapshot and reconnects for each observation.
It checks runtime identity across capture: PID plus a random instance nonce. Normal
exit removes the socket; abrupt process death may leave a stale file. Create a new
private directory for a new launch rather than treating the file as proof of liveness.
No remote TCP listener, process enumeration or debugger commands are provided.

## Snapshot contract

The only accepted request is `NEOCLR/1 SNAPSHOT` followed by newline. Other versions
or commands return `unsupported-request`. Requests and responses are bounded to
64 and 4096 bytes; read/write deadlines prevent a stalled client monopolizing the
service indefinitely. Readers are served sequentially; simultaneous attachment is
not a throughput target. Local private-directory access is the trust boundary.

The owner thread publishes a copy of its scalar counters after a collection attempt,
only when requested. Publication uses a nonblocking mutex attempt; the runtime never
waits for a reader. The diagnostics thread copies the published state under the mutex,
then formats/sends it outside the lock. It never reads guest memory or live TLS owned
by another thread. This creates no managed roots and never resets counters.

Each response identifies version, capability (`gc-snapshot`), instance and scope
(`runtime-owner-thread`). `available=false` means no collection snapshot yet.
`fresh=true` means a newer boundary was published after this request. After 100 ms
without a new boundary the service returns the cached snapshot with `fresh=false`,
a sequence, monotonic timestamp and age. The profiler must not infer zero allocation
while the owner is idle or running without collection boundaries.

Counters include allocations/reclamation, attempted/successful/failed collections,
requested payload, post-collection live payload/blocks, arena/live peaks, total/max
collection duration and 64 logarithmic pause buckets. Bucket i covers
[2^i, 2^(i+1)-1] ns; bucket zero also includes zero. Live gauges refer to the last
successful collection; maxima across contexts are not summed simultaneous heaps.
Scope is the thread that initialized the host, not all runtime threads. Public
process-wide counters will need a separate aggregation contract. The benchmark-only
reset API must not be used during an attachment session.

## Focused checks

`test_attach.py` builds a tiny native host with undefined-behavior/bounds sanitizers.
It checks absent/stale/fresh snapshots, collection accounting, CLI reconnection,
unsupported versions/commands, overlong and stalled requests, reader disconnect,
normal exit cleanup and disabled/invalid configuration. Set `CC` and `CFLAGS` to
select a matching compiler/SDK. AddressSanitizer on the local macOS 27/Xcode SDK
combination hung inside sanitizer startup before main; this is not an ASan pass.

## HTTP showcase qualification — 2026-10-12

[Recorded validation](http-validation.json) compiles `HttpProfile.rvn`, a repeated
version of the existing HttpServer greeting showcase, with Raven runtime async. It
builds matching native hosts and qualifies AOT and JIT. Both execute 128 loopback
requests with exact status/body checks and clean host teardown. No application state
machine, profiler-triggered collection or debugger session is required.

The external client attaches twice after the server has already handled requests,
reading four snapshots per session. All 16 snapshots were fresh and internally
consistent. AOT allocation counts advance from 5,813 to 8,802; JIT from 5,218 to
7,942. HTTP progress is asserted during each capture: AOT requests 9→13 and 13→17;
JIT 7→11 and 11→15. A stalled diagnostic socket also permits continued HTTP traffic.
Normal exit removes the endpoint. These checks qualify this native host, not every
runtime configuration or installed CLI.

Reproduce with a passing native showcase recipe and its matching compiler/bundle:

```sh
python3 docs/experiments/runtime-diagnostics/validate_http.py \
  --recipe docs/experiments/runtime-async/collector-walk-showcase.json \
  --output target/diagnostics-http-new
```

The output contains the compiled server, JIT host, selection report, commands and
input hashes. To inspect manually, launch its `server` with a private diagnostics
directory, then run the profiler command above against its PID. The server prints
its loopback port and exits after 128 `GET /greeting` requests. Keeping this bounded
makes teardown checks repeatable; it is a development consumer, not a production host.

Three alternating AOT rounds also compare identical guest code with no instrumentation,
with instrumented counters but no endpoint, and with an active external reader:

| Configuration | Median time for last 124 requests | Round range |
| --- | ---: | ---: |
| Uninstrumented | 2.162 s | 2.091–2.361 s |
| Instrumented, unattached | 2.154 s | 2.142–2.447 s |
| Attached | 2.373 s | 2.161–2.401 s |

The first four requests precede timing and attachment; process startup/JIT compilation
is excluded. The workload uses serial connections, no parallel load or tail-latency
analysis. Noise and overlapping ranges prevent isolating attachment cost from this
small run. An earlier local run showed a different relative ordering of instrumented
variants. Treat these as exploratory overhead data, not a negligible-overhead claim.
Future tuning should separate timer/histogram cost, publication frequency and transport
CPU time under a longer matched workload. Collection pacing is unchanged.
