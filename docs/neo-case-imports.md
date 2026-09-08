# Source union case imports

```swift
import PurchaseError.*

union PurchaseError {
    case InvalidQuantity(quantity: int)
    case OutOfStock(available: int, requested: int)
}

func Reject(available: int, requested: int) -> PurchaseError {
    let reason: OutOfStock = OutOfStock(available, requested)
    return reason
}
```

OutOfStock is still the independent type PurchaseError.OutOfStock. The return
conversion uses the carrier constructor that accepts that variant. Importing it does
not construct the carrier, infer missing generic arguments, create inheritance, or
change value/reference semantics. `new OutOfStock(1, 2)` allocates a case on the heap;
ordinary construction creates a value. `&value` is still required to form a reference
to an existing value.

Imports apply throughout the single source file, including declarations preceding
the import. Types can appear in fields, parameters, returns, local annotations,
typeof/default operands and generic arguments. Constructors can use the short name;
qualified names remain available. Match and conditional case patterns already resolve
against their input union and do not require imports.

## Lookup rules and limits

- Duplicate imports are idempotent. Different imported cases sharing a short name
  produce an ambiguity error when that name is used, not merely when imported.
- Predefined names and file-level type declarations take precedence over imported names.
  Generic function type parameters take precedence within their signatures and bodies.
- Local callable bindings and source functions take precedence for ordinary calls.
  Importing a case never replaces an existing binding.
- The owner must be a declared source union. Existing top-level variants already
  have unqualified names; importing them introduces no additional identity.
- Only wildcard imports are supported. External union cases (including System.Result.*),
  general namespace/type imports, aliases, generic union declarations and generic case
  constructor inference remain separate work. System.Console.* retains its existing
  special static-member import.

The compiler collects declarations before resolving imported type spellings. Emitted
IL/metadata contains the same qualified case identities as explicitly qualified code.
Imports do not alter accessibility, runtime storage validity or frame escape checks.

## .NET comparison and tradeoff

Microsoft's [using directive reference](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/using-directive)
(consulted 2026-09-09) describes using static importing accessible nested types as
well as static members. Neo's wildcard syntax is a small language projection of
similar name lookup, restricted here to declared union variants. It avoids repeated
carrier qualification in the order workflow without adding runtime machinery. Costs
are possible short-name ambiguity and a declaration pass in the concept compiler.
It does not claim to implement all C# import scopes or overload rules.

## Run and validate

```sh
cargo run --locked -- run examples/source/order-workflow.neo
cargo test --locked --test neo_case_imports --test neo_unions --test neo_conditional_patterns --test neo_generics
```

The workflow now imports PurchaseError cases and still prints Purchased: Coffee,
24, 7, 24, Out of stock, Orders complete and returns 0. Tests exercise independent
case construction, carrier conversion, type identity, generic-parameter shadowing,
heap construction, duplicate/ambiguous imports and local/file-level name precedence.
