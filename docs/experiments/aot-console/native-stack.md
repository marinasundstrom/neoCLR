# Native stack budget experiment

Development work, 2026-10-08. The Server close/cancellation/callback graph contains
a cycle under conservative dispatch selection. A frame-count limit cannot alone
bound physical stack usage: aggregate/root storage varies, and Cranelift adds spills
and outgoing-call storage. Keeping the acyclic restriction is safe but blocks this
consumer; receiver-flow refinement could remove some false cycles but cannot make
real recursion safe. Heap activations or stack switching require substantially more
runtime machinery and remain separate investigations.

The first slice provides a private macOS ARM64 remaining-stack probe. It obtains the
current pthread's bounds and compares the actual SP, failing closed outside those
bounds. It neither allocates nor collects. Return 9 is the experimental
StackOverflow status, with the existing standardized message “Call stack limit exceeded”.
No TLS bound cache is used. Cost is not measured yet; no performance claim follows.

`--native-stack-budget` requires `--native-gc` and enables guarded recursive CIL.
The compiler checks host entries before calling guest code and every guest function
after publishing its frame. It caps final machine frames plus conservative outgoing
argument storage at 64 KiB and leaves 256 KiB at each passing check.
That reserve covers one maximum guest frame plus 192 KiB for matched C adapters and
fault unwind. The initial helper call itself requires an ordinary usable host stack.
This is not protection for arbitrary foreign code, signal handlers, host reentry or
manually switched stacks. The default profile still rejects recursive graphs. Opted-in images must link the
matching native-stack.c helper. Function/queue signatures and public metadata do not
change. Stack faults preserve caller result storage and unlink published GC frames.
An entry guard failure has no guest frames; an in-function guard records instruction
zero and normal callers append their call sites, with the existing 64-frame truncation.

A future green-thread activation must supply its own stack bounds or use a different
execution model. Do not promote pthread ownership into the Function ABI, continuation
identity, or logical fault trace. See the [runtime review](../../runtime-scheduling-design.md#cross-runtime-reassessment--author-direction-2026-10-08).

## Comparison and evidence

.NET 10's [EnsureSufficientExecutionStack](https://learn.microsoft.com/en-us/dotnet/api/system.runtime.compilerservices.runtimehelpers.ensuresufficientexecutionstack?view=net-10.0)
checks an artificial limit preserving room for exception handling. It raises
InsufficientExecutionStackException rather than promising a particular recursion
depth. The [NativeAOT implementation](https://source.dot.net/system.private.corelib/System/Runtime/CompilerServices/RuntimeHelpers.NativeAot.cs.html)
uses current-thread bounds and caches a sufficient-stack threshold. These are runtime
implementation choices, not C# callable semantics. Sources consulted 2026-10-08;
source.dot.net is a moving source view, not a pinned implementation dependency.
Apple's [pthread stack declarations](https://github.com/apple-oss-distributions/libpthread/blob/main/include/pthread/stack_np.h)
identify the native bound APIs. The local macOS SDK supplies the tested declarations.

neoCLR already has a standardized StackOverflow fault for interpreter frame limits.
The native adapter reuses that semantic code/message while allowing a
platform-dependent depth. Its costs are a check per guest function and reserved stack
space; neither identical native/interpreter depth nor superiority over .NET is claimed.
A later optimization must retain guard coverage and verify actual machine frames.

The focused sanitized C test passes at -O0 and -O2: main-thread admission, recursive
16 KiB frames on a 512 KiB worker stopping before exhaustion, successful unwind and
reuse, and immediate rejection on a 128 KiB worker. This validates the helper boundary,
not compiled recursion, fault traces, GC cleanup or full HTTP execution.

## Guarded compiler validation

The compiled consumer passes direct and Function-dispatched recursion on a 512 KiB
worker, finite calls before/after a fault, standardized native/interpreter fault
code/message parity (not equal recursion depth), bounded rendered traces and frame
unlinking. Host callback recursion retains its strong handle through GC until explicit
release. A quiescent callback near the physical threshold and entry on a 128 KiB worker
both fail before calling guest code. Caller outputs and heap canaries remain unchanged.
The original recursion rejection remains tested without the flag; flag validation and
seven root-layout/publication tests pass.

Cranelift 0.121.2's `CompiledCode::frame_size` includes storage, spills and clobbered
registers but excludes linkage and ephemeral arguments (`machinst/abi.rs::frame_size`).
The admission check adds 16 bytes for FP/LR and a conservatively aligned eight bytes
for every parameter in the largest imported call signature, including register arguments.
This backend emits fixed I32/I64 arguments and no tail calls/dynamic stack allocations.
Reassess this calculation on compiler/ABI changes; do not use the raw frame_size alone.

The matched C sources have no recursive guest-service call path before a guard;
collector and published-frame traversal use loops. Clang stack-usage output for the
sanitized adapters reports no dynamic frames: the sum of all static function frames
is 41,904 bytes at -O0 and 13,856 at -O2. This deliberately overcounts an acyclic helper
chain and fits the 192 KiB helper reserve. System-library internals/sanitizer runtimes
are not covered by that static inventory; this is tested macOS POC evidence, not a
portable production proof. Reaudit replacement adapters and supported host/toolchains.
[Recorded checks and source hashes](../../../benchmarks/native-web/stack-budget-validation.json).

Full Server inspection with this flag gets past recursion and now rejects its no-result
entry signature: the current native entry requires Int32. Entry adaptation and queue
pumping remain before native HTTP execution. No server throughput is measured yet.
