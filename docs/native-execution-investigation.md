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
