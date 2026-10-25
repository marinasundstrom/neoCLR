# Concurrent stack/byref policy integration — 2026-10-25

The measured concurrent candidate uses compiler-proven direct/forwarded stack
writes and preserves immutable byref argument provenance. These implementations
were previously compiled only into the test runner. They now belong to the private
nonmoving concurrent backend, through one qualified policy. Ordinary hosting still
selects synchronous GC; no public concurrent selector or wider worker admission is
introduced by this change.

## Scope and boundaries

The qualified policy enables direct guard removal, forwarding of established
stack extents, and immutable byref SSA preservation together. All generated bodies
must be supplied before definition, every direct caller must establish sufficient
bounds, and exported/address-taken/unknown callees remain guarded. Circular flows
do not invent a stack proof. Original argument root storage and cleanup paths are
retained; unchanged pointer bindings may refer to changing object contents.
The SSA policy depends on the current nonmoving collector. Moving collection will
require relocation-aware byrefs or disabling this policy.

The non-test backend uses this explicit policy when live stores are selected.
Qualification retains the historical individual experiment switches by default,
and `NEOCLR_LIVE_STACK_POLICY=qualified` selects the same constant policy used by
the non-test build. Production binaries do not read those environment controls.
The standard HTTP builder now records the single qualified-policy setting instead
of requiring four independent switches. Sync compilation ignores the live policy.
Old measurement archives retain their original flags and hashes.

Normal transformation no longer constructs JSON audit rows that it immediately
discards. Explicit reports retain their original contents. This removes compiler
bookkeeping, not root publications or managed allocation. No compile-time or HTTP
performance gain is inferred without new measurements.

The implementation follows the existing researched compiler-owned-stack contract;
see the [immutable-byref design and .NET comparison context](../../../benchmarks/execution/concurrent-workloads/stable-byref-results.md)
and [measured workload effects](../../../benchmarks/execution/concurrent-workloads/stable-byref-cost-results.md).
No Raven source semantics, Runtime Contract configuration or CLI/native metadata
encoding changes; this is neoCLR backend integration.

## Async declaration issue exposed by integration

The first full qualified-policy run passed 197 tests and failed one async wrapper
emission test. The complete-closure check found an exported queue-drain declaration
with no body: callback-only async selection declared a pump/drain pair even though
no entry pump had been selected. Diagnostics now name missing declarations.
The compiler creates those pump-related declarations only when an entry drain is
selected. A regression checks absence of the unused queue-drain symbol. The closure
check is retained, including its negative missing-body test; no unknown generated
body is trusted to make the integration pass.

## Evidence

Both the historical test configuration and the explicit qualified policy pass
198 codegen tests, with 34 native-host-dependent tests ignored. Production
compilation passes. Live root validation passes 25 commands and the deliberate
missing-storage-root failure control; native async validation passes 14 commands,
including optimized/sanitized AOT and JIT.

Compressed reports and logs record exact checks. Six standard standalone AOT
objects from the integrated policy match the previously measured candidate bytes.
This preserves existing performance evidence rather than claiming another speedup.
Root/sanitizer and native async consumers, explicit missing-root controls, current
JIT/HTTP consumers and production compilation qualify the integration separately.
A repeated object comparison initially refused to overwrite existing output;
that failed harness report is retained separately. The successful final comparison
uses a fresh directory and a pinned runner. HTTP preparation now accepts
`--runner` so a concurrent local rebuild cannot replace its compiler mid-report.
Plain/JSON pipelined HTTP passes eight sync/live AOT/JIT cells (640 responses).
Both HTTP objects match the preceding qualified sync/live objects byte-for-byte.
These are correctness checks; their short timings are not new benchmark claims.
Final package/platform gates and concurrent-default acceptance remain open.
