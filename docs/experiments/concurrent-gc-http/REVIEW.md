# Serve and memory-management review — 2026-10-18

Author direction: after the HTTP comparison, inspect the implementation with
existing collectors/algorithms in mind. The author then clarified that GC must
not be assumed to be the main reason HTTP is slow. This review therefore covers
request construction, asynchronous adapters, transport, allocation, root publication
and collection separately. It proposes bounded experiments, not new shipped APIs.

**First refresh the managed benchmark cohort. Then test eliminating redundant
incoming-request URL construction/parsing.** Continue evaluating collector policy
as an independent variable. A collector replacement is not supported by the
current evidence as the first HTTP optimization.

## A benchmark-cohort finding

The [load comparison](LOAD.md) deliberately reused a fixed compiled guest while
changing native GC. Fresh allocation callers now establish a more specific limit:
that guest still has the old TaskQueue batch-list/iterator implementation. Current
`Tasks.rvn` already uses `ArrayQueue`; its earlier
[qualified change](../../../benchmarks/http-server/serve/task-queue-20261018/README.md)
removed that recurring storage. The raw profile contains 2,880 occurrences each of
batch-array construction, batch object allocation, callback-array growth and
iterator allocation. Those are **not four new defects in current source**.

Thus the measured 23–24% concurrent CPU penalty applies to the fixed older managed
cohort with current native sources. It is not a measurement of all current
managed-library improvements or current overall neoCLR capacity. The samples and
source review remain useful, but must not silently mix their revisions.

The rebuild path already exists:
[task-completion/build.py](../../../benchmarks/http-server/serve/task-completion-20261018/build.py)
accepts a source-built library cohort, compiler, backend and native-link recipe;
it compiles the Raven Serve application, compiles a new object and relinks both
policies. Rebuild/verify the matching four libraries first, record source and
compiler/backend hashes, validate the new guest, then relink both GC modes from
that same object. Require the new function map/caller evidence to show the current
queue, receive cache and Promise changes before another current-runtime claim.
This review has not performed that fresh compiler/library rebuild.

## Fresh diagnostics and what they mean

Two separate 20-second diagnostic windows offer 300 req/s; a five-second macOS
`sample` runs at 1 ms intervals. These are wall samples, not measured CPU durations.
Sampling perturbs execution: sync misses 243/6,000 arrivals and concurrent misses
360/6,000. Neither is capacity evidence. All admitted responses validate and both
servers shut down with exact completion accounting.

[Sync profile](evidence/review/sync-profile.json),
[concurrent profile](evidence/review/apply-profile.json),
[raw sync sample](evidence/review/sync-sample.txt.gz),
[raw concurrent sample](evidence/review/apply-sample.txt.gz).
The [sync](evidence/review/sync-sample-summary.json) and
[concurrent](evidence/review/apply-sample-summary.json) summaries retain each thread
separately. The summarizer checks that exclusive counts partition every thread
and match the sampler's independently collapsed leaf counts. Generated method IDs
are preserved to avoid merging overloaded/generated names.

| Exclusive leaf | Sync owner (3,482 samples) | Concurrent owner (2,825 samples) |
| --- | ---: | ---: |
| Native allocation | 795 | 691 |
| Indexed collection | 661 | 104 |
| Published stack-root hook | 600 | 477 |
| strlen | 266 | 231 |
| Zeroing | 192 | 146 |
| Conditional collection entry | 126 | see full summary |
| TLS address lookup | 116 | 164 |
| Result polling/application coordinator | absent | 118 |
| Snapshot/index construction (`build_index`) | inlined/other symbols | 72 |
| Owner sweep/cleanup (`experiment_finish`) | part of collector | 67 |

These selected leaves are not an exhaustive categorization. Inlining and differing
code generation prevent assigning every sample to one source phase. In particular,
allocation and root publication must not be relabelled as background tracing.
The concurrent worker has its own 2,825 wall samples, including 1,927 in condition
wait, 631 in indexed tracing and 123 in index construction. Waiting samples are
not worker CPU; adding thread sample totals would inflate any CPU percentage.

The incoming parser's `Push` subtree accounts for 1,115 sync owner samples (32.0%)
and 869 concurrent owner samples (30.8%). `FromIncoming` accounts for 484 (13.9%)
and 362 (12.8%), respectively. These **inclusive** counts include descendant
allocation, collection and root checks, overlap the leaf table, and cannot be added
to it or interpreted as a predicted speedup. They locate an application operation
that induces runtime work, rather than proving a slow isolated parser loop.

