# Type system: first executable slice

Every guest value has a type. Types can own members regardless of their data
representation; primitive types are not a second, memberless world. Object-oriented
mechanics remain foundational alongside free functions.

This implementation adds primitive type definitions, declared method ownership,
static methods, and read-only instance receiver snapshots. Inheritance, interfaces,
virtual dispatch, mutable/by-reference receivers, general generic definitions,
properties, and assembly-qualified identities remain future work.

## Primitive identity and representation

`int`, `int32`, `Int32`, and `System.Int32` resolve to one primitive signature kind,
whose canonical definition name is `System.Int32`. It has a runtime-known signed
32-bit representation and ordinary library-defined methods. Likewise, `Void`,
`Boolean`, `String`, and `Error` identify their canonical System definitions. `Error`
is a neoCLR addition, not a claim of a matching .NET primitive.

The System library declares these type definitions. `Module::type_definition` maps
a primitive or nominal record signature to its definition. Primitive signature
encodings remain compact tags, analogous in purpose to CLI primitive element kinds;
these tags refer to the canonical definitions rather than representing separate
types. The loader rejects a duplicate nominal encoding such as
`{"Named":"System.Int32"}`; producers must use the primitive signature encoding.

Type metadata distinguishes `Runtime` representation for known primitives from
`Record` representation for field aggregates. This is not a value/reference storage
flag. A primitive cannot be fabricated with `newobj`, declare recursive backing
fields of its own type, or be redefined as a record. Primitive values are created
by appropriate instructions and runtime operations. Either representation can be
held directly or placed behind the current explicit Ref abstraction.

User records have nominal names in the current linked image. Assembly identity,
namespace/name separation, and generic arity will need richer metadata before
multi-assembly identity is complete.

## Members and call syntax

```text
.type Point
    .field X int32
    .field Y int32

    .method instance Sum() -> int32
        ldarg 0
        ldfld 0
        ldarg 0
        ldfld 1
        add
        ret
    .end
.end
```

`.method static Name(...) -> T` and `.method instance Name(...) -> T` are declared
inside `.type` blocks. Each `.end` closes the innermost method/type. Free functions
use `.function` outside type blocks and have no declaring owner.

The prototype keeps executable bodies in its function table. A method has an
explicit `owner` type and `instance` flag; ownership is validated, not inferred from
its dotted name. The name remains fully qualified in this temporary metadata
format. A future CLI binary representation can separate simple member names and
declaring-type relationships without depending on textual qualification.

```text
call System.Int32::Parse(string)
call instance System.Int32::ToString()
call instance Point::Sum()
```

`::` produces an explicit declaring-type reference. Primitive aliases are accepted
there too: `int32::ToString()` identifies the same member. Existing dotted calls,
such as `call System.Console.WriteLine(string)`, remain accepted as qualified-name
shorthand. Free functions and static methods cannot duplicate the same qualified
name/parameter key. Static and instance calls are explicitly distinguished; the
runtime never guesses the form from values on the evaluation stack.

## Receiver contract and limits

Instance calls consume `receiver, parameter0, ...` in that order. Inside the method,
`ldarg 0` reads the receiver and `ldarg 1` reads the first declared parameter. The
receiver is excluded from the declared parameter signature, so parameterless
`Int32.ToString()` still consumes an Int32 receiver. Static methods retain ordinary
zero-based argument indexing. A missing or mistyped receiver Faults.

In this slice the receiver is a read-only **value snapshot**. `ldarg 0` copies that
snapshot under existing guest value semantics. `stfld` can produce a changed copy,
but does not update the original receiver. This is not yet .NET's by-reference
struct receiver mechanism. No borrowed address, lifetime token, automatic heap
allocation, Rust ownership rule, or mutable receiver is implied.

Read-only here concerns the receiver's own data, not deep immutability or method
purity. Embedded `Ref<T>` values retain shared identity, and their targets can still
be modified through existing heap operations. Use `heap.load` explicitly before
calling a method on a referenced target; it yields a snapshot. General mutable
receivers need explicit VM address/alias semantics and are deliberately deferred.

The [types sample](../examples/types.neoil) exercises record/static/instance methods,
receiver indexing, overloads, primitive conversion, and free-function entry points.

## Pointers and generics

`Ptr<T>` and `T*` are equivalent fundamental pointer signature forms, distinct from
`Ref<T>`. Nested pointers and pointer types inside constructed signatures round-trip
through metadata. This supports declarations and values such as `None<Ptr<Int32>>`;
it does not implement pointer creation, dereference, arithmetic, null-address
representation, native layout, or P/Invoke yet.

The existing Option/Result/Ref constructors are special-cased signatures, not general
generic definitions. A library-defined, reference-counted `Ref<T>` requires real
generic metadata, layout, and lifetime operations. It remains deferred, not implemented; heap allocation and executable pointers are
the immediate priority.
See [memory model layers](memory-model.md) for the separation between raw VM memory
and ownership policies.
