# Union convention and library fundamentals

Status: the [Preview 1 member convention](union-convention.md) is selected and ordinary
System.Option/Result implement it using explicit typed value storage and ordinary
constructors/accessors. Library APIs, native adapters and host inputs use ordinary
carriers; format 4 removed bootstrap union encodings. This direction supersedes the earlier
union case-table and dedicated-opcode design; broader tooling remains future work.

## Ordinary carrier and variant types

A **union** is a carrier that holds one value of one of its permitted variant types.
The carrier and its variants are ordinary types with value semantics. Allocation
and ownership remain separate choices. Use **enum** for named integer constants,
including flags; native overlapping storage is a separate layout capability.

Follow the attribute/member approach described in the
[.NET 11-era C# union documentation](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/union)
(reviewed 2026-09-06; preview contracts may change). Its custom union pattern marks
an ordinary type with UnionAttribute, derives alternatives from single-parameter
public constructors, and requires an object/object? Value getter. Optional typed
TryGetValue access avoids boxing when matching. A nested member-provider interface
can instead expose the construction and access members, including Create factories.
The generated C# form uses an object? backing value; adopting the convention does
not require adopting that generated representation.

For neoCLR, use ordinary custom-attribute metadata to identify a carrier and ordinary
member signatures to describe its alternatives. Construction, testing, and extraction
execute through calls, fields, and branches. The VM needs no union-specific type
category, global tag table, or wrap/test/extract opcodes. Compilers and tools can
recognize the convention without guest reflection or runtime discovery.

The [marker attribute encoding](custom-attributes.md) and library UnionAttribute in
System.Runtime.CompilerServices are implemented. The [member contract](union-convention.md)
specifies constructor and query recognition; the marker grants no special execution
semantics. Arbitrary one-parameter methods are not variant constructors. Metadata indices/tokens identify
members; source names remain authoring mappings.

## Required ordinary-type contracts

Implemented ordinary operations let the library express its carrier contract:
constructors establish a permitted variant, properties expose that value through explicit
accessor associations, and accessibility can restrict representation fields and mutation
while keeping intended constructors/readers public. Public, private, and internal are
implemented for [method calls and ordinary field operations](accessibility.md); top-level
types also support public/internal visibility. These are ordinary type-system capabilities,
not union-specific instructions. The selected convention states behavioral obligations
that the VM does not prove for arbitrary implementations.

The [constructor subset](constructors.md) establishes a whole receiver value;
ordinary methods also support [byref receivers](reference-slots.md).
[Property metadata](properties.md) supplies explicit getter/setter
associations, including Error.Message; ordinary field operations now enforce visibility.
The System carrier tests demonstrate the required value behavior. Host/native adapters
use ordinary carriers; constructor provenance at unsafe/trusted boundaries is not
guaranteed. Extensive inheritance, class virtual dispatch, reflection, and a full
runtime library are not prerequisites for this milestone.

## Intended library shapes

| Carrier | Permitted variant types |
| --- | --- |
| System.Option<T> | None, Some<T> |
| System.Result<T, TError> | Ok<T>, Error<TError> |

These are conceptual signatures, not new assembler declarations. Some<T>, Ok<T>,
and Error<TError> own their payload fields as ordinary generic records. None is an
ordinary zero-field type. Distinct wrappers preserve success/error identity even
for Result<T,T>. TError need not inherit from a universal Error type. Some<Void>
and Ok<Void> contain a real Void value; they remain distinct from None.

Case declaration syntax may later synthesize ordinary variant types and carrier
members. It adds no new runtime kind. This preserves the carrier/variant distinction
in the user's [Raven reference](https://marinasundstrom.github.io/raven/lang/spec/unions.html?q=union).
Implicit conversions, wrapping inference, and exhaustiveness checking belong to
compilers above the IL convention.

## neoCLR access and storage requirements

Typed access should be the primary path. Requiring the .NET pattern's object? Value
fallback would introduce boxing and null assumptions that do not fit neoCLR's
current model. Our eventual contract will therefore be similar, not binary identical.
No interface naming prefix is required, and no struct/class distinction determines
storage or lifetime.

TryGet members use the implemented `out(true) Case&` contract. A direct successful
Boolean branch proves the output initialized; a miss supplies no new initialization
guarantee. Runtime checks enforce required assignments. See
[reference contracts](reference-slots.md) for aliasing and forwarding, and
[verification](verification.md) for conservative proof propagation.
Constructed carriers must preserve the selected variant and value; their queries
must agree and must not change the active alternative. Absence is an explicit None
value, not an implicit null or uninitialized state. A zeroed allocation is not
necessarily a constructed carrier.

Storage remains an ordinary type implementation detail. The interpreter prototype
selects one private [System.Value field](value-storage.md), holding a complete typed
value without inactive alternatives. A discriminant plus payload
storage is a possible implementation, but the convention mandates neither an integer
tag nor its numeric values. Generic records alone do not solve storage for alternatives
without valid defaults: storing every alternative as an initialized field is not a
general solution. Before implementing native overlapping storage, specify alignment,
active initialization, copying, and pointer tracking. Do not introduce mandatory heap
allocation or ownership merely to fit every variant into a uniform slot.

A convention does not by itself enforce these invariants against arbitrary IL.
Ordinary member visibility, verification, and library implementation must provide
the available guarantees. Incorrect library implementations can violate the pattern;
a marker attribute is not a runtime proof. Extraction behavior must be specified by
the API without exceptions. Recoverable failure uses the platform's error model;
Fault remains for violated execution contracts.

## Implementation sequence

1. Completed: indexed generic references, arity checks, and field substitution.
2. Completed: ordinary closed generic record construction and field operations,
   preserving value copies, storage conversions, and exact closed type identity.
3. Completed: IL members on generic definitions with substitution of signatures/bodies
   for closed owners. Keep unrelated generic method features separate.
4. Completed: marker metadata and the ordinary constructor/property/access/storage
   foundation, with a selected [constructor/query convention](union-convention.md).
5. Completed: fully qualified System carrier and wrapper types in platform IL, with
   tests for None/Some, Ok/Error, Void, nesting, failed queries and independent copies.
6. Completed: library, native adapters, host inputs, samples and tests use ordinary
   constructed carriers, constructors, queries and accessors.
7. Completed: format 4 removes the six bootstrap union instructions, special signature
   categories and Union value representation. Old artifacts are rejected; reassemble source.

There are no union-specific IL instructions in the platform contract. Integer-backed
enums remain a separate milestone. Reflection, GC, reference counting, runtime async
and high-level pattern syntax are not prerequisites for this convention.

## Neo projection

[Non-generic declarations](neo-unions.md) now support existing source cases and
inline nested record cases, exact constructor-based conversions and whole-variant
matching. The [remaining case-construction plan](result-construction.md#case-projection-ravens-model)
covers imported cases, generic constructor inference and broader metadata discovery.
