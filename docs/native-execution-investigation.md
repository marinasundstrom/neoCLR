# ARM64 native execution and hot reload investigation

**2026-10-07 — provisional design with a first scalar AOT experiment.**
The [ARM64 scalar probe](experiments/aot-scalar/README.md) now emits and executes a
small native subset. General AOT, JIT and hot reload remain unimplemented.
The author requests investigation of future JIT and AOT, particularly AOT, while
retaining the interpreter, keeping hot reload independent of interpretation, and
making ARM64 the primary target architecture. These are direction constraints;
the backend choice, profiles and experiment sequence below are assistant proposals.
The author subsequently identifies an AOT-compiled web app as an attractive POC
and clarifies that we should start simple. Treat the web app as the motivating demo,
with small executable steps first, not as the initial compiler acceptance test.
This investigation does not replace the active Raven integration work with a native
compiler milestone. See the [roadmap](platform-roadmap.md) and
[execution architecture](execution-architecture.md).

## Subsequent author clarification: early self-contained CIL foundation

The author specifies CIL-to-native compilation and executables with dependencies
baked in, without a shared framework/managed runtime installation. Runtime services
must be linked into the application where required. This is compatible with ordinary
OS dependencies and separate optional interpreter/JIT deployments; it is not a demand
that every executable contain all execution engines.

The foundation should be exercised early as the platform evolves. The author selects
**Hello World first**, then increasingly complex executable samples until **HTTP
Server**. The [native Hello World probe](experiments/aot-hello/README.md) implements
literal UTF-8 output and self-contained native service linking. After the initial
neoIL proof, the author clarifies **neoCLR CIL** and the exact pipeline: compile Hello
World in Raven to its metadata format with IL, then compile that artifact to native
code. This now passes for Raven PE/#Neo output and its standalone NEOX encoding; the
backend reuses native metadata decoding and verification. The bounded compiler/console
bootstrap and class-library limits are recorded with the sample. The broader experimental list below is research context; this
later author-directed sample progression controls the immediate work. The author
also identifies trimming as a future step; establish explicit code/metadata retention
and dynamic-use roots, without making trimming a prerequisite for Hello World.

## Metadata beside native images (future exploration, 2026-10-07)

The author proposes retaining metadata alongside a native image as a richer interface
for calling native functions, with stable ABI conventions. This remains exploratory,
not a selected file format, loader or export ABI. The existing POC's private aggregate
calling convention and row-based symbol names do not satisfy that future contract.

The .NET Native AOT baseline exports explicitly annotated methods as C entry points
([Microsoft interop documentation](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/interop),
reviewed 2026-10-07). GNOME's [GIRepository typelibs](https://gnome.pages.gitlab.gnome.org/gtk/girepository/)
describe native C APIs for introspection and bindings (reviewed 2026-10-07). These are
useful comparison points: retain a conventional native ABI and describe it in metadata,
or define a neoCLR ABI with generated adapters. Neither is selected here.

Potential benefits are discoverable signatures, generated bindings and consistent
UTF-8/ownership contracts. Costs include metadata/image pairing, compatibility/version
rules, foreign tooling and adapter overhead. Explore stable exported identities, target
and calling convention, field layout/alignment, strings, ownership/borrows, result/Fault
encoding and callbacks. Verify image/metadata association and reject incompatible pairs.
A sidecar is independently inspectable but can become mismatched; an embedded section
simplifies pairing but couples tools to image formats. Neither requires a shared managed
runtime merely to describe an exported interface. Start a future experiment with one
scalar export and an independently compiled consumer, including mismatch rejection.
No ABI improvement or performance advantage is established yet.

## IL-free metadata for AOT reflection and interop (author proposal, 2026-10-08)

The author proposes placing a metadata file with IL stripped out beside a native
binary, converging future AOT reflection with the metadata-based interop direction
above. This is a future exploration, not an implemented capability or a requirement
for every sample in the initial stable baseline.

Separate metadata-only inspection (names, signatures, attributes and relationships)
from execution (constructors, method invocation and field access). Inspection needs
retained descriptions; execution also needs retained compiled bodies, closed generic
instantiations, layout facts, marshalling/ownership rules and generated invocation
adapters. Metadata alone cannot restore stripped code or generate a missing generic
specialization in an AOT-only application. Describe an inspectable member separately
from an invocable export; missing execution support should produce an explicit result.

.NET Native AOT already distinguishes retained reflection from dynamic code generation
and requires trimming analysis; its documented limitations include dynamic assembly
loading and Reflection.Emit. Its native exports use explicit UnmanagedCallersOnly
entry points. These are shipped comparison points, not evidence of an external
metadata-sidecar contract identical to this proposal. Primary sources reviewed
2026-10-08: [Native AOT overview](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/)
and [native interop](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/interop).

A shared metadata schema could benefit offline inspection and binding generation
without IL distribution or a shared managed runtime. Costs are retained metadata/code
size, invocation stubs, ABI evolution and deployment pairing. A sidecar is convenient
for tools but can be missing or mismatched; embedded metadata avoids a separate file
at the cost of image-format coupling. Keep a stable exported identity separate from
row numbers, symbol spelling and private AOT function indices. Pair metadata with an
image identity, architecture and ABI version; validate offsets/signatures before use.
Trimming must retain invocation roots independently from description roots.

A future bounded experiment could inspect one type and call one scalar export through
its descriptor, test a missing/mismatched sidecar, and prove that description-only
members cannot be invoked. Neither a stable ABI nor a metadata loader/trimmer is
selected here. The immediate author direction remains fixing easy and urgent native
compilation issues found by sample execution.

## Reference counting as an early native experiment (2026-10-07)

