# Native publication budget — 2026-10-24

Compare the 1,024-publication policy at `9380bc439` with 4,096 publications.
The 256-allocation budget, pressure retry, explicit/idle collections, root tracing
and worker coordination are unchanged. This affects the native JIT/AOT collector;
the interpreted collector is separate.

The baseline C sources and **execution-state.h** are copied from that revision
into the same directory so quoted includes select the baseline policy. Candidate
sources use the repository header. Managed objects are identical. Capacity runs
have profiling disabled, alternate forward/reverse configurations, and use the
same connection-close plaintext client (32 connections, two client processors,
two seconds warmup, ten seconds measurement). Two trials per configuration are
local directional evidence, not a cross-platform qualification or Kestrel parity.
AOT CPU includes server lifetime; JIT CPU starts after READY. Do not combine them.

Diagnostic 32→64-request deltas, independently of capacity:

| Handler | Old collections/request | New collections/request | Arena extent at 64, old → new |
|---|---|---|---|
| Plain | 3 publication + 1 idle | 1 idle | 73,408 → 81,216 bytes |
| JSON | 5 publication + about 1 idle | 1 allocation + 1 idle | 79,552 → 92,768 bytes |

JIT and AOT agree on the principal counts. Allocations remain 208/request for
plain and 457/request for JSON; no pressure requests or failed dispatches occur.
Arena extent is allocator high-water usage, not resident memory. GC-observed live
bytes sample only collections and cannot establish a lower true liveness peak.
The larger budget permits a fourfold publication delay before no-allocation
cleanup (4,096 rather than 1,024); neither budget is a wall-clock guarantee.

Validation uses the existing trigger and state-access runners: eight optimized/
sanitized, profile on/off, worker-hook on/off threaded trigger configurations;
exact-budget pacing and cleanup restoration; cleanup publication rejection;
shared-worker graph/fault stress; and async consumers. The pressure fixture still
forces a retry in a small heap. Twelve plain/JSON JIT/AOT HTTP invocations validate
response bodies and completion counts. Source hashes and raw outcomes are retained.

Reproduction uses `measure.py` with the same arguments as state-access, `jit.py`,
and the existing `gc-triggers/counts.py` and `state-access/validate.py` runners.
See the archived reports for exact commands and artifact identities. Historical
1,024-budget diagnostics remain in [gc-triggers](../gc-triggers/README.md).

## Retained result

Adopt 4,096 publications. Eight AOT and four JIT unprofiled trials pass:

| Configuration | Requests/s, baseline → candidate | Throughput | CPU/response |
|---|---|---|---|
| AOT, one owner | 3,744 → 4,341 | +15.9% | −13.9% |
| AOT, four owners | 9,724 → 10,896 | +12.1% | −11.9% |
| Owned JIT, one owner | 2,120 → 2,318 | +9.4% | −8.2% |

AOT one-owner p99 improves from 10.73–12.36 ms to 8.35–8.52 ms;
four-owner ranges overlap (4.02–4.47 vs 3.49–4.30 ms). JIT p99 improves
from 23.26–25.01 ms to 18.33–22.83 ms in these trials. JSON capacity is not
measured here; only its diagnostic frequency/memory behavior is qualified.

`summary.json` is produced by `../owned-root-chain/summarize.py` from
`capacity.json.gz` and `jit-capacity.json.gz`. `plain.json.gz`/`json.json.gz`
contain counted HTTP runs; `triggers.json.gz` and `consumers.json.gz` retain
validation commands, outcomes and input identities. No concurrent build/profile
work ran during the capacity windows. The modest measured memory increase and
bounded cleanup delay are accepted for this experimental native policy; broader
long-running workloads and other platforms remain qualification work.
