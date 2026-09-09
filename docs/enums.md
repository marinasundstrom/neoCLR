# Enums, flags and named constants

Enums provide nominal option values. BindingFlags is the first runtime-library use.
The initial preview supports Int32-backed definitions:

```swift
flags enum Access {
    None = 0
    Read = 1
    Write = 2
    Both = 3
}
enum Status : int { Pending, Ready, Failed = -1 }

let access = Access.Read | Access.Write
let canRead = access.HasFlag(Access.Read)
let bits = access.Value
let unknown = Access.FromValue(42)
```

The underlying type defaults to int. An omitted member value starts at zero, or is
one greater than the previous member; overflow is rejected. Signed decimal literals
are supported. Different names can share a value. Empty enums are allowed. Definitions
are nongeneric and cannot participate in inheritance or interface conformance in this
slice. Other integer widths, expressions in member initializers, casts, formatting,
parsing and general constant declarations remain future work.

`flags` is contextual at a declaration; a local may still be named flags. `enum` is
reserved. The generated helper names FromValue, Value, get_Value, Or, And, Xor, Not,
HasFlag and Equals are currently reserved as source enum member names.

## Values and operations

An enum is distinct from int and from every other enum. Parameters, locals, fields,
arrays and generic arguments keep that identity. Default initialization selects the
zero bit pattern even if it has no name. Every Int32 bit pattern is representable;
flags metadata does not restrict values to named bits or change initialization.

Neo supports same-enum `|`, `&`, `^`, `~`, `==` and `!=`. HasFlag(other) tests whether
all bits in other are present; HasFlag(zero) is true. Ordinary enum declarations also
support these operations. Flags is a designation of intended use, not an operator
permission or a promise that every combination makes sense to an API.

FromValue(int) and Value make integer conversion explicit. There is no implicit
integer-to-enum conversion, including a literal zero. The typed methods Or, And, Xor,
Not, HasFlag and Equals remain callable directly. Named factories such as Access.Read()
are also generated, but samples use the constant spelling Access.Read.

Integer `|`, `&`, `^` and `~` are now exposed in Neo as well. Binary precedence follows
the C# ordering: logical OR, logical AND, bitwise OR, XOR, AND, equality, relational,
additive and multiplicative. Use parentheses for mixed tests and bit masks.

Enums follow existing value-copy and managed-reference rules. Borrowing an enum value
requires &, and member access through that reference dereferences automatically.
No heap allocation, Object base class or boxing conversion is required by the contract.
The interpreter still uses its ordinary record value representation and helper calls;
this is not a claim of allocation-free execution or CLR-equivalent machine code.

## Metadata and execution

A type carries optional enum_info metadata: underlying, flags and named integer
members. Format 5 gains this additive metadata; older tools reject it and must be
updated before reading enum artifacts. Existing artifacts without it still load.

```text
.type Access
.enum Int32 flags
.literal None 0
.literal Read 1
.field private Bits Int32
.end
```

The loader validates an ordinary record representation with exactly one private Int32
field, no base/interfaces, no generic parameters, no abstract designation and no
custom packing/size. It rejects duplicate/invalid member names and unsupported
underlying types. Enum types cannot be base classes. Named constants are metadata,
not writable static fields.

For a validated enum, `newobj Access` constructs its value from one Int32 on the stack.
This scalar construction does not grant permission to load/store its private payload
field. Non-enum record construction keeps its existing field-access rules. Wrong enum
arguments fail typed verification and checked execution even without verification.

Neo reads named literal values from metadata and emits ldc.i4 followed by newobj.
It does not execute a user-replaceable named factory to evaluate a constant. Generated
methods implement integer bit operations and return the same enum type; ordinary call
signatures enforce enum identity. No new enum-specific execution opcode is introduced.
Handwritten IL enum definitions need not contain Neo's convenience methods; these are
frontend/library API conventions rather than loader requirements.

## Reflection and BindingFlags migration

Type.IsEnum distinguishes an enum from its managed-reference/array forms.
GetEnumUnderlyingType() returns typeof(int). GetEnumNames() returns an owned String[]
snapshot ordered by unsigned integer magnitude, preserving declaration order for
aliases. The latter two calls fault when used on a non-enum. General FieldInfo literal
queries and GetEnumValues are not yet projected; names are available through
GetEnumNames and host enum_info metadata.

BindingFlags retains its existing bits: Default=0, DeclaredOnly=2, Instance=4,
Static=8, Public=16, NonPublic=32. Its existing FromValue, get_Value, Or and named
factory methods remain available. New code can use:

```swift
let flags = System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Instance
let fields = typeof(Item).GetFields(flags)
```

Its integer layout is unchanged, but reflection now identifies it as an enum and
additional helper methods appear in method introspection. This does not expand
reflection filtering: unsupported bits still fault, and member queries remain
as documented in [reflection](reflection.md#filtering). Rebuild tools/artifacts for
new metadata. Do not treat unnamed enum values as automatic API-valid options.

## Comparison and provisional choices

The [C# enum specification](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/enums)
and [enum reference](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/enum)
(consulted 2026-09-09) establish nominal identity, integral backing, named constants,
zero defaults and bitwise operations. Neo follows these behaviors but initially supports
only Int32 and explicit FromValue/Value conversion, rather than C# casts and implicit
zero conversion. It does not require a System.Enum/Object hierarchy.

The [HasFlag contract](https://learn.microsoft.com/en-us/dotnet/api/system.enum.hasflag?view=net-10.0)
includes the zero case. [GetEnumNames](https://learn.microsoft.com/en-us/dotnet/api/system.type.getenumnames?view=net-10.0)
sorts by unsigned magnitude; .NET leaves alias order unspecified, whereas this preview
chooses stable declaration order. The [BindingFlags API](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.bindingflags?view=net-10.0)
is the numeric/API baseline, with our intentionally narrower supported filters.

A bare integer alias would lose type identity. A new runtime scalar category would
require changes across value storage, copying, native layout and debugger inspection.
The provisional implementation instead validates a one-field enum representation and
uses existing construction/call/integer instructions. This supplies useful contracts
now, at the cost of helper methods and current record-representation overhead. Future
JIT/scalar specialization requires measurement; no performance advantage is claimed.
The flags designation is explicit metadata here rather than a required FlagsAttribute
instance. Broader attributes and Enum APIs can be added independently.

Enum literals are the first constant-metadata use. General constants should extend this
foundation with an agreed set of literal types and reflection contracts, rather than
become writable slots or a second unrelated enum mechanism. Neo should evaluate and
validate constant expressions; the runtime should validate encoded literal types and
ranges. Ordinary immutable bindings remain a language feature. These are plans, not
implemented const declarations or general literal fields.

## Run and validate

```sh
cargo run --locked -- run examples/source/enums.neo
cargo test --locked --test enums --test reflection --test neo_generics
cd docs/experiments/enums-dotnet
dotnet run
```

The Neo sample prints read, 3, 1, enum and returns 0. The pinned .NET 10 probe exercises
bit combinations, unnamed/zero values, HasFlag, name ordering and BindingFlags bits.
Regression tests cover metadata/artifact validation, checked nominal calls, constants,
private payload access, generic/reference behavior and reflection queries.
