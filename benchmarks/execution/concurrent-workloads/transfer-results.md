# Store attribution and stack transfers (2026-10-25)

The four-range candidate still pays substantial guarded-versus-ordinary-sync cost.
This change first attributes out-of-arena calls, then preserves an existing stack
proof across managed instruction boundaries. It adds no language/type special case,
managed allocation, GC layout, collector ownership rule or public API.

## Attribution before optimization

`store-sites.py` instruments only the out-of-arena scalar-store branch of a generated
live collector. A bounded 16,384-entry owner-thread table counts return PCs, without
managed allocation. It reports image-relative PCs after cleanup on the invoking
thread. `summarize-sites.py` resolves them against the exact Mach-O image's local
symbols and the matching selection report, including separately reported generated
boxing/query helpers. It requires zero overflow and exact reconciliation between
site totals and the original outside-store counter. These are macOS ARM64 AOT
counts, not JIT profiling or CPU attribution; the extra native TLS and counting work
make their timings unsuitable for performance claims.

| Baseline workload | Out-of-arena calls | Result helper calls | Optional helper calls |
| --- | ---: | ---: | ---: |
| Tasks, 4,096 iterations | 372,788 | 0 | 212,992 |
| Collections, 4,096 iterations | 2,555,956 | 737,280 | 434,176 |
| Arrays, 4,096 iterations | 225,332 | 0 | 0 |
| Plaintext, 144 requests including warmup | 1,894,544 | 764,151 | 142,963 |
| JSON, 144 requests including warmup | 3,231,584 | 1,461,687 | 175,795 |

All five diagnostic workloads pass output/allocation/cleanup checks, and all site
counts reconcile. The first JSON symbolization attempt exposed synthetic helpers
outside the source-function list; the resolver now uses their explicit selection
entries rather than inventing names or omitting calls. Array hotspots include
formatting, type shape queries, callbacks and iteration, so union helpers alone
are not the whole picture.

Out-of-arena is **not** a proof of stack provenance: context/host writes and byref
arguments are included. A byref that points to a stack in one invocation can point
to a managed field in another. The counters identify where to investigate, not
which writes may bypass checks.

Disassembly showed adapter calls for destinations formed directly from `sp`.
The native compiler emits a basic block per managed instruction and carries its
evaluation stack through block parameters. The earlier proof deliberately stopped
at every block parameter. A private test-driver IR dump confirms this pattern;
the dump itself produces a byte-identical task object. The remaining constructor
receiver stores load a supplied byref from an argument slot and remain guarded.

## Bounded compiler change

