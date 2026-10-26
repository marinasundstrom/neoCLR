# neoCLR Preview 18 — 2026-10-26

Clearer platform contracts, efficient HTTP/value handling and runtime improvements.
Matching macOS ARM64 and Windows x64 SDK/editor: `0.1.18-neoclr.20261025.1`.
Raven is pinned to `ed1d08def`; newer backticked identifiers are not included.
Synchronous GC remains the default. macOS preserves the optional local PostgreSQL
provider; Windows contains the four base libraries.

[Release downloads](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.18) ·
[Changelog](../CHANGELOG.md) · [Qualification and asset identities](validation/release-20261026/README.md).

## Clearer platform contracts

`Optional<T>` replaces `Option<T>`, keeping `Some` and `None`. `Result.Failure`
replaces `Result.Error`, keeping `Ok`: the failure is the outcome, and the error is
its payload. Update constructors, patterns and affected imports, then rebuild
consumers against matching libraries and compiler tooling.

`System.Error` and typed context decorators provide optional diagnostics and
failure-context helpers without requiring every Result or union to implement an
error interface. Capture is off by default. ConditionalAttribute support and
bounded formatting accompany this work; source file/line resolution remains open.

`System.Environment` becomes a static class with fallible OS discovery, process
and OS architecture, NewLine and structured version information. Qualified calls
continue to work; former module-function imports need migration. Additional-host
qualification and Linux distribution discovery remain open.

## Efficient HTTP and value handling

Structured MediaType values, cached presets and typed HTTP content overloads make
media metadata easier to use. HTTP internals retain validated primitives and expose
structured views lazily. Raw outbound media strings now receive full syntax
validation before I/O; Accept ranges/weights are not accepted as MediaType values.

URI parsing and resolution reuse UTF-8 storage and validated components to reduce
temporary work. These changes preserve validation and value-object contracts;
they do not impose a blanket ban on allocations or change Path semantics.

## Runtime optimization, GC improvements and concurrent experiments

Native execution improves repeated root/descriptor work, managed-heap lookup,
allocation bookkeeping and redundant initialization. Async context reuse and
ownership/suspension checks are qualified within their documented host boundaries.
These changes benefit runtime execution beyond HTTP. Managed JIT preparation now
releases unused compilation inputs before guest execution while retaining code and
native-host ownership through cleanup. Memory effects vary and timing results are
mixed; this is not presented as a throughput gain. A test-only JIT placement
experiment also identifies a shared code/static-data cost, improving both collector
paths. Contiguous/fallback placement and the combined stack-bound/small-frame
layout remain opt-in experiments for this candidate. Production memory policy
and additional-platform qualification remain open. Explicit instruction-cache
publication corrects intermittent macOS ARM64 JIT execution failures during code
storage reuse, across ordinary and experimental concurrent paths.

Concurrent native GC continues as a separate experimental track. Work includes
live marking, mutation/root safety, JIT/AOT consumers, worker coordination and
collection-policy tuning. Qualified stack-bound proofs and immutable byref handling
are integrated into this private backend while preserving roots and guarded unknown
destinations. Typed scalar stores are integrated within that private candidate and reduce CPU
substantially in a reusable-buffer workload, while HTTP effects are small and mixed. The synchronous
collector remains the default; this release does not announce a broad concurrent-GC
throughput or latency advantage, generations, compaction or general virtual threads.

The current concurrent execution route is private qualification tooling, not a
supported public GC selector in ordinary hosting. Internal persistent HTTP drivers
and shared-worker experiments likewise do not imply new public server guarantees.
The native candidate implementation now belongs to the runtime source tree, with
a checked owned-heap policy and benchmark-independent source generation. Release packages retain synchronous ordinary execution and keep those experiments separate.

## Better evidence and documentation

A standard suite now covers plain HTTP, JSON HTTP and six standalone allocation
patterns, with separate throughput and profiling runs. Measurements retain CPU,
allocation/memory, collector policy and execution-mode identities. Owner-operation
histograms and metadata/scheduling diagnostics help explain costs without treating
instrumented timings as benchmarks. Results include regressions and limitations.

API references, navigation, migration examples and error-handling guidance are
updated alongside the contracts. Detailed engineering experiments remain in the
[runtime/GC record](development-history/runtime-gc.md), while the
[changelog](../CHANGELOG.md) presents the release themes.

## Measurements and limits

Both platforms pass the standardized release suite. Results retain startup and
sample ranges for interpreter, JIT and AOT CPU/JSON and plain/JSON HTTP. HTTP uses
sequential loopback requests with connection close, not peak concurrency. Windows
results are lower than the previous hosted run and use a different CPU model;
there is no controlled general throughput-gain claim. See the [macOS](../benchmarks/release-20261026/macos-preview18-final/README.md)
and [Windows](../benchmarks/release-20261026/windows-preview18/README.md) records.
Lazy JIT compilation through trampolines is a post-release investigation.
