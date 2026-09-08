# Interface inheritance

Implemented in the preview: multiple base interfaces, transitive conformance,
generic substitution in IL, base-interface views and inherited method calls in Neo.
Class inheritance, default interface bodies and variance remain future work.

```swift
interface Readable { readonly func Read() -> int }
interface Counter: Readable { func Add(amount: int) -> () }
```

A record implementing Counter must implement both contracts. A Counter& can be
passed as Readable&; a readonly Counter& can become readonly Readable&, preserving
its access restriction. Neo resolves an inherited method to its declaring interface
and inserts the projection. No explicit dereference is needed.

## Runtime contract

IL uses `.implements Base` inside `.interface Derived`, reusing the existing
TypeDef interface list. Constructed bases substitute the current type arguments
at every edge. Diamond paths to the same constructed interface are deduplicated.
Duplicate direct bases, non-interface bases, cycles (including expanding generic
cycles), and inheritance deeper than 256 active definitions are rejected at loading.
Interfaces still cannot declare storage or layout.

`interface.borrow Base` accepts an existing derived managed or native interface
view, as well as a concrete receiver. An existing view can project only to itself
or a transitive base, not downcast or discover unrelated interfaces through the
hidden concrete receiver. Managed projections retain the same SlotReference and
therefore its complete owner, location identity, readonly permission and lifetime.
They allocate no guest wrapper. Native views retain existing pointer lifetime rules;
this does not make a raw native view a GC root.

`callvirt` still names the actual declaring interface and requires a matching view.
IL authors project before calling an inherited declaration. Neo inserts this step.
An implementer must satisfy all inherited contracts with the existing exact method,
receiver, return and output rules. Conflicting same-name/parameter contracts in an
interface closure are rejected even without an implementing record. Compatible
unrelated declarations may share one concrete implementation; Neo requires a base
projection when their names are ambiguous. Redeclaration/hiding is deliberately not
modeled yet. Neo's existing no-source-overloads limitation still applies.

`Type.GetInterfaces()` now returns all direct and transitive interfaces once,
excluding the queried interface itself. Ordering is not a consumer contract.
GetMethods/GetProperties retain their declared-member behavior: inspect the returned
base interfaces to find their declarations. Closed dispatch analysis traverses the
same substituted closure; it does not silently omit inherited generic targets.

## Comparison and tradeoffs

Primary sources consulted 2026-09-08:

- [C# specification §19.2.4](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/interfaces#1924-base-interfaces)
  specifies multiple bases, transitive conformance, generic substitution and cycle
  rejection. neoCLR adopts these relationships. C# member hiding and explicit
  implementations are broader than this bounded Neo projection.
- [.NET metadata interface implementation rows](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.metadata.typedefinition.getinterfaceimplementations?view=net-10.0)
  provide the familiar metadata relationship. neoCLR keeps its existing interface
  list; it does not claim PE/CLI binary compatibility or copy CLR layout machinery.
- [.NET 10 Type.GetInterfaces](https://learn.microsoft.com/en-us/dotnet/api/system.type.getinterfaces?view=net-10.0)
  supplies the observable reflection baseline of implemented and inherited interfaces.

Flattening inherited methods into new declarations was rejected: it would lose the
identity of the declaring contract. Runtime relationship traversal keeps that identity
and allows IL frontends to share validation. Projection adapts the familiar base
conversion to neoCLR's explicit reference model rather than introducing boxing.
This is a consistency benefit, not a performance claim: closure traversal and
validation add work; caching is a future optimization only if measurements justify it.
The stricter conflict rule is provisional until explicit implementations are designed.

The [comparison probe](experiments/interface-inheritance-dotnet/Program.cs) pins
.NET SDK 10.0.100 and targets net10.0. Run `dotnet run` from
`docs/experiments/interface-inheritance-dotnet`. Observed output is `42`, `True`,
`4`, `3`: inherited dispatch, preserved reference identity, and deduplicated concrete
and interface reflection counts. neoCLR tests exercise those same outcomes plus
its own frame/readonly rules; this probe makes no claim about CLR memory layout.

No allocation, nullable storage or binding-immutability rule changes in this slice.
Existing artifacts continue to load; new inherited-interface metadata requires a
runtime supporting this feature. Reflection callers must accommodate transitive
results. The artifact schema is unchanged.

## Run and validate

From the repository root:

```sh
cargo run --locked -- run examples/source/interface-inheritance.neo --gc-stats
cargo run --locked -- verify examples/source/interface-inheritance.neo
cargo run --locked -- assemble examples/source/interface-inheritance.neo /tmp/interface-inheritance.neo.json
cargo run --locked -- run /tmp/interface-inheritance.neo.json
cargo test --locked --test interface_inheritance
```

The sample returns 42 using frame and heap implementations of a diamond hierarchy,
with a readonly base view. Tests cover source/artifact dispatch, generic call-graph
resolution, reflection, ambiguity, missing/conflicting contracts, cyclic metadata,
GC retention/identity and readonly/current-frame escape checks. Raw IL checks ensure
that skipping verification does not turn a readonly base projection writable.

Class bases now also contribute inherited interface conformance and virtual mappings;
see [class interface dispatch](class-interface-dispatch.md).

[Explicit interface implementations](explicit-interfaces.md) are now available;
default interface bodies remain planned.
