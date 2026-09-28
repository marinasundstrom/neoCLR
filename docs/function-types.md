# Function types and Function objects

**Author-selected direction, 2026-09-28; structural Function replacement and the
common/nominal descriptor split are implemented on the development feature branch.**

The author selects structural Function types, Function objects instantiated from
those types, replacement and removal of delegates, and a nominal/structural
introspection split. `NominalTypeInfo` will hold `Name`, `Namespace` and other
declaration-specific information; `TypeInfo` will expose `IsNominalType`.
This supersedes the earlier investigation-only status in [the delegate guide](delegates.md#function-type-review-reopened-2026-09-13).
The [original proposal](proposals/function-types-and-objects.md) remains broader
exploration, not blanket authorization for its dynamic, native or event facilities.

Author follow-up: named function types might be supported in the future as a
replacement for delegates. The later clarification proposes a **nominal function
type inheriting a structural Function type with the required signature**. Separate
nominal function types would retain distinct identities even with the same signature;
one must not directly convert to the other merely because their shapes match.
This refines the earlier open alias-versus-nominal discussion for that proposed
facility. Neither nominal function inheritance nor transparent aliases are implemented
or selected for the current slice. Removing delegates does not reject future named
callable contracts.

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

The development Raven `TypeInfo` is now independent of `MemberInfo`.
`NominalTypeInfo` inherits both interfaces and owns `FullName` and `Namespace`;
`MemberInfo` supplies declaration names, module/token, ownership and attributes.
The implemented split and planned Function-specific descriptor are:

| Contract | Responsibility |
| --- | --- |
| `TypeInfo` | Common type identity, equality, shape queries and `IsNominalType`; usable for parameter/result/component types. |
| `NominalTypeInfo : TypeInfo, MemberInfo` | Nominal declaration name, namespace and declaration identity metadata. |
| `FunctionTypeInfo : TypeInfo` (planned) | One specific signature and its synthesized instance `Invoke`; ordered parameter contracts and return type, without invented nominal declaration metadata. |
| Function object | Bound target and environment, plus its Function type and typed invocation. |

`IsNominalType` must agree with whether the descriptor implements `NominalTypeInfo`.
Structural descriptors must not implement that interface with empty placeholders.
The common diagnostic display is `DisplayName`, independent of nominal `FullName`. A display string is not a type identity or hash
substitute. Common descriptor equality must compare handles across implementations,
not unconditionally cast to the current `RuntimeTypeInfo` implementation.

Author clarification: `FunctionTypeInfo` describes a specific Function signature,
not just the Function family. For `(Int32) -> String`, it describes that shape and
its synthesized instance method `Invoke(Int32) -> String`. Each Function object
created from the type has that method; invocation dispatches to the object's bound
target and environment. The synthesized Invoke descriptor belongs to the shape,
while the selected target method belongs to the instance's binding. Different
bound targets do not create different Invoke signatures or Function types.

This descriptor contract is planned, not an API already emitted by the reference
assembly. Its implementation must represent a synthesized member without inventing
a nominal type name or declaration token. How synthetic method descriptors fit the
existing declaration-oriented MemberInfo/MethodInfo contracts remains an explicit
implementation question. RavenDoc's Function family page can document the common
member contract, while a concrete FunctionTypeInfo describes one actual signature.

Audit every existing member when splitting the interfaces. In particular,
`FullName`, module/token/custom attributes and declaring-type relationships must
not leak back onto all shapes via another shared base. Preserve shape inspection
without requiring consumers to cast every type to a nominal descriptor. Member support does not imply nominal identity. Structural types may have members
and extension members; member queries remain on common TypeInfo. Consumers require
a nominal view only for declaration identity metadata, not merely to discover or
use a member. Existing execution support is still bounded by the runtime; this
direction does not claim that every structural family already supports reflection
invocation.

This is the first step toward the author's split covering tuples, unions,
intersections and function types. It does not retroactively make the current
`System.Tuple` generic value declarations or Raven declared unions structural.
Named declared unions and future structural union expressions must be distinguished;
the identity and migration rules for those other families remain follow-on work.
Array, pointer, byref, generic-parameter and constructed-type classification must
also be enumerated before finalizing `IsNominalType` for every existing descriptor.

## Selective inheritance and future nominal Function types

Non-nominal types are not generally inheritable. Structural identity and member
support do not by themselves grant permission to use a type as a base. Inheritance
eligibility must be explicitly defined for each supported kind; do not enable it
for arrays, tuples, unions, intersections or arbitrary structural shapes by analogy.

The author's proposed exception is a nominal function type whose base is a Function
shape. Conceptually, two nominal types A and B could both derive from `(Int32) ->
String` and have its Invoke contract, while A and B remain distinct types. Equal
signatures must not create a direct conversion from A to B. A type's nominal identity
and its base shape's structural identity would remain separate in introspection.
This is future design, not implemented inheritance syntax or a restored Delegate API.

Assistant analysis: base-shape substitutability, explicit construction/rebinding,
and conversions back from a structural value need separate rules. In particular,
a structural intermediate must not accidentally introduce an implicit A-to-B
conversion. This identifies a validation need, not an author decision to forbid
all explicit rebinding. Multiple bases, variance, further derivation and the precise
set of inheritable non-nominal kinds also remain open.

Compared with the nominal .NET delegate identity described below, the proposal keeps
separate named contracts non-interchangeable while sharing one structural invocation
base. Its benefit is reuse of signature and Invoke metadata without losing nominal
boundaries; its cost is new inheritance, conversion and descriptor rules beyond the
current structural replacement. Existing delegate research supplies the comparison;
no new .NET behavior or performance claim is made here.

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
   inspect Function parameter/result descriptors and preserve common member queries.
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

## Native foundation checkpoint (historical)

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
and old binding encodings; removing those and migrating library/Raven callbacks
remain required. The common/nominal descriptor split is now implemented. No completed public Function introspection,
named-function-type facility or full delegate removal is claimed.

Checkpoint validation, 2026-09-28: `cargo check --locked --quiet` passes;
`cargo test --locked --test function_types --test delegates --quiet` passes all
6 structural Function tests and 26 existing callback regression tests. `cargo fmt
--all` and the Git whitespace check pass. No Raven compiler or website build was
needed for this native foundation; their migration remains outstanding.

## Nominal descriptor checkpoint

Development `TypeInfo` exposes `DisplayName` and `IsNominalType`, with represented
type equality across its separate runtime providers. Nominal declarations, including
constructed named generics and current Tuple/union declarations, implement
`NominalTypeInfo : TypeInfo, MemberInfo`. Arrays, references, pointers, generic
parameters and structural Function shapes do not. Existing general shape and member
queries stay on TypeInfo; declaration identity metadata requires a nominal view.
This is a breaking source contract: use DisplayName for diagnostic text, and narrow
to NominalTypeInfo when declaration metadata is required.

The [executable consumer and negative compilation cases](experiments/function-types/README.md)
verify provider selection, equality, object discovery, collection and the absence of
declaration-only members on TypeInfo. Native coverage now also exercises higher-order
shapes, no-result call/callvirt and nominal classification (nine tests).
Public reference metadata and documentation include the new interface and members.
A source-order-dependent Raven getter emission issue remains a documented general
compiler candidate; this change does not claim to fix the compiler.

## Structural members and documentation

Author clarification, 2026-09-28: structural types can have members and extension
members; the distinction is the absence of a declared type name. Neither member
discovery nor extension eligibility should require NominalTypeInfo merely because
a type has members. A structural descriptor's DisplayName is diagnostic formatting,
not a nominal declaration name. Individual members may still have names.

RavenDoc should document structural families such as Array, Tuple, Union,
Intersection and Function with the same member-level detail as nominal types.
The assistant proposes stable family pages plus shape/signature examples and
applicable extension-member documentation. Documentation labels and navigation
identifiers do not create nominal runtime identities; closed structural shapes
need not each generate a separate type page. Family membership, member applicability
and rendered shape signatures need explicit documentation metadata rather than
fabricated Name/Namespace values. This is a documentation direction; a generic
RavenDoc structural-family renderer is not implemented in this checkpoint.

## Raven callback migration checkpoint (historical)

The library now spells callback types with Raven function syntax. FunctionBindings
converts CLI Func/Action transport into structural native shapes and emits
function.bind. Runtime Func declarations and their generated slice are removed;
native service callback signatures use Function shapes. The isolated Raven policy
in commit `09f4c91bd` selects inhabited unit-result transport for source unit
functions, matching generic function results instantiated with unit. Other targets
retain Action. This required no new source syntax or Runtime Contract setting.

The callback fixture compiles and executes shared shapes, captured state, generic
unit results and structural type identity. An extension declared on `(int) -> int`
also executes. Named generic parameters now bind recursively inside Function
shapes; a ten-test native suite and twelve runtime-service tests pass. Full library
regeneration and bootstrap hash validation pass. Asynchronous callback validation
is tracked in the migration fixture.

[Function family member documentation](../api-docs/functions.md) describes Invoke
and extensions without declaring a nominal Function type. CLI transport scaffolds
remain inventoried with explicit documentation exclusions. Removing legacy
delegate admission/frontends and settling object equality/nullability remain open.

## Structural replacement contract — 2026-09-28

The native runtime, Raven source library, importer and archived Neo frontend now
use structural Function shapes. Native `.delegate`, `delegate.bind`, the serialized
Delegate representation and delegate operand alias are rejected. There is no
nominal fallback. Named Raven delegates are not admitted; named function types
remain possible future work. Rebuild older artifacts against matching components.
The archived frontend spells the native shape `fn<P...,R>`; Raven uses arrow syntax.
Comparer adapters are FunctionComparer and FunctionEqualityComparer.

The bounded object contract preserves the earlier callable capability semantics:
shape, closed target and retained receiver determine equality, and copies share
captures. Bound class-receiver equality now compares heap identity as slot receivers
already did. Ordinary storage defaults to null and null Invoke reports NullReference;
constructors must explicitly initialize Function fields before publication. Function
objects currently have no independent heap allocation identity or ordinary Object
conversion. These are explicit limitations of this first slice, not reasons to
retain delegates. Compared with .NET Delegate, there is no nominal callable identity
or invocation list; target/receiver equality is retained. Benefits are shared shapes
across methods and no declaration boilerplate, at the cost of rebuilding artifacts
and a bounded Object/introspection surface. Existing research above supplies the
comparison; no performance improvement is claimed.

Structural types continue to support members and extensions. Invoke is synthesized
from each Function shape, and Function extensions execute in the Raven consumer.
The common TypeInfo still owns member queries, but declaration-based GetMethods
currently has no synthetic Invoke descriptor or token. Dedicated Function parameter/
result descriptor APIs and general structural member enumeration remain follow-up
work; the current family page documents Invoke and extensions explicitly. This
initial replacement does not reclassify existing nominal Tuple/union declarations,
implement intersections, select named Function identity, or build a general RavenDoc
structural renderer. Earlier checkpoint sections above remain historical evidence.
