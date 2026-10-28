# Bounded worker-to-owner delivery — 2026-10-28

Implemented a private opt-in POSIX wake descriptor and owner-pump source, preserving
the existing pool's shared capture/result roots. Bounded delivery now resumes parked
owner executions while real TCP work progresses; the owner does not join each job.
This is a candidate for HTTP integration, not evidence of an HTTP improvement.

## Contract and safety

NEOCLR_WORKER_DELIVERY enables lazy owner registration of a nonblocking,
close-on-exec pipe. One byte represents all terminal, not-yet-dispatched tickets.
Job settlement, direct take and bounded dispatch refresh that level under the
existing pool gate. Budget exhaustion retains readability while another
undispatched terminal ticket remains. No pointer/result payload travels through
the pipe. Only the pool reads/writes it; external code may poll the borrowed fd.
Registration after completion signals immediately. Close joins/cancels normally
and leaves delivery drainable; destruction closes the descriptor after outstanding
tickets/roots are consumed. Caller removes borrowed descriptors before destruction.

The call-scoped external source polls once per owner-pump iteration, after ready
contexts and drain work run, before deciding to wait or exit, and completes operations on the owner. The fixture limits dispatch
to two tickets per iteration; the existing pool API caps a call at sixteen. The
socket wait includes one extra descriptor alongside its at-most-64 operations.
The existing timeout cap stays for DNS/timers/other sources. A -1 source descriptor
provides the otherwise-identical polling control. Providers must do bounded native
work, never invoke guest code directly, and retain their state for the whole pump.
No preemption or wall-time guarantee follows from a dispatch-count budget.

Captures are not serialized/copied: jobs mutate the same admitted pool object under
the existing heap gate and return references to it. Ticket roots keep it alive
after the original root is released; take adopts a consumer root before freeing
the slot. Every adopted reference observes the final shared mutation after
collection. Parked owner locals also survive collection. This **does not admit
arbitrary owner-heap closures into the pool** or export foreign-heap managed pointers
into Tasks. The existing pool owns its heap; general capturing HTTP/Task.Run work
still requires shared-heap/compiler/service integration. Do not replace that
product contract with copied values merely to make the transport usable.

## Qualification

- New descriptor contract passes optimized and fail-fast UBSan under both
  collector builds: coalescing, late registration, zero/limited dispatch budgets,
  direct-take suppression, wrong-owner rejection, nested dispatch rejection,
  bounded admission, stale tickets, cancellation, close/drain and descriptor cleanup.
- Shared capture identity and rooted result adoption survive forced collection;
  all eight returned references observe the same final counter. The integrated
  fixture checks exactly-once completion/resumption, a real TCP byte and parked
  locals under both collectors, optimized and UBSan.
- Existing eleven pool groups pass with compiled-worker coordination, tracing,
  local queues and delivery enabled together. This reuses the existing collector
  protocol rather than relaxing managed access rules.
- Separate async captures contain 144 genuine parks per collector, with zero lost
  events, and pass the existing exporter. Concurrent epochs actually start/finish;
  final runs also check started = finished + cancelled, with no active epoch
  at the recorded teardown boundary. Forced collections cause cancellations.
- Without the experiment flag, native workers, async carrier and socket adapter
  produce byte-identical ARM64 assembly to the pre-change sources. No production
  defaults or managed API/reference signatures change.

ThreadSanitizer builds succeed, but both pool tests terminate with signal 11 and
no diagnostic. A trivial TSan executable also terminates before its main-entry
print with the same Xcode SDK/toolchain, while its unsanitized control passes.
Thus race-sanitizer qualification is unavailable on this host, not passed. A first
smoke build using the automatically selected CommandLineTools SDK hit an SDK/linker
mismatch; the retained smoke control uses the same explicit Xcode SDK as the pool
tests. No runtime behavior was changed to work around the sanitizer startup failure.

## Measurement

macOS ARM64. Two workers, eight jobs per round, one owner and a real loopback TCP
peer thread. Each job waits 400 µs (blocking control, not async I/O), increments a
shared object under the heap gate, publishes its reference and timestamps just
before returning. The peer waits 200 µs after an owner readiness byte, then sends
one byte through the real socket adapter. The owner pumps all worker/TCP completions
without job joins. Forced pool collections before/after delivery verify roots.
256 rounds per invocation; five interleaved polling/wake pairs per collector.
No tracing/sanitizers in timed binaries. Both modes register and maintain the pipe;
only participation in the wait set differs, isolating wake responsiveness rather
than estimating the complete cost of adding pipes to an uninstrumented pool.

