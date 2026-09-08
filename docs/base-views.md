# Managed base-reference views

Implemented on top of [inherited value layout](inherited-layout.md). A managed
Derived& can now project to Base& while retaining the complete derived owner. Neo
inserts the projection for parameters, local/field initialization and returns with
an expected ancestor reference type, or accepts an explicit `as Base&`.

```swift
record Position(X: int)
record LabelledPosition(Label: string): Position
func Read(readonly position: Position&) -> int { return position.X }
func Make() -> Position& { return new LabelledPosition(42, "heap") }
```

References still access fields automatically. Allocation and lifetime do not change:
frame references remain frame-owned, heap references keep their complete allocation
reachable, and returning a reference into the current frame faults. No wrapper,
boxing or implicit heap promotion is introduced.

## Reference and field contracts

The reference carries its exposed target type separately from the type stored at
its existing owner/path. A base projection changes the target type only. Identity
compares the same location, so the derived reference and base view are reference-equal.
GetType/ref.type reports the stored derived type; typeof(Base&) continues to describe
the static signature. Projections of nested record fields preserve that field's
location, rather than treating the enclosing record as the dynamic receiver.

`castclass Base` consumes a managed record reference and returns a reference to the
same type or a declared ancestor. Generic ancestry uses substituted type identities.
Readonly access is preserved. Invalid ancestors, downcasts, interface views, native
pointers and uninitialized owners are rejected. Interface projection continues to
use interface.borrow; this does not yet introduce inherited class interface dispatch.

`ldfld` now accepts a managed record reference as well as a record value. Field
selection and accessibility use the exposed type; a base view cannot access a derived
field by its physical index. Neo emits this direct reference field read instead of
copying the complete owner with ldobj first. `ldflda` derives field addresses using
the inherited prefix layout and retains access/lifetime restrictions. Separately
stored managed-reference fields keep their own permission, as in the existing shallow
readonly contract.

A projected view is not a writable Base-sized storage slot. Whole-value `ldobj`,
`stobj`, `initobj` and output binding through it fault: they must not slice a Derived
value, replace it with Base, or fulfill an output with a partial derived replacement.
These checks apply without verification. An exact Base& pointing to an actual Base
still supports those operations. Replacing the complete Derived through its original
slot remains valid; existing base/field views continue to address that slot. There
is no implicit Derived-to-Base value copy in Neo.

The verifier validates ancestor conversions, initialized local formation and readonly
flow. It does not retain a complete projection/provenance proof through every typed
slot or call; whole-value restrictions and frame escape remain runtime checks.
Existing debugger stack/heap owner inspection continues to expose the complete value.

## .NET comparison and scope

Primary documentation consulted 2026-09-08:

- [.NET 10 castclass](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.castclass?view=net-10.0)
  supplies the familiar instruction name for checked reference conversion. CLR
  castclass consumes an object reference (O); neoCLR's operation consumes a managed
  record reference (T&), including frame-owned values. CLR's broader casts and null
  behavior are not implemented here. This is an explicit semantic adaptation.
- [.NET ldfld](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.ldfld?view=net-10.0)
  supports managed-pointer receivers. Extending neoCLR's formerly value-only ldfld
  follows that familiar field-access pattern. Native pointer receivers and static
  fields are not added by this change.

The [inherited-layout .NET probe](experiments/inherited-layout-dotnet/Program.cs)
(SDK 10.0.100, net10.0) also checks a Base<int> reference to Derived. Its last two
outputs are True for ReferenceEquals and the derived GetType. Run `dotnet run` in
that probe directory. This compares observable reference behavior, not CLR storage
representation or neoCLR's additional frame/readonly rules.

A synthetic nested Base payload would change location identity and tracing paths;
retaining one complete owner avoids that distinction. Allowing implicit whole-base
copies or writes would need a slicing/update contract, so this slice rejects them.
The benefit is predictable identity, access and lifetime across frontends. Costs
include ancestry traversal, an exposed/stored-type distinction and runtime whole-value
checks; this work makes no performance claim.

Class virtual slots, inherited methods, base constructor chaining, downcasts and
native inherited layout remain future work. The inherited-layout restriction on base
types with instance methods or implemented interfaces still applies. Virtual dispatch
can now build on the separate view and stored-owner types instead of discarding
that distinction. No general assignability or covariance rule is added to containers.

## Run

```sh
cargo run --locked -- run examples/source/base-views.neo --gc-stats
cargo run --locked -- verify examples/source/base-views.neo
cargo run --locked -- assemble examples/source/base-views.neo /tmp/base-views.neo.json
cargo run --locked -- run /tmp/base-views.neo.json
cargo test --locked --test base_views
```

The sample returns 42 using frame and returned heap views. Tests cover serialized
artifacts, nested fields, generic ancestors, identity/type inspection, GC pressure,
frame escape, readonly forwarding, whole-value faults and invalid casts. New
castclass artifacts require this runtime revision; no artifact-format bump is made.
Existing value ldfld remains valid. Rust Instruction matches must handle CastClass.
