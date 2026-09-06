# Union convention and library fundamentals

Status: design direction, not an implemented custom-union facility. Indexed generic
references and field substitution are implemented. Option and Result still use
bootstrap runtime encodings and instructions. This proposal supersedes the earlier
union case-table and dedicated-opcode design.

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
System.Runtime.CompilerServices are implemented. The union member contract remains
to be finalized; the marker grants no special execution semantics. Constructor
and factory recognition must be specified explicitly rather than treating every
one-parameter method as a variant constructor. Metadata indices/tokens identify
members; source names remain authoring mappings.

## Intended library shapes

| Carrier | Permitted variant types |
| --- | --- |
| System.Option<T> | None, Some<T> |
| System.Result<T, TError> | Ok<T>, Err<TError> |

These are conceptual signatures, not new assembler declarations. Some<T>, Ok<T>,
and Err<TError> own their payload fields as ordinary generic records. None is an
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

A TryGetValue-like member needs an explicit output contract. Addressable locals,
byrefs, and initialization rules must be settled before copying the .NET out pattern.
An unsuccessful query must not expose an uninitialized or fabricated T. Do not make
implementing Option depend recursively on already having Option-based extraction.
Constructed carriers must preserve the selected variant and value; their queries
must agree and must not change the active alternative. Absence is an explicit None
value, not an implicit null or uninitialized state. A zeroed allocation is not
necessarily a constructed carrier.

Storage remains an ordinary type implementation detail. A discriminant plus payload
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
4. Marker custom-attribute metadata is implemented. Extend attributes as needed and
   supply the member/access/storage fundamentals required by the convention.
   Finalize its construction and typed-query contract.
5. Implement carrier and variant types in the platform-written System library;
   test None/Some, Ok/Err, Void, nested carriers, failed queries, and independent copies.
6. Migrate bootstrap Option/Result signatures and some/none/ok/err/is.case/ldcase
   deliberately, with an explicit serialized-module compatibility decision.

The bootstrap operations remain supported until that migration; they are not the
proposed general union ABI. Integer-backed enums remain a separate milestone.
Reflection, GC, reference counting, runtime async, and high-level pattern syntax
are not prerequisites for establishing this convention.
