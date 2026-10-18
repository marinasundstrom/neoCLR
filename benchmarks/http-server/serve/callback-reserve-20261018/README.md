# Reserve the known first Promise callback — 2026-10-18

The [caller profile](../caller-profile-20261018/README.md) identifies 2,687 empty
callback arrays immediately replaced by first-add growth. Promise.Register now
creates its lazily allocated callback list with capacity one, since the first
callback is already known. Promises without callbacks still allocate no callback
list. Additional registrations retain geometric growth and deferred FIFO posting.

This applies the existing capacity contract, comparable to
[.NET List<T>(capacity)](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.list-1.-ctor?view=net-10.0),
without changing public Task/Promise semantics or replacing reference-type tasks.
For multiple callbacks, starting at one can add growth steps versus the former
first-add capacity of four; the measured Serve workload predominantly benefits
from avoiding the zero-capacity array. The tradeoff must remain explicit.

The shared Task completion fixture tests callbacks before/after settlement,
duplicate completion/cancellation, deferred ordering, reentrant posting and queue
reuse. It now registers nine callbacks before both completion and cancellation to
exercise growth beyond the initial reservation. Public API signatures stay unchanged.

## Validation and diagnostics

The expanded shared Task fixture passes interpreter, JIT and AOT.
[Contracts and tool hashes](evidence/contracts.json.gz). The unchanged API reference
passes its matching source/reference fingerprint check.

Same 200 measured +20 warm-up +1 stop exchanges, including startup/shutdown:

| Counter | Header-reservation baseline | Callback reservation |
| --- | ---: | ---: |
| Allocations | 61,601 | 58,914 |
| Allocated payload bytes | 3,327,856 | 3,212,315 |
| Collections | 19,062 | 19,493 |
| Root publications | 710,228 | 702,167 |
| Peak live payload bytes | 46,528 | 45,632 |
| Peak arena bytes | 68,768 | 67,776 |

Allocations fall 4.36%, bytes 3.47%, root publications 1.13%. The independent caller
capture agrees exactly with the allocation count, with no dropped records. This
workload removes exactly one allocation per first registration; no extra callback
growth offsets that saving here. Multi-callback applications may behave differently.
All allocations reclaim at shutdown and GC failures are zero. Collections rise
2.26%; idle/collection counts are timing-sensitive, as the caller-profile slice
explains. Diagnostic runs overlap tooling/other diagnostics, so elapsed GC time is
not comparative. [Counters](evidence/gc-counts.json),
[callers](evidence/callers-summary.json.gz), [requests](evidence/diagnostic.json).

Build with build-native-class-library.py and the shared task-completion build.py;
run its verify.py with the resulting cohort. Existing compiler/backend/native
host hashes and complete commands are retained in the compressed manifests.
Matched sustained Serve/Go measurements follow separately.
