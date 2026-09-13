# Reflection migration for the Raven target

The existing Type and Reflection APIs are being projected onto ordinary managed
classes. This document tracks that work; Raven source-level exposure is not yet
complete.

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
completion and package validation remain to be implemented before claiming this API
available in the POC.
