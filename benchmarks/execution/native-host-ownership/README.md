# Native host ownership and warm HTTP — 2026-10-24

This experiment returns from startup optimization to warm plaintext HTTP. It
compares the same entry-host source, 1 MiB heap, application and paced collector
under explicit external versus exclusively owned arena contracts. Both JIT and
AOT are tested with one owner. Production host defaults are unchanged.

## Measured outcome

Eight uninstrumented trials run configurations forward then reverse. Each uses a
one-second warmup and five-second measurement, 32 connections, two Go client
processors, checked 10-byte responses and control-socket shutdown. All pass.

| Mode / heap | Mean requests/s | Mean server CPU µs/response |
| --- | ---: | ---: |
| AOT external | 1,322.9 | 758.0 |
| AOT owned | 2,821.9 | 348.7 |
| JIT external | 1,043.1 | 958.5 |
| JIT owned | 1,885.0 | 527.4 |

Ownership raises mean throughput 113.3% in AOT and 80.7% in JIT, with 54.0% and
45.0% lower server CPU per response respectively. It enables the existing owned
heap's free-space index and admitted-pointer fast paths; this is not a new collector
or an execution-mode limitation. The optimized isolated-worker HTTP host already
uses ownership, so these gains must not be added to its prior capacity results.

The owned JIT host still has 33.2% lower throughput and 51.3% higher CPU/response
than the matched owned AOT host. Ownership explains a substantial host-configuration
penalty but does not explain the remaining execution-mode gap.

CPU uses `ps time` after READY and after both client runs, divided by all warmup and
measured completed/drained responses. It excludes startup and teardown, includes
intervening host work, and has approximately 10 ms resolution. Throughput uses
only the five-second measurement. Two trials per configuration on a macOS ARM64
desktop are exploratory: owned AOT ranges 2,705.6–2,938.2 requests/s, while owned
JIT ranges 1,874.8–1,895.2. Neither confidence intervals nor peer-server parity are
established. These are separate from the earlier worker-host CPU windows.

## Ownership experiment and lifecycle

`measure.py` derives a temporary host from `docs/experiments/jit-managed/host.c`.
It acquires the empty arena after context creation and releases it before each
existing post-acquisition free. Failed acquisition frees only the new buffer and
retains any incumbent owner. All scope/root cleanup and final collections remain.
The source is hashed in the raw reports; the script asserts expected edit anchors.

The existing native ownership contract permits one arena per native thread, with
release on the same thread before freeing it. Only runtime helpers may modify arena
descriptors; native code may access admitted payloads, and cross-thread arena use
or ownership transfer is not permitted. This experiment adds no worker sharing,
new ownership ABI or arbitrary native-input trust. General JIT host promotion needs
an explicit supported-helper/FFI contract and broader host failure-path qualification;
it is not enabled automatically by this report.

`lifecycle.c` checks successful allocation/zeroing and cleanup, fault return then
re-entry, invalid entry admission, refusal while another arena is owned, nested
entry refusal without disturbing its caller, and repeated invocations on two
independent threads. Four executables pass: forced/paced GC, each optimized and
UBSan/bounds. The intentional RuntimeError output verifies the fault path. These
are independent thread-owned heaps, not a shared-heap stress test. Injected malloc,
socket/task/terminal-admission failures and native Windows execution remain outside
this fixture. Existing cleanup paths are retained rather than redesigned.

## Work counts and remaining profile

Separate diagnostic hosts execute exactly two or four serial checked requests.
The counters include invocation startup/shutdown and final collections:

| Requests | Mode | Allocations | Collections | Root publications |
| ---: | --- | ---: | ---: | ---: |
| 2 | AOT | 960 | 20 | 11,823 |
| 2 | JIT | 960 | 19 | 11,823 |
| 4 | AOT | 1,376 | 27 | 19,175 |
| 4 | JIT | 1,376 | 27 | 19,175 |

The two-request increment is 208 allocations and 3,676 publications per response
in both modes. Collection totals are not identical in every run; these small
counts do not establish equal pause time, root volume or survivor distributions.
They do rule out an extra allocation/publication count in the checked JIT cases.

