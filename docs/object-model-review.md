# Object, values and erased storage — consistency review

Reviewed 2026-09-23. The author directs harmonizing with .NET where useful,
particularly reference-type and value-type semantics. This review supersedes the
older proposal that Object inheritance should leave every type value-like, and
that all Object equality should describe fields independently of type category.
It does not claim new Object methods have been implemented.

## Current layers

| Concept | Current neoCLR behavior | Consequence |
| --- | --- | --- |
| Class instance | Nominal reference type; assignment copies the reference | Aliases share mutation; a base/interface view retains the allocation |
| Value instance | Assignment copies fields; reference fields still share their targets | Value copying is shallow, not deep cloning |
| `System.Object` | Raven library class with `GetType()` | Common reference view, not an arbitrary unboxed payload slot |
| `box T` | Copies a supported value into GC-owned storage exposed through Object/interfaces | Allocation, shared boxed identity and copy independence are observable |
| `System.Value` | Intrinsic, explicitly erased complete payload; pack/is/unpack operations | Carries exact type and value-copy behavior; not .NET ValueType or Object |
| `System.ValueType` | Compiler/reference metadata role | Not a replacement name for System.Value and not a complete executable .NET ValueType implementation |

Source evidence: [Object](../runtime/raven/src/System/Object.rvn),
[Option](../runtime/raven/src/System/Option.rvn),
[Result](../runtime/raven/src/System/Result.rvn),
[TaskOutcome](../runtime/raven/src/System/Tasks/TaskOutcome.rvn),
[class semantics](class-semantics.md) and [boxing](boxed-interface-values.md).
The raw/legacy value-and-managed-byref profile is not proof of the Raven class
contract; preserve that distinction in tests and documentation.

### A reference declaration is not implementation evidence

CoreDeclarations supplies Object Equals, GetHashCode and ToString stubs to the
reference/compiler environment. The generated Object library only supplies GetType.
These stub bodies must never be executed or advertised as working implementations.
Normal Object base-constructor calls are recognized by the importer, but direct
Object construction needs its own admitted source/import/runtime case before it is
promoted as a supported application API. Keep this gap explicit in the on-site docs.
Do not remove compiler scaffolding without checking compiler synthesis dependencies.

## .NET baseline and preferred adaptation

Sources retrieved 2026-09-23, targeting .NET 10 public contracts:

- [Object](https://learn.microsoft.com/en-us/dotnet/api/system.object?view=net-10.0)
  supplies GetType, virtual Equals/GetHashCode/ToString, static equality helpers,
  protected MemberwiseClone and finalization support. This is a library surface;
  C# assignment rules, CLI boxing and GC identity are separate layers.
- [Equals](https://learn.microsoft.com/en-us/dotnet/api/system.object.equals?view=net-10.0)
  defaults to reference identity for Object, while
  [ValueType.Equals](https://learn.microsoft.com/en-us/dotnet/api/system.valuetype.equals?view=net-10.0)
  compares value contents. Overrides can define domain equality. Adopt this distinction
  as the design baseline; do not force structural equality on ordinary mutable classes.
- [GetHashCode](https://learn.microsoft.com/en-us/dotnet/api/system.object.gethashcode?view=net-10.0)
  must agree with equality. Equal objects require equal hashes; distinct instances
  need not have distinct hashes. Implement equality and hashing together. Hashes
  are not persisted identities, addresses or guaranteed stable between processes.
- [ReferenceEquals](https://learn.microsoft.com/en-us/dotnet/api/system.object.referenceequals?view=net-10.0)
  compares identity without virtual equality, including two null references. neoCLR's
  existing `ref.eq` also compares managed interior locations; retain that low-level
  operation separately from an Object-level API. Intrinsic String currently lacks
  canonical object identity, so a universal identity helper needs that case resolved.
- [ToString](https://learn.microsoft.com/en-us/dotnet/api/system.object.tostring?view=net-10.0)
  has a useful type-name default with overridable display text. This is a good first
  Object extension, provided calls through Object actually dispatch to overrides.
  Do not add a nonvirtual lookalike merely because it is easier to import.
- [MemberwiseClone](https://learn.microsoft.com/en-us/dotnet/api/system.object.memberwiseclone?view=net-10.0)
  is shallow copying, not resource duplication. Defer it until a concrete caller
  and derived-layout/readonly/resource rules are tested. GC finalization likewise
  must not be introduced as a substitute for deterministic resource cleanup.

The [executable .NET baseline](experiments/object-baseline/README.md) checks aliasing,
shallow value copies, boxing, equality/hash consistency, null identity and formatting.
It makes no performance claim and does not claim neoCLR already matches every case.

## Keep, revisit and remove

**Keep:** reference/value category distinctions, typed `Equatable<T>` contracts,
explicit boxing and GetType's concrete-type behavior. An Object base method must
not silently change ordinary value assignment or introduce boxing in generic storage.
Maintain current Result/Option ergonomics independently of payload representation.

**Revisit:** Object metadata versus executable members, override dispatch through
Object, defaults for Equals/GetHashCode, String identity and boxed-value dispatch.
Choose CLR-compatible semantic outcomes where possible; internal layout need not
copy CoreCLR. Identity hashes must survive collection without exposing pointers.
Value equality needs a deliberate policy for reference fields, floating point,
cycles and custom Equatable implementations rather than raw byte comparison.

**Retain temporarily:** System.Value and its instructions. Option, Result, TaskOutcome,
native return protocols and host input/inspection still depend on this representation.
Replacing every erased field with Object today would change copying to box aliasing,
introduce observable identity and leave exact extraction/unboxing gaps. Replacing it
with Void* instead requires ownership, tracing and copy/release machinery for managed
payloads. Neither is a rename. Keep the prior retirement goal, but require a measured,
tested migration before removing the type, instructions or artifact encoding.

Rust's [Any](https://doc.rust-lang.org/std/any/trait.Any.html) provides exact runtime
type inspection/downcasting independently of universal equality. It is a useful
comparison for separating responsibilities, not a proposed neoCLR replacement:
borrowing, owned boxes and lifetimes differ. No alternative .NET library was selected
as a replacement for Object/CLI value semantics; this review concerns foundational
runtime contracts rather than a library performance problem. A deeper source-pinned
runtime/issue review is required when implementing hashing or changing storage.

## Bounded implementation order

1. **This slice:** record actual behavior and gaps; cover Object.GetType and Value
   in the API reference; correct obsolete direction notes. No runtime API expansion.
2. **Next:** a small Raven object-display sample with a base reference and derived
   override. Implement/test Object.ToString with a type-name fallback, ordinary
   virtual dispatch and honest importer diagnostics. Check class, boxed value,
   String and null paths before declaring the surface general. Keep Console object
   overloads and automatic interpolation integration out until this dispatch works.
3. **Then:** Object.ReferenceEquals and the Equals/GetHashCode pair, with equal
   payload/distinct identity, null, boxed values, custom overrides, mutable fields
   and GC tests. Assess typed collection equality alongside this contract.
4. **Separately:** inventory and migrate one Value-backed carrier with nested managed
   fields, copy/alias tests, GC, host/native boundaries and artifact migration evidence.
   Do not remove Value merely because Object exists.

This order is an assistant recommendation under the author's consistency direction,
not approval of an entire .NET Object clone. Storage cleanup, suspension and scheduling
remain open follow-ups; this review does not pull networking forward.
