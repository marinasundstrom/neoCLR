# Interfaces inherited through class bases

Implemented 2026-09-08. A derived record inherits its base's interface conformance.
An interface call uses the inherited mapping and, when that mapping names a virtual
class method, selects the concrete owner's override. This lets a shared library base
provide capabilities without repeating declarations or forwarding methods in every
child. It completes a missing neoCLR inheritance behavior.

## Contract

Interface closure follows substituted class bases and interface bases, deduplicating
identical interface identities. Type.GetInterfaces now includes interfaces inherited
through a class base; descriptor member queries remain declared-only.

Mapping starts at the nearest class that declares the requested interface, directly
or through an interface base. Search that class and its ancestors for the matching
public instance member. Repeating an interface declaration starts mapping there again;
an inherited public member can satisfy a newly declared interface. The member must
match the full parameter, return, readonly receiver/parameter and output contracts.
Inherited class methods still require managed receivers; inherited value receivers
and native record layouts remain restricted.

A nonvirtual mapping calls that body. A virtual mapping dispatches using the stored
complete concrete type, even if the interface was formed from a Base& view. An abstract
base must declare or inherit a matching member, which may be abstract; a concrete
descendant must provide the required override. No implementation is synthesized.

The runtime projects the receiver to the selected body's declaring owner. Fields and
returned field references refer to the original storage. A heap view retains the whole
owner; a frame view cannot escape its owning frame. Readonly restrictions survive
projection and dispatch. Interface formation uses the exposed type's capabilities:
a Base& cannot reveal an interface declared only by its stored descendant.

Closed interface dispatch analysis includes concrete descendants and their selected
bodies, excluding abstract declarations. Generic arguments are inferred from substituted
interface conformance; an uninferable argument faults instead of silently omitting a
target. This does not remove the separate generic class-call graph limitation.

## .NET comparison and decision

Primary source consulted 2026-09-08: the shipped
[C# interface specification, §§19.6.5–19.6.8](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/interfaces).
C# inherits interface mappings, dispatches mapped virtual methods to overrides, allows
inherited public members to satisfy interfaces, and restarts mapping on redeclaration.
Abstract classes can defer implementation through abstract members. C# also supports
explicit implementations and member hiding; neoCLR does not yet implement those.

The pinned SDK 10.0.100/net10.0 probe checks inherited mappings, overrides, reference
identity, enumeration and mapping through inherited public members. This is evidence
of observable behavior, not a CoreCLR layout or performance claim.

Requiring repeated declarations/forwarders would avoid ancestry-aware mapping but
would burden every derived library type. A separate dispatch mechanism would duplicate
existing class virtual selection. We instead reuse the current metadata, managed
references and callvirt with no new opcode. The cost is additional ancestry traversal
in validation, mapping and analysis. Future caching/JIT work must preserve the mapping
anchor separately from the selected override; no speedup is claimed.

neoCLR adapts this behavior to explicit reference views over values or managed heap
objects, without boxing or requiring Object ancestry. Exact readonly/output contracts
remain runtime-enforced. This is a provisional preview contract; method hiding,
explicit mappings, default bodies and variance require separate decisions.

## Validation and use

```sh
cargo test --locked --test class_interfaces
cargo run --locked -- run examples/source/class-interfaces.neo --gc-stats
(cd docs/experiments/class-interfaces-dotnet && dotnet run)
```

The Neo sample returns 42 using an abstract CounterBase, an inherited Counter interface,
a derived Read override and frame/heap views. Tests cover serialized artifacts, closed
generic analysis, inherited public bodies, reflection diamonds, GC pressure, invalid
frame escapes, incompatible/missing contracts and unchecked readonly writes.

No artifact schema change is introduced. Previously rejected inherited interface
programs become valid. Reflection callers must account for the expanded GetInterfaces
results. Class member hiding remains rejected.

## Following slices

Both **explicit interface implementations** and **default interface implementations**
are planned. They are distinct: explicit mappings let a concrete type implement a
contract separately from its public surface; defaults supply behavior in an interface.
First specify explicit contract-to-body identity and visibility, then default-body
selection. See the [interface implementation plan](reflection-hierarchy-plan.md).
Neither feature is enabled by this slice.
