# Recent store ranges (2026-10-25)

The exact-base shortcut leaves substantial scans for interior-field destinations.
The private live adapter now retains four recent allocation ranges, rather than
only the last one. It keeps the last hint first, then checks exact-base stores,
then up to three older hints before falling back to the checked heap walk.

Each older hint is only a pointer, never a root or independent allocation proof.
Before reading its header, the adapter checks that it lies in the current arena
and still has a positive entry in the existing owned exact-base cache. Sweep,
reset and release already revoke these proofs. Current header/state, payload bounds
and old-snapshot identity checks remain mandatory. Eviction only causes a miss.
A round-robin cursor replaces older hints when the last payload changes; this is
not a claim of an optimal replacement policy or distinct cached ranges.

Four is a bounded measured choice, not an optimal-size claim. Compared with the
one-range adapter, it adds three native pointers and one unsigned cursor per owner
thread (28 bytes on this target before padding), no managed allocation, header,
root, compiler emission, ownership rule or public API. The tradeoff is additional
proof queries/header checks on misses. `NEOCLR_LIVE_STORE_RANGES=1` retains the
previous implementation; disabling the store cache disables all range hints.
The native validator and measurement tools accept `--store-ranges`, default four.
Historical exact-base-only comparisons require `--store-ranges 1` when reproduced
with these tools. The ordinary production collector remains synchronous.

This continues the identity/scan-metadata separation discussed in the
[existing CLR comparison](../concurrent-gc-investigation/README.md), without adopting
a new managed contract or copying a CLR object header.

## Safety

Optimized and UBSan/bounds native suites pass, including differential liveness,
mutation/deletion, overflow, snapshot identity and lifetime checks. The initial
suite is archived separately from the final suite with the older-range regression.
That regression moves a soon-to-be-reclaimed allocation into an older cache slot,
coalesces/reuses its storage, then places a plausible header inside the replacement
payload. A write crossing the real replacement boundary must reject. Removing
proof validation for older hints makes that assertion abort at line 139; the
existing newest-hint and deletion negative controls also still abort as expected.

A separate alternating-field test passes optimized and UBSan builds with one and
four ranges. After warming three object fields, 60 alternating interior stores
perform no walks with four ranges. Both configurations reject writes crossing each
object's end, and the existing fixture covers proof eviction and reclaimed bases.
The full validator includes these tests for future runs. No GC tracing exemption
was added for records, arrays or any other language construct.

## Standalone evidence

Five alternating rounds × three collectors × three samples = 45 passing AOT runs.
Each runs 4,096 complete sample iterations; allocations and bytes are identical
across ordinary sync, guarded sync and live. The guarded synchronous control runs
live-emitted code without starting any concurrent epoch and checks zero counters.
Median invocation CPU seconds, compared with the preceding one-range cohort:

| Sample | One-range live | Four-range live | CPU reduction | Four-range sync | Four-range guarded sync |
| --- | ---: | ---: | ---: | ---: | ---: |
| Tasks | 0.100277 | 0.085328 | 14.9% | 0.017139 | 0.052198 |
| Collections | 0.493688 | 0.438571 | 11.2% | 0.099810 | 0.275313 |
| Arrays | 0.124240 | 0.098231 | 20.9% | 0.053107 | 0.074942 |

First-run live range walks fall 81,423→61,197, 354,741→282,177 and
65,311→32,693 respectively. These are separate before/after cohorts with stable
ordinary-sync controls, not interleaved binaries. CPU includes worker threads,
console work and invocation cleanup, excluding compilation/process startup.

## HTTP evidence

All hosts are built before timing. One- and four-range live/guarded controls and
ordinary sync alternate across five rounds for each handler, with 16 warmup and
256 measured sequential new-connection requests per run. Every one of the 13,600
responses is checked; allocation counts and bytes match across controls per handler.

| Handler | One-range live requests/s | Four-range live requests/s | Throughput change | One-range CPU | Four-range CPU | CPU reduction |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Plaintext | 1,211 | 1,382 | +14.1% | 0.236265 s | 0.206478 s | 12.6% |
| JSON | 656 | 820 | +25.0% | 0.430732 s | 0.336209 s | 21.9% |

First-run live range walks fall from 152,457 to 62,068 for plaintext and from
219,636 to 99,874 for JSON. Guarded CPU also falls (0.219208→0.195064 and
0.353612→0.303315), supporting an adapter-cost improvement independently of marker
work. Ordinary sync remains faster at about 3,893/2,188 requests/s. These results
do not establish saturated capacity, Kestrel parity or readiness to switch GC defaults.

HTTP CPU includes startup/warmup/cleanup; standalone CPU uses the narrower invocation
boundary. Profile instrumentation is enabled in all controls. OS background work
including dasd, ANECompilerService and PerfPowerServices was active. No other project
build/benchmark overlapped timed work. There is no pause-distribution claim.

[Summary](ranges4-summary.json) and compressed `ranges4-*` reports retain commands,
input hashes, per-run diagnostics and failures of deliberate negative controls.
The remaining guarded-versus-ordinary-sync gap and millions of out-of-arena store
calls motivate examining generated store/publication work next, alongside interior
lookup. Larger caches and other replacement policies remain unmeasured alternatives.

The final four-range default also passes a freshly linked full JSON managed JIT
host: 128 correct responses, completed concurrent epochs and clean guest/process
shutdown. See `ranges4-json-jit.json.gz`; sustained JIT performance remains open.
