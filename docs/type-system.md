# Type system: first executable slice

Every guest value has a type. Types can own members regardless of their data
representation; primitive types are not a second, memberless world. Object-oriented
mechanics remain foundational alongside free functions.

This implementation adds primitive type definitions, declared method ownership,
static methods, and instance receiver snapshots. The current platform also supports
type generics and [explicit borrowed interface dispatch](interfaces.md). Inheritance,
class virtual dispatch and by-reference receiver writeback remain future work. [Property metadata](properties.md)
now explicitly associates ordinary getter/setter methods with declared signatures.
[Accessibility](accessibility.md) enforces public/internal/private method calls and
ordinary field access, plus public/internal top-level type visibility.

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

Parameters may be named, for example `Parse(string value)`, and locals use
`.local Point point`. Names are optional metadata, not part of type identity or
overload selection. `ldarg value` and `ldarg this` resolve to numeric indices.

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
through metadata. Pointer values now contain native addresses. Explicit heap
allocation/free, pointer casts, byte offsets, field addresses, and indirect access
are implemented for the supported layouts; see [heap and pointers](heap-and-pointers.md).
Pointers carry no ownership. A first native interop subset supports scalar/pointer calls; direct guest access to
externally supplied memory remains unimplemented. See [native interop](native-interop.md). Value copying a record copies its fields, including pointer addresses;
it does not copy the pointed-to storage or acquire ownership.

The existing Option/Result/Ref constructors are special-cased signatures, not general
generic definitions. A library-defined, reference-counted `Ref<T>` requires real
generic metadata, layout, and lifetime operations. It remains deferred, not implemented. The native heap/pointer subset does not depend on this wrapper.
See [memory model layers](memory-model.md) for the separation between raw VM memory
and ownership policies.


Native-sized integers use canonical System.IntPtr/System.UIntPtr type identities,
with nint/nuint aliases. Their arithmetic and native storage are executable; see
[native integers](native-integers.md). Basic generic record definitions, constructed references, and field substitution are
now available; see [generic metadata](generic-metadata.md). Generic execution remains pending.

Future modeling should place more emphasis on types to express contracts. Ref<T>
may express reference-counted ownership, while Ptr<T> expresses address access without
ownership. Such abstractions should build on the core type system instead of restoring
a permanent class/struct allocation distinction. The exact ownership operations and
the division between library code and runtime support remain open.

Fixed-width integer definitions now cover SByte/Byte, Int16/UInt16, Char, UInt32,
Int64/UInt64. Their declared storage types are separate from integer evaluation-stack
categories; see [integer storage](integer-types.md). Char remains a UTF-16 code unit
without deciding String's encoding. UTF-8 text is a [proposed direction](text-model.md).

Single/Double are also canonical System primitives. Both use one internal F stack
category (binary64 in this interpreter), with explicit precision at storage and
conversion boundaries. See [floating-point rules](floating-point.md).


Parameter, local, and field identifiers are context-specific mappings to indices;
they need not match a higher-level language's source identifiers. Parameter and
local scopes are separate, and each declaring type defines its own field scope.
Qualified field aliases such as `Point::X` resolve during assembly to the same
numeric operands as explicit field indices. See [identifier mappings](neoil.md#identifier-mappings-and-field-aliases).

## Future exploration: nullability on declarations

Explore explicitly marking locals, parameters, fields, and properties as nullable,
with non-null as the default contract, instead of making nullability part of type
identity. Under this proposal, two declarations could refer to the same type while
having different permissions to contain null. This is a direction to investigate,
not implemented syntax, metadata, or verifier behavior.

A candidate is to leave enforcement to compilers and tooling. Persist annotations
on declaration targets so separately compiled consumers can check contracts; a
frontend could reject violations rather than merely warn. The runtime would not
need to enforce these annotations as part of type identity. Annotation encoding,
defaults for unannotated imports, and any boundary checks remain undecided.

Use a consistent annotation model without separate class and struct rules. A
nullable annotation should not automatically rewrite a value's declared type to
Nullable<T>. This does not by itself supply a null representation: an inline value
whose bit patterns are all valid still needs a distinguishable state if actual null
storage is allowed. Whether that uses additional storage, an explicit indirection,
or is unsupported for a particular representation remains a separate decision.
Tooling-only annotations cannot make that physical distinction disappear. No
wrapper, boxing rule, or universal null representation is selected here.

This must remain separate from allocation and ownership. A nullable declaration
would not select heap allocation, garbage collection, or reference counting. The
library principle also remains: use Option<T> for semantic absence; reserve null
for an intentional null-reference state.

Questions to resolve before implementation:

- Which values can carry null, especially raw pointers and future reference wrappers,
  without introducing a class/struct distinction or a null state for every value?
- How should declaration annotations be encoded, including return values, property
  accessors, generic substitution, and nested generic arguments?
- Should non-null contracts be enforced by the verifier, runtime checks, a frontend,
  or a combination, and what happens at native and host boundaries?
- How do definite initialization, copying between declarations with different
  annotations, and narrowing after null checks preserve those contracts?
- How should annotations participate in method compatibility and overload resolution
  while remaining separate from type identity?

These choices should be considered alongside future frontend and metadata design;
they do not require adding nullability to the current implementation slice.

## Future exploration: inheritance openness and closed hierarchies

Make inheritance policy explicit when inheritance is introduced. Distinguish whether
an individual type permits derivation from whether an entire hierarchy is closed to
unlisted subtypes. These declarations should remain independent of value semantics,
allocation, and ownership; they must not restore a class/struct memory-model distinction.

A closed hierarchy could allow derivation among a known set of permitted types while
preventing arbitrary extensions. Its permitted members and enforcement boundary need
an explicit contract. A type that cannot be derived from is a different case from a
base type whose hierarchy permits only declared alternatives.

Before implementation, decide the default openness, where permitted subtypes are
listed, whether closure is relative to a module or an explicit list, and how indirect
derivation and separately compiled modules are validated. Also define how versioning
and generic instantiations affect closure. A compiler may use a verified closed set
for exhaustive analysis; an annotation alone must not imply such a guarantee.

This is deferred design work, not implemented inheritance metadata or syntax. Ordinary
union carriers remain a separate convention and must not require inheritance or a
closed hierarchy to represent their variants.

See the [proposed slot-reference design](reference-slots.md) for typed reference
parameters, out assignment and explicit reference receivers. This is planned work,
not a change to the currently implemented pointer or receiver semantics.
