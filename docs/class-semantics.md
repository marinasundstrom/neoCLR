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

The current class subset is record-shaped objects, including closed generic instances,
with fields, direct instance methods and constructors, plus public implicit interface
implementations. Inheritance, abstract classes, class virtual dispatch and methods with
their own type parameters on nominal classes remain unsupported.
See the [nominal interface groundwork](raven-interface-contract.md#implemented-nominal-interface-groundwork-2026-09-12)
for object-reference interface views, no-result contracts and remaining conversion limits. Existing legacy value-model
functionality in those areas remains available. Class defaults now produce typed null references, and `initobj` resets a class slot.
Constructor allocation initializes fields using supported managed defaults, including
nominal class-reference fields and ordinary `arrayref<T>` fields. The latter can be
assigned `newarr T` inside a constructor; see [array migration](managed-arrays.md#ordinary-array-references-and-migration-2026-09-12).
General `ldnull` projection remains separate work.
Native inline layout and host value-record imports reject marked classes. `notreference`
rejects nominal classes as well as explicit reference signatures.

## Constructor and field-store follow-up (2026-09-12)

`newobj instance Counter::.ctor(Int32)` allocates the class on the managed heap, initializes
supported fields, and calls its no-result constructor with the object as argument zero.
The constructor's `ret` requires an empty stack. On success, **newobj**, not the constructor
return signature, supplies the original object reference to the caller. Faults do not
produce a constructor result. The object remains a GC root throughout construction even
if the body rebinds its own `this` slot.

Ordinary non-virtual instance methods now accept the class reference directly. They can
return a value or use a no-result signature. Class `stfld` consumes its receiver and value
without producing a stack result, enforced by the typed verifier and runtime. Remove the
`pop` previously required after a class field store; this is a breaking experiment change.
Legacy value-record/byref field-store behavior is unchanged and still needs migration
before its standard CIL forms can execute.

This follows Microsoft's [`newobj`](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.newobj?view=net-10.0)
and [`stfld`](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.stfld?view=net-10.0)
contracts (consulted 2026-09-12). The benefit is preserving constructor and field-store
stack behavior directly in the runtime instead of compensating in a compiler. It requires
separate tracking of the pending newobj result and body return convention. No new opcode
or external metadata encoding is introduced; the internal no-result marker remains an
implementation detail. The old field-wise `newobj Counter` spelling remains available
for existing neoIL, alongside constructor-token operations.

The CIL decoder recognizes standard `newobj`, `ldfld` and `stfld` bytes and preserves their
MethodDef/Field/MemberRef tokens. It checks token kinds only: resolving constructor identity,
field ownership, signatures and access still belongs to metadata binding. This is not a
claim of executing Raven binaries. Class constructor chaining, including Object's base
constructor, remains explicitly rejected; do not silently discard those calls on import.

Run the [example](../examples/class_construction.neoil), which prints 42:

```sh
cargo run -- run examples/class_construction.neoil
```

System.Console still returns an inhabited Void internally, so the example retains `pop`
after that library call. That boundary is separate from the class constructor/field-store
contract and still needs library migration.

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

The constructor follow-up passed 52 focused tests. They cover default primitive fields, direct instance calls, empty
stack enforcement, failures, heap limits, GC during construction, unsupported chaining,
and the Console example. Run `cargo test --test class_constructors --test class_semantics
--test constructors --test constructor_chaining --test no_result --test cil` as one command.

## Reference defaults (2026-09-12)

Nominal class defaults now produce a typed null reference. This is a valid reference-slot
state, distinct from uninitialized storage, a managed byref or a zero-valued record.
Constructor field defaults and `initobj` on a class slot use it without allocating an
object. Resetting a slot does not modify other aliases. Null handles are not GC roots;
`ref.eq` compares them, and field access through null faults. This adopts ordinary CLR
class-reference default behavior for the experiment; earlier explicit-nullability ideas
remain historical design exploration rather than an implemented non-null class guarantee.
Generic nullable values and nullable metadata are separate work. `ldnull` projection and
String/interface/array classification are not implemented by this slice.

The reference-default follow-up passed 22 focused tests across class defaults, constructors
and reference semantics. The next acceptance result must be Raven-emitted IL calling the
real runtime library, not merely a host-generated method fixture.
