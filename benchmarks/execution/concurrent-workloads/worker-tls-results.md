# Post-inlining profiles and worker-handle TLS control (2026-10-25)

## Profiles and attribution

Refresh one-second macOS wall-stack samples for ordinary sync, the inlined live
candidate and its guarded-sync control, plaintext and JSON. All 49,248 responses
pass. These six runs are marked diagnostic-only; none of their timings is used as
throughput evidence. `--modes` selects unique configurations enabled by existing
comparison flags, avoiding compilation/profiling of unrelated historical controls.

The live top-frame counts include scalar store 105/112, TLS resolution 99/127,
owned entry 33/37 and root publication 22/14 for plaintext/JSON. Marker condition
waits contribute 525/510 observations. These are wall-stack observations across
threads, not CPU percentages; missing frames are not zero cost. Sync instead has
owned entry 103/89, TLS 92/82 and root publication 56/65 among its leading frames.

Extend `summarize-cpu.py` with immediate TLS caller attribution, preserving grouped
instruction offsets and rejecting unsupported TLS frame formats. The actual sample
format can collapse multiple offsets/addresses on one line; preserve that grouping
rather than inventing a unique location. All six caller totals match the original
collapsed TLS totals, with zero unattributed observations. Live scalar stores account
for 73/99 TLS observations. Their offsets divide into:

| Return offset in scalar store | Plaintext | JSON | State resolved |
| --- | ---: | ---: | --- |
| 80 | 37 | 49 | Heap ownership |
| 304 | 31 | 45 | Candidate/store owner |
| 428 | 5 | 5 | Marker-worker handle |

The instruction stream identifies these three resolutions. Only the last is tested
here: the worker handle already belongs to the same candidate owner thread.

## Implemented control and ownership

`NEOCLR_LIVE_GROUP_WORKER_TLS` puts the existing `shadow_worker *` handle in
`live_owner_state`; it does not move heap ownership or allocation proofs into that
service. Start, join, stop, cleanup and nulling the handle use the same code. The
marker receives its worker pointer explicitly, never through owner TLS. Independent
owners still have independent TLS. Split candidate TLS is explicitly incompatible
with this control. A forward-declared private worker structure makes the layout
available before entry checks; an initial declaration-order compilation failure is
retained as `worker-tls-safety.json.gz`, followed by the passing fixed run.

This is an internal storage-layout experiment, not a managed API or .NET semantic
divergence. It reuses existing ownership and indexing research and changes no
compiler lowering, object header, collector algorithm or shared-worker admission.
Native safety/negative controls, 144,000 model comparisons and 1,440 JIT/AOT fixture
cycles pass. ASan's previously recorded host-initialization gap remains.

Disassembly confirms five scalar-store indirect TLS-resolution call sites become
four, and 453 instructions become 449. Static sites are not resolutions executed
on every store. Mach-O thread-variable descriptors shrink 1,176 to 1,152 bytes;
thread BSS grows 158,112 to 158,128 through layout/alignment, while thread data
stays 40 bytes. These image sections are not an RSS guarantee. Binary hashes and
full assembly/size output are archived.

## Performance: no clear benefit

A balanced eleven-mode, twenty-two-round ordinary cohort passes 726 executions.
Every mode occupies every ordinal twice in each of three samples. CPU medians for
the inlined baseline / grouped-worker live variant:

| Sample | Baseline | Grouped worker |
| --- | ---: | ---: |
| Tasks | 0.0247535 s | 0.024707 s |
| Collections | 0.1428115 s | 0.142581 s |
| Arrays | 0.057937 s | 0.0579005 s |

Differences below 0.2% do not establish a useful CPU improvement. Exact outputs,
managed allocations, balanced order and metadata cleanup pass the summary checks.

Five alternating HTTP modes, three rounds and four clients validate another
61,920 responses. Live plaintext medians are 2,642 / 2,651 requests/s with CPU
0.812083 / 0.816696 seconds. JSON is 1,660 / 1,665 requests/s with CPU 1.295180 /
1.298958 seconds. Throughput changes below 0.4% accompany slightly higher CPU;
**do not promote this grouping as a performance improvement**. Keep it opt-in as
a reproducible control. Synchronous GC remains faster, about 5,173/2,906 requests/s.

HTTP managed allocation equality is not exact; retain every trial's counts in the
raw report and summary. This is not an allocation reduction. Every indexed trial
still records one 16,640-byte native metadata allocation and zero retained at
shutdown. No new native allocation is introduced by worker-handle placement.

## Larger remaining cost and next work

The inlined baseline's first performance trials classify 20,519,269 of 25,286,494
plaintext scalar stores and 34,727,845 of 40,964,638 JSON stores as outside the
managed arena: about 81% and 85%. Outside-arena does not by itself prove a
compiler-owned stack location or permit skipping validation.

A separate call-site diagnostic cohort checks 544 responses and maps every outside
store to exact executable symbols and the retained compiler selection, with zero
counter overflow. It records 2,710,373 plaintext and 4,582,821 JSON outside stores.
Leading plaintext functions include Result constructors (122,475 and 122,400
stores), request decoder Text (79,424) and header-buffer item access (65,008).
JSON's leading Result constructor contributes 418,608 stores (9.1%); CheckType and
AppendByte each contribute 104,720. These are counts, not measured CPU shares.
Raw sites and function totals are retained for codegen inspection.

Prioritize auditing compiler-generated value construction/extraction and receiver
provenance, then testing a fast path only where the compiler can prove destination
and owner lifetime. Preserve the checked native store contract and ambiguous byref
paths. A per-invocation owner-state handle is another possible way to avoid repeated
TLS resolution, but it requires an explicit lifetime/ownership contract and is not
implemented or admitted here. Do not collapse heap/candidate service ownership to
chase a small measured handle-layout effect. Root entry/publication remains a common
sync/live cost. Default admission, shared workers, broader sustained JIT workloads,
GC pause distributions and framework parity remain open.

## Reproduction

Use `--compare-worker-tls` together with existing index/exact/inlining comparison
flags. Ordinary balanced runs use `--rounds 22 --guarded-control`; HTTP selects
`--modes sync guarded-sync-inline-index live-inline-index
guarded-sync-worker-inline-index live-worker-inline-index`, four clients, 2,048
measured requests and three rounds. Safety/model/entry/record validators take
`--group-worker-tls` with existing index/exact/inlining flags. CPU profiles use
8,192 measured requests and `--profile-cpu`; sites use 256 and
`--profile-store-sites`. Diagnostics remain separate from timing comparisons.
