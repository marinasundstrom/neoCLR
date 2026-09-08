# Generic source records

Neo supports plain records with type parameters and positional construction:

```swift
record Box<T>(Value: T)
record Entry<K,V>(Key: K, Value: V)

let value = Box<int>(42)
let entry = Entry<string, Box<int>>("answer", value)
let heap = new Box<int>(42)
```

Constructor calls currently require explicit owner arguments. `Box(42)` is rejected;
bundled constructor inference remains available independently. Generic functions can
forward their own parameters, for example `Box<T>(value)` from `Wrap<T>`.
Type parameters shadow imported case names in field declarations. Duplicate parameters
and predefined type names used as parameters are rejected.

Field types are substituted at each constructed owner. A `Box<int>` contains an integer;
a `Box<Foo&>` contains a managed reference. Value assignment copies the record's fields,
so copying the latter preserves the same Foo reference. Member access dereferences
managed references automatically. `new Box<int>(42)` allocates managed heap storage;
`Box<int>(42)` constructs a value. References into local value storage still use `&`.
Existing runtime rules reject invalid defaults and escaping frame references.

The source compiler emits one generic definition, such as `.type Box<T>` with a
`.field Value T`. Calls and field operands use constructed owners such as `Box<Int32>`.
It does not generate a separate nominal declaration per construction, erase payloads
to Object, or add a new opcode. Generic metadata survives JSON artifact loading.

## .NET comparison and boundaries

Microsoft's [generics overview](https://learn.microsoft.com/en-us/dotnet/standard/generics/)
(consulted 2026-09-09) distinguishes generic definitions, type parameters and constructed
types, with substitution in fields and method signatures. Neo follows that metadata
model. Its deliberate difference is the existing value/reference addressing model:
`Box<Foo&>` can hold a managed reference as its type argument, subject to runtime
storage and lifetime checks. No automatic nominal reference-type classification is added.

The CLR's [runtime generics documentation](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/generics/generics-in-the-run-time)
explains runtime support for generic code. This compiler slice reuses NeoCLR's existing
generic runtime; it makes no claim about matching CLR JIT code sharing or performance.

Starting with data-only records supplies the type-substitution foundation for generic
source union cases with limited compiler complexity. The cost is a temporarily smaller
projection than C#: generic classes, methods on generic records, explicit initializers,
inheritance/conformance on generic records, source generic unions and
source constructor inference remain separate work. Nongeneric records/classes retain
their existing functionality. Type names remain unique within the file; overloading a
type name by generic arity is not introduced.

Next extend the source union model to generic carriers and independent cases, choosing
which carrier parameters each case needs. Then address methods and richer generic type
contracts; owner parameters and method parameters must remain distinct in metadata.

## Run and verify

```sh
cargo run --locked -- run examples/source/generic-records.neo
cargo test --locked --test neo_generic_records --test neo_generics --test neo_case_imports
```

The example prints 40 and 42, then returns 42. Tests cover closed metadata fields,
nested records, generic forwarding, value copies, managed-reference identity, heap
construction and interior references, readonly payloads, import shadowing, invalid
forms and lifetime rejection. These are correctness checks, not allocation benchmarks.

The initial [generic constraints](generic-constraints.md) now support runtime-enforced
`notvoid` and `notreference` clauses on these records.
