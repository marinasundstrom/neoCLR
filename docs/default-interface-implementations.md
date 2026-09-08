# Default interface implementation contract

Contract and executable .NET comparison completed 2026-09-08. **Default bodies are
not yet implemented in neoCLR or Neo.** This document specifies the next implementation
slice after [explicit interface mappings](explicit-interfaces.md); examples below are
proposed syntax, not runnable Neo samples.

The use case is shared behavior expressed entirely through a capability's members,
without requiring a common class base or repeating forwarding methods. A default
must work on the original frame or heap owner through a managed interface view.
It must not allocate interface storage, copy the owner or impose Object ancestry.

## Preferred contract

A public interface method may have a body. A bodyless declaration remains abstract.
A derived interface may explicitly implement a base interface member using the same
qualified declaration syntax as records. It may explicitly reabstract that member,
removing the inherited fallback for types implementing the derived interface.

```swift
// Proposed, not implemented syntax.
interface Readable {
    readonly func Read() -> int
    readonly func IsZero() -> bool { return this.Read() == 0 }
}
interface AlwaysNonzero: Readable {
    readonly func Readable.IsZero() -> bool { return false }
}
interface MustDecide: Readable {
    readonly abstract func Readable.IsZero() -> bool
}
```

A qualified override retains the original declaration identity. A same-name declaration
without a qualifier must not silently replace that contract. Existing ambiguous member
lookup still requires an explicit interface projection. Broad interface member hiding,
static virtual members, stateful interfaces and direct calls to a chosen base default
are outside the first slice.

Selection for one closed declaration and one complete concrete owner proceeds as follows:

1. Apply the existing class mapping anchor and per-class explicit/public search. A
   valid class body wins. An abstract class member still requires a concrete override;
   a default cannot rescue it. An invalid/incompatible class mapping remains an error.
2. Only if no class mapping exists, gather the original interface body plus explicit
   overrides/reabstractions of that exact declaration from the complete interface
   closure. Substitute generic arguments and deduplicate identical identities.
3. Discard a candidate when another candidate's declaring interface derives from it.
   A unique remaining concrete body wins. A unique abstract candidate means a missing
   implementation. Incomparable remaining candidates are ambiguous; never choose by
   declaration order, traversal order or the interface spelling at the call site.
4. A concrete type must have a valid result for every required contract. A class body
   may resolve an otherwise ambiguous default diamond. Abstract types may defer
   implementation, but cannot make an invalid concrete descendant instantiable.

Defaults must not cause ordinary class lookup to invent inherited methods. An interface
reference is required. Inherited class conformance remains anchored: merely adding a
public method in a descendant does not remap an inherited default; redeclaring the
interface restarts class mapping. The provisional explicit-reimplementation edge
recorded in [explicit mappings](explicit-interfaces.md) remains a separate decision.

## Receiver and lifetime rules

Enter a selected default with `this` represented by the existing managed interface
view (Value::SlotInterface), typed as the selected body's declaring interface. Retain
the same underlying SlotReference and complete concrete owner. Argument typing already
represents managed interface inputs as Interface&; a new pointer kind or boxed receiver
is unnecessary. The runtime may form the selected body's view internally after proving
conformance; this must not enable user code to downcast arbitrary interface references.

A default can call other interface members through this view. Those calls dispatch
again against the original owner, including explicit class implementations. It cannot
access concrete fields, initialize an interface value or acquire a writable capability
from a readonly receiver. Readonly receiver/parameter and output/return contracts remain
exact across replacements. A writable default may mutate through writable contract
members; a readonly default cannot call them.

An interface `this` is an initialized view, never an output slot. Forwarded field
references and returned interface views preserve the existing frame-escape checks and
complete-owner GC roots. Do not prolong a frame's lifetime to make a return legal.
Raw native pointer interface views stay excluded from default execution in the first
slice; existing pointer-backed class-body dispatch retains its current restrictions.

## Groundwork found in the implementation

The audit identifies specific changes, rather than a need for a second runtime model:

