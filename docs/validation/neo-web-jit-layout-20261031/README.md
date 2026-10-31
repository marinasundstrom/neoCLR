# Neo.Web JIT startup and execution investigation — 2026-10-31

Development work on macOS ARM64 / Apple M1. This follows the
[collector-policy comparison](../neo-web-performance-20261031/README.md).
The baseline compiler is the cancellation-qualified 7de5ada65 bundle; the archived candidate
changes only compilation-time static-layout reuse. Raven remains 813dad20c.
No GC, eager/lazy-JIT, backend optimization or heap-budget default changes.
No Raven frontend, Runtime Contract or temporary CLI representation changes.

## Profile and bounded fix

A fresh run of the previous paced-JIT cancellation case separates startup from
HTTP execution. Sampling starts about two seconds after launch for ten seconds;
a later five-second startup sample captures lowering and Cranelift work. The
request-time sample runs for ten seconds, after two seconds of continuous traffic.
The run validates 5,533 responses; profiled timing is excluded from comparisons.
Compressed samples and their uncompressed hashes are retained alongside this file.

Early startup shows metadata selection, specialization and interface resolution.
The selected image contains 3,296 functions and 1,157 types. Eager JIT prepares
this complete selected program, including all benchmark branches. Late startup includes repeated `type_initialization::needs_storage` scans alongside
backend verification and register allocation. Every static-field operation had
recounted all storage owners and preceding owners, each potentially scanning all
methods for an initializer. This work was independent of the operation.

The candidate `StorageLayout` records the dense source-order owner inventory and ordinal
prefixes once for the immutable selected module during lowering. Static operations and initializer
guards consume that same inventory. Field-only, initializer-only and combined
owners retain their identities; specialization still separates closed generic
owners. No process-global cache or borrowed mutable metadata is introduced.
The added storage is linear in selected types and lasts only for lowering.

This is a preparation optimization. It does not change .NET-like static-field and
initialization behavior, the CLI bridge, emitted storage layout or the native ABI.
The existing [static storage](../../runtime-static-storage.md) and
[initialization research](../../design/readonly-fields.md) apply without a new
semantic divergence. Native AOT shares the lowering improvement; this is not a
promise of faster steady-state requests.

In the execution sample, allocation and heap-span validation dominate, as in the
paced AOT profile. The earlier short JIT measurement is therefore not sufficient
to characterize sustained throughput or attribute the entire gap to generated JIT
code. Cold compilation and request-time runtime work remain distinct costs.

## Initial placement experiment

The first implementation stored the layout in the shared type `Profile`. Three
paired launches had readiness medians of 60.15 s baseline and 64.38 s candidate;
the candidate range was 55.77–65.19 s. That did not establish a readiness win.
`initial-profile-layout-results.json` retains every trial. Its emitted native
object was identical (`initial-object-comparison.json`).

The final implementation instead keeps the inventory local to lowering and passes
it explicitly to initialization-guard emission. This avoids enlarging the shared
profile or computing storage inventory for inspection-only profiles. The next cohort attempted child-process CPU accounting, but it measured only the
launcher: group termination prevents the launcher from reaping its compiler child.
Those raw `processCpuSeconds` values are explicitly invalid for compiler analysis;
`final-results.json` records that limitation and the script no longer emits them.
The separate primitive matrix invokes the compiler test runner directly, so its
CPU field covers that process and its threads, including compilation and traffic.

## Generated-code equivalence

The candidate rebuilt the original measured Neo.Web application image with the
original AOT command. The complete native object is **byte-for-byte identical**
to the baseline (`object-comparison.json`). This covers the shared lowering of all
benchmark branches, including static-field access and initializer guards. It is
not a claim that JIT address placement is identical across process launches.

## Validation

- Focused layout test: field-only, initializer-only, combined and absent storage,
  source-order gaps and a fresh layout after metadata changes.
- Native static-storage tests: GC-root lifetime and separate closed generic storage,
  including reset between invocations.
- Initialization consumer: readonly initialization agrees in interpreter, JIT and
  AOT. Existing rejection and initialization/fault scenarios are retained.

The application image and host are reused from the previous benchmark. In each
paired launch only `--jit-compiler` changes. Both use eager JIT, the same paced
32 MiB heap / 4 MiB byte-array host, twenty warmups and 1,000 measured requests.
Requests use fresh sequential loopback connections and validate status/body.
Compiler order alternates; no repository build runs during timed trials.
These workstation measurements do not establish production capacity or Windows
performance. Host termination by the harness is not a lifecycle test.

## Reproduction

```sh
cargo test -p neoclr-codegen --release --lib storage_layout_keeps
cargo test -p neoclr-codegen --release --test static_storage --test type_initialization
python3 docs/validation/neo-web-jit-layout-20261031/compare.py \
  --baseline /path/to/baseline/neoclr-codegen \
  --candidate target/release/neoclr-codegen \
  --recipe docs/validation/neo-web-performance-20261031/spot-results.json \
  --output target/web-jit-comparison --repetitions 3 --requests 1000 \
  --aot /path/to/aot-matched
```

The recipe records local artifact paths; restore the matching bundle/application
before replaying it. Exact input hashes and executed commands accompany results.

