# Inherited value layout: the first class-inheritance slice

Implemented as preliminary groundwork. This adds a single explicit record base,
inherited fields and BaseType reflection. [Managed base-reference views](base-views.md) are now implemented in a subsequent
slice. [Inherited method dispatch, virtual overrides and abstract classes](class-dispatch.md)
are also implemented; [constructor chaining](constructor-chaining.md) is now implemented.
Those remain the next object-model slice; this is not complete class inheritance.

## Metadata, storage and Neo

IL declares `.extends Base` inside `.type Derived`. There is no implicit Object
base. Interfaces continue to use `.implements` for interface inheritance; `.extends`
is rejected on interfaces and runtime primitives. Generic bases substitute their
arguments along the whole chain. Cycles, including expanding generic cycles, are
rejected; the preview limits a chain to 64 record definitions. Field hiding is
rejected until declaring-owner field selection has a fuller source projection.

A derived value stores inherited fields first, from the oldest ancestor, followed
by its own declared fields. These are logical managed field indices, not a promise
of native byte offsets. Metadata retains each field on its original declaring type.
Private/internal field access is checked against that declaring type, including
access from a derived type. Aggregate construction requires access to every field.

Neo supports a source record base before any implemented interfaces:

```swift
record Position(X: int)
record LabelledPosition(Label: string): Position
var point = LabelledPosition(40, "local")
point.X = 42
let shared = new LabelledPosition(42, "heap")
```

The aggregate argument order is inherited fields followed by declared fields. Neo
retains its small compiler model; this is not syntax for invoking a base constructor.
`newobj Derived` consumes the complete ordered field set and constructs a complete
value. Existing `initobj Derived` recursively defaults that whole field set where
all field types support default initialization. Ordinary copying preserves Derived
and all its fields. Managed allocation and references keep their existing meaning.
The GC scans inherited reference fields as part of the complete stored value.

A type used as a base may now declare managed-reference instance methods, including
virtual and abstract contracts. Base value-receiver constructors/methods and
inherited interface implementations remain restricted; see [class dispatch](class-dispatch.md).
Derived types may declare their own methods and interfaces. Base types may declare
static helpers. Native layout/interop for derived records is rejected pending an
explicit inherited-layout ABI. There is no implicit value slicing. [Base-reference projection](base-views.md) now
allows Derived& to be used as Base& while preserving the complete owner.

## Library API

`System.Type.BaseType` returns `Option<Type>`: Some contains the immediate,
substituted record base; None means no record base. Interfaces, primitives, arrays,
and reference signatures currently have no implicit Object or Array ancestry.
GetFields and its DefinitionIndex remain declared-only, with the index relative to
the declaring definition; follow BaseType to inspect ancestor declarations. Base
metadata round-trips through artifacts and is checked across module boundaries.

## Comparison and choices

Primary sources consulted 2026-09-08:

- [C# class specification](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/classes)
  establishes the familiar single-base relationship and an instance containing base
  and derived fields. neoCLR reuses that relationship, but separates inheritance
  from allocation/addressing mode and does not require Object ancestry.
- [.NET 10 Type.BaseType](https://learn.microsoft.com/en-us/dotnet/api/system.type.basetype?view=net-10.0)
  reports the immediate substituted base. neoCLR keeps the property name and uses
  Option for absence rather than nullable Type, consistent with its library policy.
- [C# constructor behavior](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/classes-and-structs/using-constructors)
  includes base constructor calls. This aggregate slice does not reproduce that
  behavior; load-time restrictions prevent skipping user-defined base construction.

Embedding a Base object as a synthetic field would make a base view resemble an
interior field address, changing identity and field paths. The chosen flat logical
layout preserves one complete derived value and its declared field ownership.
It adds ancestry traversal and validation cost; no performance improvement is claimed.
Native layout is deliberately unsettled instead of inferring a C++ or CLR ABI.

The [comparison probe](experiments/inherited-layout-dotnet/Program.cs) targets
net10.0 with SDK 10.0.100. Run `dotnet run` from
`docs/experiments/inherited-layout-dotnet`. Observed output: `42`, `True`, `1`,
`True`, followed by `True`, `True` for base-view identity and dynamic type.
The first four results cover inherited fields, substituted BaseType, declared-only field
reflection and .NET's implicit Object root. neoCLR deliberately differs on that
last point: a root record's BaseType is None.

The [base-view slice](base-views.md) now separates the reference's view type from the complete
stored type. Specify field access, whole-value reads/writes, copy/slicing rules,
reference identity/GetType, GC/pinning, constructor publication and virtual receiver
selection together. A base view must not replace a derived allocation with a Base
value or accidentally discard derived GC edges. Virtual dispatch builds on this distinction and remains outside this layout slice.

## Run and validation

From the repository root:

```sh
cargo run --locked -- run examples/source/inherited-layout.neo --gc-stats
cargo run --locked -- verify examples/source/inherited-layout.neo
cargo run --locked -- assemble examples/source/inherited-layout.neo /tmp/inherited-layout.neo.json
cargo run --locked -- run /tmp/inherited-layout.neo.json
cargo test --locked --test inherited_layout
```

The sample returns 42. Tests cover artifact round-tripping, substituted multilevel
fields, default initialization, copied values, inherited GC references, private-field
access and rejection of unsupported/cyclic metadata, native layout and slicing.
New artifacts with base metadata require this runtime revision. Existing artifacts
without it retain their former behavior; no artifact format version is changed.
Rust callers constructing TypeDef literals must supply `base: None` (or an explicit
base); the new optional metadata field is a source API addition.
