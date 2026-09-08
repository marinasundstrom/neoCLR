# Default interface implementation contract

Implemented 2026-09-08 in the runtime, IL and Neo, following the pinned .NET
comparison and [explicit interface mappings](explicit-interfaces.md). The runnable
[default-interface sample](../examples/source/default-interfaces.neo) returns 42
through readonly frame and heap views.

The use case is shared behavior expressed entirely through a capability's members,
without requiring a common class base or repeating forwarding methods. A default
must work on the original frame or heap owner through a managed interface view.
It must not allocate interface storage, copy the owner or impose Object ancestry.

## Implemented contract

A public interface method may have a body. A bodyless declaration remains abstract.
A derived interface may explicitly implement a base interface member using the same
qualified declaration syntax as records. It may explicitly reabstract that member,
removing the inherited fallback for types implementing the derived interface.

```swift
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
   a reabstracted replacement, but cannot make an invalid concrete descendant
   instantiable. The existing rule for an original bodyless contract is unchanged:
   an abstract record must declare or inherit a matching abstract class member.

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

## Runtime integration and metadata

Interface membership and bodylessness are now separate checks. The verifier analyzes
executable interface bodies; closed call graphs include selected defaults and their
callees. Reflection reports IsAbstract only for bodyless/reabstracted members and
preserves interface declaring owners. Debugger frames and source points name the
executed interface body and expose its original managed receiver. Host invocation of
interface bodies remains restricted; use a guest interface call.

Function.interface_implementations also maps qualified replacements on interface
owners. Those bodies are private implementation members and cannot be used directly
as callvirt contracts; calls name the original public declaration. Record mappings
remain concrete and nonvirtual; interface replacements may be reabstracted. All
replacement receiver, parameter, return and output contracts must match.

A public interface method with instructions and a managed receiver is executable.
A bodyless interface declaration remains abstract even when its serialized is_abstract
flag is absent/false, preserving old artifacts. Neo emits the explicit abstract flag
for declarations/reabstractions; an empty Void source body emits ldvoid/ret and stays
executable. Explicit abstract methods cannot contain instructions. Bodyless members
cannot have locals or native bindings. No new opcode, receiver representation or
System descriptor layout is introduced.

Class mapping now returns absence separately from invalid contracts. Only absence
permits default fallback. Runtime frame entry uses the existing SlotInterface rather
than projecting the receiver to a record body owner. Readonly checks apply on entry;
output completion is checked at runtime return, as for existing class bodies. The
verifier does not yet prove every output-parameter assignment path.

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

## Validation, use and remaining scope

Tests cover Neo and serialized roundtrips, readonly frame/heap dispatch, nested calls
to explicit class implementations, class precedence, competing/resolved diamonds,
reabstraction, inherited/redeclared conformance, generic closed targets, output checks,
readonly failures without verification, frame escape, heap pressure and reference
identity. Debugger tests stop in a default and step into its explicit callee; fault
traces preserve the selected body and source line. Existing interface, explicit-mapping
and class-dispatch regressions remain applicable.

```sh
cargo run --locked -- run examples/source/default-interfaces.neo --gc-stats
cargo test --locked --test default_interfaces --test debugger
python3 docs/experiments/default-interfaces-dotnet/verify.py
```

The comparison script requires SDK 10.0.100, selects it through the probe directory's
global.json and prints the actual runtime version (10.0.0 in the recorded run). It
checks positive behavior and three expected compiler rejections. Build outputs remain
ignored. The class-based .NET probe does not establish Neo's frame-value safety.

New artifacts with default bodies or interface replacements require this runtime.
Recompile sources against the matching revision. Existing bodyless interfaces retain
their behavior, and defaults do not add methods to implementing class metadata.

The next library work should apply this to a real capability, such as deriving a
collection's IsEmpty property from Count. Preserve familiar property APIs rather than
adding method-shaped substitutes merely for the demo. GetInterfaceMap, broader Neo
property/indexer declaration syntax, default base calls, static virtual members,
interface state and method hiding remain separate work. Defaults add shared behavior;
MemberInfo's shared storage remains appropriately served by class inheritance.
