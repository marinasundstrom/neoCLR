# Exact-base store lookup (2026-10-25)

The preceding sustained samples show overhead outside HTTP. A matched guarded
synchronous control now uses the live emitted code and native store adapter but
suppresses automatic/idle concurrent epoch starts, as in the existing HTTP
control. Every guarded run asserts all six epoch counters are zero. This is a
measurement control, not a production collector configuration.

Five alternating rounds for each of three samples and three collector variants
pass (45 runs), with identical allocations and output. Before the change, guarded
CPU is 3.45× ordinary sync for tasks, 3.23× for collections and 1.76× for arrays.
The marker is not the only source of overhead. These ratios include guarded code,
adapter work and collector scheduling differences; they do not isolate one function.

## Change and safety

After the existing snapshot identity and last-range checks, a store directly to a
currently proved allocation base now resolves its header using that proof. It
validates the current header, allocated state and requested width against current
payload bounds. Otherwise it takes the existing checked walk. The proof comes
from the owned heap's existing exact-base cache, whose reclamation/reset/release
revocation rules are unchanged. There is no new cache, root, allocation, owner
boundary, object header or compiler emission. Subsequent field initialization can
use the existing last-range hint. `NEOCLR_LIVE_NO_EXACT_STORE` retains the previous
path for controls; disabling the whole store cache also disables this shortcut.

This reuses the same .NET/CLR-inspired separation of allocation identity and scan
metadata recorded in the [existing investigation](../concurrent-gc-investigation/README.md).
It is a private implementation optimization, not a change to managed semantics.
The benefit is avoiding heap-prefix walks for known bases; the cost is one extra
proof-cache query on last-range misses. Unproved/interior destinations still require
checked lookup, and old snapshot allocations still undergo identity validation.
The first lookup diagnostic now includes proved exact bases as well as last ranges.

The complete existing native suite passes optimized and UBSan/bounds builds,
including differential liveness and deletion/stale-interior-header negative
controls (both deliberately broken controls abort). A new 300-allocation fixture
passes optimized/UBSan with the shortcut enabled and disabled: two initialization
stores per allocation use no walks when enabled, a one-byte payload rejects a
two-byte write, evicted proof entries fall back correctly, and reclaimed storage
rejects writes. The general validator includes this fixture for future runs.

## Results

A second 45-run standalone cohort passes with identical allocations, output and
lifecycle checks. Each batch contains 4,096 complete sample iterations. Median
process CPU measured only around native invocation, seconds:

| Sample | Prior live | New live | Reduction | Prior guarded sync | New guarded sync |
| --- | ---: | ---: | ---: | ---: | ---: |
| Tasks | 0.103523 | 0.100277 | 3.1% | 0.059090 | 0.057562 |
| Collections | 0.559150 | 0.493688 | 11.7% | 0.322060 | 0.293164 |
| Arrays | 0.140629 | 0.124240 | 11.7% | 0.093039 | 0.085758 |

Ordinary sync remains stable around 0.0172/0.0997/0.0530 seconds. First live-run
range walks fall from 85,520 to 81,423 (tasks), 436,662 to 354,741 (collections),
and 85,792 to 65,311 (arrays). These are separate before/after cohorts; stable
sync controls and reduced work support the result, but background load remains a
limitation. There is no claim of a universal 12% improvement.

For HTTP, both implementations are compiled before timing and alternated within
each cohort, alongside ordinary sync and both guarded controls. Five rounds × five
modes × 272 responses × two handlers = 13,600 checked responses. Allocation counts
and bytes are equal across all modes for each handler. Median live CPU:

| Handler | Previous path | Exact-base path | Sequential throughput, previous → new |
| --- | ---: | ---: | ---: |
| Plaintext | 0.239306 s | 0.238388 s | 1,196 → 1,201 requests/s |
| JSON | 0.436252 s | 0.428349 s | 646 → 656 requests/s |

Plaintext is effectively unchanged. Its guarded control was 2.7% slower with the
shortcut; JSON guarded CPU was 0.7% lower. Do not claim a repeatable plaintext win.
JSON's observed live CPU reduction is only 1.8%. The shortcut is retained for the
broader reduction in measured range work and standalone CPU, not as a solution to
HTTP throughput. Ordinary synchronous HTTP remains substantially faster.

CPU includes collector threads; HTTP totals include startup/warmup/cleanup while
standalone CPU excludes process startup and compilation. OS background services
including ANECompilerService, dasd and photolibraryd were active; no other project
build or benchmark overlapped timed work. These instrumented, sequential HTTP runs
are not saturated capacity tests. No pause-distribution claim is made.

See [numeric summary](exact-summary.json) and the `guarded-measure`, `exact-measure`,
`exact-native-safety`, `exact-boundary-safety`, `exact-http-plain` and
`exact-http-json` compressed reports for commands, diagnostics and provenance.
Remaining priorities are repeated range lookup for interior fields, unnecessary
store calls/publications, outer pauses, sustained JIT workloads and worker scaling.

A freshly linked paced live host also passes the full JSON managed JIT consumer:
128 checked responses, zero guest/process status and completed concurrent epochs
with clean shutdown. See `exact-json-jit.json.gz`; this is compatibility evidence,
not a sustained JIT performance claim.
