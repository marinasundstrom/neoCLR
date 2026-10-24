# Live plaintext GC: first matched measurements (2026-10-24)

The initial live collector works but is much slower than synchronous GC. A bounded
mutation log of 128 words overflows almost once per request, discarding marking work
and leaving a nearly full heap. Raising the experimental log to 4,096 words removes
store-overflow cancellations in three repeats and improves this workload about 13.2×.
The candidate still trails synchronous GC by about 5.6×. It is not ready to be the
runtime default.

## Scope and controls

`measure-plaintext.py` builds all hosts before timing, then alternates trial order.
Each process handles 16 warmup plus 256 measured sequential loopback requests over
new connections, checks HTTP 200 and the exact ten-byte response, verifies DONE counts
and clean exit, and checks marker/scope cleanup. Each configuration has three trials.
Both ordinary and live objects use the same retained plaintext/library selection;
object hashes and host/source recipes are retained. The ordinary object is the
previously verified full synchronous build; subsequent backend admission changes do
not change its ordinary emission path. This is macOS ARM64 AOT, one owned heap,
profiling enabled, with the same native host and pacing definitions.

Requests/s measures the post-warmup request interval. CPU is server-process user plus
system CPU across its whole lifetime, including warmup/startup/shutdown and marker
threads; it excludes the Python client and compilation. RSS is per-process macOS peak
bytes. Client latency quantiles include connection setup and are **not GC pause
quantiles**. Core GC profile timings exclude some outer coordination/cancellation
costs and cannot establish total mutator pauses. This is a sequential fixed-work
comparison, not saturation throughput or a Kestrel/framework ranking. The first sync
trial in each round is slower; raw trials and alternating order expose that variation.

## Results

Medians, with the synchronous comparison repeated in each round:

| Round / configuration | Requests/s | Process CPU seconds | Peak RSS bytes |
| --- | ---: | ---: | ---: |
| Isolation: synchronous | 3,310 | 0.0699 | 9,846,784 |
| Isolation: guarded synchronous | 671 | 0.4017 | 11,862,016 |
| Isolation: live, 128 words | 45.3 | 5.8176 | 14,417,920 |
| 1,024-word round: synchronous | 3,345 | 0.0653 | 9,846,784 |
| 1,024-word round: live | 521 | 0.5857 | 12,255,232 |
| 4,096-word round: synchronous | 3,361 | 0.0649 | 9,846,784 |
| 4,096-word round: live | 600 | 0.5005 | 12,288,000 |

The guarded-synchronous control uses the **same live-generated object, checked stores
and qualified native providers**, but disables both automatic background-epoch start
sites in a saved collector source. It preserves synchronous pacing/collection. All
candidate counters must remain zero. This isolates the combined cost of generated
barriers/providers from background scheduling; it does not identify a single hot
function or permit bypassing validation.

Each isolation process records 3,568,656 outside-arena checked stores and roughly
631,000 heap checked stores across 272 requests. The 128-word live trials cancel
248–250 epochs in the store barrier; scoped native fallback counts are all zero.
Thus the suspected UTF-8 fallback is **not** responsible in this workload. The
1,024-word trials have one or two store overflows; 4,096 has none. The larger trials
complete 248–255 of 249–256 started epochs, with one non-store cancellation;
no marker or scope remains active. The 4,096-word live heap high-water drops from
about 1 MiB to about 130 KiB. Managed allocated bytes remain 4,394,837 in every run:
this change affects collector progress and retained heap, not application allocation.

Compared with the isolation round's 128-word candidate, the 4,096-word candidate
uses about 91.4% less process CPU. Its median client latency is about 1.4 ms, with
p99 about 5 ms, versus about 21 ms / 49 ms for the original candidate. These are
bounded-work observations, not general performance guarantees or confidence bounds.

## Retained change, safety and cost

The private mutation gate now defaults to 4,096 words and accepts a compile-time
`GATE_WORDS` override between 8 and 16,384. All gate/collector translation units must
use the same value. Production synchronous GC and compiler admission stay unchanged.
The log remains bounded; writes still reserve both old and new entries, cancel before
changing payload on overflow, and join/discard before inactive retry. This follows
the existing [barrier design research](../concurrent-mutation-audit/README.md), without
changing the tracing algorithm or native ownership boundary.

The log storage grows from 1 KiB to 32 KiB per gate; drain scratch arrays also scale
with that bound. Extra native space is the deliberate cost of avoiding premature
cancellation. 1,024 words uses less space but still overflowed and performed worse in
these repeats, so 4,096 is the provisional balance, not a proven global optimum.

Optimized and undefined/bounds-sanitized gate tests pass at 4,096, including overflow,
unchanged payload, grouped publication and concurrent readers. The live suite passes
12,000 differential blocks per build plus deletion/insertion/future-address cases,
scalar stores, grouped arrays, root/heap scans and cleanup. Dropping deletion logging
still fails the negative control. The first widened-log run exposed hardcoded fixture
expectations for 64 writes; fixtures now derive overflow counts/expected values from
the configured capacity. An additional 840 generated-interface JIT/AOT cycles pass
with the new default. The updated live suite also passes with the original 128-word
capacity, including its negative control (`gate128-live-regression.json.gz`). No race-detector or full managed-mode matrix claim is made.

## Next work

Prioritize measured checked-store cost: out-of-arena calls dominate the count, and
heap stores fall back to a linear allocation walk when no snapshot lookup applies.
Investigate safe compiler provenance and owner-maintained lookup, preserving root and
heap identity checks. The guarded-synchronous control already costs roughly 5× the
ordinary baseline. Also measure repeated synchronous rechecks after concurrent finish,
which remain frequent. Keep JSON's seven type/boxing helpers, full-server JIT,
standalone workloads, pause distributions and saturated/worker scaling in the goal.

Reproduce with `measure-plaintext.py --sync-object <ordinary.o> --live-object <live.o>
--output <fresh-directory> --gate-words 128 --include-guarded-sync`, then separate
1,024- and 4,096-word runs. Do not run builds or other benchmarks concurrently.
Use `concurrent-live/validate.py --sparse-index --gate-words 4096 --output <fresh>`
and the matching gate validator for safety checks. Reports, generated sources and
initial fixture failure are archived as `plaintext-*` and `gate4096-*`; the compact
aggregate is `plaintext-measurements-summary.json`.
