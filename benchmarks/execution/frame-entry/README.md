# Frame-entry experiment — 2026-10-24

**Rejected for production.** Baseline `4d5fa423c` remains unchanged. Preserve a
poisoned-frame initialization check in the standard native root-validation fixture.
The candidate source and measurements remain reproducible, without changing the
managed frame ABI, root semantics, collector policy or interpreter.

## Hypothesis and alternatives

The post-pacing worker profile identified owned frame entry and TLS as leading
native costs. Optimized ARM64 assembly inlines descriptor scanning into entry and
saves eight vector registers on every call, even with zero/small descriptor tables.
The entry helper uses 160 stack bytes; the published managed frame is separately
104 bytes and never changes here.

First replace memset-then-overwrite with a fully initialized aggregate after input
validation. Then outline descriptor validation. The first outlined variant saves
registers but loses long-table vectorization: 256-descriptor microbench timings rise
from about 0.15–0.16 s to 0.24 s per million entry/leave pairs. Reject that variant.
A specialized two-branch large helper restores constant flag checks and vectorization.
The final candidate handles 0–4 descriptors inline with scalar validation, dispatches
larger tables to the outlined vectorizable helper, and initializes the frame directly.

The final entry helper uses 80 stack bytes and saves no vector registers. The large
helper has its own stack cost, so this is not a halving of maximum call-chain stack
usage. Descriptors, mutable root values, fault checks, owned-chain proof and leave
zeroing remain. A native noinline attribute changes call/code placement and adds
another helper: this is a compiler/target tradeoff, not a general CLR optimization.
Reuse the existing [runtime execution-state comparison](../../../docs/design/allocation-metadata-runtime-comparison.md).

## Results

Same objects, GC policy and cache size; 32 connection-close clients, two client
processors, two-second warmup, ten-second load. Two forward/reverse trials per
configuration, no concurrent builds/profiling during capacity. Counters disabled.

| Configuration | Throughput change | CPU/response reduction |
|---|---:|---:|
| AOT one owner | +0.7% | 0.9% |
| AOT four owners | +2.4% | 2.2% |
| Owned JIT one owner | +0.15% | −0.05% |

AOT one-owner and JIT ranges overlap. Four-owner AOT ranges are separated, but
this isolated gain does not justify adopting the branch/call complexity on the
current evidence. Latency tails remain mixed. No production speedup is claimed.
AOT CPU includes server lifetime; JIT CPU starts at READY. Keep those scopes separate.

The focused microbenchmark spans 0/1/4/16/64/256 descriptors, validates every frame
field from poisoned memory, and runs optimized and sanitized baseline/candidate
in both orders. Final long-table timings are similar to baseline. Short runs are
noisy, including a slow first candidate run, and are not used as capacity evidence.
Both candidates pass existing native root rejection/lifetime and worker/async tests.
The restored implementation passes the strengthened standard fixture as well.

## Evidence and reproduction

- `candidate.c.gz`: final rejected implementation; `outlined-scalar.c.gz`: earlier
  variant (comment-only wording may differ from its first test capture).
- `baseline.s.gz`, `candidate.s.gz` (aggregate-only), `outlined.s.gz`,
  `specialized.s.gz`, `final.s.gz`: assembly progression; `assembly-command.json.gz`
  identifies compiler/SDK/flags (output names vary by variant).
- `micro-initial.json.gz`, `micro-final.json.gz`: all focused runs, including noise.
- `roots-initial.json.gz`, `consumers-initial.json.gz`, `roots.json.gz`,
  `consumers.json.gz`: initial/final candidate qualification.
- `restored-roots.json.gz`: unchanged production helper with stronger initialization test.
- `capacity.json.gz`, `jit-capacity.json.gz`, `summary.json`: raw trials and summary
  produced by `../owned-root-chain/summarize.py`.

The `measured-*.py.gz` files preserve the scripts used before restoration. Current
`measure.py`, `jit.py` and `micro.py` accept `--candidate-source`: decompress the
candidate into a new output directory and pass its path. Use the publication-budget
capacity arguments, with baseline `4d5fa423c`; `micro.py` also takes `--output`.
Run capacity alone. The new test remains useful independently of the rejected change.

The next architectural question is how generated code can reuse execution state
across frame entry, publication and leave without repeated TLS lookup. This needs
an explicit owner/thread lifetime contract covering suspension, host reentry and
worker contexts, plus matched JIT/AOT validation. It is a proposal, not an implemented
ABI change or permission to skip GC/root publication.

`restored-roots-final.json.gz` reruns the fixture after removing a duplicated
initialization assertion; both restored runs pass. `reproduce-build.json.gz`
confirms the out-of-tree candidate capacity recipe still links all four hosts
after restoring production. It is build-only, not another capacity cohort.