Latency is worker callback-return timestamp to owner-context resumption: it includes
settlement, notification, dispatch and carrier work. It is not syscall-only wake
latency or Task.Run-to-completion time. Values are medians of per-run statistics,
not pooled percentiles. Timing includes root checks, forced collection and all
process threads (including the peer). Desktop load/core placement is uncontrolled.
The work is wait-heavy with a tiny live graph; no general collector advantage,
HTTP throughput, frame deadline, energy or sustainable-capacity claim follows.

### Initial screen — superseded early-source-check candidate

| Collector | Delivery | Batch wall ms | Process CPU ms | Resume p50 / p99 µs |
| --- | --- | --- | --- | --- |
| sync | 1 ms polling | 1310.9 | 113.7 | 1550 / 3552 |
| sync | Wake descriptor | 538.2 | 115.8 | 9 / 71 |
| concurrent | 1 ms polling | 1253.3 | 146.2 | 1561 / 3607 |
| concurrent | Wake descriptor | 568.9 | 149.0 | 10 / 130 |

### Second screen — same superseded source ordering

Added epoch cancellation accounting and separate trace qualification; timed work
is unchanged. Retain both cohorts: CPU and latency vary materially across them.

| Collector | Delivery | Batch wall ms | Process CPU ms | Resume p50 / p99 µs |
| --- | --- | --- | --- | --- |
| sync | 1 ms polling | 1219.9 | 221.6 | 1354 / 2801 |
| sync | Wake descriptor | 565.3 | 224.4 | 18 / 139 |
| concurrent | 1 ms polling | 1252.6 | 281.0 | 1364 / 2871 |
| concurrent | Wake descriptor | 581.0 | 278.8 | 21 / 127 |

### Accepted source-order correction

Review found that a resumed execution could submit a new worker job after the
source poll, leaving a stale idle decision. Polling now happens after ready
contexts/drain have run and before the scheduler idle/wait decision. A 32-link
await/submit regression fails against the earlier source ordering and passes
optimized/UBSan under both collectors. This also avoids waiting when the latest
bounded dispatch has made more contexts ready, improving the polling control.
The two earlier cohorts remain diagnostic history, not final candidate claims.

| Collector | Delivery | Batch wall ms | Process CPU ms | Resume p50 / p99 µs |
| --- | --- | --- | --- | --- |
| sync | 1 ms polling | 771.1 | 202.3 | 479 / 1324 |
| sync | Wake descriptor | 570.6 | 218.4 | 21 / 162 |
| concurrent | 1 ms polling | 711.5 | 258.4 | 482 / 1326 |
| concurrent | Wake descriptor | 587.9 | 321.4 | 39 / 162 |

The accepted cohort reduces batch wall time 26.0% (sync) and 17.4% (concurrent).
Median return-to-resume latency falls from 479/482 µs to 21/39 µs; p99 falls from
1,324/1,326 µs to 162 µs in both modes. CPU rises 8.0%/24.4% in ratios of medians.
That cost is material, particularly with concurrent GC; faster notification is not
a universal efficiency win. Retain individual variation and forced-collection
costs rather than selecting the earlier, more favorable comparisons.

In the accepted concurrent cohort, median epoch starts rise from 2,025 to 3,138
and finishes from 1,513 to 2,626; forced-collection cancellations stay at 512.
More frequent owner progress/idle checks and collector pacing deserve attribution
in the HTTP follow-up. These counts correlate with the higher CPU cost but do not
by themselves quantify its cause or justify changing collection policy.

Each cohort checks 40,960 worker deliveries and 5,120 TCP rounds. The polling
control relies on the existing 1 ms fallback. These gains concern wait gaps,
not faster computation, collection or a comparison with Go/.NET.

## Separate profiles and integration gate

Two-second macOS stack samples use 2,048 rounds for each collector/delivery pair.
The first captures use the earlier candidate; accepted-source captures are retained
separately after the source-order correction.
They retain expected worker sleeps, peer receive waits and owner waits; wake runs
show poll and pipe writes. These samples include blocked threads and are not CPU
percentages. Profiled timings are excluded from the tables. Raw captures and
commands are retained.

The author explicitly requires proving applicability to HTTP before eventual
integration, followed by a **new HTTP benchmark after integration**. Keep this
candidate opt-in. First qualify an HTTP adapter with correct shared captures,
owner affinity, roots, cancellation and shutdown, then run the
[fresh HTTP comparison plan](../../experiments/worker-delivery/http-plan.md).
The native fixture numbers cannot populate HTTP performance claims.

[Implementation and reproduction](../../experiments/worker-delivery/README.md).
Compressed evidence is indexed by decompressed SHA-256 in artifacts.json.
