# Lazy compilation in other runtimes — 2026-10-26

Author-requested comparison supporting the [lazy-JIT investigation](validation/lazy-jit-optimization-20261026/README.md).
Primary documentation and implementation sources were inspected on this date.
Current branch documentation is not a promise about every released configuration;
historical articles below are explicitly dated. These are architectural comparisons,
not cross-runtime throughput benchmarks.

## How work reaches executable code

| Runtime | Initial execution | Later compilation and entry management |
| --- | --- | --- |
| CoreCLR | A prestub resolves an unprepared method; quick JIT or available ReadyToRun code supplies an initial implementation. | Tiered compilation produces optimized code in the background. Entry/version management redirects future execution. |
| HotSpot | Execution normally begins in the interpreter. | C1/C2 tiers use invocation/backedge information and compilation queues; existing execution can continue while a new version is compiled. |
| V8 | Ignition bytecode supports initial execution; Sparkplug supplies a fast baseline native tier. | Higher tiers optimize in background jobs; installation is coordinated with the main thread. Lazy frontend work and cached code address other startup costs. |
| LLVM ORC (JIT infrastructure) | A lazy reexport creates a stable entry stub without immediately materializing its body. | First use triggers materialization and updates the stub target. Concurrent dispatch is configurable; ORC is not itself a managed GC or an interpreter. |

