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
