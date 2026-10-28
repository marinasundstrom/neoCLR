# Mixed HTTP and provider qualification — 28 October 2026

Development macOS ARM64 lazy-JIT diagnostic cohort; one run per mode, four clients,
1 MiB heaps, identical allocation/publication budgets (1024/16384). Baseline and
shared ready storage retain owner-affine managed execution. This does not qualify
parallel managed HTTP workers or establish production capacity.

## Correctness and collector boundary

The 24 literal endpoint methods cover four variants each of shared-capture short
tasks, allocation work, synchronous file read, synchronous file write, downstream
HTTP and nested Task.Run fanout. Downstream delays are 2/5/10/30 ms; the harness
owns its loopback peer and temporary files. First-use requests cover all routes,
then four clients each execute all routes. Bodies, capture mutations/aliases,
file contents, zero-error request counts and clean shutdown are checked.

All four cases pass: **480 requests**. DNS and multi-address connect native wrappers
were missing from native-body classification and concurrent-GC provider admission.
That omitted their published native-call frame; add classification plus exact
provider signatures and owner-host admission. The earlier mixed live case failed
at DnsCompletion.StartUntil; the same workload now succeeds. Restricted owned-worker
callback service admission remains unchanged: these are owner-host services.

DNS resolver threads keep native hostname/address copies, never guest heap access.
Callback lifetime uses existing host roots. DNS result/copy helpers only initialize
fresh noncollecting reference arrays; allocations created after the marker snapshot
are traced on the owner at remark. They neither mutate existing reference payloads
nor collect between constructing their native temporaries. This is an essential
admission invariant, not general permission for barrier-free reference stores.
Multi-address connect snapshots addresses before asynchronous admission.

File reads close descriptors before rooted text allocation; writes borrow rooted
inputs and allocate no managed objects. Ordinal Contains is read-only. Both now
have explicit audited owner-host admission and exact imported signatures.

`providers.json` records **12 successful executions**: file, rooted-producer and
socket-client contracts in optimized/UBSan builds for both collectors. The socket
case starts active epochs before DNS result construction and array copying, roots
results, collects, then validates payloads; cancellation, deadlines, resolver limits,
multi-address fallback and cleanup remain covered. File cases cover embedded NUL/
UTF-8, size limits, failed writes without truncation, missing files and collection.
Compiler mutation tests: **51 pass, 31 ignored**, no failures. Ignored tests are not
counted as validation; generated HTTP is executed separately through its retained
recipe. An initial provider build used an invalid local SDK path; the retained
successful run explicitly selects the Xcode SDK.

## Diagnostic measurements

| Mode | Checked requests | 96-request warm batch (ms) | Queue peak | Queue residence median / p95 (µs) |
| --- | ---: | ---: | ---: | ---: |
| baseline-sync | 120 | 141.7 | 13 | 289.8 / 928.1 |
| shared-sync | 120 | 150.2 | 14 | 399.4 / 949.2 |
| baseline-live | 120 | 206.5 | 12 | 265.6 / 1326.7 |
| shared-live | 120 | 220.9 | 15 | 556.6 / 1369.5 |

These are traced single-run observations, **not benchmark medians or throughput
claims**. The shared candidate is slower in this cohort; retain that regression.
Queue residence includes first-use and warm requests, so lazy compilation can
contribute to those intervals.

Each case records all 240 Task.Run admissions/dequeues, no abandonment or trace loss.
Queue peaks 12–15 improve pressure coverage over the earlier depth-one cohort.
Root-hold intervals describe callback dispatch ownership, not full managed task
execution. Marker starts/completions are 148/136 baseline-live and 158/146 shared-live;
each cancels 12 epochs and ends inactive. No claim that a marker overlapped every
request follows. Both native carrier mappings are released at teardown; peak stack
reservation remains 2,162,688 bytes and is not RSS or per-Task memory.

## HTTP/binding blocking audit and next measurements

The author asks to inspect the stack/bindings for blocking and avoidable work.
Source inspection finds:

- FileText read/write runs synchronous open/read/write/truncate loops on the owner.
  Task.Run currently changes scheduling, not the execution thread. This can delay
  unrelated handlers; tiny cached files here do not quantify cold/slow storage.
- HttpClient starts DNS for every exchange, including numeric hosts. Resolver work
  uses a bounded native thread set; results are observed by owner polling. There is
  no shared completion wake signal for DNS. The scheduler retains a 1 ms poll cap
  for DNS/timers/legacy TLS. That cap is not a measured 1 ms penalty per request.
- Request encoding occurs before lookup; parsing, response buffering and payload
  allocations remain owner work. The current trace cannot attribute their costs.

Next: separate untraced warm/load cohorts from bounded phase/CPU profiles, measure
HTTP parsing/serialization and binding duration, correlate DNS/connect/readiness
with queue waits, and include slow-file/no-I/O controls. Compare both collectors,
AOT and fixed resource limits. Do not adopt relationship tracking, a special HTTP
scheduler, or automatic blocking offload without measured benefit and ownership
qualification. General-purpose shared wake/completion service is a candidate, not
an implemented change in this slice.

## Reproduction and limits

Compile `docs/experiments/shared-scheduling/Mixed.rvnproj` with Raven neoCLR
`91c1edca8` or later and the matching native libraries. This run uses libraries from
`target/dynamic-access-20261028/bundle-generic` and a separately rebuilt compiler;
older packaged compilers still need the now-fixed explicit-overload workaround.
`prepare-http.py --mixed --trace` accepts the same bundle/runtime/runner/host and
selection inputs as the existing HTTP experiment. Retained `build.json` records
exact commands and source hashes. Then run `mixed-http.py --clients 4` and
`summarize-http-trace.py`; use new output directories. `validate-files.py` runs the
native contracts with an explicit valid SDKROOT on this host.

The project intentionally compiles current Web sources like the existing internal
HTTP benchmark. Local captures/static helper workarounds for separate Raven native
emitter limitations remain. No AOT, cold storage, offered-load capacity, full
execution-context retention or cross-runtime ranking is claimed. Website content
is updated for the next manual publication.
