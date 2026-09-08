# Neo classes, construction and default values

Neo supports ordinary `class` declarations with body-declared fields, independently
of positional `record` declarations. Both lower to the existing runtime nominal
record representation. Neither keyword forces a reference addressing mode, allocation
location or Object base. A class value can live in a frame; `new C()` constructs a
managed heap object, and C& provides a managed reference in either case.

```swift
class Counter {
    var Value: int = default(int)
    init(start: int) { this.Value = start }
}
class ZeroCounter {
    var Value: int = default(int)
}
```

Counter has the explicit initializer's signature. ZeroCounter receives a synthesized
parameterless initializer. Neo synthesizes only when no init is declared and every
own field has an initializer; empty classes qualify. Fields without an initializer
require an explicit init that assigns them. There is no implicit zeroing of all fields
and no invalid or null reference manufactured to fill a required reference slot.
Use `default(T)` explicitly where T has a valid runtime default.

This is bounded Neo policy, not new runtime constructor metadata. Another language
can choose a different synthesis policy, but must still emit constructors satisfying
the runtime's field-initialization and reference contracts. The existing runtime
constructor checks reject incomplete construction even without verifier execution,
including an executed branch that leaves a non-nullable reference field uninitialized.
Local/temporary analysis remains chiefly a language/verifier responsibility; this
slice adds no local-slot initialization metadata.

## Initialization and inheritance

Class bodies accept typed `var Name: Type` fields, optional initializers, init,
ordinary/static methods, virtual/override/abstract methods and explicit interface
implementations. Fields are public and mutable in this subset. Let/readonly/static
fields, properties declared in source, generic classes and constructor overloads
remain future work. Records keep their existing positional fields and constructor
behavior; they do not gain body fields in this slice.

A class initializer without an explicit base(...) initializer calls its direct base's
parameterless init, if available. A required base argument must be supplied explicitly.
A positional record base without init does not gain a constructor implicitly. There
is no implicit Object constructor. Abstract classes may have constructors for derived
construction but cannot themselves be instantiated.

The base chain completes first. Own field initializers then execute exactly once in
textual field order, followed by the init body. They can read already initialized
fields, including inherited fields. They execute outside constructor parameter scope;
use the init body for parameter-dependent assignments. Existing runtime restrictions
prevent publishing or invoking ordinary instance methods on an incomplete receiver.
An explicit init can overwrite a field initialized by a field initializer.

Run the [class/base-view example](../examples/source/classes.neo):

```sh
cargo run --locked -- run examples/source/classes.neo
```

It prints `42` twice and returns 42, using both a frame value and a heap reference.

## Default values are not construction or null

`default(T)` lowers to a typed temporary, ldloca, initobj T and ldloc. It uses the
existing runtime default builder; it calls no constructor and runs no field
initializers. For example, `ZeroCounter()` runs its synthesized constructor, while
`default(ZeroCounter)` obtains the typed field defaults directly. A defaultable class
with an initializer of 42 therefore yields 42 through construction and 0 through
default. Constructor-defined domain invariants are not guaranteed by initobj; this
already applies to direct IL and record default initialization.

Supported defaults include numeric zero, false, Void, and concrete records/classes
whose complete field shapes recursively have defaults. Native pointer defaults remain
an IL/interop facility; Neo has no pointer syntax. Abstract or recursive value shapes,
managed references, strings and arrays have no default in the current runtime.
In particular, no empty string or empty array is invented as a substitute for null.
Closed invalid default operations are rejected when loading; open generic default
operations are checked when instantiated/executed. Generic constraints for requiring
a defaultable argument remain future work.

A nullable reference signature could eventually default to an assigned null state.
That does not permit defaulting a non-nullable reference. Nullability belongs in the
runtime type signature, and the runtime must check storage and invocation boundaries.
The [nullability design](nullability.md) remains unimplemented; this syntax does not
introduce null, nullable types, or implicit conversions to nullable storage.

## .NET comparison and responsibility boundary

Primary sources consulted 2026-09-08:

- [C# constructors](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/classes-and-structs/constructors)
  documents initialization order and construction. C# field initializers can run
  before base constructor bodies; Neo intentionally keeps base completion first.
- [C# default values](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/default-values)
  distinguishes zeroed value defaults, null references and explicit constructors.
- [CLR initobj](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.initobj?view=net-10.0)
  does not call constructors. Its null reference behavior cannot be applied unchanged
  to neoCLR's non-nullable reference signatures.

The pinned [SDK 10.0.100/net10.0 comparison](experiments/class-construction-dotnet/Program.cs)
checks C# field/base/body order, a struct's explicit constructor versus default, and
class-reference default null. Run `dotnet run --project Probe.csproj` from that directory.
The corresponding Neo tests intentionally expect base completion before derived
initializers, and reject defaults for non-nullable references.

Adopting C# automatic field zeroing wholesale would create invalid reference values
under the current neoCLR contract. Making references nullable implicitly would violate
the agreed explicit intent model. The current compiler synthesis rule instead relies
on declared initializers and runtime checks. Its benefit is a small source projection
that preserves existing construction safety; its cost is extra initializer syntax
and less automatic initialization than C#. It is provisional compiler policy and can
be expanded once nullable/defaultable signatures and their constraints are designed.

Base-first initialization reuses the runtime's construction capability rules and
allows derived initializers to read initialized base fields. Its cost is observable
ordering differences when porting side-effecting initializers from C#; this is not a
claim of universal superiority or improved performance. The runtime owns validity,
lifetimes, readonly permissions and default semantics. Neo owns syntax, inference,
early diagnostics and constructor synthesis. See [API policy](api-policy.md).

No runtime opcode or artifact format changes are needed. `class` and `default` become
reserved Neo words. Existing records and their aggregate call signatures remain
supported. Tests cover field/default initialization, base views, heap values, ordering,
constructor bypass through defaults, generic defaults and invalid reference defaults.
