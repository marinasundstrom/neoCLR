# Readonly fields and type initialization

Status: interpreter and shared JIT/AOT type initialization implemented; compiler
producer integration remains an implementation gap. Author direction: support both instance and static
readonly fields, keeping .NET semantics as the default model.

## Contract

Readonly fields have storage. They are distinct from literal constants, and a
readonly reference does not make its referenced object immutable. Instance
initializers/constructors may assign the declaring type's readonly storage;
static initializers/type constructors may assign its static readonly storage.
Ordinary calls, imported callers, reflection writes and address escapes must not
bypass those restrictions. Raven's existing init-accessor capability requires
separate validation rather than an inferred widening of arbitrary methods.

.NET's [readonly contract](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/readonly)
permits initialization at declaration or in the appropriate constructor.
[Static initialization](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/classes-and-structs/static-constructors)
runs at most once for a type, with runtime synchronization and a retained failed
state. Field initializers execute in declaration order. These semantics require
more than admitting a readonly modifier in metadata.

## Current layers and next implementation slices

Raven/.NET already emits both forms; the shared ReadonlyFields consumer returns 42.
Native metadata carries instance readonly and static readonly flags, and access
validation restricts instance writes to constructors/init accessors. The interpreter admits stores from a verified owning `.cctor`; ordinary writes
still reject. The metadata SDK authors and imports `.cctor`, preserving initialization timing; Raven still rejects static
field initializers. Shared JIT/AOT lowering executes retained initializers through private guards. This is not yet end-to-end Raven support.

1. Qualify instance constructor initialization, declaration initialization,
   imported reads, defensive copies of mutable value fields, and rejected writes.
2. Add a verified type-initializer identity and metadata SDK emission/import,
   preserving InitOnly and explicit versus before-field-init timing.
3. Give each closed nominal type an initialization state in its existing runtime
   static-storage lifetime. Define reentrant reads, concurrent access, ordering,
   GC roots and terminal-fault retention before enabling initializer writes.
4. Implement the same protocol in interpreter and shared JIT/AOT lowering, then
   enable Raven's existing static initializer bodies and validate external consumers.

The native exception-free failure policy must be explicit: it cannot manufacture
.NET TypeInitializationException. Keep terminal fault propagation and prevent
retry and ordinary access after failure. Recursive initialization may observe default
or partially assigned fields as specified below. Generic types require independent storage/initialization per closed
type. No eager-entry-point initialization or constant substitution establishes the
full readonly contract. These are implementation requirements, not shipped features.

Broader structural work remains parked. This author-directed work does not reorder
the platform roadmap's other milestones.

## Interpreter protocol

A native `.cctor` is a unique private, static, parameterless IL method returning
no result. Its declaring owner and signature are verified; ordinary calls, function
binding and entry-point selection reject it. Only that owner's initializer may
write its readonly statics, including exact generic arguments. Initializers are
runtime roots, not ordinary user-callable methods.

The optional `before_field_init` TypeDef fact defaults to false. Strict initialization
triggers before a static method, an instance constructor, a value-type instance
method or a static-field access. Relaxed initialization currently chooses the first
static-field access. Base/interface initialization is not imposed merely because
a derived initializer runs; actual calls and field accesses establish their triggers.

Storage is default-initialized before entering the initializer. A registry keyed by
resolved closed type records running owner, completion or a retained terminal Fault.
Initializer frames use normal instruction/frame limits and GC roots; triggering
operands remain on the caller frame until completion. Static references retain the
existing invocation-owned root set. Other execution contexts wait outside the graph
and storage locks. Same-context recursion, and cycles in the initializer wait graph,
may see default/partially assigned fields without marking the type complete. Nested
completion releases only wait edges for the completed type. Failure poisons all
still-running initializers belonging to the failing execution context and never retries
them within that invocation.

