# Invocation-owned encoding providers — 2026-10-18

Follow-up profiling of the [receive-cache candidate](../receive-pool-20261018/README.md)
finds 1,989 allocations attributed to Encodings.get_Utf8 in the fixed diagnostic
workload. Both providers are stateless; only CreateEncoder/CreateDecoder own mutable
session state. Cache providers in invocation-owned static fields while continuing
to create independent sessions. Empty-array caching remains a separate investigation;
this slice changes no array representation, compiler or GC policy.

## Contract and comparison

.NET separates its [UTF-8 provider](https://learn.microsoft.com/en-us/dotnet/api/system.text.encoding.utf8?view=net-10.0)
from [stateful decoders](https://learn.microsoft.com/en-us/dotnet/api/system.text.encoding.getdecoder?view=net-10.0).
neoCLR's existing strict errors, BOM-as-data behavior, UTF-8 strings and size limits
remain unchanged; this is not a change to .NET replacement-fallback or preamble
semantics. Providers may be shared; consumers must not depend on getting a new
identity on each access. Session instances remain independent.

The cache uses existing invocation-owned static storage. Native contexts remain
thread-affine. Interpreter first readers may create competing stateless providers;
this is not a once-only singleton identity or general Lazy<T> publication contract.
No partial object is stored, no session state is shared, and the normal static root
retains cached references until context teardown. The cache holds at most two
providers plus its static-storage bookkeeping. Existing static-storage tests cover
context isolation/reset; the new consumer exercises providers under allocation
pressure, without extending that hosting contract.

The first version used concrete provider types for both fields. The three-mode
consumer passed, but UTF-8-only Serve compilation failed with a missing/ambiguous
ASCII interface implementation. Shared static-owner layout reachability appears to
introduce the unused concrete implementation without its methods. The final fields
use their Encoding interface contract, and Serve compiles. The [failed build](evidence/concrete-field-failure.json.gz)
is retained; the broader native interface-selection gap is not fixed or redefined
as a language restriction by this change.

## Validation

[Contracts.rvn](Contracts.rvn) checks strict UTF-8/ASCII errors, BOM preservation,
independent interleaved decoders, finished-session isolation, independent encoder
output, ASCII rejection and repeated access after allocation pressure. Interpreter,
JIT and AOT pass. [Evidence](evidence/contracts.json.gz). The shared verify.py now
accepts an optional --source and --expected-line; defaults preserve its Task fixture.

Rebuild the four-library cohort, use the shared task-completion build.py for Serve,
and run verify.py with --source pointing at Contracts.rvn and --expected-line
'Encoding provider contracts passed'. Compressed build manifests retain compiler,
backend, library, object, native-host and executable hashes. The matching API
reference and XML summaries are refreshed; no public signature changes.

## Diagnostic allocations

Same 200 measured requests, 20 warm-up requests and one stop exchange, including
startup/shutdown. Separate caller-attribution runs agree with the allocation total.
Diagnostics overlapped tooling and are excluded from comparative elapsed-time claims.

| Counter | Receive-cache baseline | Encoding cache | Change |
| --- | ---: | ---: | ---: |
| Allocations | 64,693 | 62,706 | −3.07% |
| Allocated payload bytes | 3,375,544 | 3,343,768 | −0.94% |
| Collections | 19,129 | 18,405 | −3.78% |
| Root publications | 717,297 | 713,322 | −0.55% |
| Peak live payload bytes | 46,480 | 46,528 | +48 bytes |
| Peak arena bytes | 68,744 | 69,072 | +328 bytes |

All allocations reclaim at shutdown and GC failures are zero. get_Utf8 attribution
falls from 1,989 allocations to three initialization allocations (provider and static
storage), not three provider objects. Counts charge the current published frame,
not an allocated-type census. [Counters](evidence/gc-counts.json),
[baseline attribution](evidence/baseline-allocation-summary.json),
[candidate attribution](evidence/allocation-summary.json),
[request diagnostics](evidence/diagnostic.json). No throughput gain is inferred
from these counts.
