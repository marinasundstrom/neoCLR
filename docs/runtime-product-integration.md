# Integrated runtime and memory-management direction

Author direction, 2026-10-13. Interpreter, JIT, native AOT and managed memory
management are parts of one neoCLR product. Historical experiment directories and
`neoclr-aot-poc` naming do not define separate products. Qualification gaps remain
explicit; integrating an engine does not qualify every target or enable a prototype.

## Build foundation

The runtime, JIT/AOT backend, shared security provider and TLS adapter now share the
root Cargo workspace and lockfile. Build both executable tools with:

```sh
cargo build --locked --workspace
```

Outputs are `target/debug/neoclr` and `target/debug/neoclr-codegen` (with `.exe` on
Windows). `--release` selects `target/release`. Existing backend manifest commands
also use this workspace and output directory. Old files under
`tools/aot-poc/target` are stale build artifacts, not fallback executables.

Run the shared conformance entry point from an ordinary macOS shell or an MSVC
Windows developer shell with Python, Rust and OpenSSL available:

```sh
python3 scripts/validate-runtime-product.py --output target/runtime-product-check
```

It retains per-command logs and a machine-readable report, combining the workspace
build, CLI mode selection, scalar interpreter/JIT/native comparisons, native GC/root
contracts and TLS native-adapter checks. ARM64 macOS also runs compiled rooted UTF-8
slicing. Platform-specific test guards still apply; this is not full HTTP, Windows
ARM64 or whole-library qualification. CI uses this same entry point.

The shared backend package and executable are `neoclr-codegen`, with sources in
`runtime/codegen`. Rebuilt SDK manifests and CLI discovery use that name without
an old-name alias. Historical reports retain their original tool names. Component
names describe roles: execution engines run code; code generation serves JIT/AOT;
memory management includes the current GC; scheduling/suspension manages progress.
Managed namespaces describe language-facing contracts rather than mirroring the
internal directory tree. This naming follows the author's integration clarification.

`cargo build -p neoclr` and the default root build still omit compiler packages
from the selected build; interpreter-only consumers do not acquire a Cranelift
dependency. Compiler dependencies remain in the shared lockfile.
The runtime's existing resolved dependency versions are retained; common backend
dependencies align with that resolution instead of carrying a second lockfile.

Native TLS static-library builds use `--profile native-ffi` to retain abort-on-panic
behavior. Ordinary runtime builds keep their existing panic policy. Explicit native
archive target directories are retained where packaging requires them. The separate
text/crypto static-library workspaces remain migration work, including preserving
text LTO/size settings. This is not a completed source-layout or packaging migration. `src/jit_cli.rs`
still launches the backend process; workspace membership alone does not create an
in-process JIT service or a complete embedding API. Neither is required merely
to call the product integrated: the author clarified that architectural components
may remain separate. The library slice below improves shared ownership and testing
without changing the installed launcher.

## Native collector source ownership — 2026-10-13

The shared native collector implementation and private header now live in
`runtime/native/gc`. JIT and AOT host recipes still accept the former
`docs/experiments/aot-console/native-gc.c` and `.h` paths through forwarding files;
there is one implementation, not a second collector. Build-kit support manifests
include the canonical files, and current profiling tools instrument their contents.
Benchmark baseline lookup supports both the historical and current source paths.

Root-frame publication, root descriptors and fault-root enumeration now also live
in `runtime/native/gc/root-probe.c` and `.h`, with forwarding files at the former
paths. The collector includes the canonical root header directly. Profiler and
pacing instrumentation read the canonical implementation; historical benchmark
lookup understands both layouts. The packaged support closure includes these
files automatically, and relocation checks compile roots both directly and through
the compatibility path. The root implementation is unchanged apart from includes.

Arena/context types and allocation/text services now live in
`runtime/native/memory/text-arena.c` and `.h`. The old paths forward to these files,
and the collector includes the canonical memory header directly. GC builds retain
their collector allocation path; non-GC builds retain aligned arena allocation
and caller-owned reset lifetimes. The implementation and private ABI sizes are
unchanged. Native support manifests and profiling/validation provenance cover both
source trees.

Fault layout/rendering now live under `runtime/native/faults` and the provisional
POSIX attachment transport under `runtime/native/diagnostics`. Their former paths
forward to the canonical implementation. Native GC, roots and arena headers now
reference runtime-owned headers directly, without dependencies on documentation
experiment headers. Remaining native services are still migration work. The private header names
those dependencies explicitly; it is not a stable or collector-independent ABI.
The integrated validator includes a relocated-package test that compiles and runs
the collector through the forwarding path, directly, and from a profiler-style
copy using only packaged support sources. Native GC/root validation fingerprints
the canonical implementation as well as the compatibility files.

