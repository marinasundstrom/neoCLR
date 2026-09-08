# Generic constraints

The initial runtime-enforced subset restricts the outermost form of a type argument:

```swift
record Measurement<T>(Value: T) where T: notvoid, notreference

func CopyValue<T>(value: T) -> T where T: notreference {
    return value
}
```

Neo supports these clauses on plain generic records, generic free functions and
static generic methods on nongeneric records/classes. Clauses follow the record field
list or function return type. Repeat `where` for different parameters. Explicit and
inferred method arguments undergo the same runtime checks; source record constructors
still require explicit arguments. No new operation on T is enabled by these clauses.

| Constraint | Rejected argument forms | Still allowed |
| --- | --- | --- |
| `notvoid` | Void itself | `Box<Void>`, arrays or native pointers involving Void, subject to their existing validity rules |
| `notreference` | Mutable/readonly managed references and the legacy interface-reference form | Ordinary values, including values containing references; native pointers, subject to existing pointer rules |

These are independent restrictions and may be combined. They do not imply unmanaged
layout, deep copying, immutability, defaultability or heap-only storage. In particular,
`notreference` does not inspect the object's fields. Passing an existing Foo& to a
bare T parameter infers Foo& and can violate the constraint. To deliberately pass a
copy of Foo, first bind a Foo-valued local using the existing value-read projection.
Constraints never silently change inference into a copy or a reference conversion.

## Metadata and enforcement

neoIL attaches constraints to the declaring type or method:

```text
.type Measurement<T>
.constraint T notvoid notreference
.field Value T
.end

.function CopyValue<T>(T value) -> T
.constraint T notreference
ldarg value
ret
.end
```

The directive accepts the parameter name or its zero-based index. Method constraints
refer to method parameters; type constraints refer to owner parameters. Method
constraints must precede instructions and labels. JSON stores a `generic_constraints`
list of `{parameter, kind}` rows (`NotVoid` / `NotReference`) on TypeDef and Function.
Unknown rules, duplicate rows and out-of-range indices are rejected.

Definitions are validated during loading. Concrete constructed signatures and resolved
generic calls check the argument restrictions, including substituted base/interface
signatures. Instantiated field queries check locally known substituted field contracts.
Host function resolution uses the same checks. Serialization, linking and signature
substitution retain the rows; source syntax is not required for enforcement.

Symbolic forwarding is allowed in this initial subset. An unconstrained Forward<T>
may call a constrained function using T, but resolving the concrete constrained call
must succeed before that body executes. An invalid forwarding path can therefore fault
at execution instead of source compilation. This is not a complete proof that every
possible instantiation of a generic body is valid, nor does it authorize removing
runtime checks. Closed violations visible during verification fail earlier.

This is an additive format-5 metadata extension. Unconstrained artifacts omit the new
field. Older readers use strict unknown-field rejection on these definitions and reject
constrained artifacts rather than silently ignoring their contracts. Rebuild tools when
using the new metadata. Neo also reserves `where`, `notvoid` and `notreference` as names.

## .NET comparison and next steps

Microsoft's [constraint guide](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/generics/constraints-on-type-parameters)
(consulted 2026-09-09) describes C# `where` clauses, base/interface constraints and the
class/struct distinction. Its `notnull` constraint produces warnings in a nullable
context. Neo reuses the familiar clause structure, but addressing mode is separate
from nominal type identity, and these two restrictions are runtime contracts.

The benefit is enforceable argument intent across Neo, IL and host calls without
reintroducing nominal reference/value types. The cost is new metadata and checks,
plus a bounded frontend that currently defers some forwarding failures until runtime.
No JIT or allocation improvement is claimed. A compiler-only prohibition was rejected
because it would not protect IL/artifact consumers; a recursive reference-free rule
was rejected because it would express a different object-layout contract.

Base/interface constraints remain the next constraint slice: define conformance for
value versus reference arguments, make constrained members available to Neo lookup,
and validate substitution/dispatch together. Constructor constraints, implication
checking between generic declarations and reflection APIs for enumerating constraints
remain future work. `notnull` is deliberately rejected until nullable type metadata
exists; it must distinguish nullable type arguments from uninitialized storage and
must not become an always-successful placeholder. See the
[nullability groundwork](runtime-groundwork-review.md).

## Run and validate

```sh
cargo run --locked -- run examples/source/generic-constraints.neo
cargo test --locked --test generic_constraints --test generic_metadata --test neo_generic_records --test neo_generics
```

The example prints 42 and returns 42. Tests cover source/IL contracts, JSON validation,
host resolution, symbolic forwarding, shallow reference restrictions, pointers,
substituted base and field contracts, and malformed or unsupported constraints.
