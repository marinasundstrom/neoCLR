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
bounds. It neither allocates nor collects. Return 9 is reserved for the experimental
StackOverflow status; this first slice does not yet emit it from compiled CIL.
No TLS bound cache is used. Cost is not measured yet; no performance claim follows.

The proposed compiler integration must check all host entries before calling guest
code and every guest function after publishing its frame, cap **final machine**
frames at 64 KiB (not just explicit slots), and leave 256 KiB at each passing check.
That reserve covers one maximum guest frame plus 192 KiB for matched C adapters and
fault unwind. The initial helper call itself requires an ordinary usable host stack.
This is not protection for arbitrary foreign code, signal handlers, host reentry or
manually switched stacks. A compiler guard and a matched-adapter audit are required
before admitting recursion; the helper alone does not weaken admission.

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
The proposed native adapter reuses that semantic code/message while allowing a
platform-dependent depth. Its costs are a check per guest function and reserved stack
space; neither identical native/interpreter depth nor superiority over .NET is claimed.
A later optimization must retain guard coverage and verify actual machine frames.

The focused sanitized C test passes at -O0 and -O2: main-thread admission, recursive
16 KiB frames on a 512 KiB worker stopping before exhaustion, successful unwind and
reuse, and immediate rejection on a 128 KiB worker. This validates the helper boundary,
not compiled recursion, fault traces, GC cleanup or full HTTP execution.
