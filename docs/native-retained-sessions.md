# Retained native sessions — private experiment, 2026-10-09

The stateless HTTP host can recover by discarding an invocation. A stateful host
instead needs to retain guest objects between calls, prevent accidental heap reset,
and release exactly what it owns at shutdown. This slice adds a private C session
around the existing strong roots and generated zero-argument callback dispatcher.
It is not yet an application-facing Raven hosting API or a stateful HTTP showcase.

## Ownership and transitions

Open a zero-initialized session on the context's owning thread, or adopt the
explicit bootstrap handle described below. A null anchor root prevents another Main/entry reset even when the session
has no retained objects. A context may have only one session. The session storage,
context, heap buffer and compiled image must remain alive and must not be copied.

Retain up to 16 live guest allocations with opaque handles. A callback handle also
keeps its captured object graph alive; collection between calls preserves identity
and mutations. Only callbacks retained by that session may be dispatched. The
existing generated dispatcher still checks the exact selected fn<Void> shape.
Wrong-thread, copied-session, stale/foreign-handle and active-frame operations are
rejected. Collection and shutdown cannot reenter a running callback.

A guest fault poisons further session dispatch. State mutations before the fault
are not rolled back, and manually clearing diagnostics does not restore admission.
The host may inspect/render the fault and collect while its diagnostic roots remain
live, then explicitly close. This is stricter than stateless HTTP recovery because
the remaining mutable graph may no longer satisfy application invariants.

Close first checks that there are no guest frames and that the context's complete
host-root set is exactly the session's owned set. Outstanding roots from sockets,
tasks or another service reject close without releasing session state; shut those
services down and retry. After heap validation, close releases its handles and
discards diagnostics and the entire owned heap, including the entry-scoped intern
pool. Old handles remain invalid, and a new entry/session can then be admitted.
Close does not run guest finalizers or implicitly close unregistered OS resources.

## Explicit bootstrap handoff

The private AOT option `--native-host-bootstrap` requires `--native-gc` and selects
an alternative C export, `neoclr_bootstrap_callback_v1(int32_t, uint64_t *, context *)`.
The selected static root must take zero arguments or one Int32 and return exactly
`fn<Void>` (no callback arguments, inhabited Void result). Ordinary entry compilation
continues to reject function-valued roots; the bootstrap object does not export
`neoclr_entry_v4`. This is not an async Main replacement or a project-profile option.

After successful guest execution, the export creates a strong root without a GC
point and transfers its opaque handle to the caller. Collection is safe immediately
on return, before adoption. `neoclr_session_adopt_v1` transfers that same handle
into a zero-initialized session and adds its reset-prevention anchor. It requires
that the callback be the context's only host root and that guest frames have unwound.
On failure, the caller still owns the handle and may retry or release it. On success,
the caller must not independently release or replace the session-owned handle.

Bootstrap leaves its output untouched on guest faults, null output/context pointers, null
callbacks or root-table exhaustion. Existing context roots/frames reject bootstrap
before diagnostics or heap state are reset. Host admission/ownership failures return
a status without manufacturing a guest fault. Successful admission starts a new heap
and clears old diagnostics, as an entry does. A failed bootstrap may leave guest
allocations or diagnostics to inspect/discard; it exports no handle. As with the
existing entry ABI, trusted callers supply valid, non-aliasing output/context storage.

This extends the existing GCHandle ownership comparison below: returning a borrowed
heap pointer would require a fragile no-collection interval. Returning an opaque
strong handle makes lifetime transfer explicit, at the cost of a temporary root and
an adoption step. Arbitrary export shapes, asynchronous factories, code replacement
and thread migration remain deferred. Existing Function metadata and native callback
lowering are reused; Raven syntax, CLI bridge encoding and Runtime Contract settings
are unchanged. The native backend owns this temporary export ABI.

## Comparison and tradeoffs

.NET's [GCHandle](https://learn.microsoft.com/en-us/dotnet/api/system.runtime.interopservices.gchandle)
provides explicit roots for native consumers, and
[Free](https://learn.microsoft.com/en-us/dotnet/api/system.runtime.interopservices.gchandle.free)
must be called once for an allocated handle (Microsoft documentation reviewed
2026-10-09). neoCLR already has private strong roots; the new work groups their
ownership at a host lifetime boundary rather than copying .NET's API surface.

Leaving callers to coordinate raw handles would preserve flexibility but makes
premature reset and incomplete teardown easier. An automatically recovering session
would need a rollback or application-consistency contract that does not exist.
This bounded, fail-closed session provides explicit ownership and retryable shutdown
admission at the cost of a 16-root limit, one-thread affinity and whole-session
discard after guest faults. No performance advantage over CLR hosting is claimed.

The implementation is host/GC policy only: no Raven semantic, Runtime Contract,
CLI bridge, library or native metadata encoding change. The C structures and function
names are experimental, not a stable public ABI. Future runtime-owned activations
must replace thread-affine callback ownership before suspension migration, green
threads or reload can safely retain this state. Code/image lifetime remains the host's
responsibility; this slice does not make callbacks portable across compiled images.

## Validation and remaining integration

`scripts/validate-native-session.py` runs two matching gates on macOS and Windows:

- A lifecycle kernel checks retained identity through collection, wrong-thread and
  copied-owner rejection, bounded/atomic retention, reentry, foreign service roots,
  fault diagnostic roots, poisoned dispatch, stale handles and intern-pool shutdown.
- A generated native consumer binds static and captured callbacks, mutates a counter
  across calls/GC, checks explicit callback handoff and fault dispatch, then closes and
  restarts. The existing Rust test also compares its fault with the interpreter.

The generated fixture now receives its callback through the explicit export, without
heap scanning. It collects before adoption and checks reset rejection, foreign-root
admission, stale handles, bootstrap faults and atomic root-quota failures at export
and adoption. Focused compiler tests reject missing/duplicate opt-in and wrong root
shapes, and inspect-mode reporting records the capability. The local macOS handoff
gate passes; cross-platform evidence for this extension is recorded separately in
[native-bootstrap-handoff-validation.json](native-bootstrap-handoff-validation.json).
A retained-state HTTP consumer remains the next integration work.
The website's existing statement that general reusable hosting is unfinished remains
accurate; there is no new public API reference entry for this private C experiment.

[Retained validation evidence](native-retained-session-validation.json) records the
local passing gates and
[Action 37972822432](https://github.com/marinasundstrom/neoCLR/actions/runs/37972822432)
at `c77e2b07`: both lifecycle and generated-callback consumers pass on macOS ARM64
and Windows x64 with identical fault output. All 21 macOS and 25 Windows downloaded
artifact hashes are verified; 15/16 tracked source inputs respectively match the
commit, allowing Windows checkout line endings.