The pass records branch arguments only for a single incoming **edge**, then follows
that transfer in addition to stack-address instructions and checked constant
offsets. It counts both arms of a branch separately even when they have the same
predecessor and target. Entry parameters, loaded pointers, multiple-edge joins,
unanchored cycles and arithmetic overflow remain unknown. A value-count bound
prevents cyclic analysis; every proved store still fits the declared stack slot.
Validation finishes before IR edits or new imports, and framed fault cleanup keeps
its existing dominance checks. Root publication and tracing of changing root values
are unchanged. This extends the existing stack-store qualification in
[concurrent-store](../concurrent-store/README.md#stack-provenance-qualification),
consistent with the CLR distinction between stack storage and heap write barriers.

Forty-two focused compiler tests pass. New JIT cases exercise chained transfers,
external byrefs, duplicate-target branch edges, hook rejection and output behavior;
bounds reject before mutation, while overflow and cycles cannot establish a proof.
Six hundred forced/paced entry cycles pass across JIT and AOT native hosts, including
O2 and UBSan/bounds C builds, faults and recovery. Cranelift optimization settings
are unchanged; C host optimization is not a claim of optimized managed code.

Two cargo-launched test binaries stalled before producing test output on this host;
sampling one showed `_dyld_start` before `main`.
After verifying an ad-hoc-signed isolated copy ran normally, those stalled launches
were terminated. The signed runner passes the tests and builds retained consumers.
A diagnostic-only build initially compared an optional function name incorrectly;
that compile error was corrected before the final IR/JIT driver was used.

## Matched execution results

All three standalone consumers and both full HTTP selections recompile against
the same retained assemblies. Ordinary synchronous objects are reused because the
pass is live-only. Forty-five standalone runs (five rounds × three modes × three
samples) preserve exact outputs, allocation counts/bytes and lifecycle checks.

| Sample | Prior live CPU | New live CPU | Outside calls, prior → new |
| --- | ---: | ---: | ---: |
| Tasks | 0.085328 s | 0.083976 s | 372,788 → 282,668 |
| Collections | 0.438571 s | 0.433332 s | 2,555,956 → 2,146,348 |
| Arrays | 0.098231 s | 0.097986 s | 225,332 → 213,036 |

These separate before/after cohorts show large count reductions but only small CPU
changes; do not interpret them as a statistically established standalone speedup.
The instrumented count cohort is separate from these normal-adapter timings.

HTTP compares previous/new generated objects with identical four-range providers,
both guarded-synchronous controls, and ordinary sync. All builds finish before five
alternating rounds of 16 warmup plus 256 measured requests per mode and handler.
All 13,600 responses pass; allocated counts and bytes match across all five controls.

| Handler | Previous live requests/s | New live requests/s | Previous CPU | New CPU | CPU reduction |
| --- | ---: | ---: | ---: | ---: | ---: |
| Plaintext | 1,385 | 1,491 | 0.206566 s | 0.191798 s | 7.1% |
| JSON | 837 | 899 | 0.336053 s | 0.310467 s | 7.6% |

Outside-store calls fall 3,568,656→2,710,373 and 6,094,176→4,582,821 per process.
Inactive/new and guarded-old heap-store counts are identical between the old/new
live objects in the first runs, corroborating the intended stack-only reduction.
Guarded CPU also falls; ordinary sync remains much faster at 3,884/2,204 requests/s.
This is sequential fixed-work evidence, not saturated capacity or framework parity.

HTTP CPU includes startup/warmup/cleanup; standalone CPU surrounds only native
invocation. Both include collector threads. OS background activity remains a limit;
no project compiler or other benchmark overlapped the normal timed cohorts. There
is still no full outer-pause distribution or worker-scaling acceptance result.

## Tools, provenance and next questions

Use `--profile-store-sites` with either measurement harness, then pass its results
and matching build report to `summarize-sites.py`. Use `--previous-live-object` for
interleaved compiler comparisons. `recompile-live.py` preserves source assemblies
and synchronous objects while rebuilding live objects. The ignored compiler driver
can dump one function before store lowering using `NEOCLR_LIVE_STORE_IR_FUNCTION`
and `NEOCLR_LIVE_STORE_IR_OUTPUT`; production builds do not read these test-only
variables. No Runtime Contract configuration or CLI encoding changes.

[Summary](transfer-summary.json), `sites-baseline-*`, `transfer-sites-*`,
`transfer-build`, `transfer-http-build`, `transfer-entry-safety`, `transfer-measure`
and `transfer-http-*` compressed reports retain the evidence. `transfer-tests.txt.gz`,
`transfer-value20.clif.gz` and `transfer-before-value855.asm.gz` retain focused test,
IR and disassembly evidence. The retained URI-efficiency Raven bundle is unchanged;
assembly/compiler hashes are recorded rather than claiming a new Raven build.

Counts alone do not establish CPU dominance. Remaining opportunities include
repeated per-field validation/copy work, byref specialization only where proved,
and repeated publication/metadata work. Inspection also confirms that these paths
still use Cranelift's default settings rather than explicitly selecting `speed`,
as the earlier scalar/compiler comparison documented. Qualifying generated-code
optimization for **both** collectors belongs in the next comparison; changing that
setting without root/fault/lifetime checks would not establish GC correctness.

The current signed diagnostic driver also passes the full JSON managed JIT consumer
with 128 correct responses, completed concurrent epochs and clean shutdown. See
`transfer-json-jit.json.gz`. This qualifies execution, not sustained JIT throughput.
