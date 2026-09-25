# Managed network cancellation

Development networking/web slice 4, 2026-09-25. The DNS/socket provider prerequisite
for HTTP forwarding. Compare [cancellation contracts](../../cancellation-design.md)
and the existing [.NET socket cancellation baseline](../socket-api/README.md).

```sh
python3 docs/experiments/network-cancellation/verify.py \
  --toolchain-root /path/to/matching/neoclr-bundle \
  --runner target/release/examples/measure_async
dotnet docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll --network-budget-checks
```

Use a current native runner, bridge, reference assembly and assembled System library.
The verifier builds in a temporary directory and runs with a 256-object GC budget.
Only localhost resolution and an ephemeral loopback listener are used. The fixture
checks pre-cancellation before invalid argument admission for all connect forms,
DNS, accept and transfers; pending DNS/accept/send/receive cancellation; unchanged
receive storage; listener/connection reuse; registration removal after successful
DNS/connect/receive; source disposal without cancelling accept; and zero-size receives cancelled before polling or preserved after completion. The bridge probe
checks all eight public token overloads, invalid parameter rejection and private
shared-deadline visibility. Native pending-connect cancellation remains covered by
the existing owner-controlled Rust test; the managed fixture does not assume whether
an OS reports loopback connection establishment immediately or asynchronously.

The callbacks deliberately inspect IsCancelled before reading a result. They expose
provider acknowledgement ordering directly; this is a contract fixture, not the
recommended shape of an ordinary HTTP application. HTTP forwarding is not implemented
by this slice. Website build and cross-platform matrices are skipped as directed.

## Checked outcome

On 2026-09-25 the focused managed fixture passed on Darwin arm64: 343 allocations,
nine collections, peak 72 live objects and zero live objects at invocation teardown.
All eight public token signatures, invalid-parameter rejection and private deadline
visibility checks passed. Full library regeneration and source snapshot validation
passed; the test bundle equals the regenerated library. API reference snapshot checks
passed without a website build. This is not cross-platform or general async evidence.

## Deferred compiler/bridge observations

The first large async version encountered several target-toolchain limitations:
a generic Settled<T> helper failed emission with a MetadataLoadContext mismatch;
overloaded captured Settled helpers generated colliding display-class storage;
awaiting Task<Void> left an unexpected Void on the imported MoveNext return stack;
and retaining a pattern-bound address across awaits produced an unsupported-family
result. Adding diagnostics to that state machine exposed an uninitialized-field
constructor fault. A nested callback capturing a pattern-bound connected Socket
also observed a null receiver. These observations need separate minimal reproductions
before assigning a Raven versus importer/runtime cause or making general fixes.
They are not claimed fixed here. The retained fixture uses separate callback-stage
functions and passes the connected socket as an ordinary parameter. No compiler
source, Runtime Contract setting, or emission policy changes are made.
