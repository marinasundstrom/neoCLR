# Inline native header validation (2026-10-25)

The [sustained CPU profiles](packed-tls-results.md) place `block_valid` beneath the
store adapter in both guarded-sync and live execution. The experiment asks Clang/GCC to inline that existing predicate. Its body and every caller's
validation order remain unchanged. This exposes header loads and conditions to
the caller's optimizer; it does not replace current validation with a remembered
proof, remove fields from validation, or alter snapshot identity/ownership rules.
The final generator keeps the previous declaration by default;
`NEOCLR_LIVE_INLINE_VALIDATION` explicitly enables the experiment.
Ordinary native GC, managed code, root descriptors and all public/native ABIs are
unchanged. This is an internal code-generation tradeoff under the existing CLR
comparison, not a changed managed GC contract or a new default collector.

Both native variants pass the O2/UBSan suite, including the deletion-log and two
stale-range negative controls, detached identity, bounds, reclamation and the two
independent-owner isolation checks. Six hundred entry and 840 record-array cycles
pass with the new candidate through JIT/AOT and forced/paced hosts. Reuse unchanged
compiler and previous full-server JIT evidence; this slice rechecks the changed
native implementation through those focused generated-code consumers and full
HTTP AOT execution.

The ARM64 store adapter grows from 429 to 545 instructions while its seven
`block_valid` call sites disappear. In the complete native store-test executable,
`__text` grows from 44,032 to 44,360 bytes (328 bytes, about 0.75%); its page-rounded
`__TEXT` segment remains 49,152 bytes. These are code-size measurements for this
fixture, not claims about all binaries or managed allocation savings. Raw adapter
assembly and exact size output are retained with the qualification results.


## Matched results and decision

The initial five-round short cohort (256 measured plus 16 warmup requests per
trial) passes 13,600 HTTP responses and equal allocation checks. It used the initial
experimental generator with `live` meaning inline and `live-outlined` the control.
Plaintext inline CPU was 0.164301 versus 0.158525 seconds (worse), while JSON was
0.256142 versus 0.258678 (slightly better). This mixed result justified a longer
comparison rather than adopting a default from a favorable subset.

The final opt-in interface reverses those labels explicitly: `live` is the unchanged
outlined default and `live-inline` is the experiment. Three alternating rounds at
2,048 measured plus 16 warmup requests validate another 61,920 responses:

| Handler | Outlined live requests/s | Inline live requests/s | Outlined live CPU | Inline live CPU |
| --- | ---: | ---: | ---: | ---: |
| Plaintext | 1,845 | 1,857 | 1.142877 s | 1.139172 s |
| JSON | 1,073 | 1,091 | 1.903235 s | 1.886738 s |

Throughput differences are about 0.7%/1.7%, CPU reductions 0.3%/0.9%. Guarded-sync
CPU is 1.000636 versus 0.979070 seconds for plaintext and 1.657542 versus 1.636897
for JSON. Ordinary sync remains much faster (4,809/2,441 requests/s). All allocation
counts/bytes match: plaintext 429,856 / 33,057,881 and JSON 943,817 / 54,065,775 per
2,064 responses. These sequential loopback medians do not establish a statistically
significant gain, saturated capacity or framework parity.

Ninety standalone executions pass exact outputs and allocation/lifecycle equality.
Inline versus outlined live CPU medians are 0.077373 versus 0.078503 seconds for
tasks, 0.391291 versus 0.396553 for collections, and 0.090278 versus 0.090492 for
arrays. They are separate cohorts with small differences. There is no managed
allocation reduction in any sample.

**Decision:** keep header validation outlined by default. Retain an explicit
experiment/control, not a claimed optimization. The profile identified real
validation work, but removing its call boundary alone does not materially close
the synchronous/concurrent gap. Next investigate redundant work under explicit
ownership/lifetime guarantees and execution-state entry/root descriptor setup,
rather than continuing to force inlining because a function appears in a profile.
Precise per-type heap layouts remain a larger, separately qualified requirement;
current native object scans are conservative, so scalar stores cannot simply skip
barriers based on their source-language type.

## Evidence and reproduction

`inline-header-*` / `outlined-header-*` contain initial safety, generated consumers,
assembly and measurements; `header-long-*.json.gz` use the final opt-in labels.
The original generated native sources are retained for the initial safety/entry
fixtures (entry and record-array sources are identical). They preserve the initial
inverse preprocessor control so that reported compile commands remain reviewable.
The final predicate bodies are identical to those qualified variants.

Use `concurrent-live/validate.py [--inline-validation]` for current native checks,
`concurrent-workloads/measure.py --guarded-control [--inline-validation]` for the
retained ordinary build, and `concurrent-store/measure-plaintext.py
--compare-inline-validation --include-guarded-sync --pairs 3 --requests 2048` for
matched HTTP. Use the retained `optimizer-build` and `optimizer-http-build` objects;
finish compilation before timing. No production/default GC or compiler change is
part of this result. Existing worker/shared-heap and pause/scaling acceptance
requirements remain open.
