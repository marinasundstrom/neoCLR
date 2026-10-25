# Combined native execution layout — 2026-10-25

Follow the [grouped stack-bound](../grouped-stack-bounds-20261025/README.md) and
[split storage-validator](../split-storage-validation-20261025/README.md) candidates
with matched composition measurements. Do not add percentages from separate runs.
The pair changes storage/code layout, not root visibility, stack admission or GC
mutation rules. These are shared runtime changes; sync is measured alongside the
concurrent candidate.

## Procedure and safety scope

`prepare-stack-bounds.py --combine-validation` builds a preserved original
stack-bound control and a current grouped-bound variant with
`NEOCLR_GC_SPLIT_STORAGE_VALIDATION`. It validates the preserved baseline source
against the original build manifest and checks all other inputs unchanged. Both
variants use the same generated application objects and native host/GC settings.
`prepare-stack-jit.py` respects the paired manifest's explicit layout names.

Standalone `compare-root-validation.py --stack-bounds --combine-validation` uses
qualified scalar-store objects, canonical concurrent sources and five alternating
rounds, excluding the first round from medians. Individual boundary/owner/recursive
stack tests and malformed descriptor/slot checks remain applicable: neither
implementation changes in this composition. HTTP checks wire output and lifecycle;
standalone checks output and matching allocation work.

`prepare-paired-owner-profile.py` copies and instruments each AOT recipe with the
existing checked owner-operation histogram implementation. It retains all recipe
flags, collects every outermost operation (including begin/finish/cancel), and marks
the build diagnostic-only. A separate HTTP run and `summarize-owner-pauses.py`
validate and pool counts and histogram bounds. Quantiles are bucket intervals,
not exact pause values; owner wall times include scheduling and clock overhead.
They are neither total GC CPU nor global application stop-the-world durations.
No profiling is mixed into the throughput cohort.

This reuses existing runtime/CLR design research: keep changing roots and current
stack bounds correct while reducing lookup and frame-entry cost. It introduces no
public API, native ABI, allocation policy or collector-default change.

## AOT results

Three interleaved rounds at 16,384 requests per case pass 393,600 responses,
including warmup. This is four times the per-case traffic of the preceding layout
cohort. Median invocation results:

| Handler | GC | Control RPS | Combined RPS | CPU change |
| --- | --- | ---: | ---: | ---: |
| Plain | Sync | 7,298 | 7,725 | −5.24% |
| Plain | Concurrent | 6,118 | 6,312 | −3.01% |
| JSON | Sync | 4,187 | 4,343 | −3.63% |
| JSON | Concurrent | 3,567 | 3,655 | −2.43% |

Sixty standalone concurrent AOT runs pass across six patterns. Median CPU changes
(excluding warmup): compute −0.89%, reuse-buffer +0.07%, steady −3.78%, bursts
−6.39%, retained-window −6.00%, mixed-phases −5.10%. Allocation counts and bytes
match. This supports composition of the two changes without adding their earlier
percentages. Sync still leads HTTP; these are shared runtime improvements rather
than evidence that concurrent marking is faster.

Raw AOT reports: `http-aot.json.gz`, `http-build.json.gz`,
`standalone-aot.json.gz`. These short loopback measurements are not server saturation
or industry-framework rankings.

## Separate owner-operation profile

Eight diagnostic cases pass 32,896 responses; all histogram counts, totals and
candidate lifecycle counts reconcile through the existing checked summarizer.
P99 outermost GC operation buckets are identical between layouts:

| Handler | Sync p99 interval µs | Concurrent p99 interval µs |
| --- | ---: | ---: |
| Plain | [32.768, 65.536) | [131.072, 262.144) |
| JSON | [32.768, 65.536) | [65.536, 131.072) |

The combined concurrent profile spends 17.12 ms in begin and 37.14 ms in finish
for plaintext; JSON spends 47.70 ms and 82.30 ms respectively. These disjoint
operation totals are diagnostic wall times. Guarded writes are reported separately;
including their many short calls in one percentile would obscure GC-operation tails.
This supports a shared execution CPU gain without a demonstrated pause improvement.
It does not justify making concurrent GC the default. Sync still has shorter
owner-operation p99 buckets in these workloads.

Raw profile reports: `owner-profile.json.gz`, `owner-build.json.gz`, and
`owner-summary.json.gz`. Existing finish-subphase research attributes substantial
cost to final collection, but those older proportions must not be applied as exact
current shares. Reuse that research when choosing the next collector-specific work;
removing a small local phase cost has previously produced little whole-request gain.

## Concurrent JIT and integration decision

Three matched rounds pass another 196,800 responses with identical runner, fallback
JIT placement and released compilation inputs. Plaintext median throughput rises
5,383 → 5,517 RPS with invocation CPU −4.79%; JSON rises 3,100 → 3,324 RPS with CPU
−6.55%. Allocation counts and bytes match. Compilation stays synchronous and is
outside invocation timings. Reports are `http-jit.json.gz` and `jit-build.json.gz`.

Across these cohorts the pair improves HTTP CPU and the allocation-heavy standalone
patterns, with reuse-buffer effectively flat. This supports integrating the pair
for the qualified macOS ARM64 native execution path in the next bounded slice,
retaining an explicit control and verifying generated code matches the measured
candidate. That integration is not performed by this measurement slice. Do not
extend the result to Windows, change the collector default, or claim an improvement
in GC-operation p99. Runtime execution and collector pause work remain separate.
