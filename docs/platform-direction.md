# neoCLR direction: familiar foundations, selective changes

Direction clarified 2026-09-13. neoCLR is an independent managed runtime, with Raven
as the current frontend experiment. Preserve useful .NET behavior and investigate
specific improvements without requiring developers to relearn ordinary programming.
Reviews include other platforms, .NET API discussions and alternative .NET libraries,
following the [broader research scope](design-research.md#broader-api-review-scope-2026-09-13).
Starting without legacy constraints creates options; it does not establish that a
replacement is simpler, safer or faster. Small reversible prototypes can gather
evidence immediately; major platform contracts require grounded comparisons before
adoption. Keeping current behavior is always an alternative.

## Foundations to retain

Keep value and reference type categories, ordinary class and array references,
managed garbage collection, generics, interfaces, virtual dispatch and familiar
construction and call behavior. Value semantics do not imply a short lifetime or
stack-only storage: values can be embedded in heap objects. Managed byrefs provide
explicit access to storage; native pointers remain a separate low-level facility.
Reopening the value/reference split is not a current priority.

Prefer existing CLI metadata and instruction semantics at the compiler boundary.
Compatibility means a useful target for existing compilers, not execution of arbitrary
unchanged .NET assemblies. Runtime mechanisms are primary; Raven should project them
with small target mappings. Leave the historical Neo compiler outside this migration.

## Differences with a purpose

| Area | Direction and benefit to evaluate | Costs and limits |
| --- | --- | --- |
| Errors and absence | Continue Result for recoverable failures and Option for optional outcomes; generic Void represents no-payload completion | API migration and compiler adaptation; terminal faults remain distinct from recovery, without a guest Exception hierarchy |
| Mutable arrays | Keep invariant element types; reject unsafe widening early and observe real-program ergonomics | Deliberate CLR covariance incompatibility; a future covariant read-only view needs its own contract |
| Text | Build modern operations around valid UTF-8 and explicit text units | UTF-16 interoperability and offset conversions remain necessary; existing familiar contracts must not silently change |
| Spans and managed references | Investigate bounded memory views and consistent lifetime/access guarantees across languages | Metadata, verification, GC, aliasing and eventual JIT costs must be demonstrated; no new span contract is selected |
| Callables | Revisit function types versus nominal delegates while retaining current callbacks | Closure lifetimes, type identity, variance, events and compiler mapping remain open |
| Nullability | Revisit whether metadata can express enforceable nullable/non-nullable uses consistently | Construction, defaults, arrays, generics, reflection and compiler compatibility make this a cross-cutting change; encoding remains open |

The [array contract](array-variance.md), [text model](text-model.md) and
[nullability review](nullability.md#review-reopened-2026-09-13) distinguish delivered
behavior from candidates. Implementing a missing CLR feature is compatibility work,
not evidence that neoCLR has improved the CLR.

## Bounded memory views: investigation brief

Start with a buffer-processing function that accepts a slice, reads or mutates its
elements, and allocates no copy. Compare a managed array-backed view with a view of
frame storage. A return into a caller-owned buffer differs from returning a view of
the callee's dead frame. A read-only view prevents writes through that view, not writes
through other aliases or mutation of objects stored in the buffer.

.NET already combines runtime support and language restrictions here. Its managed
byrefs and byref-like types are not just library conventions. C# supports ref fields
and restricts escaping/capture; modern C# also supports constrained byref-like generic
use. Span and Memory serve different lifetime needs. We should examine those reasons
before proposing to unify their APIs or remove restrictions.

Compare three approaches: adopt the existing Span/Memory split; retain those public
roles over more uniform neoCLR internals; or introduce a new bounded-view contract.
The second is a useful experiment, not an accepted solution. Start with existing
signatures and IL; propose extra metadata or instructions only for a demonstrated gap.

Record which facts the compiler proves, the loader/verifier enforces, and execution
must check. Test bounds, empty/end slices, dead-frame escape through returns, fields
and delegates, GC retention, writable/readonly aliases, native pinning and disposal.
Before supporting suspension, decide which backing storage may survive it. Do not
infer moving-GC safety or native pointer stability from the current interpreter.
Measure allocation and access costs before making optimization claims.

## API scope and ergonomics

The [runtime API plan](runtime-api-plan.md) identifies expected API families and
scenario-driven additions. Preserve the ergonomics of ordinary C# and other
.NET-language code; novel internals should not require manual reference handling.
The [callable review](delegates.md#function-type-review-reopened-2026-09-13) is open,
not a decision to replace delegates.

## Order of work

The [runtime API priorities](runtime-api-plan.md#immediate-implementation-priorities-2026-09-13)
now put System.Array<T>, minimal collection contracts and basic implementations first.
Build a usable prototype foundation before expanding text/encoding and clock APIs.
Continue maintaining the Raven/runtime demonstration and reproducible tools throughout.

Bounded memory views, nullability metadata, callable alternatives and runtime async
remain research tracks; they need not block ordinary collection APIs. Prototype them
when a concrete question can be answered without treating the experiment as a settled
platform contract. Debugger integration and other recorded release requirements remain
on the roadmap. This is an implementation order, not a release-date promise.

## Initial primary evidence

Consulted 2026-09-13:

- [C# ref structs](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/ref-struct): language-level escape restrictions, ref fields and version-labelled generic/interface support.
- [.NET memory and span types](https://learn.microsoft.com/en-us/dotnet/standard/memory-and-spans/): distinct memory-view APIs. Read-only access must not be mistaken for immutable backing storage.
- [Runtime ECMA-335 augments](https://github.com/dotnet/runtime/blob/main/docs/design/specs/Ecma-335-Augments.md): runtime managed-pointer and byref-like generic contracts beyond the base CLI specification. This moving design document is orientation, not evidence that every described feature shipped in a particular runtime.

Pin a .NET runtime revision and compiler version before the implementation comparison;
then run equivalent positive and negative examples. Follow the
[design research process](design-research.md), including frontend, metadata, runtime
and library responsibilities separately.

## Async and time: preserve the application model

The author wants runtime support for async/await, with the representation still open;
Task-based APIs remain relevant. Treat the result of an asynchronous operation and
the mechanism for suspending a frame as separate contracts. A Task<T>-shaped public
API could coexist with runtime-owned suspension, or with a compatible compiler
lowering. Compare both before considering a different public completion abstraction.

A first design probe should complete synchronously and suspend/resume once. Specify
GC roots and retained frames, cancellation, cleanup, scheduling, repeated awaiting,
debugger stacks, and references crossing suspension. Evaluate Task<Void> and
Task<Result<T,E>> as candidates; distinguish a recoverable Error result, cancellation
and a terminal runtime fault. Do not assume .NET task exception aggregation fits the
neoCLR fault model. Async does not itself require a new thread or task-per-thread model.
No async representation, opcode or implementation is selected by this discussion.

For time, extend the existing Date/Time values with an independently supplied clock
that application code can instantiate/substitute in tests. The
[date/time follow-up](date-time-design.md#injectable-clock-follow-up-2026-09-13)
compares a narrow clock with a broader provider. Modern .NET already supplies a
mockable TimeProvider; the opportunity is a coherent initial API and error model,
not claiming that dependency-injected time is new.

Primary baselines consulted 2026-09-13:
[.NET Task-based asynchronous pattern](https://learn.microsoft.com/en-us/dotnet/standard/asynchronous-programming-patterns/task-based-asynchronous-pattern-tap)
and [TimeProvider](https://learn.microsoft.com/en-us/dotnet/standard/datetime/timeprovider-overview).
These establish public API precedents, not a chosen runtime suspension implementation.