The author asks whether reference types should initially use reference counting,
“Before we consider JIT for AOT”. This is recorded as an open question, not a chosen
replacement for tracing. Memory management is independent of code-generation timing:
AOT requires no JIT, and either native execution mode can call a statically linked
memory manager. The value/member work is [recorded separately](experiments/aot-values/README.md):
reference-free records (including nested storage) now execute natively, without selecting a heap strategy.

The existing interpreter has a nonmoving tracing heap and reclaims unreachable cycles
([implementation](../src/gc.rs), [contract](garbage-collection.md)). .NET's baseline
also identifies live objects from roots and traces the reachable graph, though its
production collector has generations and compaction
([Microsoft GC fundamentals](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/fundamentals),
reviewed 2026-10-07). Adopting native reference counting would be a new backend strategy,
not merely filling in an absent managed lifetime policy.

Reference counting can provide an incremental experiment with explicit allocation,
retain/release and destruction of owned reference fields. Its costs include updates
on ownership changes, cleanup on every return/Fault path, cascading reclamation, and
additional coordination for shared objects. Plain counts cannot reclaim strong cycles.
[Rust Rc](https://doc.rust-lang.org/std/rc/index.html) documents both cycle leakage and
the single-threaded/non-atomic restriction; Weak breaks selected ownership cycles,
but adding weak edges manually is not equivalent to automatically collecting arbitrary
managed cycles (reviewed 2026-10-07). No performance advantage is established here.

**Provisional assistant recommendation:** keep the value/member slice first. Compare
an isolated, explicitly single-threaded reference-counting experiment with a simple
nonmoving native tracer using registered roots. The tracer preserves existing cycle
semantics but requires native roots and collection boundaries; reference counting
requires ownership lowering and a cycle strategy before general managed compatibility.
Neither is automatically the cheaper complete implementation. Do not select a
production policy or promise deterministic destruction based on this discussion.

For either approach, define per-type reference descriptors and copying/overwriting
rules first. Inline value types containing references still need reference management;
borrowed byrefs must preserve their owning allocation and cannot be treated as owning
object references indiscriminately. Keep layout/lifetime operations internal to the
native backend/runtime boundary, with no new retain/release obligations in Raven
source or portable neoCLR CIL. Replacing a strategy later can require recompilation;
it is not necessarily an ABI-compatible runtime swap.

A reference-counting trial must test aliases, self-assignment, reference-bearing
value copies, field replacement, returns, early branches, Fault cleanup, interior
references and a cycle that exposes its limit. An unrestricted cycle must either be
collected by an added mechanism or excluded by a clearly enforced experimental profile;
leaking it cannot be reported as parity with the interpreter. Weak references,
concurrent ownership, finalizers and resource cleanup are separate contracts. Timely
external cleanup remains Dispose/Close, not an implicit promise from counting.
No reference-counting implementation or benchmark is added in this planning slice.

## First native value/member slice

Raven Counter and Int32/Boolean record consumers now exercise native constructors,
fields, accessors, borrowed receivers, copied arguments/results and typed branch joins.
A subsequent nested-record slice adds inline payload storage, aggregate field copies and
interior borrows with cycle/size rejection; the Raven Envelope sample runs independently.
The [value profile](experiments/aot-values/README.md) documents the private flattened
ABI, interpreter comparisons and unsupported cases. It reuses the original native
metadata reader/verifier and arithmetic-Fault lowering; no new Raven encoding is added.
Ordinary output parameters now pass borrowed storage through calls, with conservative
whole-slot assignment proof because interpreter callee-output checks are dynamic.
A Raven output-member probe runs natively; conditional output contracts remain deferred.
Byte storage and unchecked tag conversions now preserve Int32 evaluation-stack
semantics and low-eight-bit storage truncation, with Raven and boundary-value probes.
Overloaded calls now resolve through exact signatures and optional definition identities;
private per-function native symbols remain local to a single compilation. Stable mangling
for separate compilation is a future contract, not selected by this implementation.
An [ordinary Raven Some/None app](experiments/aot-union/README.md) now runs natively
with explicit closed-world code selection, covering both cases, patterns and copies.
A subsequent [local generic Result/pattern slice](experiments/aot-values/README.md#generic-result-and-pattern-bindings-2026-10-07)
adds one closed value instantiation per local definition; let-else and if-let execute
natively, while plain positional let deconstruction remains a producer gap.
Generated formatting/boxing, generic library unions and reference-bearing payloads remain
subsequent work. The author adds console input after unions, exercising input/parse
outcomes. Neither an input service nor a heap manager is implemented by these value records.

## Findings and proposed direction

JIT and AOT are plausible extensions of the existing architecture, but adding a code
generator alone will not produce a usable managed runtime. The larger task is
specifying native representations, runtime boundaries and code lifetimes.

Retain one platform contract with three execution implementations. Keep the
interpreter as a supported option for embedding, debugging, short-lived workloads
and rapid iteration; do not require every deployment to include it. Give AOT the
first native experiment, with ARM64 as the primary architecture. Introduce JIT
through the same lowering and runtime contracts when a concrete workload needs it.

Treat hot reload as a separate capability: interpreter bodies, JIT code and
precompiled native modules can all be replaceable. AOT means compilation before
execution; it need not mean every process has permanently immutable code. However,
a statically linked image without replacement machinery cannot accept arbitrary new
native bodies. Reloadable AOT needs an explicit module/update mechanism, compatible
runtime contracts and host permission to load replacement executable code.

## Repository evidence and gaps

Inspected checkout: `codex/structural-types`, commit `a081c6e3`, on 2026-10-07.
Structural Function behavior here is experimental, not evidence of support on main.
The author subsequently corrects the working branch to main and requests a cherry-pick
of the isolated AOT slice. Main already has newer native metadata/System bootstrap
work; historical CLI-bridge assumptions in this initial assessment are not the current
main milestone. The scalar neoIL probe is independent of either input transport.
Pre-existing edits to the dynamic-dispatch and structural-types proposals are outside
this investigation. The subsequent author instruction to continue AOT produced the
isolated scalar tool and its linked execution evidence. No Raven compiler, bridge
encoding or runtime code changes are made; no new Raven bundle revision is claimed.

| Existing foundation | Evidence | Native execution gap |
| --- | --- | --- |
| Immutable resolved program | [LoadedProgram](../src/program.rs), [contract](loaded-program.md) | A snapshot is not a persistent reloadable session or stable native ABI. |
| Closed call graph and service uses | [analysis](../src/reachability.rs), [reachability](reachability.md), [services](runtime-services.md) | Complete roots for dispatch, reflection, initialization, layouts and native helper dependencies still need a backend-specific audit. A graph is not proof of compilability. |
| Explicit scalar/record layout | [layout](target-layout.md), [implementation](../src/memory.rs) | Pointer width and alignment do not establish an ARM64 calling convention, native object layout or every managed type layout. |
| Interpreter values and frames | [Value](../src/value.rs), [VM](../src/vm.rs) | Rust enums, slot cells and operand vectors are implementation details; do not freeze them as the generated-code ABI. |
| Tracing managed heap | [collector](../src/gc.rs), [GC contract](garbage-collection.md) | Native register/stack roots, safepoints, transitions and cooperation with workers need explicit contracts even for nonmoving collection. |
| Verification and logical diagnostics | [verifier](verification.md), [stack traces](stack-traces.md) | Preserve required checks and map native locations to versioned logical methods. Verification coverage must match the compiled subset. |

The existing Raven/CLI import path can supply a bounded backend experiment. Native
metadata is an eventual replacement for temporary transport, not a reason to import
ordinary CLR object layout or to couple code generation to Raven syntax. Keep target
contracts language-neutral and preserve UTF-8 String and grapheme Char semantics.

## .NET and other runtime comparison

Sources below were consulted on **2026-10-07**. Microsoft deployment documents use
their .NET 9+/10 context; Visual Studio documents describe product/compiler-dependent
support. They are shipped-product documentation, not proof that every combination
works. LLVM/Cranelift references are rolling documentation, not pinned dependencies.
Pin exact compiler versions before experiments; no upstream implementation-source
claim or local .NET comparison execution is asserted here.

| Baseline or alternative | What it establishes | Implication and cost for neoCLR |
| --- | --- | --- |
| [.NET Native AOT](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/) | Publish-time native compilation without a runtime JIT, with runtime support retained. It constrains dynamic assembly loading/code generation and requires trimming; ARM64 targets exist. | Follow explicit deployment capabilities and root analysis. Retaining introspection metadata is distinct from retaining executable reflection targets. Do not describe all reflection as impossible in AOT. |
| [.NET ReadyToRun](https://learn.microsoft.com/en-us/dotnet/core/deploying/ready-to-run) | Precompiled code can coexist with IL and runtime JIT, including replacement by tiered compilation. | Hybrid execution is a legitimate separate profile, but does not satisfy an AOT-only deployment guarantee. Extra representations increase size and testing obligations. |
| [Visual Studio Hot Reload](https://learn.microsoft.com/en-us/visualstudio/debugger/hot-reload) and [supported edits](https://learn.microsoft.com/en-us/visualstudio/debugger/supported-code-changes-csharp) | Compiler/runtime/tooling cooperation enables edits while running; signatures, active statements and application configuration constrain support. | Hot reload is not intrinsically an interpreter feature. Define our supported edit set and reject incompatible edits rather than promise arbitrary state migration. |
| [Erlang/OTP 27 code loading](https://www.erlang.org/docs/27/system/code_loading.html) and [BeamAsm](https://www.erlang.org/docs/27/apps/erts/beamasm.html) | Versioned current/old modules coexist; BeamAsm compiles BEAM at load time on ARM64 and x86-64. | Concrete precedent for native execution plus replaceable code. Borrow explicit version lifetime, not Erlang's process, term or call semantics wholesale. |
| [Mono AOT design](https://www.mono-project.com/docs/advanced/runtime/docs/aot/) | Historical .NET ecosystem implementation separates ordinary and full AOT and accounts for wrappers/trampolines that otherwise require runtime generation. | Audit hidden generated stubs, generic adapters and interop, not just method bodies. Historical details are not a current Mono platform-support matrix or an independent modern backend recommendation. |

C# edit restrictions belong partly to compiler-generated state machines and tooling;
Native AOT deployment restrictions are not a CLI instruction-set rule. neoCLR must
make its own choices at the metadata, loader, runtime and deployment layers.

No modern independent .NET backend was evaluated deeply enough to recommend reuse.
Mono is historical alternative-runtime evidence, not a claim of independence from
today's .NET ecosystem. A relevant first-hand integration discussion is
[dotnet/runtime #102048](https://github.com/dotnet/runtime/discussions/102048): it asks
about combining AOT libraries with a JIT host and highlights resource-lifetime issues.
It motivates an unload test; this investigation does not treat discussion advice as
a supported hot-reload contract or copy an unloading design from it.

## Backend options

| Option | Benefit to investigate | Cost / reason not to select it yet |
| --- | --- | --- |
| Cranelift for AOT and JIT | Rust implementation; ARM64 code generation; object and JIT modules expose both output paths. A practical first integration candidate. | Need to validate target ABI, debug information, root reporting, relocation and packaging against neoCLR's requirements. Compiler speed and output quality must be measured locally. |
| LLVM for AOT and potentially ORC JIT | Established object generation and optimization infrastructure; ORC supplies JIT/linking components. Attractive for an AOT-focused product. | Larger integration/toolchain surface, version management and semantic lowering burden. ORC does not supply neoCLR GC or state migration. |
| Emit C, then use a native compiler | Useful reference prototype and inspectable runtime-helper boundary. | Must explicitly preserve integer, aliasing, memory and Fault semantics instead of inheriting C undefined behavior; awkward source mapping and no direct shared JIT path. |
| Hand-written ARM64 emitter | Direct control over a tiny baseline or diagnostic experiment. | Own register allocation, relocations, ABI, unwind/debug information and maintenance. ARM64-first alone does not justify this cost. |

Primary backend sources: [Cranelift](https://cranelift.dev/),
[object module](https://docs.wasmtime.dev/api/cranelift_object/index.html),
[JIT module](https://docs.wasmtime.dev/api/cranelift_jit/index.html),
[LLVM object generation tutorial](https://llvm.org/docs/tutorial/MyFirstLanguageFrontend/LangImpl08.html),
[ORC](https://llvm.org/docs/ORCv2.html) and
[JITLink](https://llvm.org/docs/JITLink.html).
These establish available building blocks, not measured neoCLR results.

**Provisional recommendation:** use Cranelift for the first bounded feasibility
probe, keeping LLVM as an AOT alternative. Select a production backend only after
the same consumer and runtime-boundary tests. Avoid committing now to maintaining
Cranelift JIT plus LLVM AOT: two lowerings double important correctness work.

## ARM64 first, with explicit OS contracts

ARM64 is the author's primary architecture direction, not a removal of existing
Windows x64 or other interpreter support. Propose macOS ARM64 first because the
repository already has a qualified local runtime/Raven workflow, then Linux ARM64
for a second ABI and server deployment check. This OS order is a proposal, not an
author-selected support matrix; Windows ARM64 remains a separate qualification.

Each artifact needs a target triple, endianness, pointer/layout rules, minimum CPU
features, object format, runtime ABI version and platform dependencies. ARM64 does
not mean one universal binary: Mach-O, ELF and PE/COFF and their platform conventions
need separate support. Use [AAPCS64 2025Q1](https://github.com/ARM-software/abi-aa/blob/2025Q1/aapcs64/aapcs64.rst)
as a baseline, then verify each platform's deviations rather than assuming AAPCS64
alone qualifies an OS. Check argument extension, aggregate returns, stack alignment,
callee-saved registers and native callbacks with a C consumer.

JIT publication must obey executable-memory policy, write/execute transitions and
instruction-cache synchronization. Apple's
[JIT entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.cs.allow-jit)
is one platform constraint to investigate; it is not an assurance that every Apple
application environment permits JIT or downloaded native replacements. AOT module
loading also depends on signing, loader and distribution policy. Cross-compilation
requires the target linker/SDK/sysroot as well as an ARM64 code generator.

## Windows x64 scalar object foundation (2026-10-09)

The author-selected next portability slice now adds explicit
`--target x86_64-pc-windows-msvc` to `neoclr-aot-poc`. The existing default remains
`aarch64-apple-darwin`, including when running the compiler on another host; target
selection is explicit rather than inferred from the build machine. The scalar
backend uses Cranelift 0.121.2's target-default calling convention and emits x64 COFF.
Mach-O build-version metadata is emitted only for Mach-O. Existing scalar ABI v2
status/result-pointer semantics and temporary length-prefixed literal text remain
unchanged. No Raven metadata or .NET compiler mapping changes are introduced.

```sh
cargo build --locked --manifest-path tools/aot-poc/Cargo.toml
tools/aot-poc/target/debug/neoclr-aot-poc \
  docs/experiments/aot-scalar/scalar.neoil Calculate scalar.obj \
  --target x86_64-pc-windows-msvc
```

Initial local cross-target tests inspect COFF architecture, exported entry symbol and
relocations for scalar calls, control flow, arithmetic faults and literal-console
output. They also reject unsupported targets/profiles and preserve existing output.
The retained Raven-produced Hello World PE/#Neo artifact also emits an x64 COFF
object through `@entry --console`; this reuses the recorded source/compiler fixture.
Those initial checks establish object emission, not Windows executable qualification.
[Evidence](experiments/windows-aot-scalar-validation.json) distinguishes those checks
from the then-pending Windows run. The six existing inspection tests and seventeen scalar
tests pass on macOS, including actual C-consumer/interpreter parity.

On a Windows x64 MSVC developer command prompt, run:

```sh
cargo test --locked --manifest-path tools/aot-poc/Cargo.toml --test windows_scalar
```

The Windows-only execution test links a C consumer with MSVC's static CRT and
checks eight Int32 inputs for each of four workloads, comparing with interpreter
results and fault codes. The bounded host locks stdout and sets redirected streams
to binary mode, preserving UTF-8 bytes, LF and embedded NUL. Interactive console
code-page/display policy is not qualified. The
[focused Windows workflow](../.github/workflows/windows-aot-scalar.yml) runs this gate
on relevant pushes/PRs or manual dispatch.

**Windows execution qualified (2026-10-09):** the dedicated
[GitHub run](https://github.com/marinasundstrom/neoCLR/actions/runs/37947583061)
at `7cfe27222d875686f6ce5b0bfa052b8a64d05e88` passes all four tests with no skips
on Windows Server 2022 x64. The MSVC-linked consumer passes 32 input comparisons
covering calls, loops/branches, divide/overflow faults, UTF-8, NUL and LF output.
The [retained report](windows-aot-execution-validation.json) includes toolchain
identity, native outcomes and verified downloaded artifact hashes. The Raven Hello
World fixture is object-emission-tested; its Windows executable run remains a
separate next consumer. This supersedes the initial execution-pending status, not
the managed-service/unwind limitations below.

The dedicated job uses Windows Server 2022 with an explicit x64 MSVC developer
environment. Manual dispatch accepts a `revision` commit/ref, and the report records
the resolved checkout SHA. `scripts/validate-windows-aot.py` retains the Cargo test
log, linker logs, input programs, COFF objects, C-consumer executables and all 32
native outcomes in a GitHub artifact on success or failure. It requires the native
execution completion record; zero tests, a skipped native test or a partial run
cannot be reported as qualification. The workflow has a 35-minute timeout and
14-day artifact retention. It does not publish a package or run website deployment.

After the workflow is on the selected remote ref, dispatch an exact candidate:

```sh
gh workflow run windows-aot-scalar.yml --ref main -f revision=COMMIT_SHA
```

Relevant code pushes/PRs also trigger this focused gate automatically; a matching
automatic run can provide the same evidence without dispatching a duplicate.

Baseline: Microsoft's [x64 calling convention](https://learn.microsoft.com/en-us/cpp/build/x64-calling-convention?view=msvc-170)
specifies register arguments, shadow space, alignment and unwind requirements;
[_lock_file](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/lock-file?view=msvc-170)
documents the CRT stream lock (reviewed 2026-10-09). Reuse the existing .NET/CLI
arithmetic and status-propagation comparison from the scalar experiments. Selecting
the Windows convention is required interoperability work, not an improvement over
.NET Native AOT. Cranelift object emission here does not add Windows unwind tables,
SEH interoperability or native stack walking; guest faults use explicit statuses.
Those gaps must be addressed before broader native hosting/suspension qualification.

Windows inspection, closed-world libraries, value/managed profiles, GC, task/socket
services, native project kits and stack guards remain explicitly unsupported.
The Windows C adapter is an acceptance host, not the Windows implementation of
the macOS console/HTTP kit. With bounded Windows linking/execution now qualified, use actual
Raven-produced scalar/console metadata in execution before extending services or advertising
Windows project publication. Windows execution was performed in GitHub Actions;
no local Windows host or emulator was used.

The next Windows gate now includes the retained Raven Hello World fixture in both
PE/#Neo and standalone NEOX containers. MSVC links with `/MT`; the test copies only
the executable into a fresh directory and checks exit zero and byte-exact interpreter
stdout/stderr. Its separate completion marker requires both containers, so a skipped
native test cannot qualify the run. Execution of this extension is pending. This
qualifies the retained compiler fixture, not a fresh Raven source build on Windows.
Like .NET Native AOT, startup is native; this probe covers only literal console output
and does not claim comparable library coverage. The C adapter remains provisional.

## Shared native contracts to establish

1. **Lowering and checks.** Resolve and verify a declared IL subset into a typed
   control-flow representation before backend IR. Preserve checked arithmetic,
   division edge cases, floating-point behavior, null/bounds/access/lifetime checks,
   copies and initialization. Do not add LLVM flags or alias assumptions stronger
   than the platform guarantees. Unsupported opcodes must fail compilation.
2. **Calls and services.** Specify scalars, aggregates, receivers, byrefs, interface
   dispatch, callable adapters, Void/unit and runtime context passing. Use explicit
   helper entry points initially; no Rust enum layout across the boundary. Keep
   normal Result/Option data separate from terminal Fault propagation. A first probe
   can use status plus out-result to avoid unwinding across Rust/C/native frames.
3. **Managed memory.** Start with the existing nonmoving policy; prototype explicit
   shadow-stack roots before sophisticated stack maps if that simplifies correctness.
   Register live references around allocating calls and define safepoints on loops
   and transitions. Byrefs, interior references, callbacks, suspended async work,
   cancellation and workers must remain accounted for. Nonmoving does not mean
   untraced or safe to keep unreported pointers in native registers.
4. **Reachability and generics.** Seed entry points, exports and declared reflection
   targets; close over calls, bindings, dispatch candidates, constructors, static
   initialization, layouts and service helpers. Initially specialize required closed
   instantiations with bounds; diagnose unbounded demand. Code sharing/dictionaries
   remain alternatives when size measurements justify them.
5. **Metadata and artifacts.** Preserve type/member identities and UTF-8 contracts;
   version implementation bodies separately from logical identity. Cache against
   content, target features, compiler/runtime ABI, dependencies and optimization or
   reload policy. Keep introspection data by policy and invocation stubs by explicit
   roots. A native executable or exported library can link runtime support without
   containing an interpreter or compiler.

## Hot reload across execution modes

Proposed initial unit: a compatible method-body update in an explicitly reloadable
module. Stable entry cells select a versioned body. Interpreter entries select IL,
JIT entries compiled bodies, and AOT entries precompiled module exports. A host owns
persistent state separately from code images; today’s fresh-execution embedding
helpers do not already provide this model.

Proposed lifecycle:

1. Prepare and verify an update without changing active code. Check identity, ABI,
   layout, dependencies, metadata and runtime capability requirements.
2. Publish a complete generation atomically at a defined safe boundary. New external
   entries use that generation. For the first experiment, an in-flight invocation
   retains its generation for internal calls, avoiding a partially updated call graph.
3. Let active frames finish on old bodies; do not rewrite native instruction pointers
   or promise on-stack replacement. Preserve static/application state and do not
   automatically rerun initializers. Suspended continuations retain their generation.
4. Reclaim old images only after frames, continuations, callbacks, callable handles
   and native references can no longer enter them. Initially pin callable handles to
   their creating generation; changing that behavior needs an explicit contract.
   Bound retained versions and report a deferred/rejected update when they cannot
   drain. Atomic pointer replacement alone does not solve reclamation.

For the first contract, reject signature, field layout, base/interface shape, generic
constraint and closure/async state-machine shape changes. Support these later only
through explicit migration or restart. Report an incompatible edit clearly in every
mode. Interpretation simplifies code replacement but does not make these changes safe.

| Proposed deployment profile | Replacement policy | Tradeoff |
| --- | --- | --- |
| Interpreter with reload | Verified IL generations | No native compilation delay, but interpretation overhead remains. |
| JIT with reload | Compile replacement bodies, then publish | Runtime compiler and executable memory required. |
| AOT with reload | Build replacements outside the process and load compatible native modules | No in-process JIT required, but compatible modules, loader permission and ABI/metadata checks are mandatory. |
| Sealed AOT | No runtime code replacement | Permits stronger whole-program optimization and simpler deployment. |
| Explicit hybrid | Opted-in interpreted/JIT replacement over an AOT base | Faster iteration is a hypothesis; larger runtime and mixed-mode correctness surface. |

For reloadable boundaries, initially disallow inlining and devirtualization that bake
in replaceable implementations; calls use generation-aware indirection. Later track
and rebuild dependent callers or add invalidation/deoptimization. Sealed regions can
still optimize aggressively. Indirection, retained versions and reduced optimization
are real costs; hot reload independent of mode does not mean free hot reload in every
artifact. A restart with saved state is useful but is not in-process hot reload.

## POC destination: an AOT-compiled web app

**Author follow-up, 2026-10-07:** an AOT-compiled web app would be an attractive POC,
but start simple. Proposed progression:

1. A scalar native executable and C-callable function, with no managed allocation.
2. AOT Hello World through an explicit UTF-8 console service, then one managed object
   and a collection boundary. Each step must execute without the interpreter/JIT.
3. A minimal web app with one fixed route and a small UTF-8 response, using the
   existing bounded HTTP runtime services. Compile guest request handling to native
   code; identify linked runtime services explicitly. A native launcher that invokes
   interpreted guest handlers does not satisfy this demo.
4. Only after that works, add a small JSON response or application-owned counter.
   Demonstrate AOT hot reload separately by replacing compatible handler code while
   preserving state, once native module lifetime and ABI checks are ready.

The first web demo should reuse the existing accept-loop/handler model, not require
a new WebApplication framework, database, TLS stack or general reflection support.
Inspect the selected consumer's actual reachable IL and services before promising
its scope: networking, task/scheduler behavior, object lifetime and UTF-8 responses
may expose native requirements absent from the scalar probe. Document the supported
subset and fail explicitly for unsupported code instead of hiding an interpreter.

Acceptance for the web demo: build a target-specific ARM64 native artifact, launch
it without a compiler or IL interpreter, send a request from an independent HTTP
client, verify status/body, and shut down cleanly. Record runtime dependencies and
artifact evidence. One successful request is a feasibility demonstration, not proof
of production readiness, scalability or a performance gain. Hot reload is not a
prerequisite for this first web demo.

## Bounded experiments and acceptance

These are proposed tasks, not approved implementation dates. Step 1 now has a
[partial scalar implementation](experiments/aot-scalar/README.md): wrapping arithmetic,
direct acyclic calls, Int32 locals, branches, loops and stack joins. The author-directed
cherry-pick, control-flow and arithmetic-Fault slices are validated on main. Checked
arithmetic and division/remainder now propagate two Fault categories through a
versioned status/out-result ABI. Recursion, general Fault diagnostics and execution
budgets are not yet supported. Later steps remain future work.

1. **ARM64 scalar AOT:** lower a small verified arithmetic/branch/direct-call subset
   to an object and executable; export a scalar function to a C host. Test overflow,
   division faults, recursion and rejected instructions against interpreter results.
   Inspect the artifact for real native bodies and absence of interpreter/JIT fallback.
   Pin compiler, target triple, flags and helper ABI in the evidence.
2. **Small managed consumer:** add UTF-8 output, one allocated object surviving a
   collection, a closed generic and one service call. Exercise a native-to-runtime
   transition and Fault propagation. Require GC root/lifetime correctness before
   trying a library-scale Web API. Reject missing services/roots and ABI mismatch.
3. **Minimal AOT web demo:** follow the progression above after managed/service
   boundaries work. Use an independent client and retain the interpreter consumer
   as a semantic comparison, not a runtime fallback.
4. **AOT reload proof:** preserve a counter in a host-owned session while replacing a
   calculation body with a separately compiled native module. Hold one old invocation
   active, observe new entries using the new generation, and prove old code stays
   alive until safe. Reject a layout change; test failed preparation without partial
   publication, callbacks and a retained generation. No JIT/interpreter fallback.
5. **JIT and interpreter parity:** run the same supported semantic and update cases
   through each backend. Add concurrent publication, cancellation, suspended async
   frames and mixed calls only as those contracts become supported.
6. **Backend/deployment evaluation:** compare equivalent release workloads on recorded
   ARM64 hardware: cold/warm startup, build/rebuild and update latency, throughput,
   executable size, peak memory and retained-version growth. Include an appropriate
   .NET JIT/Native AOT comparison and selected other languages/platforms with matched
   behavior and recorded toolchains. Follow the benchmarking plan below.
   No neoCLR performance advantage is claimed before this evidence exists.

First feasibility evidence can decide Cranelift versus LLVM and helper/ABI shape.
Broader generics, reflection, thread coordination, full debugger support, state
migration and deployment policy remain open. Keeping these explicit is how an
AOT-first experiment can progress without redefining the platform around its subset.

## Later benchmarking against .NET and other platforms

**Author direction, 2026-10-07:** eventually compare with .NET and other
languages/platforms through benchmarks. Benchmarking is a planned evaluation stage;
no results are recorded here. Begin once the relevant native consumer works correctly.
The first AOT proof remains small. Comparing an AOT web app does not require waiting
for neoCLR JIT or hot reload to be implemented.

Use .NET JIT and Native AOT as explicit baselines. Select additional platforms to
answer concrete questions: for example, Go for a compiled managed web service and
Rust for a native implementation with a different memory-management model. These
are candidate comparisons, not a fixed leaderboard or selected framework/toolchain.
Record neoCLR interpreter, AOT and later JIT results separately; never label a mode
as tested before it exists.

Proposed workload progression:

- A small computation with consumed results, checking scalar/call lowering without
  network or console overhead. Avoid drawing whole-platform conclusions from it.
- Process startup to first useful output, then server startup to first valid response.
- A fixed UTF-8 HTTP response, followed by equivalent JSON and allocation workloads
  only when all compared implementations support the same behavior.
- Reload latency and retained memory as a separate capability comparison when
  supported; a process restart and in-process hot reload are different operations.

For web comparisons, match request/response bytes, status, encoding, protocol,
connection reuse, concurrency, payload limits and validation/error behavior. Report
both common-subset runs and any separately labelled idiomatic application runs.
A bounded neoCLR HTTP service and a full framework can differ substantially in
features and costs; document those differences instead of attributing every gap to
the compiler or language. Retain correctness checks during load, including error
counts and response validation.

Measure startup, sustained successful requests per second, p50/p95/p99 latency at
stated offered loads, CPU use, steady and peak resident memory, and complete deployed
artifact size. Separate compiler/build and incremental rebuild time from execution.
Report saturation and failures rather than hiding unsuccessful requests. Separate
JIT warmup/tiering from steady state; distinguish fresh-process startup from OS disk
cache state. For HTTP, record load-generator capacity and placement, and ensure it
is not the bottleneck; use a separate host when measuring network service capacity.

Use the same ARM64 machine/OS and resource limits for a comparison, release builds,
recorded compiler/runtime versions and flags, and equivalent inputs. Record CPU,
core count, power/thermal conditions and dependencies. Repeat independent runs,
vary execution order, and publish variability with sample counts and measurement
windows. Keep raw data, scripts and exact revisions reproducible. Do not extrapolate
one machine's result to all ARM64 targets or collapse startup, throughput and memory
into one claim that a platform is faster.

Initial acceptance is a repeatable, honest baseline that identifies bottlenecks,
not a required win over .NET. Use measured findings to prioritize optimizations;
add focused regression checks only after a metric and meaningful tolerance are
established. Select and verify benchmarking tools when the experiment is scheduled.

## HTTP-driven native reclamation requirement (2026-10-08)

The [RoutePattern lifetime consumer](experiments/aot-console/README.md#route-outcomes-and-sustained-allocation-2026-10-08)
now runs ordinary library routing across repeated requests, retaining a pattern and an
earlier capture. The native invocation arena accumulates 117,507 bytes by 128 requests
and exhausts a 64 KiB budget. The interpreted 16-request entry performs eight pressure
collections while preserving those live references. Object counts and native arena bytes
measure different quantities; no throughput or representation-efficiency claim follows.

This provides the first concrete workload for the existing proposed nonmoving native
tracer. The author's HTTP-driven direction includes GC integration when required;
scoped demonstrations do not constrain the HTTP API to arena lifetimes. Per-request
resets cannot preserve the references retained by this consumer.

The next implementation sequence is provisional and remains within that earlier direction:

1. Define typed allocation descriptors and root categories independently of native lane
   width. Objects need reference-bearing field layouts; inline unions/records and erased
   payloads need their actual discriminants/layouts. Strings/Chars may point into immutable
   image data or allocated text, Object views may carry the private String tag, and reserved
   String arrays expose only initialized slots. Integers that occupy pointer-sized ABI
   lanes must not accidentally become roots.
2. Establish explicit native roots at allocating calls for arguments, initialized locals,
   live evaluation-stack values, pending allocations/results and managed interior borrows.
   Include interface aliases and fault messages; preserve roots on success and fault exits.
   Validate root registration before allowing allocation pressure to reclaim anything.
3. Integrate single-threaded nonmoving tracing and reuse of unreachable storage. Start with
   the existing supported native shapes and reject unsupported root/storage categories.
   Preserve cycle/alias semantics and distinguish genuine live-budget exhaustion from
   cumulative allocation. Write barriers depend on the eventual collector/concurrency
   policy; a nonmoving collector alone does not settle those requirements.
4. Re-run the same fixed-budget workload over increasing request counts, checking retained
   pattern/capture contents, unreachable cycles, interior references, faults, and host roots.
   Then follow remaining HTTP library/service dependencies, including scheduler and network
   resource ownership. Native GC service bindings and stable hosting metadata remain separate
   contracts to validate; linking a collector into the image is compatible with standalone AOT.

This is a test-driven integration sequence, not a claim that descriptors, native safepoints
or tracing have shipped. Reuse the CLR/GC comparisons and sources above; measure collector
costs once collection exists rather than substituting an arena-size comparison for a GC
benchmark.

The first implementation slice now exposes private typed storage recipes through AOT
inspection, including reference-object payloads and argument/local/result layouts.
[Contract and evidence](experiments/aot-console/README.md#typed-native-tracing-layouts-2026-10-08)
cover integer exclusion, nested offsets, cyclic edges, erased String tags, managed borrows
and nominal byte-array views. This completes only the static classification portion of
step 1: dynamic allocation descriptors, initialized/live roots and emitted registration
remain open. It does not change the proposed collector or the existing CLR comparison.

A subsequent slice seeds traceable local storage and erased discriminators at ordinary
native function entry. It retains guest definite-assignment checks and exposes selected
lanes in inspection. This prepares part of step 2 without registering roots or safepoints;
arguments, evaluation stacks, scratch results, initialization through managed borrows and
host/fault roots still need a complete scanning contract. Optimizers may eliminate stores
until root publication makes them observable, so this is not a collector-readiness claim.

Pre-operation planning now uses the checked CFG stack shapes to classify retained stack
values and consumed operands, including erased tag/payload spill lanes. Constructor
receivers and successful results are separate activation phases; native adapter/dispatch
bodies remain explicitly uncovered. See the [contract and evidence](experiments/aot-console/README.md#pre-operation-stack-root-plans-2026-10-08).
At that planning checkpoint, step 2 still emitted no root frames or spills. Conservative stack retention trades simpler coverage for potentially longer
lifetimes; existing CLR comparison and collector performance questions remain unchanged.

An opt-in executable probe now materializes the planned stack lanes and exposes them to
a synchronous read-only linked callback. Native tests validate actual values and routing
checks preserve output/fault behavior. See the [probe contract](experiments/aot-console/README.md#executable-stack-root-probes-2026-10-08).
This closes the observability gap for pre-operation stack spills, while persistent root
frames, ancestor roots, native adapters and result/fault lifetimes remain unimplemented.
It adds diagnostic overhead and does not enable collection or claim CLR-equivalent roots.

Diagnostic frames now persist across ordinary managed calls, carry host-context identity,
and unlink before every completed-IR return. Tests cover caller snapshots, deepest-frame
fault propagation and reentry; the real route host requires an empty chain at return.
[Details and evidence](experiments/aot-console/README.md#diagnostic-frame-lifetimes-2026-10-08)
record the private v2 probe migration. This is a lifetime audit of incomplete root sets,
not activation of the proposed collector. Native wrappers and non-stack root categories
remain outstanding; ancestor scanning adds diagnostic cost.

Diagnostic frames now additionally expose typed addresses of traceable argument/local
lanes. Seeded local roots are safe to observe internally, later writes remain visible,
and discriminator reads avoid uninitialized padding. The adapter never follows borrowed
pointees. [Contract and validation](experiments/aot-console/README.md#argument-and-local-storage-in-diagnostic-frames-2026-10-08)
record the 72-byte frame/enter-v2 migration and added storage cost. Pointee initialization,
owner recovery, result activation, adapter and host/fault roots remain open before tracing.

Constructor storage is now published after initialization and before the constructor call;
successful call/constructor results have a separate diagnostic activation phase. Faulting
calls publish no result roots. Live slot addresses preserve constructor writes observed
from nested callbacks. [Contract and evidence](experiments/aot-console/README.md#constructor-and-call-result-activation-2026-10-08)
record the 104-byte frame/enter-v3 migration. Callee-to-caller result handoff, native adapter
internals, borrowed ownership/initialization and host/fault roots still precede collection.

Ordinary-call handoff now registers seeded caller-owned result lanes before invocation.
The same addresses remain observable while the callee removes its frame; only success
activates the returned-result phase. [Contract and validation](experiments/aot-console/README.md#caller-owned-result-handoff-2026-10-08)
record exact pending/handoff counts and the private transient-v2 migration. This closes
that diagnostic lifetime gap without resolving native adapter internals, borrowed
pointees or host/fault roots, and does not enable collection.


Native wrapper diagnostic frames (2026-10-08) now preserve typed argument copies across
bound service calls and interface dispatch. Early/null/service faults and normal returns
unlink these frames, while guest fault traces continue to omit synthetic wrappers.
[Contract and validation](experiments/aot-console/README.md#native-wrapper-argument-frames-2026-10-08)
cover live service-side observation and the routing consumer. This reuses the CLR root-map
comparison above: explicit argument storage makes boundary lifetime observable at the cost
of extra stack copies/hooks. It is not a complete native transition/handle protocol;
service temporaries, borrowed pointees and host/fault roots still block collection.


Fault-context diagnostic enumeration (2026-10-08) now exposes initialized message and
frame-name slot addresses both during unwinding and after host return. Dynamic user
messages retain the existing v4 arena lifetime; separate contexts are independent and
entry reset retires prior fault slots. [Contract and validation](experiments/aot-console/README.md#fault-context-root-slots-2026-10-08)
include exact interpreter rendering parity. The CLR comparison still distinguishes this
read-only slot view from registered hosting handles and runtime-owned exception objects:
the host continues to own context/arena lifetime, with no reclamation or lifetime extension.


## Basic compiled GC checkpoint (2026-10-08)

The author directs continuation until native compilation has basic GC. That checkpoint
now uses explicit `--native-gc` emission and a statically linked nonmoving collector.
The [contract and comparison](experiments/aot-console/native-gc.md) explain allocation
kinds, conservative object candidates, interior-owner recovery and why services do not
collect internally. This scopes the previously open native-temporary/borrow problem to
complete operation boundaries without claiming general native transition or host handles.
The [real Raven routing consumer](experiments/aot-console/route-native-gc-validation.json)
completes 1,024 requests within a fixed 64 KiB buffer, with retained captures and exact
fault/output parity; the arena-only predecessor failed that budget at 128 requests.
Precise maps, measured pressure scheduling and full HTTP service integration remain next.
