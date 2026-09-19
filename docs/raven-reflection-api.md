# Reflection migration for the Raven target

The current Raven profile exposes eight sealed Info interfaces with internal providers.
Both typeof(T) and Object.GetType() return TypeInfo. RuntimeContext.Current exposes
ExecutingAssembly; AssemblyInfo supplies ReferencedAssemblies, modules and retained
loaded types. MetadataToken is on the Info interfaces, scoped by Module for type,
member and parameter definitions. Dynamic loading remains future RuntimeContext work.

All public collection results use System.Collections.Sequence<T>, including member,
parameter, generic-argument, interface and enum-name queries. Use Count instead of
Length; indexing, enumeration and Iterable query extensions remain available. The
public interface has no mutation methods, while implementations own fresh snapshots.
Rebuild consumers and replace array result annotations with Sequence<Element>.
See [the current contract and limits](introspection-design.md#complete-sequence-migration--2026-09-19).

The sections below retain the implementation history; earlier Type/Info and array
spellings describe superseded stages, not the current Raven contract.

## Production interface/provider slice — 2026-09-19

TypeInfo, ParameterInfo, MemberInfo, FieldInfo, MethodInfo and PropertyInfo are
Raven-authored sealed interfaces. RuntimeTypeInfo and the other Runtime*Info classes
are internal implementations; concrete providers are sealed leaves and their
shared RuntimeMemberInfo base is a closed hierarchy. Native metadata snapshots are
materialized into these providers while existing filters, storage order, accessor
selection and independent parameter-array copies are preserved.

The reference carries Raven's ClosedHierarchyAttribute with its exact permitted
family. Internal provider definitions remain in reference metadata so Raven can
resolve those entries, but applications cannot name or construct them. Compiler
checks reject outside implementations; the target importer independently rejects
foreign implementation metadata and validates the library's permitted family.
This does not add general raw-neoIL closed-hierarchy enforcement.

MemberInfo's direct permitted cases are the public FieldInfo, MethodInfo and
PropertyInfo interfaces. The shared provider base does not implement MemberInfo;
callers need no internal-provider arm. The consumer sample
`docs/experiments/raven-target/samples/library-introspection-interfaces.rvn` executes
all three cases without a fallback. The saved-project gate rejects each missing
arm with RAV2100. Raven's current diagnostic names the hidden leaf; accessible-case
wording remains a [general compiler candidate](introspection-design.md#exhaustiveness-over-memberinfo).

The target importer now translates reference `isinst`, reference conditional branches
and the emitted non-null comparison into checked neoIL type tests. There is no new
Runtime Contract configuration and no change to Raven's type-pattern emission.
C# reference generation promotes inherited interface accessors to virtual/final;
the reference projection normalizes only RuntimeMemberInfo's shared accessors to
Raven's ordinary method flags. Admission still checks exact method contracts.

Validation: all 73 source slices regenerate and snapshot hashes match; 12 descriptor
admission cases, seven TypeInfo cases, five ParameterInfo cases and 29 focused
reflection/type-inspection tests pass. All 770 Raven-profile declarations have
source or explicit native-service ownership (54 services). The language-server
check reports completion kind 8 (interface) for all six public Info contracts and
keeps Runtime*Info providers out of public completion. All 71 saved-project checks
pass across the initial 42 samples and the resumed 29 checks after correcting the
old FieldInfo-class-base assertion. The 21 reference/boxing checks cover successful
and failed type tests, nulls, object identity, boxed interface implementations and
intrinsic strings. A broader cargo run was stopped after targeted checks passed;
this is not a new full-suite validation claim.

Existing callers must rebuild: Info member calls now use interface dispatch. The
historical Neo profile retains value descriptors. Type references in these
interfaces still use System.Type and collection results still use arrays in this
intermediate slice. Retiring Type/Info, establishing RuntimeContext assembly discovery
and Object.GetTypeInfo, and deciding collection result contracts remain open.
The historical descriptions below must not be read as the current Raven declaration
kind. See the [maintained design](introspection-design.md) for the selected target.

## Target design update — 2026-09-17

The [refined proposal](introspection-design.md) replaces the planned public
Type/TypeInfo split with `*Info` interfaces and runtime-backed RuntimeContext v1.
The historical implementation described below uses classes, System.Type and Type.Info;
it is the earlier migration baseline, superseded in part by the production slice above. Further class hierarchy ports
are superseded by interface-contract/provider work. Reflection and Emit remain
separate; offline metadata loading and typed introspection are deferred.
BindingFlags remains in use during this migration.

An [isolated Raven probe](experiments/raven-target/introspection-v1/README.md) now
exercises TypeInfo/MemberInfo interfaces with internal runtime-backed adapters and
DeclaringType returning TypeInfo. It uses the current runtime rather than replacing
its production descriptors. The production identity migration remains open.

The follow-up [RuntimeContext POC](experiments/raven-target/introspection-v1/README.md)
runs actual Raven `typeof(Date)` through
`RuntimeContext.Current.GetTypeInfoFromHandle`, returning the TypeInfo interface
with an internal RuntimeTypeInfo implementation. This opt-in compiler contract
does not change the default compiler's System.Type behavior or migrate the installed
core yet. The legacy Type/Info examples below still describe that installed core.

The [checked interface declaration path](raven-system-library.md#checked-interface-declarations--2026-09-17)
is now integrated and exercised by the Raven-authored Clock contract. It is a
prerequisite for the Info interfaces, not a claim that these class-based descriptors
have already migrated.

The isolated prototype now includes a field-query path: TypeInfo.GetFields(flags)
returns descriptive FieldInfo interfaces whose Type and DeclaringType both use
TypeInfo. Existing filtering and invalid-flag behavior are preserved. The current
demo uses an eagerly populated Iterable, not a settled replacement collection API;
type acquisition still wraps the legacy runtime identity. No dynamic field access
or production class/interface replacement is introduced.

The target acquisition contract is runtime-owned handle/type-of resolution to a
TypeInfo interface backed by a hidden RuntimeTypeInfo. RuntimeContext must yield
the same or value-equivalent descriptor. The prototype's direct internal adapter
construction is temporary bootstrap code, not the intended public API; resolver
identity and context equivalence still need implementation and tests.

## Introspection namespace migration — 2026-09-16

### TypeInfo Raven port

TypeInfo's existing base/interface, enum and member queries are now authored in
`runtime/raven/src/System/Introspection/TypeInfo.rvn`. The same runtime metadata
services produce the snapshots. Bootstrap-only adapters copy their results into
managed arrays and retain the existing filtering behavior (default flags 28).
Flag overloads now name their parameter `flags` instead of `arg0`; positional calls
are unchanged. Regenerate reference metadata and consumers for named calls.

The importer checks the single opaque handle layout and exact internal factory
contract. Bootstrap generation preserves that factory's internal visibility.
Info remains an instance property on Type; the member hierarchy remains neoIL.
No invocation APIs, independent metadata provider, Raven compiler change or Runtime
Contract configuration change is introduced. This follows the shared-model scope
clarification in the [proposal](reflection-model-review.md).

Validation: `verify_type_library.py --type-info`, executable introspection consumers,
runtime reflection suites, bootstrap freshness and website checks.

### ParameterInfo Raven port

`runtime/raven/src/System/Introspection/ParameterInfo.rvn` now implements all six
ParameterInfo readers in the Raven profile. Runtime factories still supply the
snapshot data. The importer validates the exact six-field order, types and core
Type identity before admitting the implementation; added or reordered storage is
rejected. The constructor is private and initializes every field. Public member
signatures and snapshot behavior are unchanged.

The profile substitutes the generated Raven declaration for the complete old
ParameterInfo declaration. The bundled historical neoIL profile retains its existing
body. The MemberInfo hierarchy remains neoIL-authored. This reuses the
existing descriptor/.NET comparison and adds no compiler configuration or opcode.
Run `verify_parameter_info_library.py` for positive and negative authoring checks
and `verify_introspection_namespace.py` for executable Raven consumers.

The existing TypeInfo, MemberInfo, FieldInfo, MethodInfo, PropertyInfo,
ParameterInfo and BindingFlags now belong to `System.Introspection`. Change Raven
imports to `import System.Introspection.*` and rebuild reference metadata,
applications and System together. The old descriptor names have no aliases. The
bundled neoIL profile and its samples use the same new identities.

This implements the namespace portion of the [proposal](reflection-model-review.md).
Type stays in System and is Raven-authored; Info remains an instance property for
this slice. Apart from TypeInfo and ParameterInfo, descriptor bodies still use neoIL. Extension Info, Raven descriptor
bodies, optional runtime invocation and Emit are subsequent work. Existing query,
allocation, filtering and visibility behavior is preserved.

This reuses the .NET comparison below: the public namespace separates descriptive
operations from future execution capabilities, with a source/metadata migration
cost. Namespace changes alone add neither capabilities nor performance guarantees.
No Raven compiler changes or Runtime Contract configuration changes are needed.

Validation uses the runtime reflection/hierarchy/enum suites, the Type authoring
gate and `verify_introspection_namespace.py` (three Raven consumers and rejection
of the old descriptor namespace), plus regenerated bootstrap and coverage checks.

## Type source and metadata boundary (2026-09-15)

`runtime/raven/src/System/Type.rvn` now owns the Raven target's Type implementation.
Type exposes identity, names, category/shape flags, element type and closed generic
arguments. `type.Info` returns `System.Introspection.TypeInfo`, which retains the same
opaque RuntimeTypeHandle and exposes member enumeration, base/interface discovery
and enum metadata. It does not eagerly build an entire member graph. Current
queries can allocate snapshots; no allocation-free or cache guarantee is made.

Use `typeof(int).Info.GetMethods()` instead of `typeof(int).GetMethods()`. The Raven
reference surface no longer exposes these queries on Type. Member hierarchy
descriptors remain NeoIL-authored; TypeInfo and ParameterInfo are Raven-authored. The original Neo profile retains its
Type forwarding methods only for the source migration process.

Type's private constructor is emitted and checked like other class constructors.
Its sole handle field is validated because native factories also construct Type
snapshots. RuntimeTypeHandle has no valid default value: a constructor must assign
it, and reading it before assignment or returning without assignment faults.
GetTypeFromHandle remains the compiler's token-to-Type operation; Type has no public
instance constructor or API for constructing instances of the described type.

This differs from .NET's TypeInfo deriving from Type: neoCLR separates the identity
view from metadata lookup without making them interchangeable. The broader closed
hierarchy and TypeInfo's possible MemberInfo base remain later work; this port does
not claim runtime enforcement of closed hierarchies. In particular, a top-level
type's absent DeclaringType needs a defined contract before adding that base.

Validation: `verify_type_library.py`, `tests/raven_reflection.rs`, and the saved
project suite cover authoring admission, constructor initialization, base views,
metadata queries and the migrated consumer syntax. Raven's compiler is unchanged.

## Snapshot storage

The adapted library declares Type, MemberInfo, FieldInfo, MethodInfo, PropertyInfo
and ParameterInfo as classes. MemberInfo remains an abstract base. Trusted runtime
reflection factories allocate snapshots according to those declarations, including
nested Type objects inside descriptors, arrays and Option payloads. Returning a
base view retains the complete descriptor and its nested metadata objects through GC.
Allocation observes the managed heap object limit; snapshot construction faults when
the remaining budget is insufficient. Snapshots never retain the inspected guest object.

This follows the class/reference categories of .NET reflection while retaining
neoCLR's existing metadata-only API and Option-returning queries. It is not dynamic
invocation, caching or canonical Type identity. Each query can allocate fresh snapshots;
Type.Equals compares metadata identity. Allocation cost is an explicit consequence
of replacing copied descriptor records with familiar class references.

The adapted library omits its old internal byref descriptor constructors: trusted
factories construct the complete snapshot layout. Public descriptor construction is
not offered. Direct Type.Equals and its [Equatable contract](raven-fundamental-interfaces.md) are present. These are provisional target-profile
implementation choices, not changes to Neo's original library profile.

This reuses the [runtime reflection design](reflection.md) and
[class-semantics comparison](class-semantics.md). Tests in
[tests/raven_reflection.rs](../tests/raven_reflection.rs) exercise inherited access,
returned base views, nested GC roots and allocation limits. Existing reflection,
reflection hierarchy and adapted collection tests pass. Raven metadata, source samples,
completion are covered by the subsequent source projection below; package validation
remains required before shipping this API in the POC.

## Raven API projection

The current source compiler can now use `typeof(T)` for admitted closed types, all existing Type query methods, and the public getters on
MemberInfo, FieldInfo, MethodInfo, PropertyInfo and ParameterInfo. Class assignment
and base casts preserve identity. Descriptor arrays are projected as managed arrays;
the adapter copies the owned snapshot vector while retaining descriptor references.
Option<Type> and Option<MethodInfo> queries support typed case matching.

[The executable sample](experiments/raven-target/samples/library-reflection.rvn)
covers names, shapes, generic arguments, enum metadata, fields, method parameters,
properties, accessor Options, base views and filtering. BindingFlags now uses standard CLI enum metadata and the syntax described below. Name retains neoCLR's
existing qualified-name behavior. DefinitionIndex describes the current metadata
snapshot and is not a stable cross-build identifier.

Run the saved sample through `run_project.py` with the collection profile prepared
as described in the [experiment instructions](experiments/raven-target/README.md).
`verify_project.py --collections` includes it. `verify_editor.py --reflection`
checks completion for Type, MethodInfo and PropertyInfo, including inherited members.
The editor can also show inherited Object APIs that remain outside the executable
importer; their appearance alone is not a claim of support.

The importer rejects incompatible descriptor hierarchy and member signatures. The
runtime remains introspection-only: MethodInfo.Invoke, field/property mutation,
arbitrary application classes, new dynamic invocation facilities and universal generic
shapes are not introduced here. Package validation remains separate work
before completing the existing-API audit.

## BindingFlags enum metadata

BindingFlags is an Int32-backed CLI enum with FlagsAttribute and the existing
Default, DeclaredOnly, Instance, Static, Public and NonPublic literals. It has no
methods of its own in exported metadata, following
[ECMA-335](https://ecma-international.org/publications-and-standards/standards/ecma-335/)
enum and TypeDef requirements (consulted 2026-09-13). The former wrapper spelling is a breaking
change within this experiment:

| Former wrapper expression | Raven enum expression |
| --- | --- |
| `BindingFlags.Public()` | `BindingFlags.Public` |
| `BindingFlags.FromValue(bits)` | `(BindingFlags)bits` |
| `flags.Value` | `(int)flags` |
| `left.Or(right)`, `And`, `Xor` | `left | right`, `left & right`, `left ^ right` |
| `flags.Not()` | `(BindingFlags)~(int)flags` |
| `left.Equals(right)` | `left == right` |
| `flags.HasFlag(mask)` | `(flags & mask) == mask` |

Raven currently defines unary complement for integral operands; the explicit casts
above follow that documented language contract. Its target-typed leading-dot syntax
now handles chains and grouping: `.Public | (.Instance | .Static)`. A corresponding
Raven experiment compiler fix has ordinary CLR regressions. neoCLR adapts CLI integer
stack operations at enum storage and call boundaries; it retains the existing nominal
record layout internally and needs no new opcode. This representation cost can be
revisited independently of the standard metadata contract. Runtime introspection can
still see the implementation's helper methods; those are not exported enum members.

[The flags sample](experiments/raven-target/samples/library-flags.rvn) exercises casts,
operators, comparison and reflection filtering. Unknown bit combinations remain
representable, matching the existing FromValue behavior. Validation checks the enum
underlying field, literals, absence of methods and FlagsAttribute. This slice does not
claim import support for arbitrary application enum declarations.
