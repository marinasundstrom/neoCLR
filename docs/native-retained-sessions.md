# Retained native sessions — private experiment, 2026-10-09

The stateless HTTP host can recover by discarding an invocation. A stateful host
instead needs to retain guest objects between calls, prevent accidental heap reset,
and release exactly what it owns at shutdown. This slice adds a private C session
around the existing strong roots and generated zero-argument callback dispatcher.
It is not yet an application-facing Raven hosting API or a stateful HTTP showcase.

## Ownership and transitions

Open a zero-initialized session after bootstrap returns, on the context's owning
thread. A null anchor root prevents another Main/entry reset even when the session
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
  across calls/GC, checks wrong-shape rejection and fault dispatch, then closes and
  restarts. The existing Rust test also compares its fault with the interpreter.

The generated fixture discovers its callback through private heap inspection solely
to isolate codegen, as the prior callback test did. Real applications need an explicit
bootstrap/export handoff; scanning the heap is not an application hosting mechanism.
That handoff and a retained-state HTTP consumer are the next integration work.
The website's existing statement that general reusable hosting is unfinished remains
accurate; there is no new public API reference entry for this private C experiment.

[Retained validation evidence](native-retained-session-validation.json) records the
local passing gates and
[Action 37972822432](https://github.com/marinasundstrom/neoCLR/actions/runs/37972822432)
at `c77e2b07`: both lifecycle and generated-callback consumers pass on macOS ARM64
and Windows x64 with identical fault output. All 21 macOS and 25 Windows downloaded
artifact hashes are verified; 15/16 tracked source inputs respectively match the
commit, allowing Windows checkout line endings.
