# Buffered HTTP JSON byte input — 2026-10-19

This probe exercises 600 buffered JsonContent.ReadNode calls, with canonical output
checks: three numeric/Unicode/escaped fixtures, each backed by an array and an
ArrayList. It does not use sockets or measure HTTP server throughput. Both arms
use the optimized parser from 9970f9874; the candidate bypasses whole-document
HttpText.Decode through the new ReadOnlySlice<byte> deserialization overload.

Array-backed content borrows its bytes; other sequences still take one snapshot.
Complete strict UTF-8 validation precedes JSON syntax errors, preserving HTTP's
previous error precedence. Returned strings/DOM values own their storage.

| Metric | Baseline | Candidate |
| --- | ---: | ---: |
| Allocations | 15,821 | 15,221 |
| String allocations | 4,017 | 3,417 |
| Allocated payload bytes | 550,785 | 528,785 |
| Root publications | 586,360 | 610,560 |
| Every-boundary collections | 586,363 | 610,563 |
| Paced collections | 9,165 | 9,543 |
| Paced peak sampled live payload bytes | 2,232 | 2,257 |
| Paced peak sampled arena bytes | 5,576 | 5,584 |

Exactly one document string per read disappears: 3.79% fewer total allocations and
3.99% fewer allocated payload bytes, with no dropped attribution records. Array,
object, closure and boxed-value allocation counts are unchanged. The full UTF-8
prevalidation scan raises root publications 4.13%; fewer allocations do not imply
lower CPU or peak live storage. Profiled GC time is diagnostic, not a speed score.

The reusable [relinker/profiler](../../json-byte-input/20261019/gc-followup/README.md)
is retained with the earlier parser investigation.
It explicitly selects every-boundary or paced synchronous GC; no production
collector default changes. No experimental concurrent-GC claim is made.

`evidence.json.gz` retains build manifests, caller summaries, collector profiles
and unsuccessful qualification attempts. Initial test construction used unsupported
jagged byte arrays, then exposed missing reflection binding for typed scalar reads.
An empty reflection scope and catalog scopes were rejected. The final harness
retains one runtime type identity without constructors/properties to bind the
ObjectMapper services; this is test configuration, not a new public retention rule.
The current compiler is Raven codex/neoclr dcf7ac41f; no compiler source changed.

## Validation and timing

All 20 focused JSON/stream cases pass in interpreter, JIT and AOT (60 executions).
They include scalar mapping, independent result ownership, nonzero byte windows,
UTF-8 boundaries and malformed sequences, quotas/error precedence, sequence aliases
and refusal to consume an unbuffered body. Native and CLI API snapshots validate.

Four alternating uninstrumented pairs per synchronous policy retain all runs,
including first-run startup effects. Paced mean CPU is 0.237126 → 0.244776 seconds
(+3.23%); every-boundary mean CPU is 0.719386 → 0.733509 seconds (+1.96%).
The allocation reduction is established, but this probe does **not** show a CPU
speedup. Full-document UTF-8 validation remains an optimization candidate; preserve
error precedence when evaluating a fused/error-path validation approach.
