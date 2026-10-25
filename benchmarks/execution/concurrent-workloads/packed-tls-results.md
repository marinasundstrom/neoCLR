# Private concurrent-owner TLS (2026-10-25)

The [scalar-copy investigation](direct-store-results.md) identified four macOS TLS
resolutions on the ordinary outside-arena store path. The private live candidate
now groups its active flag, context and scalar-store counters in one thread-local
`live_owner_state`. `owned_heap` remains the independent ownership/proof service.
The marker continues receiving its snapshot explicitly; it does not look up a
mutator's state through its own TLS. Root, heap, native ABI, mutation, validation
and cancellation contracts are unchanged. This retains the existing thread/heap
separation from the CLR comparison; it changes private native storage placement,
not managed semantics or a public API.

The struct uses 56 native TLS bytes on this ARM64 target, including padding, versus
52 bytes of fields before individual TLS alignment/descriptor overhead. No managed
allocation is added or removed. All field initialization, updates and lifetime
remain as before; counters remain per thread across that thread's heap scopes.
`NEOCLR_LIVE_SPLIT_TLS` keeps the previous separate declarations as a comparison.
The automatic scheduler's declarations no longer depend on rewriting the active
flag declaration. The first compile exposed declaration order: the new context
pointer preceded its type. Make `state.h` include `native-gc.h` explicitly; retain
that failed build alongside the corrected evidence.

## Qualification

Packed and split variants pass the O2/UBSan native suites and all three deliberate
barrier/stale-proof negative controls. A new two-thread test starts simultaneous
concurrent epochs on separate owned heaps, rejects each foreign context without
writing, preserves independent active epochs/counters and releases both heaps.
The parent thread's counters remain unchanged. This proves isolation for these
owned contexts, not shared-heap GC or general cross-worker object support.

Six hundred entry and 840 record-array cycles pass across JIT/AOT, forced/paced
and O2/UBSan hosts. Managed objects are reused from the qualified `speed` build;
there is no compiler or helper-admission change. A fresh full JSON JIT consumer passes 128 responses, 134 completed epochs and
clean shutdown. Native assembly confirms the
outside-arena path now resolves two TLS descriptors instead of four, using the
same owner pointer for active/context/counters. No validation was removed.

## Matched execution

Five alternating HTTP rounds use identical generated code and native providers,
comparing packed/split live and guarded-sync plus ordinary sync. All 13,600
responses pass, with equal allocation counts/bytes and clean lifecycle checks.

| Handler | Split live requests/s | Packed live requests/s | Split live CPU | Packed live CPU |
| --- | ---: | ---: | ---: | ---: |
| Plaintext | 1,723 | 1,772 | 0.168598 s | 0.167072 s |
| JSON | 1,014 | 1,059 | 0.274527 s | 0.258725 s |

Observed live throughput rises 2.8%/4.4%, CPU falls 0.9%/5.8%. Packed guarded-sync
CPU is 0.147089/0.229380 seconds; ordinary sync is 0.055870/0.091056 seconds at
4,400/2,381 requests/s. Sync remains substantially faster. Counts/bytes remain
57,120 / 4,394,837 for plaintext and 124,873 / 7,163,755 for JSON per 272 requests.
These are short sequential loopback cohorts, not saturated capacity or a CPU-time
breakdown; small effects, especially plaintext CPU, remain sensitive to background
activity. No statistical significance or framework parity is claimed.

Ninety ordinary executions retain exact output/allocation/lifecycle checks, five
rounds per mode/variant at 4,096 iterations. Packed versus split live CPU medians
are 0.078374 versus 0.080025 seconds (tasks), 0.396100 versus 0.405905 (collections),
and 0.089767 versus 0.090890 (arrays). These separate cohorts suggest small effects;
the retained counts/bytes match previous measurements. Interpreter behavior and
previously recorded helper admission gaps are unchanged.

## CPU profiling follow-up

`measure-plaintext.py --profile-cpu` starts macOS `sample` after warmup for each
owned server process. It requires at least 8,192 requests, checks the profiler's
exit/report, records its command/hash and cleans up the owned profiler on failure.
It labels the whole cohort diagnostic-only: sampling can disturb timing, and
wall-clock stack samples (including sleeping threads) are not CPU percentages.
This complements fixed-work CPU measurements and call-site counts rather than
substituting one for another.

Six diagnostic runs (8,192 measured plus 16 warmup requests, three modes, two
handlers) pass another 49,248 response/allocation/lifecycle checks. Their timing
fields are deliberately excluded from performance conclusions. The one-second
captures report 542/546 observations per thread in live plaintext/JSON; the worker
has 538/530 top-of-stack condition-variable waits. That is evidence of waiting in
this sequential workload, not a CPU-utilization percentage or proof that concurrent
marking cannot help a saturated server.

| Live top-of-stack function | Plain observations | JSON observations |
| --- | ---: | ---: |
| experiment_gc_store | 98 | 116 |
| block_valid | 73 | 124 |
| _tlv_get_addr | 109 | 62 |
| stack_roots | 27 | 19 |
| enter_owned | 26 | 13 |

Guarded-sync likewise shows store/block-validation/TLS prominently. Ordinary sync
shows TLS (81/67), `enter_owned` (66/75), and `stack_roots` (64/59), alongside poll
and other kernel operations. Call trees place repeated `block_valid` calls below
the store adapter. These observations corroborate mutator-side costs instead of
assuming marker parallelism is the current bottleneck. Next examine redundant
header loads/validation within proved store paths, and execution-state entry/root
publication common to both collectors. Preserve validation and stale-proof tests;
do not remove checks solely because they appear in a profile. Pauses, sustained
concurrent request load, shared-worker correctness and default acceptance remain
separate requirements.

`summarize-cpu.py` verifies each report hash and extracts top frames and thread
observation counts. It retains the five-observation reporting threshold and does
not interpret omitted functions as zero. Raw captures and diagnostic reports are
`packed-tls-cpu-*.json.gz` and `packed-tls-*-cpu.txt.gz`; readable summaries sit
beside the timing summary. Native failures/successes, layout assertion, JIT and
assembly evidence are retained under `packed-tls-*` and `split-tls-*`.