| Area | Current behavior | Required change |
| --- | --- | --- |
| interfaces.rs | `is_contract` means any interface-owned method; bodies are rejected | Separate interface membership from abstract/bodyless status; validate bodies and qualified interface overrides |
| metadata.rs / vm.rs | `argument_types` describes Interface&; Frame already accepts SlotInterface | Enter defaults with that view instead of projecting to a record body owner |
| interfaces.rs mapping | Missing and invalid class mappings both return faults | Represent absence separately so fallback cannot hide an invalid contract |
| inheritance.rs | Abstract/override validation assumes record owners | Route interface reabstraction/replacement through interface rules |
| verifier.rs | All interface-owned functions are skipped | Analyze executable interface bodies, including readonly and output flow |
| reachability.rs / program.rs | Interface-owned functions are treated as non-executable | Include selected bodies and their callees in closed graphs; keep arbitrary host entry into interface bodies restricted |
| reflection.rs | Every interface member reports IsAbstract | Report bodyless/reabstracted versus executable members accurately; retain interface declaring owners |
| frontend.rs | Interface methods parse as declarations only | Parse/lower bodies and qualified replacements; type `this` as an interface view with no record fields |

Reuse Function.interface_implementations for qualified interface replacements if its
validation can distinguish record and interface owners cleanly. Do not encode the
mapping in a body name. Preserve old bodyless artifacts: an omitted abstract flag on
an existing empty interface declaration must not accidentally create an executable
method. Choose a canonical abstract/body representation during implementation and test
old serialized fixtures. This remains a validation detail, not permission to infer
ordinary class abstractness from an empty body.

## .NET evidence and alternatives

Primary sources consulted 2026-09-08:

- [Shipped C# interface specification, §§19.4 and 19.6](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/interfaces):
  bodies, qualified derived-interface implementations and reabstraction are part of
  the current language baseline. This is not just the earlier C# 8 proposal.
- [TypeBuilder.DefineMethodOverride](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.typebuilder.definemethodoverride?view=net-10.0):
  declaration/body metadata mappings are independent of body names. This informs
  metadata reuse, not a claim that neoCLR implements the full CLI MethodImpl model.

The pinned .NET probe checks fallback, a most-specific implementation, a resolved
interface diamond, public/explicit class precedence, reabstraction satisfied by a
class, inherited versus redeclared conformance, a default calling an explicit ref-return
member, original-owner mutation/identity, and interface-owned reflection metadata.
Negative builds reject an unresolved diamond (CS8705), missing reabstracted member
(CS0535), and ordinary class access to a default-only member (CS1061).

Shared class methods remain preferable when sharing storage or construction is the
purpose. Compiler-generated forwarding bodies could provide surface convenience but
would bake fallback choice into each consumer and require every frontend to duplicate
selection/lifetime rules. Runtime defaults centralize those rules and permit independent
capabilities at the cost of more complex resolution, body verification and dependency
analysis. Changes to interface bodies can alter existing callers' behavior; adding a
new derived default can introduce ambiguity. No blanket binary-compatibility or
performance improvement is claimed.

neoCLR deliberately keeps explicit value/reference addressing and exact readonly/output
contracts. The .NET probe uses classes; it does not demonstrate unboxed frame-value
behavior in .NET or establish neoCLR's receiver safety. Those are implementation tests
required below, not conclusions from the comparison. No CoreCLR source/layout or JIT
optimization claim is made.

## Implementation acceptance and library use

Before enabling syntax, implement and test selection plus receiver entry together.
Cover source, raw IL and serialized artifacts: class precedence; nested defaults calling
explicit members; identical versus competing diamonds; reabstraction; generic closed
selection/collisions; private/inaccessible bodies; readonly violations; output assignment;
invalid frame escape; heap collection pressure; and complete-owner identity. Run negative
cases without prior verification. Closed graphs must include defaults and their dynamic
callees without omitting unresolved generic targets. Debugger stacks/source locations
must name the executed interface body. Reflection must distinguish an abstract contract
from a default body without synthesizing class member rows.

Start with a small capability demonstration whose default calls a required member.
Then evaluate a collection convenience such as IsEmpty derived from Count, keeping
familiar property APIs in the library. Add it only where a real collection consumer
benefits. Reflection capability defaults should follow an actual shared behavior need;
MemberInfo's shared storage is already served by class inheritance. GetInterfaceMap,
additional accessor declaration syntax and broad framework expansion remain separate.

Run the comparison, including expected compiler failures:

```sh
python3 docs/experiments/default-interfaces-dotnet/verify.py
```

The script requires SDK 10.0.100, selects it using the probe directory's global.json,
and prints the actual runtime version. Build outputs stay ignored. The comparison
is executable evidence for this contract slice; it does not enable defaults in Neo.
