# Function types and Function objects

**Author-selected direction, 2026-09-28; native foundation implemented on the feature
branch. Full replacement and introspection split remain incomplete.**

The author selects structural Function types, Function objects instantiated from
those types, replacement and removal of delegates, and a nominal/structural
introspection split. `NominalTypeInfo` will hold `Name`, `Namespace` and other
declaration-specific information; `TypeInfo` will expose `IsNominalType`.
This supersedes the earlier investigation-only status in [the delegate guide](delegates.md#function-type-review-reopened-2026-09-13).
The [original proposal](proposals/function-types-and-objects.md) remains broader
exploration, not blanket authorization for its dynamic, native or event facilities.

Author follow-up: named function types might be supported in the future as a
replacement for delegates. They are not part of this first structural slice.
Whether such a name is a transparent shape alias or introduces nominal identity
is open; do not infer that choice from the word "named". Removing delegates now
must not be documented as rejecting all future named callable contracts.

## Shape and instance

A Function type describes the invocation contract. A Function object is an instance
of that shape and holds the method reference required for invocation, together
with a receiver or closure environment when needed. Its `Invoke` contract follows
the shape. The selected method is instance data, not part of type identity.
Two methods with the same shape can therefore produce objects of the same Function
type, even when their names, declaring types or modules differ.

For example, `(Int32) -> String` identifies one shape within a resolved type
universe. Parameter names and source aliases do not create distinct shapes.
Nominal types used as components retain their existing module-scoped identities:
unrelated nominal classes with identical fields are still different parameters.
The bound receiver is not an exposed parameter; it is retained by the object.
The symbolic target method is distinct from the shape's `Invoke` operation.

Assistant recommendation for the first implementation: exact structural matching
of ordered parameter types, return type, and the existing readonly, out and
conditional-out contracts. Preserve the distinction between an inhabited `Void`
result and the internal no-result convention; compiler lowering must normalize
source unit consistently. Do not silently introduce variance or signature adapters.
Closed generic substitutions must be reflected in shape equality and hashing.
Transparent aliases are erased before comparison; recursive aliases must be rejected unless
a separate recursive-type contract is designed.

Reuse existing checked binding behavior: validate method access at binding, resolve
virtual/interface implementations, retain heap receivers and shared captures, and
reject frame-backed captures and forged host bindings. Function objects must work
as arguments, results, fields and generic arguments, with GC tracing through those
containers. Direct method calls need no Function object. Calling an object uses
the checked target and ordinary argument/result checks.

These recommendations preserve existing execution safety; they do not settle
public Function equality, nullability, object identity, allocation strategy or the
final public method-inspection API. Those choices need resolution before adopting
their production contracts. A managed method reference must not expose a raw native
entry address. Native ABI projections and multicast/event collections are separate
work, and are not reasons to retain a neoCLR Delegate feature.

## Introspection split

The current Raven `TypeInfo` extends `MemberInfo`, which supplies `Name`, `Module`,
`MetadataToken`, `DeclaringType` and custom attributes. Adding `IsNominalType` alone
would leave structural shapes pretending to be declarations. The proposed split is:

| Contract | Responsibility |
| --- | --- |
| `TypeInfo` | Common type identity, equality, shape queries and `IsNominalType`; usable for parameter/result/component types. |
| `NominalTypeInfo : TypeInfo, MemberInfo` | Nominal declaration name, namespace and declaration metadata; nominal inheritance, members and visibility where applicable. |
| `FunctionTypeInfo : TypeInfo` (proposed name) | Ordered parameter contracts and return type; no invented declaration name, namespace or metadata token. |
| Function object | Bound target and environment, plus its Function type and typed invocation. |

`IsNominalType` must agree with whether the descriptor implements `NominalTypeInfo`.
Structural descriptors must not implement that interface with empty placeholders.
Keep a common diagnostic type display, independent of nominal `FullName`; its
final spelling needs selection. A display string is not a type identity or hash
substitute. Common descriptor equality must compare handles across implementations,
not unconditionally cast to the current `RuntimeTypeInfo` implementation.

Audit every existing member when splitting the interfaces. In particular,
`FullName`, module/token/custom attributes and declaring-type relationships must
not leak back onto all shapes via another shared base. Preserve shape inspection
without requiring consumers to cast every type to a nominal descriptor. Reflection
consumers that need declared fields/constructors must explicitly handle nominal
types and reject or branch on structural types.

This is the first step toward the author's split covering tuples, unions,
intersections and function types. It does not retroactively make the current
`System.Tuple` generic value declarations or Raven declared unions structural.
Named declared unions and future structural union expressions must be distinguished;
the identity and migration rules for those other families remain follow-on work.
Array, pointer, byref, generic-parameter and constructed-type classification must
also be enumerated before finalizing `IsNominalType` for every existing descriptor.

## Comparison and tradeoffs

Primary sources checked 2026-09-28:

- [C# specification §21](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/delegates)
  specifies nominal delegate identity and a matching `Invoke` member. Equal
  signatures do not make separately declared delegates interchangeable. The chosen
  neoCLR shape identity intentionally changes that rule.
- [C# 10 lambda design](https://github.com/dotnet/csharplang/blob/main/proposals/csharp-10.0/lambda-improvements.md)
  discusses inferred function types at the language level and conversions to
  delegates. It is a design document, not evidence of a structural CLR runtime type.
- [Go specification: function types and type identity](https://go.dev/ref/spec#Function_types)
  supplies an implemented comparison for signature-based function shapes. Go's
  named types, multiple returns and lack of neoCLR managed-reference contracts mean
  its full rules cannot simply be copied.
- The existing [CLR comparison and executable probe](delegate-contract.md) provide
  the baseline for method binding, captures, lifetime and dispatch. No new .NET
  execution or runtime-source verification is claimed by this planning change.

Keeping nominal delegates minimizes migration but contradicts the selected identity
model. Adding only function syntax over delegates preserves that machinery but
does not provide runtime structural identity. First-class structural metadata
matches the author direction and removes the need for a named callable declaration
per shape. It costs changes across type serialization, linking, verification,
compiler/importer mapping, introspection and all callback consumers. CLR delegate
metadata needs explicit translation at an interoperability boundary; renaming
`Func` to `Function` does not solve identity. No speed or allocation improvement is
claimed. No independent .NET library replacement has been evaluated: a library
alone cannot change CLI nominal type identity. Further source/interop investigation
belongs to the compiler phase, not an unsupported claim of compatibility.

## Migration order and acceptance

1. Resolve the remaining core object, nullability, equality and descriptor-classification
   contracts. Define structural metadata, textual syntax, substitution and canonical
   identity independently of nominal `TypeDef` declarations. Test same-shape identity
   across modules and distinct nominal component identities.
2. Introduce checked Function binding and invocation and migrate tracing, scheduler,
   workers, initialization, verifier, reachability, debugger and host boundaries.
   Run positive static/generic/bound/virtual/interface/closure consumers and negative
   signature, readonly/out, frame-lifetime, private-access and forged-artifact cases.
3. Split introspection descriptors and migrate callers, including reflection, JSON
   mapping, hashing and formatting. Verify interface tests and `IsNominalType` agree;
   inspect Function parameter/result descriptors and preserve nominal member queries.
4. Integrate Raven on its isolated target branch, update reference emission and
   importer lowering, and regenerate the authored runtime. Compiler transport may
   need CLR-compatible metadata internally, but must not leak nominal delegates
   into neoCLR's resulting type identities or public contract. Record the precise
   boundary and remove generated delegate declarations from neoCLR artifacts.
5. Migrate `Func` APIs, comparers, queries, Task.Run, I/O callbacks, applications and
   samples. Remove delegate declarations/opcodes/runtime variants and old admission
   paths; define an explicit old-artifact rejection or conversion policy. Keep
   historical release notes and external .NET comparison probes intact.
6. Refresh matching compiler reference, generated library, API inventory, XML and
   documentation snapshot, then run focused consumers spanning synchronous callback,
   shared closure, asynchronous work and reflection. Update current website behavior
   only when supported by this evidence. Publication is separate.

The status owner is the [runtime/language tracker](tracking/runtime-language.md).
The steps above describe full acceptance, not completion of the feature branch.

## Native foundation checkpoint

The native metadata model now represents Function shapes directly, outside nominal
`TypeDef` declarations. Internal IL uses `fn<ParameterTypes...,ResultType>` and
`function.bind fn<...> = Target(...)`, followed by ordinary
`call instance fn<...>::Invoke(...)`. This is internal textual encoding, not a
selected Raven source syntax. `readonly`, `out` and `outtrue` parameter contracts
and a `noresult Void` result preserve the current calling distinctions.

Resolved identity recursively includes component identities and reference/output
contracts. Type substitution, module-reference checks and visibility checks visit
the components. The checked Function object retains its target and receiver using
the existing collector and invocation paths. The native host names are `Function`
and `Value::Function`; this is a host-source compatibility change.

`tests/function_types.rs` exercises declaration-free calls, a closed generic target,
artifact round trips, structural identity, shared shape storage, reference/output
distinctions, incompatible binding rejection, retained receivers under collection
and rejection of frame receivers. The existing delegate suite remains regression
coverage during migration. The transitional parser still admits delegate declarations
and old binding encodings; removing those, migrating the library/Raven and implementing
the descriptor split remain required. No completed public Function introspection,
named-function-type facility or full delegate removal is claimed.

Checkpoint validation, 2026-09-28: `cargo check --locked --quiet` passes;
`cargo test --locked --test function_types --test delegates --quiet` passes all
6 structural Function tests and 26 existing callback regression tests. `cargo fmt
--all` and the Git whitespace check pass. No Raven compiler or website build was
needed for this native foundation; their migration remains outstanding.
