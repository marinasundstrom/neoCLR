# Readonly fields and type initialization

Status: existing instance support under qualification; native static initialization
is an implementation gap. Author direction: support both instance and static
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
validation restricts instance writes to constructors/init accessors. All native
static readonly writes currently reject; native callable authoring rejects .cctor
and Raven rejects static field initializers. Default-initialized static storage
alone does not complete the requested feature.

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
retry/partial publication; choose the exact retained fault contract with the runtime
implementation. Generic types require independent storage/initialization per closed
type. No eager-entry-point initialization or constant substitution establishes the
full readonly contract. These are implementation requirements, not shipped features.

Broader structural work remains parked. This author-directed work does not reorder
the platform roadmap's other milestones.
