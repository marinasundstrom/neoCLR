# Small-frame descriptor validation

2026-10-25. Follow the [post-forwarding profiles](forward-gap-results.md) with
activation counters before changing metadata lifetime. This adds an opt-in native
Clang code-generation control, not a GC ownership or root-validation exemption.

## What the counters establish

Nine standalone 4,096-iteration batches and six 512-response HTTP runs cover sync,
guarded-sync and live modes. All outputs pass; live runs complete 1,792 concurrent
epochs with clean index release. Each owned frame validates its storage once;
the validation-call and descriptor totals equal the owned-entry totals in all
15 runs. Existing ancestor proofs already avoid repeated scans at publication.
Transient descriptors remain separately validated. The counters are diagnostic,
with no timing or allocation-gain claim, and cover the invoking owner thread only.

| Live workload | Frame entries | Descriptors | One-slot entries | Entries with at most four slots |
| --- | ---: | ---: | ---: | ---: |
| Tasks | 319,536 | 815,187 | 44.9% | 85.9% |
| Collections | 2,023,472 | 3,973,203 | 51.0% | 89.7% |
| Arrays | 475,184 | 491,603 | 48.3% | 95.7% |
| Plaintext (512 responses) | 1,987,205 | 4,354,737 | 67.8% | 88.8% |
| JSON (512 responses) | 2,989,189 | 7,039,153 | 62.5% | 88.7% |

The initial profiler expected an implementation in the compatibility wrapper and
rejected it before compiling. The corrected driver instruments the actual
`runtime/native/gc/root-probe.c`; both the failed setup and final reports are retained.
No runtime defect or failed managed execution is implied by that setup rejection.

## Implementation and safety

ARM64 disassembly shows the vectorized validation loop inlined into `enter_owned`.
It saves eight SIMD registers even for a zero/one-slot activation. The Clang-only
`NEOCLR_GC_SCALAR_STORAGE_VALIDATION` control disables vectorization/interleaving for
that loop. It preserves each address, width and flags predicate, the full scan,
all abort behavior and every frame/ancestor/root lifetime rule. GCC and default
builds keep their existing behavior. No ABI, public API, Raven emission, Runtime
Contract or temporary CLI representation changes.

The scalar host reduces this native frame from 160 to 96 bytes and removes the
SIMD saves. This is stack/code-generation evidence, not a managed-allocation saving.
Large scans may lose SIMD throughput; the option is experimental, not a universal
claim that scalar loops are preferable. A future split small/large path must be
measured and qualified separately.

The existing owned-root and checked-root validation driver passes 138 commands
with the option, including ordinary/UB-bounds-sanitized, worker/pacing configurations,
valid consumers and expected aborts for invalid addresses, widths, flags, plans,
contexts and transient handoffs. These worker checks validate the common root
service; they do not admit the concurrent collector to shared heaps. There is no
new ASan or cross-platform qualification. This reuses the existing CLR/root-map
research: dynamic slot addresses remain per-activation facts, even when layout
metadata is compiler-owned. No metadata is cached across activations here.

Complete managed JIT consumers also pass: 1,024 plaintext/JSON responses and six
standalone batches (stack elision off/on), with 2,559 completed concurrent epochs
and clean index release. The HTTP hosts explicitly select scalar validation; the
standalone runs reuse that native library. The managed object/compiler is unchanged.
Exact qualifier commands and three raw JIT reports use the `scalar-frame-*-jit`
prefix. HTTP compilation used `NEOCLR_TEST_OPT_LEVEL=speed` and the retained
`test-runner-forward-elision`; the standalone driver selects speed itself.

## Matched performance

Five interleaved modes retain sync, current/previous guarded-sync and current/previous
live controls. Managed objects are identical within each current/previous pair;
only the scalar-validation define differs in native compilation. All 150 standalone
executions and 82,560 HTTP responses pass. No compiler/qualification jobs overlapped
the performance runs.

| Workload | Paired live CPU change | Paired elapsed/throughput change |
| --- | ---: | ---: |
| Tasks | −0.9% | elapsed −1.0% |
| Collections | −2.1% | elapsed −2.6% |
| Arrays | −0.6% | elapsed −0.6% |
| Plaintext HTTP | −1.9% | throughput +1.8% |
| JSON HTTP | −1.2% | throughput +1.1% |

These are descriptive paired medians, not confidence bounds. The smaller standalone
changes should not be overstated. Guarded-sync also improves slightly. Standalone
and JSON allocation counts/bytes match. Plaintext retains one-object/528-byte
scheduling variation; no allocation reduction is claimed. Raw memory/GC counters
remain available; no new pause-distribution advantage is established.

Retain the option and vectorized control. Synchronous GC remains faster/default;
shared-worker, migration and broad pause/default acceptance remain open. Next
consider separating small-frame validation from the large vectorized helper,
without removing activation checks or combining heap and root ownership.

## Evidence and reproduction

`profile-frame-work.py --root target/concurrent-workloads --output <new-directory>`
rebuilds diagnostic hosts from the retained forward-elision recipes. Raw reports
are `frame-work.json.gz`, initial setup `frame-work-initial.json.gz`, and the exact
instrumented source `frame-work-instrumented-roots.c.gz`. Counter order is entries,
entry descriptors, stack publications, lanes, transient publications, transient
descriptors, storage-validation calls, storage descriptors. Histogram buckets 0–16
are exact and bucket 17 contains all larger counts. `frame-entry-code.json` records
binary hashes; before/after disassembly is retained separately.

`scalar-frame-measure-commands.json.gz` records exact benchmark invocations and the
compressed driver. Raw ordinary/plaintext/JSON reports share the prefix. Recompute:

```sh
python3 benchmarks/execution/concurrent-workloads/summarize-stack-arguments.py \
  --root target/concurrent-workloads --prefix scalar-frame --scalar-validation \
  --output target/concurrent-workloads/scalar-frame-summary.json
```

The scalar mode requires identical object paths and exactly one candidate define.
Four negative controls reject a missing define, a define on the control, changed
objects and missing trials. The missing-sync-trial case initially exposed a summary
gap: all five modes are now required in each of four HTTP pairs. The prior valid
forward-elision summary still reproduces byte-for-byte. Validation commands and
raw outputs, including expected aborts, are in `scalar-frame-safety.json.gz`.
