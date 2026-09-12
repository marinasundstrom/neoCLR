# Nominal class references: first runtime slice

Implemented 2026-09-12 on the Raven experiment branch. This begins the intended alignment
with .NET value/reference type semantics; it is not the completed migration.

## Runtime contract

The experimental neoIL declaration `.type class Counter` selects reference semantics.
A `Counter` local, parameter, return or field holds an ordinary object reference.
Assignment and passing copy that handle. Member reads/writes access its managed heap
object. A `Counter&` remains a byref to a storage slot, so a function can replace the
caller's class reference through it. The ordinary class reference does not point into
a call frame and can be returned safely without an escape conversion.

Unmarked `.type` definitions retain existing value behavior during migration. Copying a
value copies its fields; any class references inside it still share their target objects.
No change is made to the existing String, array, delegate or System library projections.
Neo's source compiler has not yet been migrated to this declaration model.

Internally `TypeDef.is_reference_type` records classification and `Value::ObjectReference`
carries a heap-only handle. Its signature type is the nominal class, not `ByRef<Class>`.
The existing GC traces these handles in stacks, fields and returned values, with allocation
limits and diagnostic identities. `ref.eq` accepts ordinary class references. The host can
inspect the returned allocation through the execution's heap; importing a counterfeit
owned class record as a host argument is rejected.

This internal field and neoIL spelling are not proposed CLI extensions. A future metadata
reader should derive nominal classification from standard CLI type definitions/signatures.
No new instruction has been added. Existing artifacts omit the field and retain their
behavior; Rust consumers constructing TypeDef literals must supply the new field.

## Comparison and limits

The baseline is the distinction described in Microsoft's
[value-type documentation](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/value-types)
and [reference-type documentation](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/reference-types)
(consulted 2026-09-12), together with ECMA-335 Partition I's object-reference/managed-pointer
categories. Adoption restores familiar sharing and copying behavior; it is not a novel
performance improvement. A separate object handle avoids rewriting every imported class
signature as a managed byref. The cost is migrating the existing runtime's assumptions
about copying, construction, dispatch and library contracts.

The first class subset is non-generic record-shaped objects with fields and static/free
functions. Instance methods/constructors, inheritance, abstract classes and interface
implementation are rejected for marked classes until their receiver contracts are migrated.
Existing legacy value-model functionality in those areas remains available. Class defaults
and null are not implemented: `initobj`/default initialization rejects class creation rather
than manufacturing a copied inline object or an invalid reference. Native inline layout
and host value-record imports reject marked classes. `notreference` now rejects these
nominal classes as well as explicit reference signatures.

There are two deliberate temporary IR limitations. Existing field-wise `newobj Counter`
constructs initialized fields directly; it is not yet the standard constructor-token
operation. Existing `stfld` produces the updated receiver as an IR result and requires
`pop` when unused. These are inherited neoIL contracts, not changed meanings assigned to
standard CIL bytes. The CIL decoder still rejects field stores/construction. Resolve these
runtime/IR contracts before admitting those instructions; do not hide them by claiming the
Raven program already runs. Null and ordinary instance construction are the next necessary
work for that program.

## Validation and reproduction

`cargo test --test class_semantics --test heap_references --test gc_diagnostics --test generic_constraints --test record_inputs`
passed 36 tests. Class tests cover shared mutation, independent value copying, returning
heap objects, reference identity, nested GC roots, collection/budgets, parameter rebinding,
byref slot replacement and unsupported metadata/defaults. Existing heap, constraints and
host-input regressions also pass. No full-suite result is claimed.

The [.NET comparison](experiments/class-semantics/Program.cs), run with .NET SDK 10.0.100,
shows shared mutation 42, an unchanged value copy source 1, ordinary parameter rebinding
leaving the object at 42, and byref replacement changing it to 9. Its checked-in result is
separate evidence of expected behavior; it is not a Raven execution test.

```sh
cd docs/experiments/class-semantics
dotnet run --project Probe.csproj
```

The [Raven milestone](raven-target-experiment.md#next-milestone-useful-raven-subset) still
requires constructor/field CIL semantics, core type mapping, metadata resolution and real
System binding. Raven was not modified in this slice.
