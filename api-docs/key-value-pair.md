# KeyValuePair<K,V>

Development (2026-10-20). A value record in `System.Collections` with `Key: K`,
`Value: V`, a two-argument constructor, initialization-only accessors and
`Deconstruct(out Key: K, out Value: V)`. Copying a pair copies its components;
reference components keep sharing their objects.

<a id="equality"></a>

## Equality and hashing

The compiler supplies `EquatableTo<KeyValuePair<K,V>>` and these members:

| Member | Contract |
| --- | --- |
| `func Equals(other: KeyValuePair<K,V>) -> bool` | Compares both components using the declared K and V equality contracts. |
| `override func Equals(other: Object?) -> bool` | False for null or another closed pair type; otherwise typed equality. |
| `override func GetHashCode() -> int` | Combines component hashes; equal pairs have equal hashes. |
| `static func ==(left: KeyValuePair<K,V>, right: KeyValuePair<K,V>) -> bool` | Same comparison as typed Equals. |
| `static func !=(left: KeyValuePair<K,V>, right: KeyValuePair<K,V>) -> bool` | Negates typed equality. |

Null components compare equal to null and contribute a zero component hash.
Text uses canonical equality without rewriting its UTF-8 bytes. A component whose
declared type supports EquatableTo uses that contract; otherwise it uses virtual
Object equality. Merely storing a typed-equatable object as Object does not change
Object's equality policy. Reference types without overrides retain identity equality.
Hashes are neither stable serialized IDs nor collision-free identifiers.

The development generic path currently boxes value components and resolves type
relationships. These operations can allocate. The tested native scope includes
Int32, Int64, Boolean, String, nullable references, authored equality contracts,
Index/Range and nested generic records. Floating-point/enum generic Object adapters,
ordinary structs without value overrides and generic record formatting remain gaps;
unsupported component contracts can cause a terminal Fault or native compilation
rejection. This is an implementation limit, not a different intended equality model.

For example, the tested pair consumer compares `(1, "é")` and `(1, "e\u0301")`
as equal. See [execution and allocation evidence](https://github.com/marinasundstrom/neoclr/tree/main/docs/experiments/record-generics).
The pinned renderer currently assigns both operators the same generated URL;
this reference preserves both signatures.

Index and Range now also override `Equals(Object?)` and `GetHashCode`, preserving
their typed contracts through Object and generic parameters.
