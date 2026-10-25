# Explicit borrowed-store lowering qualification

2026-10-25, following the [native store-state prototype](store-handle-results.md).
This slice qualifies explicitly selected framed functions. The complete managed
pipeline, HTTP objects, ordinary provider list and JIT binding policy are unchanged.
No new environment switch silently enables borrowed stores.

## Emission and admission

The internal lowering implementation can acquire one candidate-state handle after
frame linkage and saved-state initialization. Acquisition is omitted when there are
no lowered stores. The handle stays an internal SSA value passed only to the new
store hook; managed parameters/results do not carry it. Fallible stores continue to
return through the existing production frame cleanup. A missing frame, a frame not
in the entry block, an entry backedge, or linkage failing to dominate stores/returns
is rejected before IR mutation. The tests select this path explicitly; ordinary
lowering continues to pass false.

This does not prove scheduler migration safety by dominance alone. The caller must
separately establish the same-native-thread activation contract. Existing generated
root-state handles provide that model, but broader borrowed-store admission remains
a next step. Arbitrary/foreign pointers are not validated as safe capabilities.
Heap ownership, current context/epoch checks, destination bounds, barriers and native
reclamation remain the collector's responsibility, unchanged by this emission.

The private version-1 live host query advertises additive bit 16 only with
`NEOCLR_LIVE_STORE_HANDLE`. Borrowed-store tests require all existing bits (1/2/4/8)
plus 16. An old host remains valid for ordinary live stores but cannot satisfy the
new contract. Ordinary call-closure providers reject the new imports; only the
explicit test provider set admits their exact signatures. A feature bit is a trusted
host compatibility declaration, not isolation or proof of arbitrary native code.

## Validation

Four focused tests cover single acquisition and cleanup after success/first-store/
second-store failure, missing/late-frame rejection without IR mutation, feature-bit
requirements, and ARM64 Mach-O/x64 COFF object imports. The object test verifies that
ordinary providers reject the new hooks and the explicit provider signatures accept
them. Fifty focused mutation/admission regression tests pass; 31 environment-dependent
tests remain ignored in that general run.

The native consumer passes 300 concurrent cycles: 100 AOT, 100 UB/bounds-sanitized
AOT and 100 JIT. Each epoch delays marking, tries an invalid destination and checks
frame cleanup, then overwrites references through generated stores and verifies both
old/new closures survive while dead storage is reclaimed. The ignored JIT fixture is
run explicitly by the driver. A legacy capability-only host is rejected before
resolving store hooks. This is real native-collector generated-code evidence, not a
full managed application, HTTP, worker migration or pause qualification.

The initial cargo runner remained at `_dyld_start` with zero CPU and no test output;
a retained startup sample records that state. It was explicitly terminated and its
terminal status checked before running a separately copied/signed runner. That runner
passed. An initial validation-driver omission failed to pass the object-output
environment to the test runner; the corrected driver passes it and checks the object
exists. Neither initial result is counted as collector evidence.

No request-level CPU/throughput or allocation improvement is claimed. The prior
amortized microbenchmark excluded acquisition; now the next step is to integrate
this path into already admitted complete nonmigrating call closures, including host
binding and all value/replacement bodies, then measure that actual acquisition cost
in plaintext, JSON and standalone JIT/AOT workloads against sync/guarded controls.
Shared-worker and concurrent-default acceptance remain open.

## Bridge and evidence

Native semantic intent is an activation-bounded same-thread state borrow. The
private native ABI adds a pointer argument to the scalar-store hook and a no-argument
acquire hook. There is no new CLI encoding, Runtime Contract configuration or Raven
semantic/emission change. Existing CLI bridge limitations remain temporary and
unchanged. This reuses neoCLR's root-state contract and the existing .NET/Go ownership
research linked from the native prototype. The neoCLR backend owns the lowering;
the native experimental host owns the pointer lifetime and compatibility declaration.

Reproduce with `concurrent-store/validate-borrowed.py --runner <signed-test-runner>
--output <new-directory>`. The retained SDK recipe is required. Evidence:
`borrowed-store-unit.log`, `borrowed-store-regression.log`,
`borrowed-store-native.json.gz`, and `borrowed-runner-startup.txt.gz`.
The validation report records native sources, object, runner and command hashes.
A comment-only clarification to lowering followed compilation; emitted behavior was
unchanged. Full compiler admission/performance is intentionally not represented as done.

Follow-up: [complete managed qualification](borrowed-managed-results.md) corrects
the experimental borrow bit from 16 to 32 and records acquisition-inclusive regressions.
The earlier bit/admission description above describes this historical slice.
