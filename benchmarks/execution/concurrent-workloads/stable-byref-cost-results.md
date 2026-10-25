# Immutable-byref workload costs

2026-10-25. Follow-up to [provenance and root qualification](stable-byref-results.md).
The prototype improves the concurrent candidate on JSON and two standalone samples,
but synchronous GC remains faster. It stays test-only; this is not concurrent-default
acceptance or a comparison against other web frameworks.

## Matched controls

All builds use speed optimization and existing direct/forwarded stack-extent proofs.
The only candidate compiler difference is `NEOCLR_LIVE_SSA_BYREF_ARGUMENTS`.
Rebuilding with that switch disabled reproduces the retained tasks, collections,
arrays and JSON control objects byte-for-byte. The ordinary builder now exposes
`--stable-byrefs`, clears an ambient byref switch when not selected, and records
its compiler environment. Its disabled run deliberately inherited the switch to
check that an accidental environment setting cannot select the candidate.

Retained managed inputs and sync objects are unchanged. Each live comparison uses
the same native collector, exact allocation index and inline base lookup. Native
hosts compile before timing; builds and measurements run sequentially. The earlier
plaintext cohort is not rerun or counted in this evidence.

## Standalone AOT

Ten interleaved rounds across three modes and three samples pass **90 runs**.
Each invocation runs 4,096 workload iterations and checks exact output, collection
accounting, shutdown and released index metadata. CPU includes collector worker
work and console output, but excludes compilation and process startup.

| Sample | Sync CPU ms | Previous live CPU ms | Candidate CPU ms | Candidate change |
| --- | ---: | ---: | ---: | ---: |
| Tasks | 14.066 | 22.554 | 21.395 | -5.1% |
| Collections | 80.805 | 123.584 | 116.942 | -5.4% |
| Arrays | 46.531 | 53.687 | 53.528 | -0.3% |

These are medians of short invocations, not capacity estimates or confidence
intervals. Treat the array result as flat. Candidate wall-time medians are about
20.96/115.17/53.69 ms respectively. All modes have exactly equal allocation counts
and allocated bytes for a given sample:

| Sample | Allocations per invocation | Allocated bytes |
| --- | ---: | ---: |
| Tasks | 65,548 | 2,261,361 |
| Collections | 237,580 | 7,864,689 |
| Arrays | 86,028 | 1,892,721 |

Candidate and previous-live peak live bytes and arena use are also equal. Candidate
peak live bytes are 9,329/10,113/6,239, versus sync 497/1,694/593. The optimization
reduces executed bookkeeping, not managed allocation or collector retention.

## JSON HTTP AOT

Four alternating-order rounds of sync/previous-live/candidate, four independent connection-
close client lanes, 16 warmups and 2,048 measured requests validate **24,768 responses**.
Every process checks response content, completed epochs, zero failures, closed native
scopes and index teardown. This is the existing fixed-work single-owner HTTP fixture,
not the newer isolated-owner capacity fixture or a multi-owner scaling test.

| Mode | Median requests/s | Process CPU us/response | Median trial client p99 ms |
| --- | ---: | ---: | ---: |
| Sync | 3,121.0 | 322.32 | 1.402 |
| Previous live | 2,005.5 | 527.29 | 2.120 |
| Candidate live | 2,355.3 | 455.31 | 1.838 |

The candidate improves throughput **17.4%** and lowers CPU/response **13.7%** against
previous live. Process CPU includes startup, warmup and shutdown and is amortized
over all 2,064 successful responses per process. Throughput excludes warmup. Client
p99 includes client/transport/scheduling effects and is not a GC pause distribution.

Both live versions allocate exactly 943,817 objects / 54,065,775 bytes per process.
Sync ranges from 943,817–943,818 objects / 54,065,775–54,066,303 bytes because of the
existing scheduling-dependent path; allocation variation is explicitly permitted
and reported. No allocation reduction is claimed. Both live versions peak at
67,911 live bytes / 144,872 arena bytes; sync medians are 52,690 / 101,248. Median
process RSS is 13.37 MB for the candidate versus 13.52 MB previous-live and 10.67 MB
sync; these small RSS differences do not establish a memory improvement.

JSON static eligible store removals grow from 20,727 to 31,311. This is a count of
compiled sites, not executed calls. The prototype still retains original argument
root storage and relies on nonmoving GC. No type-specific collector path is added.

## Next decision and evidence

Keep the prototype selected explicitly. Its CPU/throughput gain now extends beyond
plaintext, without observed managed allocation or retained-live-byte changes. The
remaining synchronous gap is material: candidate JSON uses about 41% more CPU per
response and handles about 25% fewer requests/s in this fixture. Continue broader
JSON JIT qualification, then isolated-owner scaling and owner-pause/retention
profiling before regular experimental selection or a default change. Existing
safety evidence does not qualify shared heaps, moving collection or TSan.

[Evidence](stable-byref-evidence/) includes `cost-summary.json`, `ordinary-build-*`,
`ordinary-control-*`, `ordinary-measure-*`, `json-build-*` and `json-measure-*`.
Reports retain commands, compiler environment, source/binary hashes, all trials and
object audits; the JSON build driver is archived too. No failed measurement was
excluded. These observations reuse the existing compiler-proven stack-address
design and its CLR comparison; no new API, Raven emission, metadata encoding,
Runtime Contract, ownership or native ABI change is proposed here.

The subsequent [JIT and owner-operation profile](stable-byref-pause-results.md)
qualifies JSON JIT and provides guarded controls and pause histograms. It finds no
pause advantage and leaves concurrent-default acceptance open.