This follows the CLR distinction between allocation and initialization and its
[deadlock-aware initialization and cached failures](https://github.com/dotnet/runtime/blob/main/src/coreclr/vm/methodtable.cpp).
The [CLI specification](https://ecma-international.org/publications-and-standards/standards/ecma-335/)
I.8.9.5 and II.10.5.3 define trigger timing and recursive initialization. A single
boolean cannot represent in-progress and failed states; a lock held across managed
calls would deadlock recursion and prevent GC/worker progress. This service belongs
in the runtime because it controls managed execution, storage and synchronization.

Two existing platform differences remain explicit: static lifetime is one invocation,
not a CLR load context; failures are the original terminal Fault, not catchable
TypeInitializationException wrappers. They are not promises of permanent divergence.
The registry tests cover ownership, retained failures and multi-context wait cycles;
interpreter consumers cover triggers, closed generics, recursion, metadata round trips
and rejected writes/calls. Native compiler production and threaded managed consumers still need qualification.
Runtime consumers also qualify initializer-created reference pressure in all three modes.


## Native guards and qualification

Closed-world selection retains each selected owner's initializer, and generic
specialization retains the corresponding body for each closed owner. Shared JIT/AOT
code emits private guarded calls at static accesses and strict method/constructor
boundaries. Even a type without fields receives initialization state. Relaxed types
retain their initializer but ordinary static calls do not trigger it.

The current native context is thread-affine and its static storage is invocation-owned.
A private static-storage header word distinguishes unstarted, running, complete and
failed states. Recursive calls observe running state without invoking the body again.
A fault status is cached; the original detailed fault remains in the terminal invocation
context. The header is not guest-addressable and does not change instance layouts,
field offsets, context ABI or the C static-storage allocation service. Native contexts
must not share this table across threads; broader shared native initialization is not
established by this qualification. Interpreter ownership/wait-cycle handling remains
separate from this thread-affine protocol.

Implicit calls publish caller arguments, locals and pending stack operands before
allocation or initializer execution. Entry root plans describe their implicit initializer,
and cctor bodies use ordinary GC-visible frames. The storage table remains a context
root. Internal guard calls bypass ordinary method-access checks only after metadata
verification; guest calls to `.cctor` remain forbidden. Intrinsic primitive/array
initializers and strict initializers on special native replacement bodies are rejected
rather than silently bypassed; this is an admission gap, not a new language rule.

`cargo test -p neoclr-codegen --test type_initialization` exercises the same native
metadata in interpreter, managed JIT and sanitized macOS ARM64 AOT. Cases cover readonly
storage, same-type recursion, two-type cycles, separate closed generic state,
fieldless strict initialization failure, relaxed timing and collection pressure with
both a static reference and a pending caller reference. Failure preserves the caller's
result slot; the host checks that GC roots and invocation storage are released. Existing
native static-storage tests additionally cover repeated entry/reset and arena/GC modes.
No Raven consumer or public SDK `.cctor` API is claimed by these runtime fixtures.


## Metadata SDK authoring and projection

`TypeBuilder.AddTypeInitializer()` creates a private static parameterless no-result
method, distinct from instance construction. Detached `.cctor` declarations use the
same signature/flag validation. `SetBeforeFieldInit(bool)` selects relaxed field-triggered
initialization; strict timing is the default. Native emission uses the canonical owner
name plus `..cctor` and the optional `before_field_init` fact. Native declaration reads
and CLI reference projections retain that fact and the constructor flags instead of
inventing relaxed timing. Reference bodies remain nonexecutable placeholders.

Readonly static stores require the declaring initializer and its exact open generic
owner; an initializer cannot write another closed construction. Direct calls/bindings
and entry selection reject. Interface initializers are not implementation contracts.
Intrinsic primitive/enum initializer authoring remains unsupported. The host SDK APIs
are covered by the [manual metadata reference](../../api-docs/experimental-metadata.md).
`dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- --type-initialization`
checks executable CLR behavior, CLI/native/reference declarations and invalid stores,
methods and entry points. Runtime backend qualification above is unchanged. Raven
source emission and imported-consumer qualification remain the next integration step.

## Guest introspection and reflection follow-through

The host SDK's declaration support is not guest API parity. A source audit on
2026-10-16 found these remaining integration requirements:

- `src/reflection.rs` enumerates only `TypeDef.fields`; static storage and literal
  declarations need distinct descriptor identities/ordinals and binding filters.
- Guest `System.Introspection.FieldInfo` exposes type/access/static facts, but no
  `IsInitOnly`, `IsLiteral` or raw-constant accessor. Follow the corresponding .NET
  FieldInfo concepts while retaining Result for fallible execution.
- Guest constructor discovery recognizes only `..ctor`. Exclude `..cctor` from
  ordinary method discovery and expose a dedicated type-initializer description,
  plus initialization timing. Description must not execute initialization or grant
  ordinary invocation authority.
- `src/reflection_members.rs` executes public instance-field access only. Static
  reads/writes need the same runtime initialization guards as ordinary code; readonly
  writes must remain rejected. Qualify reflective method calls and construction
  against strict initialization in interpreter, JIT and AOT, including failures.

These are tracked gaps, not implemented guest APIs. .NET's load-context lifetime and
exception wrapping remain distinct from neoCLR's current invocation-owned state and
terminal Fault contract. The existing SDK/runtime tests do not establish reflection
execution support for these additions.
