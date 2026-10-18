# HTTP header reservation — 2026-10-18

The encoding-cache allocation profile attributes the largest allocation groups to
array reservation. The request decoder starts its header ArrayList empty, then
reallocates through 4, 8, 16, 32, 64 and 128 elements for ordinary headers. Reserve
min(128, configured header limit) once in the decoder constructor. Larger headers
retain normal geometric growth and byte-by-byte limit validation.

This applies the existing capacity contract, comparable to [.NET List<T>(capacity)](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.list-1.-ctor?view=net-10.0),
without changing ArrayList defaults or adding pooling. It trades up to 128 bytes of
initial payload per decoder for fewer allocations/copies on ordinary requests.
Tiny requests may retain more capacity; idle/pending decoders also reserve it.
No transport buffer escapes, ownership changes, shared mutable arrays or GC policy
changes. Streaming and buffered requests use the same header parser.

The regression fixture tests exact and one-byte-over header limits at 127, 128,
129, 256 and 512 bytes, plus a one-byte limit. Larger inputs arrive in separate
sends crossing the reserved prefix. These are correctness tests with 60-second
read/write budgets, not performance qualifications. Public API signatures remain
unchanged.

## Diagnostic counters

Same 200 measured requests, 20 warm-up requests and one stop exchange as the
encoding-cache baseline, including startup/shutdown. Separate allocation-site and
GC runs agree on 61,601 allocations. Diagnostics overlapped build/tooling work;
elapsed GC time is not comparative evidence.

| Counter | Encoding-cache baseline | Header reservation |
| --- | ---: | ---: |
| Allocations | 62,706 | 61,601 |
| Allocated payload bytes | 3,343,768 | 3,327,856 |
| Collections | 18,405 | 19,062 |
| Root publications | 713,322 | 710,228 |
| Peak live payload bytes | 46,528 | 46,528 |
| Peak arena bytes | 69,072 | 68,768 |

Allocations fall 1.76%, bytes 0.48% and root publications 0.43%, but collections
increase 3.57%. All allocations reclaim at shutdown with zero GC failures.
No throughput gain follows from these counters.

## Validation

All 12 cases pass in interpreter, JIT and AOT (36 outcomes). The fixture uses
buffered Accept to isolate the shared parser; the unchanged Serve load fixture
exercises server-managed concurrency separately. [Contracts](evidence/contracts.json.gz).
The matching API snapshot passes its reference/source fingerprint check; no public
signature changes. Compressed build manifests retain exact compiler, backend,
library, native host and executable provenance.

Reproduce by building the four-library cohort with build-native-class-library.py,
then task-completion-20261018/build.py with --source Probe.rvn. Run verify.py with
--build, --library, --runtime, --backend, --host and --output. Build the unchanged
Serve/Server.rvn using the same helper without --source. Run allocation-sites.py
and the profile-native.c diagnostic separately from root-validation/compare.py.

Matched sustained-load results follow in a separate measurement slice.