A separate allocation-only capture uses 200 measured requests, 20 warm-up and one
stop exchange. It reconciles 85,838 allocations / 4,116,692 payload bytes with zero
lost records. Counts include process setup and teardown; four-frame caller chains
may truncate and are not allocated-type identities. The retained
[caller data](evidence/review/callers.json.gz),
[build](evidence/review/callers-build.json.gz),
[function-map input](evidence/review/callers-input.json.gz) and
[request outcomes](evidence/review/callers-requests.json) document the exact cohort.
For example, callers containing Promise construction account for 14,547 allocations
and 1,418,021 bytes. That supports checking Task representation, but it is not a
fresh count for the current class library. The same trace records 20,403 idle
collector entries and 11,783 paced-boundary triggers; idle includes boundary
triggers, so those counts must not be added.

## Whole-path code audit

Source fingerprints are retained in [review-inputs.json](evidence/review/review-inputs.json).
These are current sources; only findings also visible in the retained guest may
be related directly to its samples. The table deliberately includes non-GC work.

| Area / inspected implementation | Concrete observation | Proposed experiment and preservation requirements |
| --- | --- | --- |
| Incoming construction: `HttpRequestDecoder.Parse`, `HttpRequest.FromIncoming`, `HttpRequest.Get` in [server](../../../runtime/System.Web/src/System.Web/System/Web/Http/HttpServer.rvn) / [client](../../../runtime/System.Web/src/System.Web/System/Web/Http/HttpClient.rvn) | Decoded method/target/Host become a concatenated absolute URL, then `Uri.Parse`, then another HTTP URL scan/encode and substring decode. Temporary default headers/content are replaced. This still exists in current source and appears in both profiles. | Share authority validation and construct the incoming request directly from validated parts. Preserve Host/port restrictions, target spelling, query/percent encoding, scheme, body/content-type ownership and current rejection behavior. Do not simply bypass validation because the header parser succeeded. Test malformed authorities, duplicate Host/framing, fragments and partial reads. |
| Parser byte path: `PushCore`, `Text`, `HeaderName` | Input bytes copy into a growable header list, then selected ranges decode. Common header names already avoid decoding; a 128-byte header prefix is already reserved. | Count per-request decoding, list growth and root publications before considering a native parser. Prefer bulk append/range operations with equivalent incremental parsing and limits; retain borrowed-buffer lifetime rules. The original full-request string/split model is not the current implementation. |
| Response creation: `HttpContext` initializer, `EncodeResponse`, `SendNext` | A default response is eagerly constructed even when Serve's callback supplies another; headers start with an empty growable list and are appended byte by byte. Buffered output already uses one exact final allocation; it no longer grows the body into the header list. | First test bounded header reservation/exact sizing and avoiding unused default response work while retaining the public Response behavior. Separate larger-body vectored I/O from the tiny response case. Preserve response snapshot ownership, framing checks, HEAD and status restrictions. |
| HTTP transport adapters: [HttpConnection](../../../runtime/System.Web/src/System.Web/System/Web/Http/HttpConnection.rvn) | Plaintext receive/send traverses HttpConnection → HttpSocketStream → Socket, with async result/error conversions; TLS needs the stream abstraction. | Count promises/continuations for plaintext separately from TLS; prototype a direct plaintext internal adapter if it removes work. Preserve deadlines, cancellation acknowledgement, partial-write loops, error translation and deferred completion ordering. Do not delete async behavior or the deliberately queued handler to make the benchmark look faster. |
| [Task/Promise](../../../runtime/System.Runtime/src/System.Runtime/System/Concurrency/Tasks/Tasks.rvn) | `Promise<T>` stores one result in `ArrayList<T>(1)` plus a Task and optional callback list. First-callback reservation and reusable TaskQueue already exist. Completed callbacks still post to the queue. | Test an explicit completion flag and inline result slot before changing Task's public reference semantics. Validate generic records/unions/references, cancellation, repeated reads, callback FIFO/reentrancy and GC retention in interpreter/JIT/AOT. Do not pool publicly retained task identities or assume ValueTask is a free replacement. |
| [Receive caches](../../../runtime/System.Web/src/System.Web/System/Web/Http/HttpServer.rvn) | One 256-byte and one 4 KiB buffer can be retained per server. Tiny requests start small and grow after fragmentation. Streaming borrows have separate ownership. | Measure cache misses under concurrent/large reads before expanding pooling. Keep bounded retention, clear ownership, and never return a borrowed stream buffer early. The tiny request often fits 256 bytes, so increasing every buffer is not an established throughput fix. |
| Native transfer: [socket-listener.c](../../../docs/experiments/aot-console/socket-listener.c), `transfer` | Per operation, malloc scratch, create callback/buffer roots, copy outgoing data or later copy received data. This exists beneath managed pooling. Tables scan up to 64 entries. | Count transfers, malloc bytes, copies and short reads/writes. A bounded native scratch cache may remove allocator traffic; retaining send buffers directly would change snapshot/mutation guarantees and needs an explicit lease/immutable contract. Small-request CPU savings remain unmeasured. |
| Readiness/scheduling: `socket_poll`, `neoclr_runtime_async_pump_v1` | Poll scans bounded operation tables. The POSIX pending path uses a pause; Windows has a separate readiness wait. Pumping moves root ownership and parks/resumes fibers. TaskQueue drains all posted work. | Measure empty polls, readiness-to-dispatch delay, callbacks per request, drain budgets and root transitions. Evaluate a readiness wait/event backend and a bounded drain budget separately, preserving timer/cancellation wakeups and FIFO fairness. A low syscall leaf count does not rule out off-CPU latency. |
| [Native allocation](../../../runtime/native/gc/native-gc.c), `neoclr_gc_allocate_v1` | Every allocation walks/validates all used block headers, remembers first fit, then zeroes storage. This is mutator allocation cost even if the symbol begins `gc_`. | Reduce upstream allocations first. Later evaluate segregated free space / allocation regions, but retain the current malformed-tail-before-mutation contract or explicitly design a separately trusted internal allocator. A free list alone cannot remove validation cost. A prior split-search experiment was already rejected. |
| [Root hooks](../../../runtime/native/gc/root-probe.c) | Publication walks frames/descriptors and checks plan strings repeatedly. Existing marker cache is a search hint, not a validity cache. | Count hook calls and bytes inspected; consider immutable registered descriptors/binary metadata and fewer redundant publication sites. Requires image-lifetime and compiler contracts; address-only caching or skipping validation is unsafe. Public HTTP changes are not necessary. |
| [Concurrent adapter](apply.c), [capture](../concurrent-gc-walker/snapshot.inc), [coordinator](../concurrent-gc-walker/reclaim.inc), [worker](../concurrent-gc-worker/worker.c) | Cycle start validates/copies the arena and roots; worker rebuilds an index; owner validates identities, imports marks and sweeps. Poll takes a mutex at each visit. Initial load discards 33.6% of snapshots. | Attribute capture/copy/trace/import/sweep separately. Test allocation-byte/live-heap debt and bounded idle cleanup rather than starting every idle opportunity. Consider a cheap completion notification and bounded scratch reuse. Preserve pressure fallback, root invalidation, publication ordering, same-address protection and shutdown join. Epoch stability alone is not proof of applicability. |

