# Cached allocation-header validation (2026-10-25)

## Question and boundary

The previous CPU profiles locate real work in `block_valid`; forcing it inline
removed calls but did not materially reduce CPU. This experiment instead retains
validated span, payload length and allocation kind beside the last allocation
hint. It reuses that validation only after current owned-base admission and exact
field comparisons. Current state, arena extent and destination bounds remain
checked. Changed fields take full validation and refresh the tuple after success.
The runtime's legitimate intern-cell-to-bytes conversion is explicitly tested.

This adds 24 bytes of private native TLS, with no managed allocation or object
header change. The other three hints retain full validation. Snapshot allocation
identity validation still runs first; old-object mutation still uses the same gate
and deletion logging. Reclamation/reset/release revoke the existing ownership
proofs. Merely remembering an address cannot grant access after reuse/coalescing.
Matching metadata on a newly admitted allocation establishes the same constraints;
it does not require preserving an old allocation's identity across reclamation.

This is an internal nonmoving owned-heap optimization, not a new public contract,
shared-heap permission or record-specific GC rule. It follows the existing
[research and qualification context](packed-tls-results.md) rather than copying
CLR object headers. Production GC and compiler defaults remain unchanged.

## Correctness

Enabled, disabled and cache-disabled native suites pass optimized and
undefined/bounds sanitizer checks. Negative controls reject missing deletion
logging and revoked newest/older hint admission. A new negative control bypasses
the tuple comparisons and must abort on corrupted metadata. Tests reject changed
span, payload size, kind and state, reject writes beyond a shortened extent, and
accept the runtime's valid intern-kind transition followed by another cached write.
Existing independent-owner isolation, reclamation, reuse and stale interior-header
checks remain in force. Passing tests does not prove every thread interleaving.

Current generated-code qualification passes 600 entry and 840 record-array JIT/AOT
cycles with the retained private `speed` backend setting. No compiler change was
made, so earlier complete compiler/full-server JIT evidence is reused. The current
HTTP measurements link freshly generated native collectors to those same retained
AOT objects. Worker/shared-heap, pause distribution and saturated framework
comparisons remain separate acceptance requirements.

## Measurements

Ninety standalone executions pass exact outputs, cleanup and equal allocation
counts/bytes. Live CPU medians (proof / full-validation control) are 0.074390 /
0.074849 seconds for tasks, 0.373437 / 0.375286 for collections, and 0.084762 /
0.084765 for arrays. These separate cohorts show less than 1% difference.

Three interleaved rounds of 2,048 measured plus 16 warmup requests across five
configurations and two handlers validate 61,920 responses:

| Handler | Full-validation live requests/s | Proof live requests/s | Full-validation live CPU | Proof live CPU |
| --- | ---: | ---: | ---: | ---: |
| Plaintext | 1,848 | 1,834 | 1.094467 s | 1.105035 s |
| JSON | 1,088 | 1,093 | 1.824658 s | 1.819503 s |

Plaintext throughput falls 0.8% and CPU rises 1.0%; JSON throughput rises 0.4%
and CPU falls 0.3%. Guarded-sync CPU also moves in opposite directions: plaintext
0.944814 to 0.975603 seconds, JSON 1.596288 to 1.564323. Ordinary sync remains much
faster, at 4,062/2,151 requests/s. Allocation counts/bytes match across all modes:
plaintext 429,856 / 33,057,881 and JSON 943,817 / 54,065,775 per 2,064 responses.
These mixed small differences do not establish a performance benefit.

**Decision:** retain full validation by default. Expose cached-header proof only
through `NEOCLR_LIVE_HEADER_PROOF` and qualification-tool opt-ins. This tested
hypothesis does not justify extra default TLS state or cache maintenance. The next
candidate is validation already retained by the snapshot, where reuse could avoid
extra cache writes; that remains unimplemented and needs its own lifetime/identity
review. Common execution-state/root costs and precise heap layouts remain open.

The initial reports use `live` for proof enabled and `live-no-header-proof` for its
control. The final interface deliberately uses `live` for the unchanged default
and `live-header-proof` for the opt-in. The original generated source is archived
as `header-proof-original-source.c.gz`; report commands preserve the old inverse
macro. Final native safety runs verify both opt-in and default spellings. A final 160-response
HTTP smoke run verifies the renamed measurement modes (not performance evidence). No
performance conclusion depends on silently relabeling those original trials.

## Reproduction and evidence

Use `concurrent-live/validate.py --sparse-index [--header-proof]`, or
`--disable-store-cache` for the uncached path. Use `concurrent-workloads/measure.py
--guarded-control [--header-proof]` with retained `optimizer-build` objects.
HTTP uses `concurrent-store/measure-plaintext.py --include-guarded-sync
--compare-header-proof --pairs 3 --requests 2048` with retained
`optimizer-http-build` objects. Each trial includes 16 warmup responses. Compilation
and correctness runs finish before timed work; the measurement cohorts run
sequentially. Raw `header-proof-*.json.gz` reports preserve commands, input hashes,
outputs, failures and individual trials; `header-proof-summary.json` contains
medians. These are sequential loopback/fixed-work comparisons, not saturation
capacity or general framework parity.
