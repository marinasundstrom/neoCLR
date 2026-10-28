# Owned generated callbacks with active concurrent GC — 2026-10-28

Private macOS ARM64 worker prerequisite for HTTP integration. The callback bridge
still serializes managed execution through the pool gate; the HTTP owner heap is
not attached, and Task.Run scheduling/defaults are unchanged.

## Implemented boundary

The live-store compiler now admits a restricted native-host-bootstrap profile.
Generated bodies, static initialization, exports and callback adapters pass existing
store lowering and complete call-closure validation. Native callbacks may use the
existing admitted primitive boxing/reference dispatch, type queries, integer
formatting and limited string operations. Async/socket/TLS/owner scheduling and
arbitrary native providers remain rejected for this profile. This does not change
Raven language or public managed APIs.

Bootstrap images export a private collector contract (ordinary=0, live stores=1).
`NEOCLR_WORKER_OWNED_CALLBACKS` requires exclusive heap adoption; before bootstrap,
the adapter compares the image contract with the linked collector's own query.
No caller-provided "live" switch can substitute for that match. A mismatched ordinary
image is rejected without heap mutation. Parallel worker-participant mode is a
compile-time incompatible combination.

The gate protects adoption, generated invocation and release. All pool captures,
results, job diagnostics and local roots remain published through the adapter
frame; static storage is retained across invocations. Release joins/cancels any
marker before withdrawing those roots or destroying the temporary context. Native
static storage uses the existing guarded provider in live builds. This maintains
shared identity rather than copying captures.

## Accepted checks

`accepted.json.gz` retains exact commands, inputs and hashes. Both modes use a
128 KiB pool and matching allocation/publication pacing budgets (1,024/16,384).
Ordinary and live-store objects come from the same selected Raven input and matching
compiler library test executable. Native C checks use optimized and UBSan builds;
generated machine code itself is not UBSan-instrumented.

| Configuration | Chained deliveries | Worker allocations | Worker collections | Worker epochs started / finished / cancelled |
| --- | ---: | ---: | ---: | --- |
| Sync optimized | 64 | 160,196 | 192 | 0 / 0 / 0 |
| Sync UBSan | 64 | 160,196 | 192 | 0 / 0 / 0 |
| Concurrent optimized | 64 | 160,196 | 128 | 64 / 64 / 0 |
| Concurrent UBSan | 64 | 160,196 | 128 | 64 / 64 / 0 |

Each callback mutates the retained aliased cyclic graph, replaces edges, keeps a
displaced live local, allocates 2,500 boxes and advances shared static state. The
owner forces collection before taking the ticket. Successful jobs hand the same
closure back through a rooted handle; one deliberate fault preserves diagnostic
text and later jobs recover. Every worker invocation returns with settled epochs.
Owner epochs also settle (92/28/64 optimized, 121/57/64 UBSan); their counts depend
on pump scheduling. This establishes actual worker marking, not just selection of
the concurrent collector. Counts are diagnostic evidence, not performance rankings.

The existing four-worker shared-host fixture also passes both modes optimized/
UBSan: 87 successful callbacks and one isolated fault, shared statics/identity,
collection, cancellation winning at settlement, foreign-root admission recovery,
bootstrap failure and clean teardown. Managed callbacks remain serialized. The
foreign-root pressure control now expects runtime admission failure in owned mode
before root-slot allocation; ordinary mode retains its previous memory-limit code.
The first reuse of that control expected the old code and failed; its report is
retained separately. No runtime fault was hidden or converted into a success.

Compiler mutation-store checks: 51 pass, 31 intentionally ignored executable-host
cases; admitted host-I/O rejection and the new callback/TLS rejection are included.
The selected Raven consumer supplies executable generated-code evidence. Ordinary
worker and GC assembly are byte-identical to the preceding HEAD. TSan remains
unavailable on this host, as documented in earlier slices.

## Reproduce and remaining work

Build the `neoclr-codegen` library test executable with Cargo. Obtain a closed-world
bootstrap selection JSON for `worker-delivery/Generated.rvn` using the matching
bundle/libraries (the previous generated fixture records such a command). Then:

    SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk python3 docs/experiments/worker-delivery/validate-owned-callbacks.py --selection <command.json> --runner <neoclr-codegen-library-test> --output <new-directory>

The retained runner/source hashes identify the exact inputs. The private ignored
selection test is reused to compile each collector profile; its historical name
mentions HTTP but this consumer is a worker fixture, not an HTTP test.

The author now directs a bounded "good enough for now" HTTP integration, deferring
minor optimizations. Do not tune ownership metadata further without an application
need. Next connect the qualified boundary to useful HTTP processing while preserving
owner roots and service affinity, then execute the [fresh HTTP plan](../../experiments/worker-delivery/http-plan.md).
No HTTP throughput improvement, general parallel Task.Run or production adoption
is claimed by this prerequisite. Existing HTTP measurement tables stay unchanged.

Website content checks: 26 tests pass. Publication remains a separate manual step.
