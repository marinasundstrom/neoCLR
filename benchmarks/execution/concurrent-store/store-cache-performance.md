# Owned allocation lookup in the live store barrier (2026-10-25)

The live collector's scalar store adapter now remembers its last allocation payload.
It can reuse that range only while the existing owned exact-base cache still proves
that address is an allocated base. This removes repeated linear heap walks without
removing extent checks, detached-snapshot identity validation, mutation logging or
owner checks. Retain the optimization in the private candidate; synchronous GC is
still substantially faster and remains the production default.

## Contract and safety

The range hint is one pointer per native owner thread. It is not a GC root and
does not retain allocations. It is not an independent allocation registry: sweep revokes the existing exact-base proof before poisoning
or coalescing, and entry reset/release clears it. A hint cannot be dereferenced
without a current proof. Reallocation at the same base reads the new descriptor and
bounds. Active snapshot lookup and identity validation precede the hint. No heap
object, managed header, public API, compiler emission or ownership contract changes.
Two lookup counters add 16 native bytes per thread for experimental diagnostics.
This remains an owned, nonmoving-heap implementation.

Reuse the [barrier research](../concurrent-mutation-audit/README.md). CLR typed
reference barriers operate with different tracing information; neoCLR's conservative
live payload scan must also coordinate scalar writes. This cache accelerates the
candidate's extra allocation-extent lookup, not a new language semantic or a claim
that CLR needs the same check. Its costs are a small thread-local hint, proof-cache
traffic and a proof lookup; cache collisions can only turn hits into misses.

Both cached and disabled-cache builds pass optimized and undefined/bounds-sanitized
native tests, including 12,000 differential block comparisons per build, mutation,
overflow, root scanning and reclamation. A hundred range-lifetime rounds check
smaller reuse, bounds, sweep and owner reacquisition. A new coalescing fixture places
a plausible allocation header inside a larger replacement object: stale cached
identity must not permit a write across the replacement's real boundary. Removing
the exact-base proof fails specifically at that check (SIGABRT); deleting SATB
logging still fails its separate negative control. Six hundred generated entry
JIT/AOT cycles and 720 primitive-box/type-query cycles pass through the new adapter.
The complete JSON JIT server also passes 128 checked responses, 134 completed
concurrent epochs, guest exit zero and clean shutdown (`cache-json-jit.json.gz`).

The original disabled-cache control failed to compile because its unused hint was
still declared. The hint declaration now follows the same conditional as its use;
the complete uncached control and safety suite run successfully. Preserve that
initial failure as `cache-original-disabled-build-failure.json.gz`.

The legacy `NEOCLR_LIVE_LINEAR_STORE` debug path also remains uncached. Its targeted
range/reuse checks pass optimized and sanitized builds. A full-suite probe shows
that this pre-existing path does not reject changed detached descriptor identity;
it is **not** the equivalent uncached control used in these measurements. That control
is `NEOCLR_LIVE_NO_STORE_CACHE`, which preserves snapshot identity checks and passes
the full suite. Preserve the legacy rejection and targeted results separately as
`cache-legacy-linear-full-rejection.json.gz` and `cache-legacy-linear-ranges.json.gz`.
The default-mode performance run precedes this conditional-only compatibility fix;
normal cached/uncached paths are unchanged when the legacy macro is absent.

## Matched HTTP results

MacOS ARM64 AOT, one owned heap, 4,096-word mutation log, existing retained plaintext
and JSON selections. For each sample, all five hosts build before timing. Five
alternating rounds run each host with 16 warmup and 256 measured sequential requests
on new loopback connections. Every response, DONE count and cleanup is checked:
13,600 responses across the two samples. All live trials complete epochs, all
controls have zero background epochs, and disabled caches have zero cache hits.

Medians (server process CPU includes startup, warmup, marker threads and owned
cleanup, sampled on the owner immediately after invocation; later process-exit
teardown is excluded):

| Sample | Configuration | Requests/s | Process CPU seconds |
| --- | --- | ---: | ---: |
| Plaintext | Synchronous | 3,962 | 0.0646 |
| Plaintext | Guarded synchronous, no cache | 608 | 0.4530 |
| Plaintext | Guarded synchronous, cached | 1,133 | 0.2283 |
| Plaintext | Live, no cache | 338 | 0.8162 |
| Plaintext | Live, cached | 1,191 | 0.2382 |
| JSON | Synchronous | 1,983 | 0.1059 |
| JSON | Guarded synchronous, no cache | 344 | 0.7976 |
| JSON | Guarded synchronous, cached | 745 | 0.3572 |
| JSON | Live, no cache | 248 | 1.1123 |
| JSON | Live, cached | 638 | 0.4335 |

The guarded-synchronous control uses the live-generated object and checked providers
with automatic epochs disabled. It isolates barrier/provider cost from concurrent
scheduling. Its CPU falls 49.6% for plaintext and 55.2% for JSON; this supports lookup
cost as a cause independent of background marker progress. Live CPU falls 70.8% and
61.0%, with observed sequential throughput improvements of 3.53× and 2.57×.

Background macOS maintenance was active before and after measurement; the recorded
snapshot is `cache-background-activity.txt.gz`. Treat absolute rates and live scheduling
effects as provisional, not saturation capacity or framework comparisons. Uncached
plaintext varies from 329 to 438 requests/s, versus 1,128–1,205 cached; JSON ranges
245–260 versus 634–646. Raw trials and min/max summaries preserve the variation.
No builds or other benchmarks from this task ran during the measured request intervals.
Client latency quantiles include connection setup and are not GC pause quantiles;
inner GC profile maxima do not capture all coordination/cancellation pauses.

Plaintext live walks fall from a median 585,349 to 155,724, with 419,094 range hits.
JSON walks fall from 764,945 to 227,799, with 553,720 hits. Managed allocation stays
identical across every configuration: 4,394,837 plaintext bytes and 7,163,755 JSON
bytes per process. Median live peak arena use falls from 134,448 to 81,360 bytes for
plaintext and 131,192 to 100,080 for JSON. These are retention/progress changes, not
fewer application allocations. Median live RSS is about 12.09 MB and 14.37 MB.

## Evidence and next work

`cache-plain-matched.json.gz` and `cache-json-matched.json.gz` contain raw trials,
commands and input hashes; `cache-measurements-summary.json` contains medians/ranges
and input hashes. Safety evidence is in `cache-proof-validation.json.gz`,
`cache-disabled-validation.json.gz`, `cache-entry-validation.json.gz` and
`cache-box-validation.json.gz`.

Reproduce each sample with `measure-plaintext.py --sample plain|json --sync-object
<ordinary.o> --live-object <live.o> --output <fresh> --include-guarded-sync
--compare-store-cache --pairs 5`. The `--disable-store-cache` option retains a separate
uncached control; the native validator accepts it too. Measurements now use the same
owner-thread reporting boundary as the qualified HTTP harness.

Next prioritize the remaining guarded-store overhead: 3,568,656 out-of-arena calls
for plaintext and 6,094,176 for JSON in each measured process. Cached guarded controls
still cost roughly 3.5× and 3.4× ordinary synchronous CPU. Preserve conservative
provenance and ownership checks while investigating these calls. Extend comparison
to standalone workloads, complete outer pause distributions and saturated/worker
scaling before any default switch; this result does not establish general race safety.
