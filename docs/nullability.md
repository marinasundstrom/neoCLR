# Explicit nullability as a type-signature characteristic

Direction agreed 2026-09-08; not implemented. Null is a special runtime state.
It does not mean that the payload is numerically zero or that its bytes are zeroed.

## Type contract and slot state

Nullability belongs to the type signature used by a local, parameter, field, return,
array element or generic argument. The nominal Counter definition is unchanged.
Each use selects Counter or its nullable form. A separate Boolean on every kind of
slot would not compose through return values and nested signatures.

The slot holds the current state, validated against that signature:

| State | Meaning |
| --- | --- |
| Uninitialized | No assigned value; reading is invalid |
| Present(T) | An assigned T, including valid zero/default-valued payloads |
| Null | An assigned null state, legal only under an explicitly nullable signature |

Zero, false and a zero-filled record remain ordinary present values. A null check must
not infer absence from their contents. Physical representation can use a discriminator,
a suitable spare representation or another implementation technique that preserves
these semantics; no byte-zeroing contract is implied.

Managed references remain non-nullable unless explicitly qualified. Values may also
be explicitly nullable; this does not introduce a value-type/reference-type classification.
The runtime must carry and validate the characteristic across storage and APIs.
Binding immutability remains a language concern, as recorded in [mutability](mutability.md).

## Composition matters

Use N(T) here as explanatory notation, not as a proposed library wrapper or implemented
Neo syntax. The following are different contracts:

- N(Counter): a Counter value whose storage can be present or null.
- N(Counter&): a reference value that can be present or null. Clearing it removes
  this reference; another alias can still keep the Counter alive.
- N(Counter)&: a non-null reference to storage that can contain a Counter or null.
  Clearing that addressed nullable value changes the shared storage.
- N(readonly Counter&): a nullable readonly reference. Replacing the containing slot
  is distinct from mutating the target through that reference.
- readonly N(Counter)&: a readonly view of nullable storage. It may observe a null
  state written through another writable alias, but cannot clear that storage.

The runtime must preserve qualifier placement through generic substitution, reflection
and array element signatures. Source punctuation such as ? and grouping is not settled
and must not be added to the implemented grammar before a compiler projection exists.

## Clearing storage

A clear/reset-to-null operation assigns the null state to explicitly nullable storage.
It is different from local.reset, which currently resets declaration storage to
uninitialized, and from constructing a present default value with initobj.

The old payload must stop contributing GC edges through this slot after a successful
clear. This does not promise immediate collection, disposal or destructor execution.
A null payload must never be traversed by the collector or accessed as a present T.
Copying a nullable value copies its state and, if present, its payload under normal
value/reference rules.

A non-nullable destination rejects null at the boundary. Reading the payload of null
must fault predictably unless a defined checked operation establishes presence.
The exact check/narrow operations, effects of intervening alias writes, and behavior
when clearing owned storage with live interior references need a dedicated slice.
Existing lifetime protection must not be bypassed by calling an operation reset.

## Optionality and the library

Prefer Option<T> for optional results and domain alternatives. Nullable storage has
a narrower primary purpose: explicit state that may be cleared or reset to null.
Do not automatically convert Option.None into null, infer nullability from a reference,
or treat uninitialized slots as Option.None.

The representation and any explicit conversions between Option<T> and N(T) remain
design decisions. Nested combinations must retain their meaning if supported.
Native pointers retain their current interop behavior for now; reconciling Ptr/null
operations with future qualified signatures needs an explicit migration and ABI decision.

## .NET comparison and implementation gate

Sources checked 2026-09-08:

- [.NET nullable value types](https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/null-safety/nullable-value-types)
  already distinguish a nullable value's null state from the underlying default value.
  Nullability in .NET is therefore not limited to reference types.
- [C# nullable reference types](https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/null-safety/nullable-reference-types)
  use annotations and static analysis without making string and string? different
  runtime types.

neoCLR's proposed improvement is one explicit runtime signature characteristic across
its value and managed-reference addressing forms, with non-nullable defaults and
Option-based optionality. Costs include payload-state representation, checked access,
GC behavior, substitution and metadata migration. It does not imply a faster runtime
or automatically safer native pointers.

Before implementation, specify clearing and alias behavior, checked access, defaults,
copying, GC tracing and the reflection contract. Acceptance must include present zero
versus null, null versus uninitialized, nested qualifier placement, removal of old GC
edges, writable/readonly access, and failure when null enters a non-nullable signature.

## Construction and defaultability boundary

The reference-holding slot accepts null only when its declared type signature permits
null. Non-nullable references must receive valid references; defaulting them to a zero
address or other invalid reference is forbidden. An uninitialized slot is not a legal
null/default value and cannot be read or complete a required constructor field.

The current [Neo default(T) projection](classes-and-defaults.md) uses checked initobj
without adding nullable signatures. Once nullable signatures exist, their null default
must be an assigned null state, distinct from a present zero and uninitialized storage.
The runtime must enforce this through generic substitution, calls/returns, fields,
arrays and host/artifact boundaries; compiler diagnostics are additional assistance.
Constructor synthesis and how much initialization a language inserts remain language
policy. Future defaultability metadata/constraints should be considered alongside
nullable defaults, rather than weakening non-nullable reference validity.