The AOT control is relinked from the byte-identical application object, using the
original native build recipe with pacing enabled and heap/array budgets changed to
match JIT (32 MiB / 4 MiB). `aot-link.json` retains the command and binary hash.
The three AOT trials run after the JIT cohort; no compilation overlaps timing.
This removes the earlier budget mismatch but retains each execution mode's normal
host/entry and code-placement paths.

## Final Neo.Web layout comparison

Three alternating pairs, same measured image and paced unowned host; 1,000 requests
per trial. The complete rebuilt AOT object remains byte-identical.

| Compiler | Ready median s (range) | Warm median req/s (range) |
| --- | ---: | ---: |
| Baseline | 56.09 (55.80–62.16) | 578.2 (561.7–580.0) |
| Local-layout candidate | 56.47 (54.30–56.61) | 587.0 (582.4–590.4) |

This removes repeated layout work but **does not establish a decisive whole-process
startup improvement**. The overlapping readiness ranges and code-equivalent output
do not justify attributing warm-throughput differences to this cache. The matched
32 MiB paced AOT control has median 666.3 req/s (628.4–667.0). The earlier 56.7 req/s
JIT spot check is not a reliable sustained estimate; this cohort does not establish
why that short run was slow. Remaining metadata/backend preparation dominates the
larger 3,296-function image's readiness.

The final comparison validates 9,189 responses including warmups/first requests.
Initial placement trials and separate profiles remain retained, not pooled into
that cohort. The cache's benefit is eliminating repeated preparation work with
linear temporary storage; no user-visible speedup or runtime-default change is
claimed from these results.

## Direct HttpServer eager/lazy and GC screen

The author subsequently made the primitive HTTP API the primary baseline and
requested a roadmap for throughput and execution continuity. A standalone program
now uses HttpServer directly, with no Neo.Web or DI setup. Its argument-free image
selects 1,147 functions and 361 types; do not compare its readiness to the larger
framework image as a pure JIT-policy effect.

The first fixture exposed an unused process-argument entry. Concurrent lowering
rejected it (`live stores do not admit process-argument providers`); both failed
attempts and four successful sync trials are retained in
`initial-primitive-argument-results.json`. The final fixture removes that unused
dependency. It does not implement process-argument admission for concurrent GC.

Two launches per cell, reverse cell order in round two, twenty warmups and 1,000
measured requests per launch. All 8,168 responses pass. Both hosts use the same
32 MiB heap / 4 MiB byte-array limits, **exclusive owned-heap admission** and GC
profiling instrumentation. Sync pacing is 256 allocations / 4096 publications;
concurrent uses the standard private policy's 1024 / 16384 and 50% log watermark.
These are explicit experimental profiles, not installed-package defaults.

| Mode / GC | Ready s | First request ms | Warm req/s |
| --- | ---: | ---: | ---: |
| Eager / sync | 6.99 | 9.2 | 1549.4 |
| Eager / concurrent | 9.93 | 4.4 | 1508.5 |
| Lazy / sync | 5.18 | 1170.6 | 1953.9 |
| Lazy / concurrent | 5.46 | 2367.4 | 1437.2 |

Lazy mode reduces readiness here but moves compilation into the first request.
Concurrent GC is not a throughput winner in this tiny sequential workload. That is
not evidence against the author's heavy-load direction: this screen does not test
warm traffic during cold compilation, sustained concurrency, allocation pressure,
queue growth or pause distributions. It records configured collector modes and
validated responses, not epoch/GC-pause counts. Those require separate diagnostics.

The private test runner executes the current compiler with qualified live-store
admission. It is not a public concurrent-GC CLI selector. The sync and concurrent
host build commands, policy and hashes are retained. Correctness-only interpreter
and default stress-AOT smoke checks validate another eight responses. Harness
termination is not a collector shutdown qualification.

The primitive and Neo.Web cohorts differ in ownership and instrumentation; do not
subtract their throughput to infer framework overhead. The
[roadmap](../../design/http-throughput-roadmap.md) requires a matched overhead study
and a real sustained-load generator before asserting server capacity or parity
with another runtime. Runtime research and heavy-load tuning plans are documented
there separately from measured capabilities.

```sh
python3 docs/validation/neo-web-jit-layout-20261031/primitive-matrix.py \
  --build target/neo-web-jit-investigation/primitive-noargs \
  --runner /path/to/release/neoclr_codegen-test-runner \
  --sync-host /path/to/sync-owned-paced-host.dylib \
  --concurrent-host /path/to/qualified-concurrent-host.dylib \
  --output target/http-primitive-matrix --repetitions 2
```

`primitive-build.json` retains the source/tool/input identities; large compiler
stdout is represented by byte count and hash, with the full local build log retained.
No runtime policy promotion follows from this study. The author's subsequent
lazy-JIT/paced-sync release-default direction is a separate qualification slice.

## Selection decision

The author subsequently directed selecting release settings for proven throughput,
including experimental settings, with concurrent GC retained as research into this
or alternative implementations. The static-layout candidate has no decisive
whole-process gain in this screen, so it is **not integrated**. Its exact final
source patch and all trials remain retained for future investigation. The executable
candidate remains locally retained under `target/neo-web-jit-investigation`; its
hash is recorded. The separate release-default slice evaluates lazy JIT and paced
synchronous GC, including a matched owned-heap comparison.
