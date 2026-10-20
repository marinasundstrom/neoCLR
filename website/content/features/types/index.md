# Reference types and value types

Choose a type by what copying it should mean. A class gives an object a shared
identity. A struct represents a value whose fields are copied when the value is
assigned or passed by value. neoCLR follows the .NET reference/value model, with
Raven providing the declarations and compiler-generated record members.

| Operation | Reference type, such as a class | Value type, such as a struct |
| --- | --- | --- |
| Assignment or by-value argument | Copies a reference to the same object | Copies the value's fields |
| Mutation through one copy | Other aliases see the same object's changes | The other value's own fields are independent |
| Reference fields inside it | Refer to other objects | Still refer to the same objects after copying |
| Default equality | Object identity for ordinary classes | Same runtime type and equal instance fields |
| Conversion to Object | Retains object identity | Boxes a copy in managed storage |

Neither kind of assignment performs a deep clone. Copying a struct containing an
array reference does not copy the array's elements. Readonly access prevents writes
through that access path; it does not make every referenced object immutable.

## A small value, with an optional Object view

This struct represents two numbers:

```raven
{{STRUCT_DEFAULT_DECLARATION}}
```

Two independently constructed values can have the same contents:

```raven
{{STRUCT_DEFAULT_USAGE}}
```

`first.Equals(second)` and `boxed.Equals(another)` compare the fields. The two
separate boxes have different identities, so `Object.ReferenceEquals(boxed, another)`
is false. Equal values have equal hash codes, but a matching hash does not prove
equality. Never persist hash codes or use them as unique identifiers.

Ordinary structs use runtime-provided default equality and hashing. A custom
`Equals(Object)` override takes precedence; pair it with a matching `GetHashCode()`.
Reference fields use their own equality contracts: a class without an override
compares by identity, while a class with an override can compare its contents.

`EquatableTo<T>` is an explicit, user-implemented typed contract. It allows a
comparison to accept `T` directly instead of requiring an Object argument. It does
not make conversion of a struct into an interface object allocation-free. Typed
and supported constrained calls can avoid boxing; an Object or interface conversion
can still need a box.

## Records express compiler-generated behavior

A record class is still a reference type; a record struct is still a value type.
Raven generates their equality and other supported record members. That generated
behavior is separate from the runtime's ordinary struct fallback. Default struct
value equality does not automatically declare `EquatableTo<T>` or add `==` and `!=`
operators to every ordinary struct.

String is a reference type with content equality. neoCLR deliberately uses canonical
Unicode equality over preserved UTF-8 storage, so string fields follow that text
contract rather than .NET's default ordinal String equality. See
[strings and UTF-8](../strings/) for explicit ordinal operations.

## Allocation follows storage and lifetime

“Value type” does not mean “always on the stack.” A value can live in a local,
a register, an array element, a class field or a managed box. A value embedded in
an existing object does not require its own object allocation. References inside
values still keep their targets reachable by the GC.

Small values can avoid temporary wrapper objects, but copying large values has a
cost. Boxing also allocates storage and copies the value. Choose classes when shared
identity and ownership fit the problem, and structs when independent copied values
fit it. Allocation for a useful result or owner is appropriate; repeated temporary
boxes and copied buffers deserve attention.

`Slice<T>` and `ReadOnlySlice<T>` are borrowed value views with additional lifetime
rules. `Memory<T>` and `ReadOnlyMemory<T>` retain an owner and can cross asynchronous
suspension. Copying any of these views shares the underlying storage rather than
copying its contents. See [slices and memory](../memory/).

## Development coverage

Ordinary struct defaults are development functionality. The example and nested,
generic, empty and reference-field cases pass interpreter, JIT and AOT checks.
Floating fields also pass these checks, including NaN and signed-zero equality.
Int32-backed enum fields use exact enum type and underlying-value equality.
Enum display now supports names, flags and unnamed numeric values.
Numeric primitives now support Formattable and boxed floating-point display. This
does not yet cover every .NET field shape or dispatch path. The [Object API guide](/docs/objects.html) records the
supported contracts and [implementation evidence](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/struct-defaults/README.md)
tracks execution-mode validation.

[Object API](../../docs/objects.html) · [Garbage collection](../gc/) ·
[.NET comparison](../../comparison/)

Development qualification now covers signed, unsigned and native-width integer equality, hashes and decimal display through Object, including narrow sign/zero extension and UInt64 boundaries. These use existing scalar boxes and formatting services; Object conversion still allocates a box. Floating boxed display uses the shared numeric formatting implementation; custom formats and providers remain gaps.

Development primitive interface support preserves each scalar value type's declared
interfaces when boxed. Interface calls borrow that box's payload and invoke the same
value method used by direct calls; converting to an interface does not create a second
box. This includes narrow/unsigned/native-width integers, floating values, Boolean and
Char. Exact generic arguments remain significant, and the runtime does not invent
undeclared equality interfaces. `Int32` and `Int64` also implement `Formattable` for
bounded culture-aware D/F/N formatting; other numeric format coverage remains a gap.

The development primitive library explicitly implements `EquatableTo<T>` for all
signed, unsigned and native-width integers, `Single`, `Double`, `Boolean` and `Char`.
Typed `Equals` calls do not require boxing; storing a value in an interface does.
Floating `Equals` treats NaNs as equal and positive/negative zero as equal, matching
Object equality and hashing; IEEE `==` retains its NaN behavior. Char equality keeps
canonical grapheme semantics. This does not synthesize the interface for user structs.

Numeric primitives implement `Formattable`: integers support bounded G/D/F/N;
Single and Double support G/R/F/N/E, shortest round-trip output and explicit precision.
`Parse(text, culture)` accepts the corresponding decimal/group conventions and
validates grouping; ordinary `Parse(text)` stays invariant. See [formatting and
culture](../time/) for the supported profiles and limitations.
