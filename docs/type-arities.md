# Type definitions distinguished by generic arity

The prototype now permits types with the same name and different numbers of generic
parameters. This is the first prerequisite for a non-generic Result companion alongside
the generic Result<T,E> carrier. Actual nested definitions remain the next slice.

```text
.type Container
.end
.type Container<T>
    .field Value T
.end
.type Container<T,E>
    .field First T
    .field Second E
.end
```

These are three distinct definitions and metadata IDs. `Container` selects arity zero,
`Container<Int32>` selects arity one, and `Container<Int32,String>` selects arity two.
Parameter names do not distinguish definitions: Container<T> and Container<U> are
duplicates. Wrong arity fails resolution; it is not inferred from a method's return type.

Method overload identity includes its owner, so all three definitions can declare
`Describe() -> String`. Calls use explicit owner references such as Container::Describe()
and Container<Int32>::Describe(). Bound member IDs retain their existing signature
guards, and closed generic substitution does not lose the selected definition.

## Metadata and cross-layer behavior

TypeDef continues to store its name and explicit generic parameter list. Definition
lookup uses name plus arity, then the existing definition ID identifies the resolved
row. Constructed signatures already carry the complete argument list. No arity is
guessed from display names or a backtick suffix.

Generic method **definitions** now record an explicit open constructed owner:
Container<!0> or Container<!0,!1>. Non-generic owners remain ordinary named/primitive
signatures. A method definition cannot claim a closed owner such as Container<Int32>,
or reorder/repeat its declaring parameters. Method uses specialize the open owner
normally. This removes ambiguity between a companion and its generic carrier.

Layout/field lookup, host input schemas, type identity, method access, type visibility,
scoped references and direct-module reference checks all select by arity. Same-name
types do not share private access. `[A]Container` and `[B]Container<Int32>` may resolve
to different supplied modules; a false scope or missing required reference is rejected.
Same-name, same-arity definitions still cannot coexist across the prototype load set:
general duplicate-name module isolation remains a separate limitation.

This updates the prototype's generic method-owner encoding within JSON format 3.
Reassemble older generic-method artifacts whose owner is an unqualified Named signature;
there is no ambiguous legacy-owner inference. These preview encodings and hosting APIs
remain changeable. Ordinary nongeneric owner signatures are unchanged.

```sh
cargo run --locked -- run examples/type_arities.neoil
```

The [sample](../examples/type_arities.neoil) prints `companion`, `one parameter`,
`two parameters`, `42`, then `=> Void`. It exercises same-name static methods and
generic record field access. Tests cover serialized round trips, layouts, host identity
and receiver shape, cross-module visibility/reference checks, duplicate arity and
invalid method-owner metadata.

Next, add real nested ownership for ordinary types and generic cases under a non-generic
companion, then migrate library case references. See the [companion design](nested-types.md).
This does not add inheritance, generic outer-parameter capture or union-specific IL.
