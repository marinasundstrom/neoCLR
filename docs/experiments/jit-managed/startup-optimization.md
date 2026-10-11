# Shared metadata startup optimization

2026-10-11. Follow-up to the [first managed JIT benchmark](README.md#benchmark-result).
The author asks for continuing JIT optimization and continued interpreter investment.
This slice targets shared startup costs, leaving generated code and interpreter dispatch
unchanged. Application size and sustained work are separate axes: larger metadata can
increase loading costs, whereas repeated CPU work can amortize JIT compilation.

## Mechanism and contract

The existing runtime candidate index groups type/function names into ordered candidate
rows. Structural validation, reference-use validation and typed verification now each
open an immutable lookup scope. Resolution still checks exact identities, arity,
signatures, accessibility and ambiguity. Duplicate names remain multiple candidates;
unknown names stay absent. The index is scoped to the borrowed module and thread,
restored after nested calls/unwinding and rebuilt after mutations between scopes.

There is no persistent cache, skipped validation, lazy verification, metadata format or
public API change. The cost is temporary index allocation during analysis; the benefit
to evaluate is fewer full metadata scans. Shared loading improves AOT compilation and
interpreter startup as well as JIT. It does not optimize instruction dispatch or native
machine code. Relative to the [.NET execution-layer baseline](../../native-execution-investigation.md),
this is metadata resolution work; tiering, lazy JIT, PGO and code caching remain separate
choices. No performance comparison with .NET is claimed.

Managed JIT results now expose `metadataLoadSeconds` (input/dependency/System decoding,
object-root selection and context construction) and `selectionAndVerificationSeconds`
(closed-world linking, source verification, specialization and selection). Existing
`compileAndFinalizeSeconds` still includes selected-code admission/verification,
helper loading and native lowering/finalization. The fields are disjoint elapsed phases,
not a complete profiler: process startup, option parsing, reporting and teardown remain
outside them. They do not expose a stable public timing API.

## Reproduction and checks

Preserve the previous release-built compiler and interpreter before rebuilding this
slice. Re-run `verify.py` with the same qualified Index/Range artifacts; it checks all
20 tests, no-match filtering, bounds faults, missing-helper rejection and identical AOT
object bytes. Then run:

```sh
python3 docs/experiments/jit-managed/benchmark.py \
  --qualification target/jit-startup-validation/validation.json \
  --build target/jit-iteration-release-baseline/index-range-build \
  --baseline-jit target/jit-before-startup-optimization \
  --baseline-runtime target/interpreter-before-startup-optimization \
  --repetitions 10 \
  --output target/jit-startup-benchmark/benchmark.json
```

The optional baseline flags run the old tools with the same input, bindings and helper
library, and retain their hashes. Each mode gets one unmeasured warm-up followed by ten
fresh-process samples. Five modes rotate starting order across ten rounds, so each
occupies each position twice. All 55 executions must produce identical successful
20-test output. No concurrent build/test workload is launched during measurement.
OS caches and background host load remain uncontrolled; this is one local workload.

Focused checks: 56 integration tests across loaded programs, typed/stack verification,
accessibility, overloads, references, library references and friend assemblies; three
index tests cover duplicates/mutations, missing/type-arity lookup, nested scopes,
unwinding and thread isolation. All pass. No website build or public API snapshot is
needed for this internal optimization.

## Local measurements

Apple M1/macOS 27, release-built tools. [Raw samples and hashes](startup-benchmark.json),
[consumer qualification](startup-validation.json). The previous binaries and current
binaries are compared in this same run, rather than against earlier wall-clock results.

| Fresh-process mode | Before median (min–max) | After median (min–max) | Median reduction |
| --- | ---: | ---: | ---: |
| JIT including compilation | 3.217s (2.640–5.185s) | 2.531s (1.774–3.633s) | 21.3% |
| Interpreter | 3.919s (3.309–6.517s) | 2.697s (2.104–3.074s) | 31.2% |

The unchanged prebuilt AOT executable has an 11.01ms median (10.34–23.01ms range).
Its build cost is excluded; JIT compilation is included. All modes run the same existing
20-test assembly, and the AOT object recompiles byte-for-byte identically after the change.
The JIT stage medians are 109.73ms metadata loading, 1,801.06ms selection/verification,
543.26ms lowering/finalization and 6.65ms invocation. Medians of individual phases need
not sum to the median process time.

Host variability is substantial. The new JIT is faster than the baseline in nine of ten
rounds, and the interpreter in all ten. These observations support the targeted change
for this workload; they do not prove a universal percentage, steady-state throughput,
or that the interpreter is approaching native execution speed. In particular, the current
JIT/interpreter process medians are dominated by startup, not comparable dispatch timing.
Do not combine this run's absolute times with the earlier benchmark to estimate a trend.

Remaining work: profile selection/verification more deeply, then measure a sustained
workload and scale metadata size independently. Any persistent cache or lazy compilation
needs explicit invalidation, semantic validation and lifetime contracts. Async/HTTP JIT
qualification remains a separate prerequisite for service-throughput comparisons.

Follow-up: the [retained-code CPU experiment](../jit-cpu/README.md) varies computation
with fixed metadata and reports invocation timing for all three modes. Managed async/HTTP
lifetime remains the next functional JIT gap; this scalar result does not close it.
