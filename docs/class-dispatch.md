# Inherited methods, virtual dispatch and abstract classes

Implemented as the next [base-view](base-views.md) slice. Methods operate on managed
reference views of complete concrete values. Inheritance does not force heap
allocation or permit implicit base-value slicing.

## Neo and IL

```swift
abstract record Counter(Value: int) {
    readonly abstract func Read() -> int
    func Add(amount: int) -> () { this.Value = this.Value + amount }
}
record OffsetCounter(Offset: int): Counter {
    readonly override func Read() -> int { return this.Value + this.Offset }
}
```

Neo currently expresses class definitions using its existing record syntax;
`abstract record` declares a non-instantiable base. `abstract func` is bodyless and
implicitly virtual. `virtual func` provides an initial implementation, and
`override func` replaces an inherited virtual implementation. The supported modifier
order is readonly, abstract, then virtual/override. An abstract intermediate can
reabstract a virtual member with `abstract override func`.

IL uses `.type abstract Counter`, `.method instance virtual byref Read()`,
`.method instance override byref Read()` and `.method instance abstract byref Read()`.
Readonly follows the virtual/override modifier and precedes byref. Metadata stores
virtual, override and abstract flags on methods, and an abstract flag on types.
There is no mandatory Object base or separate reference-type classification.

## Dispatch and validation

Neo resolves inherited method names to their declaring owner and inserts the
appropriate base-reference projection. A nonvirtual inherited call uses `call`.
Virtual and abstract calls use `callvirt`. Both require a reference receiver; Neo
can take the address of a local value for ordinary member syntax.

Class callvirt names the declaring class and exact signature. The runtime selects
the nearest implementation in the complete stored type's ancestry, then creates an
internal receiver view typed to that implementation's owner. That internal projection
is not an exposed downcast facility. It preserves identity, readonly permission,
GC ownership and frame lifetime. Overrides can access their own derived fields,
mutate eligible storage and return derived field references under existing lifetime
rules. `call` to a concrete virtual declaration deliberately invokes that declared
body, without dynamic selection. A direct call to an abstract declaration is invalid.

Concrete virtual methods currently require public IL bodies and managed byref receivers.
Override matching preserves parameter and return types, readonly receiver and
parameter contracts, and unconditional/conditional output contracts. Covariant returns,
method hiding/newslot shadowing, sealed overrides and explicit override mappings are
not implemented. Nonvirtual inherited methods may use existing visibility rules;
private members do not become accessible to derived callers.

Abstract records may contain fields and concrete byref methods. Every nonabstract
record must resolve every inherited abstract declaration to a concrete override;
abstract intermediates may defer implementation. An abstract method must have no
body or locals and belong to an abstract record. Abstract values cannot be created
by aggregate construction, constructor allocation, initobj, defaulted array elements,
native layout or owned host input. Derived aggregate construction still initializes
all inherited fields. An abstract type without abstract methods is also non-instantiable.

Managed base constructors now support [constructor chaining](constructor-chaining.md)
and publication checks. Inherited value-receiver methods remain restricted. Inheriting implemented interfaces
through a class base is also still restricted; existing interface inheritance works
independently. No default interface method bodies are added in this slice.

## Inspection and analysis

Type.IsAbstract and MethodInfo.IsVirtual/IsAbstract expose the familiar predicates;
interface types and their bodyless contracts report abstract, and interface contracts
report virtual. MethodInfo.IsOverride is an explicit neoCLR convenience for the new
reuse contract, rather than a claim of an identically named .NET property.
GetMethods remains declared-only; inspect BaseType for ancestor declarations.

Closed dispatch analysis enumerates concrete class targets and excludes abstract
bodies. Runtime generic class dispatch supports substituted signatures, but closed
call-graph inference for generic class implementers currently reports a limitation
instead of silently omitting possible targets. Interface generic analysis retains its
existing behavior. Debugger frames retain the selected implementation and source
mapping; abstract declarations have no executable sequence points.

## .NET comparison

Primary sources consulted 2026-09-08:

- [.NET 10 callvirt](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.callvirt?view=net-10.0)
  selects behavior from runtime receiver type. neoCLR reuses callvirt and adapts its
  receiver to managed reference views, including frame values. CLR also permits
  nonvirtual calls through callvirt; this bounded class implementation requires a
  virtual declaration and uses call for nonvirtual members.
- [CLI Partition II, method attributes](https://download.microsoft.com/download/7/3/3/733ad403-90b2-4064-a81e-01035a7fe13c/ms%20partition%20ii.pdf)
  distinguishes virtual new slots from reuse. neoCLR's explicit virtual/override
  metadata records that distinction without claiming a CLR vtable representation.
- [CLI Partition III, call](https://download.microsoft.com/download/7/3/3/733ad403-90b2-4064-a81e-01035a7fe13c/ms%20partition%20iii.pdf)
  distinguishes statically selected call from virtual selection; concrete base-body
  calls retain that behavior. The CLI sources establish the baseline, not a claim
  about a particular current CoreCLR layout or optimization.

The [comparison probe](experiments/virtual-dispatch-dotnet/Program.cs) pins SDK
10.0.100 and targets net10.0. Run dotnet run in that directory. It checks abstract
base dispatch, inherited mutation, explicit base-body selection and reflection of
the original virtual declaration. On SDK 10.0.100 the probe printed 42, 1, 42,
True and True, matching those checks. It does not compare performance or frame lifetimes.

Using a copied Base receiver would lose derived state and mutation semantics. The
chosen reference-view model avoids that ambiguity and centralizes override validation
for all frontends. Costs include ancestry lookup, receiver projection and stricter
preview contracts. Caching/vtables and generic graph inference remain future work;
no speedup is claimed. Default interface bodies need their own ambiguity/override
rules rather than inheriting these class rules accidentally.

## Run and compatibility

```sh
cargo run --locked -- run examples/source/virtual-dispatch.neo --gc-stats
cargo run --locked -- run examples/source/abstract-classes.neo --gc-stats
cargo run --locked -- verify examples/source/abstract-classes.neo
cargo run --locked -- assemble examples/source/abstract-classes.neo /tmp/abstract-classes.neo.json
cargo run --locked -- run /tmp/abstract-classes.neo.json
cargo test --locked --test class_virtual --test abstract_classes
```

Both samples return 42. Tests cover serialized artifacts, frame/heap receivers,
multilevel overrides, mutation/reference returns, GC pressure, readonly violations
without verification, abstract construction/import rejection and invalid contracts.
Rust Function/TypeDef literals require the new flag fields; old JSON defaults them
false. New flags and expanded MethodInfo descriptors require a matching runtime and
System library. No published artifact format version is changed.
