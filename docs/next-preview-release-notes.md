# Next preview — draft release notes

Unreleased. Version, publication date and final artifact identities are not yet
selected. The previous published release is [Preview 17](preview-17-release-notes.md).
These notes describe the intended release scope; the
[readiness record](validation/release-preparation-20261025/README.md) tracks what
still needs integration or qualification.

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

## Runtime execution and garbage collection

Native execution improves repeated root/descriptor work, managed-heap lookup,
allocation bookkeeping and redundant initialization. Async context reuse and
ownership/suspension checks are qualified within their documented host boundaries.
These changes benefit runtime execution beyond HTTP.

Concurrent native GC continues as a separate experimental track. Work includes
live marking, mutation/root safety, JIT/AOT consumers, worker coordination and
collection-policy tuning. Typed scalar stores reduce CPU substantially in a
reusable-buffer workload, while HTTP effects are small and mixed. The synchronous
collector remains the default; this draft does not announce a broad concurrent-GC
throughput or latency advantage, generations, compaction or general virtual threads.

The current concurrent execution route is private qualification tooling, not a
supported public GC selector in ordinary hosting. Internal persistent HTTP drivers
and shared-worker experiments likewise do not imply new public server guarantees.
Final release packaging must distinguish supported execution from retained experiments.

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

## Before publication

Fresh matching source/compiler/API, macOS ARM64 and Windows x64 package/editor
checks, release benchmarks and artifact identities are still required. Development
macOS evidence does not certify new packages or additional platforms. No tag,
release upload or website publication has been performed for this draft.
