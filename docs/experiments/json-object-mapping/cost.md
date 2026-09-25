# Mapped JSON execution cost — 2026-09-25

The opt-in HTTP mapping fixture passed in isolation but reached its existing
15-second transport deadline during an overlapping run. This investigation measures
cost without changing that deadline or expanding the serializer contract.

## Finding and bounded change

A three-second macOS sample of the release runner executing the standalone mapped
report caught LoadedProgram preparation repeatedly calling vm::resolve. Much of the
sampled work was allocating, copying and freeing method-identity strings during the
lookup scan. Inspection confirmed that the resolver cloned every candidate identity
before rejecting an unrelated method name, instance/static form or generic arity.

Move those cheap rejection checks before identity construction. All surviving
candidates still undergo identity, owner, substitution, parameter, constraint and
ambiguity checks. This reduces incidental allocation in the existing algorithm;
it does not introduce cached resolutions, change reflection access or skip validation.
Lookup remains linear in the function table. A prepared index or specialization
cache could remove more work but would require a separate identity/lifetime design;
this small change avoids adding that state for an already-demonstrated allocation
problem.

Reuse the [reflection comparison](../../reflection-execution.md) and
[execution architecture](../../execution-architecture.md): familiar .NET creation
and accessor behavior remains unchanged. This is an implementation improvement to
neoCLR's prototype lookup, not a new API, JIT claim or performance comparison with
.NET/CLR.

## Measurement

The measure_async example now reports assembly_ms, load_ms, verify_ms and run_ms
on stderr, separately from managed-heap counts. Assembly includes source reads;
loading includes linking/metadata validation; verification is the explicit typed
analysis; run includes runtime entry checking, execution and any I/O wait. These
wall-clock figures are diagnostic observations, not microbenchmarks. Run timing is
also reported when execution returns a Fault; earlier preparation failures still
return directly.

Compare release binaries against the exact same imported App.neoil and System.neoil,
heap limit 512 and instruction limit 100000000, running them sequentially without
other builds/tests. The baseline uses revision 2789002d plus only the timing additions;
the candidate adds the resolver check reordering. A preliminary baseline overlapping
a compiler build is excluded from the comparison. The original sampling profile
observed preparation only, so it cannot by itself explain the HTTP timeout.


Sequential local release observations on macOS arm64:

| Phase | Baseline (ms) | Early rejection (ms) |
| --- | ---: | ---: |
| Source read/assembly | 12,314 | 1,814 |
| Loading/link validation | 9,436 | 1,034 |
| Typed verification | 5,331 | 502 |
| Execution | 3,511 | 1,484 |
| Total | 30,592 | 4,834 |

Both return the same successful JSON mapping output and exactly 195 managed
allocations, peak 72, four collections and zero live objects. Lookup is used during
preparation as well as execution; assembly is not only text parsing. The retained
[result record](cost-results.json) includes exact input hashes. These are single
before/after observations, not statistical estimates or a bound on future latency.

Focused regressions: 40 tests pass across overload resolution, generic methods,
loaded programs, reflection construction and property execution. The standalone
fixture also verifies constructor/accessor use, string and stream round trips,
structured error cases and managed GC. No compiler change or library regeneration
is required; the website build remains skipped.


The mapped HTTP pair passes with the optimized runner and unchanged transport
policy: 335 client / 336 server allocations, seven collections each and zero live
objects. Whole-execution timing was 12,206 ms for the client and 14,247 ms for the
server. Those intervals include waits and, for the server, client preparation;
they are not isolated request-processing times or the client's exact exchange
budget. The large remaining interval means the earlier load-sensitive timeout is
not claimed resolved. Reproduce this focused check with the HTTP verifier's
`--mapped --case pair` options; avoid overlapping benchmark runs.

Website content was reviewed: its existing opt-in status and latency limitation
remain accurate. Only the sample archive inventory gains these evidence files;
no website build or publication was performed.