Two additional runs capture three-second `sample` profiles at 1 ms after warmup.
Profiling runs are excluded from capacity aggregation. Main-thread sample counts
are 2,123 AOT and 2,143 JIT. Selected collapsed self counts:

| Native symbol | AOT | JIT |
| --- | ---: | ---: |
| stack_roots | 351 | 223 |
| _tlv_get_addr | 324 | 250 |
| collect_indexed | 257 | 149 |
| neoclr_gc_enter_owned_v1 | 126 | 198 |
| neoclr_gc_leave_v1 | 39 | 114 |
| neoclr_native_stack_check_v1 | 21 | 96 |

These are wall-clock stack samples, not CPU attribution or per-request normalized
costs. JIT generated code is not symbol-attributed in this capture. Different
throughput and sampled locations make raw counts unsuitable for declaring any
one helper slower. Both paths still spend substantial observed work in root/GC/TLS
services. The native entry helper's disassembly is retained for both artifacts;
its matching validation prologue does not establish whole-function performance.

Source inspection gives a specific further hypothesis: managed JIT passes
`long_range_calls=true` to shared lowering; `prepare_calls` clears `colocated` on
all function references. AOT uses the ordinary object/linker path. Cranelift JIT
0.121.2's local `backend.rs` explicitly requires long-range libcalls on AArch64
because short relocations may not reach definitions. JIT also uses non-PIC code
and a dynamic helper library. These settings are correctness constraints, not
safe flags to disable based on throughput. A controlled call-layout experiment
and generated-code attribution are needed to determine their contribution.

## Decision and research comparison

Keep the host candidate experimental. Reuse the existing [.NET/Java/Go allocation
research](../../../docs/design/allocation-metadata-runtime-comparison.md): explicit
allocation ownership can amortize validation and bookkeeping without enlarging
managed headers. Unlike CoreCLR allocation contexts or HotSpot TLABs, this native
arena contract forbids cross-thread arena operations; the speedup does not justify
silently assuming that restriction for every managed JIT program.

The next bounded work is to qualify an explicit owned JIT host configuration,
then isolate long-range call/code-layout costs while preserving relocation reach.
For improvements shared by JIT and AOT, root entry/exit, repeated storage metadata
and collection tracing remain measured candidates. Do not remove safepoints or
infer a generational-GC benefit from these counts alone. No public API, compiler
bridge contract, deployed bundle or website headline changes in this slice.

## Evidence and reproduction

`capacity.json.gz`, `profile.json.gz`, `counts.json.gz`, `lifecycle.json.gz`, raw
sample/disassembly archives, `summary.json` and `validation.json` preserve commands,
outputs, source/artifact/dependency identities and the three distinct measurement
purposes. Baseline source is `f348d4606`. Four summary tests check cohort completeness,
profile exclusion and CPU units/denominators. All eight capacity, two profile, four
fixed-request and four lifecycle runs pass.

This local harness requires the retained artifacts from the [mode experiment](../gc-mode-comparison/README.md),
including the wrapped ordinary report, current compiler, matching HTTP object,
bundle, native link recipe and checked client. Set Xcode DEVELOPER_DIR/SDKROOT.
Run capacity and profiling sequentially without competing builds/load:

```sh
python3 benchmarks/execution/native-host-ownership/measure.py --ordinary target/gc-mode-comparison-final --output target/native-host-ownership
python3 benchmarks/execution/native-host-ownership/measure.py --ordinary target/gc-mode-comparison-final --output target/native-host-profile --profile
python3 benchmarks/execution/native-host-ownership/counts.py
python3 benchmarks/execution/native-host-ownership/lifecycle.py
python3 benchmarks/execution/native-host-ownership/summarize.py benchmarks/execution/native-host-ownership/capacity.json.gz
python3 -m unittest discover -s benchmarks/execution/native-host-ownership -p 'test_*.py'
```

Use fresh output directories; counts/lifecycle currently consume the named capacity
build directory above. These are experiment tools, not clean-checkout installers.
