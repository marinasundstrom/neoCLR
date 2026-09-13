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
properties, accessor Options, base views and filtering. The existing BindingFlags
factories and combinators are exposed as a provisional value wrapper, including its
Value property. This does **not** yet project enum-literal/operator syntax or claim
that its reference metadata is a CLI enum; that metadata migration is still required.
The runtime query itself sees the original enum definition. Name retains neoCLR's
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
shapes are not introduced here. General Equatable/interface projection and the enum
metadata boundary remain separate work before completing the existing-API audit.