CoreCLR's [current prestub implementation](https://github.com/dotnet/runtime/blob/main/src/coreclr/vm/prestub.cpp)
protects the managed transition and enters preemptive GC mode around preparation.
The [compilation configuration documentation](https://learn.microsoft.com/en-us/dotnet/core/runtime-config/compilation)
describes quick initial code versus background optimization and the startup/quality
tradeoff. ReadyToRun is a separate source of initial code, not evidence that every
method is JIT-compiled at launch. The older [code-versioning design](https://github.com/dotnet/runtime/blob/main/docs/design/features/code-versioning.md)
explains multiple implementations of a logical method; it is design context,
not a substitute for checking current source.

The [Java 25 command specification](https://docs.oracle.com/en/java/javase/25/docs/specs/man/java.html)
explicitly contrasts normal background compilation with `-Xbatch`. That flag does
not mean that all unused methods are compiled at program startup. HotSpot's
[compilation policy](https://github.com/openjdk/jdk/blob/master/src/hotspot/share/compiler/compilationPolicy.hpp)
uses compiler-queue load to adjust transitions and thresholds; its comments describe
prioritizing active methods and removing stale requests. This provides a concrete
scheduling precedent: compilation demand should be bounded and responsive to load.

V8's [Sparkplug design article (2021)](https://v8.dev/blog/sparkplug) describes a
quick compiler that emits machine code from bytecode without a conventional
optimizing IR, retaining interpreter-compatible frames. This trades optimization
opportunities for fast compilation and easier stack/profiler/OSR integration.
The [lazy-parsing article (2019)](https://v8.dev/blog/preparser) separately explains
deferring frontend work for functions that may never run. It supports distinguishing
lazy lowering from merely postponing machine-code generation.

ORC's [lazy-reexport contract](https://llvm.org/docs/ORCv2.html#laziness) preserves a
canonical stub address and updates its destination after first-use materialization.
Taking that address need not compile the body. Publication depends on materialization
and dependency readiness. Its [AST-lazy tutorial](https://llvm.org/docs/tutorial/BuildingAJIT4.html)
goes further than delaying LLVM code generation: custom materialization can retain
an AST until needed. These are useful precedents for neoCLR's deferred method lowering
and stable method identity. They do not supply neoCLR's root or heap safety contract.

## GC and compiler-thread coordination

CoreCLR's [threading documentation](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/threading.md)
distinguishes managed cooperative execution from preemptive runtime work and specifies
coordination when returning to managed execution. neoCLR does not currently implement
that moving-GC transition system: its compiler receives no managed references, and
the blocked caller retains published roots under a nonmoving, single-mutator contract.

V8's [background-compilation documentation](https://chromium.googlesource.com/v8/v8/+/HEAD/docs/compiler/concurrency-and-background-compilation.md)
describes local heap/isolate views, brokered object references, safepoints and parked
compiler work that promises not to access the heap. It also separates background
optimization from main-thread installation. The lesson for neoCLR is specific:
future profiling or specialization must not casually let its compiler inspect mutable
managed objects. Compiler isolation today avoids that problem; it is not equivalent
to V8's richer concurrent heap-access protocol.

None of these sources establishes that concurrent GC makes lazy compilation faster.
Compilation, marking and application execution compete for CPU and memory bandwidth.
Whether overlap helps depends on live data, allocation, spare cores and I/O slack.
Our GC comparison therefore retains CPU, wall time, first-use stalls, scheduling
phases and identical placement controls, rather than treating thread count as a gain.

## Implications for neoCLR

These are analysis and candidate directions, not implemented commitments:

1. **A compiler thread is not an execution fallback.** The current caller waits for
   full native compilation. An interpreter or quick tier could let that same work
   progress sooner; merely adding compiler threads cannot do so. Integrating the
   existing interpreter with native execution requires explicit argument, heap,
   exception, suspension and root-map transitions. It is not a configuration toggle.
2. **Separate cold and warm entry costs.** A stable address need not imply a permanent
   call/return wrapper frame. Evaluate a frameless branch stub or a suitable internal
   tail-call ABI, preserving cold-call arguments, stack bounds and atomic publication.
   Opportunistically bypassing already-compiled callees is a smaller experiment; it
   does not remove trampolines from all warm calls or establish a replacement policy.
3. **Keep background work demand-driven.** Prioritize needed/hot methods and cap
   work under load. Compiling every selected method in the background would recreate
   the author's unwanted work, even if moved off the startup thread.
4. **Treat scheduling as a root-lifetime contract.** Yielding a guest fiber during
   compilation could let other ready work run. It requires safe suspended arguments,
   reentrancy, once-only publication, errors/cancellation and teardown; the current
   synchronous resolver does not implement it. A callback pump inside the resolver
   without those contracts would be an unsafe shortcut.
5. **Measure frontend and code-placement costs separately.** Metadata selection is
   still eager; lazy parsing/IR precedents do not show that neoCLR's verifier can simply
   be skipped. Arena locality is a shared JIT improvement, so eager arena controls are
   required. Persistent code caches would additionally need ABI/input invalidation,
   relocations and matching GC metadata; raw address-bearing code is not a disk cache.
6. **Do not recover whole-program GC proofs from a partial program.** Lazy mode's
   conservative checks remain until local ABI contracts or sound summaries justify
   their removal. A tiering or interpreter design must preserve this safety boundary.

The author permits breaking changes. If measurements favor a different internal ABI
or execution transition, compatibility alone is not a reason to reject it. Describe
its migration and safety consequences, then qualify both collectors and real HTTP/I/O
consumers. This permission is not evidence that such a redesign already works.

## Local startup boundary to profile

`runtime/codegen/src/linking.rs::prepare` constructs a `LoadedProgram` and verifies
original module scopes before canonical projection. This includes method bodies
beyond the eventual executed set. Its source comment identifies access, signatures,
field access, definite initialization and borrows. Removing that call would weaken
the contract; deferring body verification would instead need original-scope evidence
and validation before a transformed method becomes executable. This observation
identifies a profiling target, not an implemented lazy-verifier change.

## Load, heavy handlers and shared resources

The author's follow-up asks which combination handles each workload best, including
multiple web apps and complex handlers. Runtime labels alone do not answer that.
.NET distinguishes **workstation/server GC** from **non-concurrent/background GC**;
these are separate choices. Its documentation explicitly describes interference
when many server-GC processes share too few CPUs. This supports measuring the
aggregate host and each application, not assuming additional GC threads always
improve throughput. neoCLR's experimental collector is not CLR server GC.
[Workstation/server GC](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/workstation-server-gc).

.NET background collection also retains suspension points: its generation-2 work
can overlap execution, while foreground collections of younger generations still
suspend managed threads. “Concurrent” therefore does not mean “no blocking,” and
neoCLR's different nonmoving heap should be assessed through its own pause, CPU
and offered-load measurements rather than importing CLR tuning expectations.
[Background GC](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/background-gc).

Node's scheduling guidance provides a related precedent for heavy handlers:
a long callback can delay other clients even with asynchronous I/O. It discusses
partitioning work and offloading appropriate operations while accounting for worker
cost and fairness. Our inference for neoCLR is that reducing compilation stalls
alone will not solve long managed compute sections. Safe yielding, an explicit
worker/heap ownership model, or process isolation address different bottlenecks;
they must be measured separately. Existing bounded handler tasks do not establish
parallel managed execution. [Node event-loop and worker-pool guidance](https://nodejs.org/learn/asynchronous-work/dont-block-the-event-loop).

The current experiments use isolated processes sharing a host, not multiple tenants
inside one managed heap or enforced per-app CPU/memory quotas. Fixed offered arrivals,
misses, per-app tails, cold versus warm execution and uneven traffic shares make
that boundary explicit. A best tested rate under a provisional latency target is
not an estimate of untested production capacity.

## Concrete scheduler follow-up boundary

Source review of `jit_lazy.rs::resolve` finds a zero-capacity response channel and
a blocking receive. The native fiber scheduler already parks published root chains
under their owning fiber, but its wait/completion path is tied to owned asynchronous
operations. A proposed compiler wait must participate in that ownership protocol;
calling arbitrary ready callbacks inside the resolver would not establish it.

The follow-up needs a completion object that remains valid if a waiting call is
cancelled or the host begins shutdown. Publication must precede waking the fiber,
and a compiler must not become stuck delivering a result to a parked or cancelled
consumer. Multiple cold callers need deduplication and bounded admission, with
recursive calls, compilation failure, late completion and teardown tests. Preserve
the nonmoving-heap/root contract and compiler isolation from managed references.
The shared native event loop's polling policy also needs attribution: replacing a
blocking receive with repeated polling is not automatically lower-latency scheduling.

Measure an already warm route while another route first compiles, alongside the
current all-cold case. This distinguishes useful independent application progress
from simply moving the same wait elsewhere. A fast initial execution tier addresses
the cold call's own progress; scheduler suspension addresses unrelated ready work.
Both remain proposals, and neither entails compiling every unused method.