## What to borrow from existing collectors and libraries

These are reviewed primary sources, with our assessment separated from their claims.
No external percentage is a predicted neoCLR gain. Previously reviewed research in
[concurrent-gc-evaluation](../../design/concurrent-gc-evaluation.md) remains relevant.

- **.NET:** [background GC](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/background-gc)
  overlaps generation-2 work while foreground ephemeral collections can interrupt it.
  [Allocation fundamentals](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/fundamentals)
  describe allocation-pointer fast paths. Our assessment: cheap allocation and
  avoiding work are separate from concurrent tracing; generations are not a
  prerequisite for the next POC experiment. We cannot copy a moving-heap design
  without native-borrow and root relocation contracts.
- **Go pacing/allocation:** the [GC guide](https://go.dev/doc/gc-guide) relates GC
  frequency to allocation/live heap and the memory/CPU tradeoff. The
  [allocator](https://go.dev/src/runtime/malloc.go) uses size classes, spans and
  per-P caches. Our assessment: replace call-count-only scheduling experimentally,
  and investigate allocation metadata before more parallel marking. Keep heap
  limits and explicit/pressure collection; using more memory must be reported.
- **Go Green Tea:** the [design](https://go.dev/blog/greenteagc) groups tracing by
  pages with seen/scanned state to improve locality; sparse cases can lose. The
  [Go 1.26 notes](https://go.dev/doc/go1.26#runtime) describe its default status.
  Our assessment: a useful later tracer experiment if trace/cache cost dominates.
  Our variable-size arena and conservative object-word scan lack Go's size-class
  layout and precise pointer maps. Do not import its reported gains or AVX-512
  kernel to this Apple M1 workload by analogy.
- **Go hybrid barrier / SATB:** the [current barrier source](https://go.dev/src/runtime/mbarrier.go)
  documents deletion/insertion protection and bulk-copy entry points; the
  [historical proposal](https://github.com/golang/proposal/blob/master/design/17503-eliminate-rescan.md)
  explains the graph invariant. Our assessment: a nonmoving barrier-based collector
  could avoid whole-arena copies and coarse invalidation, but needs all reference
  stores, bulk copies, roots and C memory ordering covered. Current copied snapshots
  are not barrier-based SATB. Do not remove park/resume invalidation to improve a
  discard counter without proving replacement root-publication rules.
- **LXR/Immix:** the [2022 extended research paper](https://arxiv.org/html/2210.17175v1)
  combines block/line allocation, coalescing reference counting, occasional tracing
  and limited copying. It illustrates that short pauses alone do not guarantee
  application throughput. Our assessment: learn from region allocation and reducing
  tracing frequency, rather than introducing reference-count barriers/cycle handling
  into this POC now. Its JVM results do not establish neoCLR performance.
- **Tasks and streaming:** [.NET Task<T> source](https://source.dot.net/System.Private.CoreLib/src/runtime/src/libraries/System.Private.CoreLib/src/System/Threading/Tasks/Task_T.cs.html)
  stores its result in a field, supporting investigation of our one-result list.
  [ValueTask documentation](https://learn.microsoft.com/en-us/dotnet/api/system.threading.tasks.valuetask?view=net-10.0)
  makes consumption constraints explicit; it is not a transparent Task replacement.
  [System.IO.Pipelines](https://learn.microsoft.com/en-us/dotnet/standard/io/pipelines)
  provides a model for segmented buffers and consumption/backpressure. Our assessment:
  compatible ownership/lease concepts can guide future spans; defer exposing slices
  until retention rules are enforced. Pooling public mutable responses is not equivalent.

Direct OpenJDK JEP fetches failed in this review; no new claim about ZGC/Shenandoah
release status or implementation is based on those failures. Their relocation/barrier
tradeoffs remain covered by the explicitly historical prior research. Research-backed
review is not a decision to adopt a collector framework.

## Bounded sequence and acceptance criteria

1. Refresh the full compiler/library/guest cohort and rerun a small safety check,
   allocation callers and separate whole-path sampling. Preserve the old comparison
   as a native-policy experiment; do not overwrite it. This prevents repeating
   already-landed TaskQueue, callback-storage and receive-cache work.
2. Test direct incoming-request construction with shared authority validation.
   This is the first HTTP-specific candidate supported by both current source and
   inclusive sample evidence. Use current behavior as an oracle; count allocated
   bytes/objects and UTF-8/URI operations before timing.
3. Independently test header construction / unused default response work, then
   Promise result storage and plaintext adapter depth if fresh callers still rank
   them highly. Keep public Task identity and cancellation semantics stable.
4. Evaluate GC cadence and cheap completion observation as separate candidates,
   including a synchronous variant with comparable cadence. Otherwise a pacing
   improvement could be mislabelled as a concurrency improvement. Measure owner
   phase times, worker CPU, stale bytes, live heap and scratch high-water usage.
5. Consider allocator/descriptor contracts or page-local tracing only after fresh
   attribution justifies their greater scope. Defer relocation and hybrid barrier
   integration until a complete compiler/runtime store contract exists.

For every candidate: focused invalid-input/lifetime checks, same rebuilt guest
cohort across native arms, allocations/bytes before and after, then uninstrumented
alternating offered loads with all arrivals, CPU/request, sampled CPU peaks and
RSS. Reject a rate as qualified when the generator-lag target fails; retain the
trial and investigate separately. Include larger headers/body streaming and slow
clients for buffer changes, and an independent load generator before a strong
sustained-capacity claim. The ten-byte connection-close benchmark does not measure
keep-alive, TLS, routing, JSON or large-body throughput. Do not change its wire
contract in only one arm. No new optimization or speedup is claimed by this review.
