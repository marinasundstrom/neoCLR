# Reflection migration for the Raven target

The existing Type and Reflection APIs are being projected onto ordinary managed
classes. This document tracks the implemented projection and its remaining boundaries.

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
not offered. Direct Type.Equals remains present; projecting its Equatable contract
awaits the general value/interface migration. These are provisional target-profile
implementation choices, not changes to Neo's original library profile.

This reuses the [runtime reflection design](reflection.md) and
[class-semantics comparison](class-semantics.md). Tests in
[tests/raven_reflection.rs](../tests/raven_reflection.rs) exercise inherited access,
returned base views, nested GC roots and allocation limits. Existing reflection,
reflection hierarchy and adapted collection tests pass. Raven metadata, source samples,
completion are covered by the subsequent source projection below; package validation
remains required before shipping this API in the POC.

## Raven API projection

The current source compiler can now use `typeof(T)`, `TypeOf<T>.Of(value)` for
admitted closed types, all existing Type query methods, and the public getters on
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
shapes are not introduced here. General Equatable/interface projection and package validation remain separate work
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
