# Neo union declarations

Neo supports non-generic unions of declared source value types. Variants remain
separate nominal types; a union is a carrier accepting one of those types through
its constructors. This is not inheritance or a new runtime type category.

```swift
record Card(LastDigits: string)
record Cash()
union PaymentMethod(Card | Cash)

union Shape {
    case Circle(radius: int)
    case Square(side: int)
    case Rectangle(width: int, height: int)
}
```

The first form accepts existing types. The second generates ordinary nested record
cases, named Shape.Circle, Shape.Square and Shape.Rectangle. A case without a field
list, or with (), is an empty record. Inline cases currently contain fields only.

```swift
let circle = Shape.Circle(3) // type is Shape.Circle, not Shape
let shape: Shape = circle   // calls Shape(Shape.Circle)
let explicit = Shape(Shape.Square(4))
```

Assignment, return and known parameter contexts can construct the expected carrier
from an exact case type. Explicit carrier construction also takes one case value.
The generated carrier has one public single-parameter constructor per variant;
those signatures define which types it accepts. No extra runtime membership table
or conversion opcode is introduced. Ordinary non-union constructors do not gain
implicit-conversion behavior. A case may be accepted by more than one carrier.

This slice generates constructors from source declarations. Importing arbitrary union
metadata into Neo's type/constructor lookup is still separate work; it does not claim
to recognize every externally authored carrier. Existing bundled Option/Result
matching continues to use their constructor/accessor convention.

## Matching whole variants

```swift
func Measure(shape: Shape) -> int {
    return shape match {
        Circle(let circle) => circle.radius * circle.radius,
        Square(let square) => square.side * square.side,
        Rectangle(let rectangle) => rectangle.width * rectangle.height
    }
}
```

The binding is the entire variant value, so multiple fields need no new pattern
syntax. `Circle(_)` discards it; `Circle` tests the case without binding, even if it
has fields. A wildcard `_` can cover remaining cases. Expressions and statements
require coverage, and unknown, duplicate or unreachable cases are rejected.
Patterns use the case's simple type name; duplicate simple names in a union are
rejected instead of guessed. Existing System.Result's `Ok(let value)` and Option's
`Some(let value)` continue to extract their single payload; this slice does not
change those published sample conventions into whole-wrapper bindings.

## Storage, lifetime and limitations

Carriers and variants follow ordinary value-copy rules. Copying a Circle copies its
fields. Copying a variant containing Item& preserves that reference's identity and
access mode. The existing aggregate checks prohibit hiding a frame-backed reference
inside a stored case; managed heap targets remain GC-traced through carrier storage.
No automatic promotion, boxing to Object, destructor support or new cleanup policy
is added. Read [type design](type-design.md) when choosing value/reference fields.

Carrier storage uses the same private System.Value field and ordinary value.pack,
value.is and value.unpack operations as the library unions. Generated constructors,
case tests and typed extraction methods implement the contract. Calls and branches
implement matching; checked extraction faults on the wrong case. These representation
choices have existing copy/allocation costs, and no performance improvement is claimed.
Generated helper methods appear as IL frames; source call-site mapping is retained.

Only declared non-abstract source records/classes and other source union carriers
are accepted as case types in this slice. Self-cases, unknown types, duplicate case
names, empty unions and direct `default(Union)` are rejected. Select a case explicitly.
Inline variants are real nested metadata types. Generic unions/cases, imports of case
types, generic constructor inference, primitive/interface/reference case alternatives,
qualified patterns, guards and destructuring are not implemented here. Field types
can use the existing type system, including generic library types and managed references.

## Comparison and validation

The [C# union documentation](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/union)
consulted 2026-09-08 describes constructor-based conversions, exhaustive matching and
a custom marked-carrier contract whose single-parameter constructors define its cases.
We treat this as the evolving .NET 11-era comparison, not an established older CLR
requirement. The documented generated C# storage is object?, with boxing for value
cases and null/default considerations. Neo retains its existing System.Value carrier
and explicit reference model, and rejects a source union default without a chosen
variant. This reuses the [ordinary union design](unions-and-enums.md).

Generating ordinary case types and constructors fits current runtime facilities.
A dedicated runtime union kind would add metadata/opcodes; a Result-specific compiler
shortcut would not support independent user variants. We choose ordinary types, at
the cost of generated methods and existing erased-storage overhead. Exhaustiveness
and conversion eligibility are language checks; the union marker alone does not prove
that arbitrary handwritten IL obeys the carrier's semantic contract.

Run from the repository root:

```sh
cargo run --locked -- run examples/source/unions.neo
cargo run --locked -- debug examples/source/unions.neo
cargo test --locked --test neo_unions --test neo_match
```

The sample prints 1234, cash, 9, 16, 6 and returns 0. Tests exercise both declarations,
constructor signatures and nested identity, source/JSON verification, copy behavior,
whole-case matching, reference payloads, GC pressure, invalid conversions, exhaustiveness,
shadowing and runtime frame-escape rejection. General imported case inference remains
on the [case-projection plan](result-construction.md#planned-case-projection-ravens-model).

See [conditional union bindings](conditional-patterns.md) for `if let` and
`let … else`, including scope, failure-path rules and the order-workflow example.
