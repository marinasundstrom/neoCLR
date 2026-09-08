# Conditional union bindings in Neo

`if let` tests one case and introduces an immutable binding in its success block.
`let … else` keeps the successful binding available in the surrounding block; its
failure block must exit. Both evaluate the input exactly once.

```swift
let Some(product) = FindProduct(&products, "Coffee") else { return 5 }
let Ok(receipt) = Purchase(product, 2, &notifications) else { return 1 }
receipts.Add(receipt)

if let Error(error) = Purchase(product, 99, &notifications) {
    if let OutOfStock(details) = error {
        WriteLine(details.available)
    }
}
```

The [order workflow](../examples/source/order-workflow.neo) supplies these functions
and a custom PurchaseError union with InvalidQuantity and OutOfStock variants.
Source-union bindings receive the whole variant, including all its fields. Bundled
Option/Result bindings retain their existing match projection and receive the payload.
These forms reuse [union matching](neo-unions.md); they do not destructure fields.
Use `Case(_)` to discard a bound value and a bare case name for a payload-free library
case or a source case test. Unlike match arms, no second `let` appears inside parentheses.

An `if let` binding is unavailable in its else block or after the conditional.
`else if let` is supported. A guard binding is unavailable in its failure block,
and cannot shadow an active name. Failure must provably leave the path via return,
break or continue (the latter two require an enclosing loop); a conditional whose
branches both exit also qualifies. Infinite-loop divergence is not currently proven.
A falling-through failure block is a compilation error.

Bindings follow ordinary copy and managed-reference rules: extracting a reference
payload preserves its identity; extracting a value copies its fields, including any
shared references. Immutable binding does not make the referenced object immutable.
Taking an address of these scoped bindings remains restricted, as with match bindings.
Closure capture uses the existing capture cells and runtime lifetime checks.

## .NET comparison and scope

Microsoft's [C# pattern matching guide](https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/functional/pattern-matching)
(consulted 2026-09-08) shows conditional pattern variables and compiler flow checks.
Neo chooses explicit conditional-binding statements instead of implementing C#'s
broader `is` expression vocabulary. This makes the order success path linear and
keeps failure handling local, at the cost of two additional statement forms and a
limited pattern grammar. General property/positional patterns and Boolean pattern
composition remain future work.

This is compiler syntax and flow analysis. Existing carrier tests, extraction calls,
locals and branches implement it; no new runtime opcode or metadata contract is added.
It does not add nullable-state handling or strengthen arbitrary handwritten carrier IL.

## Run and validate

```sh
cargo run --locked -- run examples/source/order-workflow.neo
cargo run --locked -- debug examples/source/order-workflow.neo
cargo test --locked --test neo_conditional_patterns --test neo_control_flow --test neo_closures --test neo_match --test neo_unions --test reference_experience
```

The workflow prints Purchased: Coffee, 24, 7, 24, Out of stock, Orders complete,
and returns 0. Tests cover exactly-once effects, nested cases, early return, loop
transfers, closure captures, binding scope and rejected fallthrough/mutation.
