# Readonly fields and type initialization

Status: interpreter type initialization implemented; native codegen and compiler
producer integration remain implementation gaps. Author direction: support both instance and static
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
still reject. The metadata SDK still rejects `.cctor`, and Raven still rejects static
field initializers. Shared JIT/AOT admission explicitly rejects initializer-bearing
modules until its guards are implemented. This is not yet end-to-end Raven support.

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
and rejected writes/calls. Native compiler production, JIT/AOT execution, threaded
managed consumers and initializer-created reference pressure still need qualification.