Local GC/root contracts, two compiled UTF-8 GC/non-GC tests, package relocation and
allocation/graph benchmark instrumentation checks pass. The one-round benchmarks
are tooling smoke checks, not performance evidence: collector algorithms and policy
are unchanged. The [hosted gate](https://github.com/marinasundstrom/neoCLR/actions/runs/38434699266)
at `ad3292e8a` passes socket and relocated-package checks on macOS and Windows,
plus Windows GC/root checks. [Raw reports](validation/runtime-product/gc-source-layout/)
retain command/input provenance and hosted job/step outcomes. The local report
predates the implementation commit but fingerprints the relocated sources. The
superseded initial hosted run was cancelled after an MSVC command spelling correction.
Website capabilities and public APIs are
unchanged; no publication is part of this work.

The root-source follow-up passes 20 local GC/root executions, two compiled
GC/non-GC UTF-8 tests, two profiler attachment tests and relocated-package checks.
The [hosted gate](https://github.com/marinasundstrom/neoCLR/actions/runs/38435362775)
at `ab9aa0a2f` passes socket and relocated-package checks on macOS and Windows,
plus Windows GC/root checks. [Raw root-migration reports](validation/runtime-product/root-source-layout/)
retain commands, source hashes and job outcomes. The local report predates the
commit and identifies the tested source through its input hashes. Root ABI sizes, suspension ownership, collection
policy and fault behavior are unchanged. No Raven metadata/Runtime Contract change
is required, and this slice does not alter the non-GC execution profile.

The arena/context follow-up passes local GC/root and relocated-package checks.
The package test now runs the same arena contract through direct and compatibility
paths, both with GC and with no collector linked. It verifies zero initialization,
alignment, unchanged outputs on errors, allocation limits, UTF-8 including embedded
NUL, separate context storage, collector reclamation and non-GC caller-owned reset.
The [hosted gate](https://github.com/marinasundstrom/neoCLR/actions/runs/38436090755)
at `8355c040a` passes on macOS and Windows, including all four arena contract
combinations (direct/compatibility, GC/non-GC), existing relocation/instrumentation
checks and socket contracts; Windows also passes the full native GC/root gate.
Local validation includes 20 GC/root executions, two compiled GC/non-GC UTF-8
consumers and two profiler attachment tests. [Raw reports](validation/runtime-product/arena-source-layout/)
retain source hashes and hosted job outcomes. The local report predates the commit
and fingerprints the tested canonical sources. This is source integration, not a
performance optimization or a new public memory-manager API. The backend process/hosting boundary remains separate follow-up work.

The fault/diagnostics follow-up preserves fault codes, context/frame layouts,
first-fault behavior and output rendering. A relocated-package test checks both
rendering entry paths, including no-fault output, embedded NUL, instruction indices,
truncation text and unchanged input context. Local rendering and POSIX attachment
lifecycle/disabled-endpoint tests pass. Six compiled fault-detail tests and 20 native
GC/root executions also pass locally. The [hosted gate](https://github.com/marinasundstrom/neoCLR/actions/runs/38436723275)
at `fe27a8995` passes package rendering and socket checks on macOS and Windows,
Windows GC/root checks and macOS attachment lifecycle/disabled-endpoint checks.
[Raw reports](validation/runtime-product/fault-diagnostics-layout/) retain source
hashes and hosted job outcomes. The local report predates the commit and records
the tested inputs. The consolidated native services no longer include headers from
`docs/experiments`; forwarding files preserve existing host recipes.
The diagnostic transport remains opt-in and POSIX-only; including its source in
support manifests does not link or enable it in ordinary hosts. Windows attachment
support and a stable diagnostics ABI remain gaps. Existing profiler protocol and
owner-thread snapshot publication are unchanged. No public API or website
capability changes accompany this source relocation.

## Backend library and hosting driver — 2026-10-13

The backend now has a library target. Its executable is a thin adapter over
`neoclr_aot_poc::run(Vec<OsString>)`, which accepts the existing backend arguments
and returns a status or driver error. JIT and AOT use the same selection, verification,
lowering and ownership code as before. `--jit-run` returns the guest status after
native cleanup instead of calling `process::exit` inside the backend. Only the
executable decides its process exit status. Guest faults still terminate the current
invocation; this does not make faults recoverable inside managed code.

This Rust entry point is an internal, provisional synchronous driver, not a stable
managed API or language-neutral hosting ABI. It retains CLI stdout/stderr and artifact
behavior. Calls serialize around native helper state; callback reentry is unsupported.
Native host aborts still terminate the process. Code and helper-library ownership
remain inside each invocation; no callback, task or code pointer may outlive it.
There is no compilation cache, tiering or concurrent-session claim.

The dependency direction remains backend → runtime; interpreter-only builds still
exclude compiler dependencies. The installed runtime CLI still launches the backend
process. **Integration does not negate modularity.**
**Author clarification, 2026-10-13:** experiments/POCs should become
integrated product components so they can be optimized together; they may remain
separate architectural components. A process boundary is not by itself a defect.
The author also emphasized that integration should make profiling and debugging
tools easier to build incrementally. Keep runtime observation points reusable by
separate tools; add measurements needed for the current performance question
without making a complete tooling suite a prerequisite.
Evaluate any in-process composition or runtime-core split against measured startup,
throughput, memory, observability and lifetime needs, rather than making it a
mandatory migration. Avoid a cyclic runtime → backend → runtime dependency.
AOT remains a separate publishing entry point into the same backend library.
The .NET runtime/compilation-layer comparison remains relevant, without prescribing
identical component or process boundaries. A full embedding surface is future work.

A hosting integration test invokes the library repeatedly, checks rejected writes
preserve artifacts, compares library/CLI AOT bytes, exercises success/fault/success
scalar JIT calls and rejects foreign targets. Existing scalar tests check interpreter,
JIT and AOT result/fault parity. This first slice does not qualify repeated managed
HTTP sessions inside one embedding process. The [macOS/Windows product gate](https://github.com/marinasundstrom/neoCLR/actions/runs/38437665710)
at `2d2eab607` passes. The [expanded Windows gate](https://github.com/marinasundstrom/neoCLR/actions/runs/38438253770)
at `8c4e930bb` also passes, adding native execution through the Windows AOT C host.
[Reports and focused test logs](validation/runtime-product/backend-hosting/) retain
exact revisions and commands. The gates include workspace build, CLI selection,
hosting reuse/artifact checks, platform-applicable scalar execution tests, GC/root,
TLS/security and package checks; macOS also runs compiled rooted UTF-8 tests.
The Windows AOT fixture covers calls, branches, faults and UTF-8 output. This does
not imply full managed HTTP embedding/session qualification. Public execution
defaults and website capabilities are unchanged.

This closes the current integration qualification. Per author direction, return to
profiling-led runtime performance work rather than extending structural migration
as a prerequisite. Keep useful component boundaries and add reusable observation
points for separate profiler/debugger tools as concrete measurements require them.

## Memory management is a runtime service

**Architectural direction, not a new public ABI.** GC remains the default memory
manager for managed programs. The author also wants future constrained-device
profiles without tracing GC and distributions with specialized collectors. Select
an implementation when building a runtime distribution; do not implement live GC
replacement or collector migration inside a running execution.

The runtime/memory-manager boundary must cover:

- Aligned, zero-initialized allocation, overflow/limits and terminal exhaustion;
  result publication and admitted allocation retry must preserve existing roots.
- Object/reference layout descriptions, roots/handles, interior references and
  suspended-frame ownership. A moving manager requires updateable references or
  explicitly supported pinning; today's conservative addresses are insufficient.
- Reference stores and bulk copies, with collector-specific barriers when needed.
  Build-selected implementations can inline fast paths; a per-allocation or
  per-reference-store virtual call is not required by this architecture.
- Safe suspension/quiescence, concurrent mutation rules, host attachment and resource
  teardown. Managed liveness and deterministic native-resource release are separate
  responsibilities; fault exits and abandoned operations must remain safe.
- Limits and truthful diagnostics. Collector-specific generations/tuning remain
  optional capabilities, not invented properties of every memory manager.

A bounded arena/region profile could be useful for a closed invocation with a proven
lifetime boundary. It cannot reset memory while objects, host handles or suspended
operations escape that boundary. Reject unsupported program capabilities or prove
those lifetimes; do not silently leak, reset live objects, or call disabling GC a
complete small-device implementation. Reference counting is another possible
implementation, with cycle and store-overhead costs, not a selected replacement.

Distribution compatibility must account for layout, root and barrier requirements.
AOT artifacts and native hosts must match the selected memory profile. JIT and
interpreter execution must use its capabilities consistently. Exact profile identity,
metadata and ABI encoding remain to be designed before alternate managers ship.

## Comparison and tradeoffs

Primary sources reviewed 2026-10-13:

- [.NET collector architecture](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/garbage-collection.md)
  separates allocation and reclamation responsibilities and describes the execution
  engine supplying allocation context. We retain familiar managed lifetime semantics
  while leaving implementation choice behind a runtime-owned boundary.
- [.NET Native AOT build integration](https://github.com/dotnet/runtime/blob/main/docs/workflow/building/coreclr/nativeaot.md)
  links a runtime library with generated code. Native publishing is not equivalent
  to removing runtime services. Separate compilation tools can belong to one product.
- [Embedded Rust collections](https://doc.rust-lang.org/embedded-book/collections/index.html)
  distinguishes allocator-backed and fixed-capacity storage. Predictable capacity
  is useful for constrained devices, but Rust's ownership guarantees do not transfer
  automatically to arbitrary managed object graphs.

The proposed variation is distribution-level memory-manager choice, not a promise
that arbitrary managed programs run without GC. Benefits are footprint/workload
specialization and direct fast paths. Costs are capability validation, collector
conformance, matching native artifacts and a larger supported-profile matrix.
No alternative collector or no-GC device distribution is implemented by this slice.

## Next integration slices

1. Keep one build and focused conformance entry point for CLI/interpreter, JIT,
   compiled native consumers and GC/root contracts; retain platform-specific tests.
2. Move shared native runtime implementation out of documentation experiment paths,
   keeping fixtures and historical evidence separate. Update packaging/import recipes
   together; do not duplicate implementations or promote incomplete APIs by renaming.
3. Maintain the shared backend library and explicit hosting/lifetime contract.
   Optimize component boundaries using measurements; a single process or crate is
   not a completion criterion. Keep source ownership, build/release provenance,
   diagnostics and execution-mode conformance integrated across components.
4. Consolidate static-library builds, distribution manifests and backend naming.
5. Extract the memory-manager contract from current allocation/root/cleanup behavior,
   then qualify build-selected implementations. Finish current helper-root and idle
   cleanup work before changing pacing; measure improvements independently of this
   build integration. Generational GC remains a candidate, not a selected policy.

## Qualification — 2026-10-13

The local integrated gate and the hosted macOS job at `bb5a3d2b0` pass: workspace
build, CLI selection, 24 scalar execution/compiler tests, two compiled rooted-text
tests, 20 native GC/root executions, security/TLS adapter checks and packaging
policy tests. Reports are retained in [validation/runtime-product](validation/runtime-product/).
The local report predates the implementation commit; hosted evidence has the exact
revision and input hashes. The root-only dependency graph excludes Cranelift.

The first Windows run exposed line-ending conversion of the hashed Cranelift
license, fixed by preserving notice bytes in checkout. The next run passed notice
checks, the workspace build and CLI selection, but exposed a CRLF-sensitive
negative compiler fixture. That fixture now normalizes line endings and verifies
each invalid-input mutation is effective; the focused local rerun passes. The [Windows rerun](https://github.com/marinasundstrom/neoCLR/actions/runs/38431039111)
at `4d1efde54` passes the full integrated gate, including all 12 applicable scalar
compiler/JIT tests, 20 GC/root executions, TLS/security and packaging checks.
The retained Windows reports include exact inputs and commands. These are build/test
integration fixes, not a measured runtime throughput improvement.

## Runtime-owned suspension carrier — 2026-10-13

The cooperative carrier now lives in `runtime/native/async`, with forwarding files
at the historical experiment paths. macOS retains its existing `ucontext` carrier.
Windows uses [CreateFiberEx](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createfiberex)
and [ConvertThreadToFiberEx](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-convertthreadtofiberex),
reviewed on this date, beneath the same operation, scheduling and parked-root
logic. Unlike .NET compiler state machines, these application activations retain
native stacks; the benefit is ordinary call-chain suspension, at a stack reservation
and switching cost. This is private runtime support, not a new public ABI.

The carrier still has fixed private tables of 128 operations and 128 activations;
this integration does not remove those capacity limits. The Windows carrier is
thread-affine and uses a 1 MiB reservation per activation.
It preserves pre-existing host fiber conversion and only undoes conversion it owns.
Completed/abandoned Windows fibers are deleted, not cached; Windows stack caching
needs a separately qualified reusable entry loop. macOS guarded-stack caching is
unchanged. Stack probes validate actual thread/fiber bounds and preserve the stack
reserve, rather than rejecting all Windows fibers.

The shared GC gate runs owned-body survival, abandonment, pressure rollback and
idle-drain tests on both OS carriers, plus a Windows host-owned-fiber case. Local
macOS passes all 35 GC/root cases after migration. The hosted macOS/Windows [carrier gate](https://github.com/marinasundstrom/neoCLR/actions/runs/38462503102)
also passes, including Windows host-owned fibers. See the [retained reports](validation/runtime-product/runtime-await-sdk/).

Author clarification: backward compatibility is not a constraint for this
experimental integration. Existing forwarding paths serve active internal recipes
during migration; they are not supported external contracts. Consumers may move
together with the runtime. State-machine lowering is retained as a validation and
benchmark control, without a long-term compatibility commitment. Matching build
requirements still matter for correctness.

## SDK-default runtime-await integration — 2026-10-13

Development SDK builds now set `RavenNeoClrRuntimeAsyncHelper` to
`RuntimeAwaitProvider.Bridge` in `NeoCLR.ClassLibrary.props` and include the matching
`RuntimeAwait.rvn` and its `RuntimeGenerator.rvn` companion from the SDK library
directory. Their canonical sources live in `runtime/raven/compiler-services`;
callers no longer copy experiment files. The companion supplies the existing demand-driven `yield`/`await for`
contract, including cancellation and disposal. These source-included,
assembly-private providers use the existing explicit Raven Runtime Contract. They do not add public managed API or change
ordinary Raven/.NET lowering. Existing source-provider type names remain private
compatibility details, not a new module-authoring convention.
Remove manually copied provider sources when upgrading; otherwise the SDK and
project would declare the same helper types. Projects supporting old and rebuilt
bundles can condition their legacy source include on
`RavenNeoClrRuntimeAsyncProviderImported != true`.

To select compiler state machines explicitly, place this property **after** the SDK
import in the project (an empty environment variable alone is not an override):

```xml
<PropertyGroup>
  <RavenNeoClrRuntimeAsyncHelper></RavenNeoClrRuntimeAsyncHelper>
</PropertyGroup>
```

The SDK conditionally excludes its provider for that selection. Historical
state-machine benchmark controls select it explicitly. Library bootstrap projects
retain their existing lowering; default application lowering does not retroactively
rewrite referenced assemblies. The PostgreSQL library retains its explicit runtime
provider selection, now from the canonical source.

AOT project builds admit the shared suspension bindings on macOS ARM64 and Windows
x64 and link the carrier only when selected code uses it. Async console programs
receive the task/scheduler invocation host as well. Installed JIT helper builds
include the carrier and expose it on both platforms; old JIT helper binaries must
not be paired with new compiler/runtime inputs. Generated Windows code keeps its
stack probes and the carrier preserves floating-point state through OS fibers.

Native intent is runtime-owned suspension/resumption with rooted activation
lifetimes. The current provider still adapts `Task` through managed callback
notifications and private runtime intrinsic declarations; source inclusion,
per-assembly helper metadata and callback allocations are temporary costs. A future
compiler/runtime provider contract can remove that bridge without changing Task's
operation semantics. The native module compiler uses this explicit contract; it
does not redefine ordinary .NET async behavior or promise Windows ARM64 support.

Validation: `scripts/validate-runtime-await-sdk.py` rebuilds SDK-default applications
and an explicit state-machine control, inspects application lowering, stages a
matching installed JIT bundle, then exercises interpreter/JIT/AOT language, cleanup,
fault, HTTP server/client and generator cases. The hosted Windows HTTP workflow can run this
gate with `runtime_await=true`. All 21 macOS SDK cases pass, including generators and the explicit state-machine
fallback; source hashes remain unchanged through execution. Windows application
qualification is running. The raw carrier gate passes on both platforms. See
[retained evidence](https://github.com/marinasundstrom/neoCLR/tree/main/docs/validation/runtime-product/runtime-await-sdk).

The Windows application gate exposed a remaining console-profile admission guard:
task/scheduler/timer bindings are now accepted for its integrated async host, while
file/socket exclusions and rooted-stack prerequisites remain. Six cross-target
backend checks cover admission, GC publication and stack probing. The HTTP workflow
can reuse a prior matching toolchain/runtime artifact via `reuse_bundle_run` for an
SDK-only rerun with freshly rebuilt runtime/backend tools; `sdk_smoke_only` limits
a naming/package rerun to the cleanup consumer in all three modes. This does not qualify arbitrary
mixtures of runtime ABI versions.
