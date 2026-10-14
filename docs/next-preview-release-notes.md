# Next experimental preview — candidate notes

This candidate is being qualified; Preview 14 remains the published release.
The new SDK/extension build is `0.1.15-neoclr.20261014.1`, paired with Raven native
compiler `fac4ac736024800151d6b285e68407ded864f655`. Download links and final
runtime revisions will be added only after the extracted packages pass qualification.

## Changes to try

- Runtime-owned await is now selected by the SDK's native compiler contract.
  Interpreter, JIT and AOT share the runtime's suspension/resumption model,
  including Windows x64 fibers. Explicit state-machine lowering remains available.
  Earlier development SDK gates pass all 21 cases on macOS and Windows; fresh
  candidate package checks are pending.
- Runtime, codegen, native memory management and GC are maintained together.
  The backend tool is `neoclr-codegen`. Components retain their own boundaries.
- HTTP parsing uses bounded byte scans for URI components and known header names.
  Thirteen known names are lazily shared through invocation-owned static storage,
  with separate storage for closed generic owners and GC retention through await.
  All 132 local three-mode header checks pass; current Windows qualification is pending.
- Collector refinements avoid repeated admission work and empty tracing work for
  text/byte leaves. Borrowed inputs are rooted across allocation retries. Retained
  benchmarks measure specific workloads; they do not establish a general speedup
  over .NET or Go. See the [GC evidence](../benchmarks/gc-index/README.md).
- HTTP JSON helper errors preserve `HttpError | JsonError`; sample mapping errors
  retain their original causes. Recompile consumers and update wrapper patterns.
- The API reference again includes GitHub declaration-file links, including
  PostgreSQL provider pages. These links identify files, not exact member lines.

## Migration and limits

Rebuild applications with the matching compiler, libraries and runtime. Native
hosts must be rebuilt for the 1,080-byte runtime context; do not mix old helpers
with new runtime binaries. Task APIs remain in `System.Concurrency.Tasks`, and
stream operations use `await`.

Static field support does not yet include source type initializers or readonly
static writes. Constructed generic static classes remain a compiler gap; ordinary
generic classes with static members have separate closed-owner storage.

HTTP/2/3, general connection pooling and unrestricted reflection remain outside the
implemented contract. PostgreSQL authentication, transport and deployment limits
remain as described in [Preview 14](preview-14-release-notes.md); no expanded SQL
platform claim is made here. Windows ARM64 and Linux JIT/AOT are not qualified by
this candidate. Profiling tools remain provisional; source debugging is not available.

## Release gates still open

The [validation procedure](next-preview-validation.md) requires canonical source
checks, matching extracted macOS ARM64 and Windows x64 packages, default-JIT
samples, runtime-await and HTTP checks, editor acceptance and recorded hashes.
Current qualification must pass before installation or publication. Website
publication is a separate manual operation.
