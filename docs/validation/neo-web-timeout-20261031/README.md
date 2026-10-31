# Neo.Web interpreter timeout fix — 2026-10-31

Follow-up to `2c86afc6b` and the [original qualification](../neo-web-generic-20261031/README.md).
Local macOS ARM64 development validation; not a release or Windows qualification.

## Reproduction and finding

The retained interpreter and unchanged framework assembly reproduce the original
`HTTP text request failed: TimedOut` failure (4/5 tests). A temporary request trace
reaches the concurrent generic-handler requests before the failure; no diagnostic
code is committed. Both client and server execute on the same cooperative dispatcher.
The socket backend's five-second per-transfer deadline is distinct from HttpClient's
100-second timeout, the server's phase deadlines and the 300-second test watchdog.

A two-second sampling profile places 98 of 171 interpreted main-thread samples under
`Access::check_arrays` and another 41 under `frame_array_usage`. Heap accounting
walks and locks every allocated slot before each instruction, even between mutations.
Per-slot counts were already cached, but repeatedly summing those counts remains
expensive for this combined application/test graph. The profile supports an
execution-cost diagnosis; it is not a per-request latency or throughput measurement.

## Fix and contract

Cache complete numeric heap array totals until heap allocation, a payload-changing
slot write/reset (including interior aliases), or reclamation invalidates them.
Slot invalidation signals a heap-owned atomic flag. Existing fixed-size byte writes
retain their counts. The shared graph gate excludes concurrent mutation during a
budget walk; atomic invalidation also crosses worker handoffs. Cache storage owns
no managed values and does not root allocations.

Every check still adds current frame/participant/service payloads and applies the
caller's limits. Failed recounts do not publish incomplete totals. Tighter limits,
cumulative roots, native-style whole-array writes, interior aliases and collection
retain their original quota behavior. No instruction-boundary checks, collection
policy, socket deadlines, HTTP options or test timeouts changed.

This extends the existing [slot-summary design](../../experiments/http-json/slot-budget-summary.md)
and [managed-array contract](../../managed-arrays.md). CLR array identity, assignment
and bounds semantics are unaffected; neoCLR's host logical-payload quota is its own
execution policy, not a claim that CLR scans arrays at each instruction. Compared
with full rescans, the benefit is avoiding repeated heap traversal; costs are one
invalidation reference per heap slot, atomic write invalidation and synchronized
numeric cache storage outside logical payload quotas. A fully incremental counter
would require broader ownership/copy accounting. Frame rescans remain a measured
cost and are outside this fix.

## Validation

- The **same framework assembly and library inputs** pass 5/5 with the new interpreter
  in three runs, covering real loopback HTTP, concurrent generic handlers, scoped
  dependency injection, middleware and joined shutdown. Two runs are unprofiled,
  sequential repeat checks. Exact commands, hashes, durations and output are in
  [results.json](results.json).
- **63 focused tests pass:** 12 slot tests, 12 shared-heap tests, 14 managed-array
  tests, 15 reference-array tests and 10 runtime GC tests. The new regression checks
  cached totals against smaller quotas, subsequent allocation, cross-thread alias
  mutation, repeated failed recounts and reclamation. Existing tests cover native
  replacement, nested aliases, byte writes, output epochs, cumulative roots and
  collection under payload pressure.
- The candidate profile no longer shows the full heap walk dominating the sampled
  interval; frame scanning now dominates. Profiles cover different execution
  intervals and are diagnostic evidence, not a quantitative speedup comparison.

Build/check commands:

```sh
cargo build --release -p neoclr --bin neoclr
cargo test --lib slots::tests
cargo test --lib shared_heap
cargo test --test managed_arrays --test reference_arrays --test runtime_gc
```

The shared-heap log also includes three integration binaries filtered to zero tests;
the separate array/GC log contains their actual 39 tests. Logs/profiles have trailing
whitespace normalized. Initial diagnostic runs overlapped compilation/profiling;
baseline total duration cannot be compared with the passing candidate, because the
baseline returns from HTTP assertions early. No broad performance claim follows.
Native/JIT behavior and public APIs are unchanged; the earlier native/JIT evidence
is retained. Website text was reviewed/updated; no website build or API snapshot
refresh is needed for this internal runtime change and prose-only status update.
This closes the reproduced Neo.Web timeout, not every historical HTTP timeout or
an arbitrary-load latency guarantee. No publication was performed.
