# JSON parser safepoint follow-up — 2026-10-19

The original JSON probe used the build helper's every-boundary synchronous GC,
not the opt-in paced synchronous mode used in the HTTP load cohorts. The recorded
31.6% CPU regression is real for that configuration; it must not be presented as
an HTTP paced-GC or concurrent-GC result. No collector default is changed here.

build.py relinks the same baseline and initial borrowed-parser guest objects with
explicit every-boundary/paced flags, plus separate instrumented variants. report.c
records cumulative main-thread allocations, root publications, collections, GC
timer and peak sampled live/arena bytes at process exit. profile.py checks each
process output and collector failures. This diagnostic helper targets the macOS
benchmark host; it is not a public Native API or a cross-platform harness.

## Initial borrowed-parser diagnosis

| Metric | Original parser | Initial borrowed parser |
| --- | ---: | ---: |
| Allocations (either policy) | 46,816 | 31,616 |
| Root publications (either policy) | 1,207,661 | 1,588,261 |
| Every-boundary collections | 1,207,664 | 1,588,264 |
| Paced collections | 18,873 | 24,820 |
| Paced peak live payload bytes | 1,881 | 1,795 |
| Paced peak arena used bytes | 5,240 | 4,904 |

Root publications increase 31.5%. The paced 64-boundary fallback dominates this
workload, so collections still increase despite fewer allocations. Instrumented
GC time falls from about 990/1,318 ms (original/borrowed) in every-boundary mode to
23/28 ms with pacing. Instrumented timing is diagnostic, not the performance score.

Four alternating uninstrumented paced pairs use mean process CPU 0.4905 versus
0.6257 seconds: the initial borrowed parser still regresses about 27.5%. Pacing
substantially reduces absolute runtime but does not erase the extra parser work.
These are whole-process round trips with serialization, not isolated parser timings.

## Bounded parser correction

Reuse the first string-token scan to decode unescaped contents directly; instantiate
the escape reader only for escaped/control-containing tokens. Cache immutable view
lengths within loops. Check keyword lengths before comparing bytes and use explicit
start/count windows at internal boundaries instead of constructing Range paths.
Decode numeric text from the existing token window. Owned DOM results, escape
validation, quotas and malformed-input precedence stay intact. Native checked
access, root publication and collector policy are unchanged.

This follows byte-oriented parsing as used by System.Text.Json; it removes redundant
work within neoCLR's provisional bounded parser rather than introducing another
public adapter. It does not replace GC optimization or establish HTTP throughput.


## Revised allocation and collection counts

| Metric | Original parser | Revised borrowed parser |
| --- | ---: | ---: |
| Allocations | 46,816 | 29,616 |
| Allocated payload bytes | 1,480,065 | 1,002,065 |
| Root publications | 1,207,661 | 1,083,461 |
| Every-boundary collections | 1,207,664 | 1,083,464 |
| Paced collections | 18,873 | 16,933 |
| Paced peak live payload bytes | 1,881 | 1,795 |
| Paced peak arena used bytes | 5,240 | 5,000 |

Allocation count is 36.7% below the original parser and 6.3% below the initial
borrowed version. Payload bytes fall 32.3% from the original. Diagnostic allocation
traces reconcile with zero dropped records. Root publications fall 10.3% from the
original and 31.8% from the initial borrowed implementation. This is evidence of
less work at the same safe boundaries; it does not disable those boundaries.

Peak live bytes are sampled at collections and are not total process memory.
The initial borrowed parser's paced arena peak was 4,904 bytes, so the revised
5,000-byte peak is slightly higher despite lower allocation volume. Pacing and
reachability at individual collection points affect these peaks.

## Reproduction

Original guest: target/json-input-baseline; initial borrowed guest:
target/json-input-candidate; revised guest: target/json-scan-candidate.
Relink each with build.py; use profile.py for diagnostics and ../compare.py for
four alternating uninstrumented pairs per policy. Build/test/profiling activity
must finish before timing. All use Raven codex/neoclr dcf7ac41f and the same native
runtime/compiler. The final class library is target/json-scan-libraries.

Compressed evidence preserves each guest/relink build, input hashes, allocation
traces, policy profiles, timings and focused contract reports. MANIFEST.json
checksums the stored files. Earlier measurements remain in the parent directory.

## Final uninstrumented timing

All builds, profiles and tests finished before these paired runs. Four rounds
alternate original/revised and revised/original. Every process validates.

### paced

| Round | Original CPU seconds | Revised CPU seconds | Original wall seconds | Revised wall seconds |
| --- | ---: | ---: | ---: | ---: |
| 1 | 0.4633 | 0.4447 | 0.4654 | 0.7429 |
| 2 | 0.4573 | 0.4087 | 0.4590 | 0.4108 |
| 3 | 0.4551 | 0.4083 | 0.4568 | 0.4100 |
| 4 | 0.4606 | 0.4075 | 0.4625 | 0.4091 |

Mean CPU: 0.4591 versus 0.4173 seconds (9.1% lower).

### every-boundary

| Round | Original CPU seconds | Revised CPU seconds | Original wall seconds | Revised wall seconds |
| --- | ---: | ---: | ---: | ---: |
| 1 | 1.3812 | 1.2556 | 1.5736 | 1.4543 |
| 2 | 1.3550 | 1.2113 | 1.3572 | 1.2130 |
| 3 | 1.3586 | 1.2241 | 1.3607 | 1.2325 |
| 4 | 1.3533 | 1.2099 | 1.3553 | 1.2122 |

Mean CPU: 1.3620 versus 1.2252 seconds (10.0% lower).


The initial CPU regression is resolved in this fixed round-trip workload under
both synchronous modes. First-process startup effects remain visible in wall time;
no samples were discarded. These timings do not establish sustained HTTP capacity,
near-limit JSON performance or concurrent-GC behavior.

All 16 stream cases pass in native AOT, interpreter and JIT (48 executions), and
all 21 DOM cases pass in interpreter/JIT (42 executions). Existing checks cover
empty/Unicode/control/escaped tokens, surrogate failures, duplicate names, numeric
spelling, quotas, fragmented input and stream ownership. Native API rendering and
legacy reference fingerprints are refreshed; four native snapshot checks pass.
No public signatures, compiler backend, native checks or GC policy changed.
