# Explicit typed value storage

The interpreter supports `System.Value`: one complete value whose static payload type
has been explicitly erased. `Value` is its short signature alias. It is a runtime-known
representation with an ordinary System type definition, like String; it is not a
universal base class, reference type category or implicit conversion target.

| Instruction | Stack effect | Contract |
| --- | --- | --- |
| `value.pack T` | T → System.Value | Store a complete T, preserving its exact closed type |
| `value.is T` | System.Value → Boolean | Exact type comparison; false exposes no payload |
| `value.unpack T` | System.Value → T | Retrieve a value copy; a mismatch raises a terminal Fault |

These operations work independently of union attributes, variants and case names;
they can also implement a heterogeneous value slot. Their spellings and canonical
`Value` type encoding are additions to prototype JSON format 3. Older readers reject
them; no CLI binary opcode assignment or compatibility is claimed. Compared with CLR
boxing/casting, this contract has no mandatory reference identity, null sentinel,
heap allocation, subtype conversion or shared mutable box. Distinct instruction
spellings make those semantic differences explicit.

Packing uses normal storage conversion: `ldc.i4 257; value.pack Byte` stores Byte(1).
It matches Byte, not Int32. Extraction applies normal stack normalization, so unpacking
Byte puts Int32(1) on the stack. Generic operands are substituted normally. Void is a
valid complete payload. Packing System.Value itself creates another explicit layer.

There is no empty or null System.Value and no fabricated default payload. Locals still
require initialization. Copying a stored record copies its value, including nested
erased values; extracting and updating a copy does not mutate the original. Pointers
and Ref handles retain existing alias/lifetime contracts: erasure does not retain an
allocation, extend frame lifetime, or acquire ownership.

The Rust interpreter uses an owned tree internally. That host implementation may
allocate, as records and strings already do; it does not create an addressable guest
heap object or promise allocation-free execution. Each pack limits payload traversal
to depth 64 and 16,384 nodes to bound recursive value shapes. These are prototype
limits, not a total memory quota. Service analysis reports `ValueStorage`; future
backends must supply its copy, identity, checking and storage behavior.

Native layout/ABI, pointers to erased payload storage, guest ownership/destruction,
and JIT/AOT implementations remain unspecified. Native storage containing System.Value
and `sizeof System.Value` are rejected. [Bounded host import](erased-inputs.md) now
validates explicit erased payloads, including nested record fields, so guest-produced
carriers can be supplied to later invocations. Pointer/Ref payloads remain unsupported;
shape validation does not certify constructor or union invariants.

## Carrier decision and proof

Use one private System.Value field for the first interpreter carrier prototype. This
avoids valid inactive T/E fields, implicit null, mandatory native heap storage, and
overlapping native layouts for arbitrary String/record payloads. This prototype
representation is not required by the union convention and is not a frozen native ABI.
Whole-value constructors suffice; addressed receivers and partial initialization are
not prerequisites for this representation.

The [ordinary carrier sample](../examples/ordinary_carrier.neoil) defines Success<T>,
Failure<E>, and Outcome<T,E> as ordinary records. Public overloaded constructors
accept permitted wrappers; the storage field is private. IsSuccess and typed accessors
use ordinary methods with the general value operations above. Success<Int32> and
Failure<Int32> stay distinct when payload types coincide. No union marker or bootstrap
union opcode participates in the sample.

A match can test the unchanged carrier, branch, then call its appropriate checked
accessor. A false test exposes no uninitialized T. Calling the wrong accessor violates
an execution contract and Faults. This initial predicate/checked-accessor lowering
does not implement .NET's out-parameter TryGetValue signature or finalize the
compiler-recognized member convention. Ordinary visibility does not prove that every
implementation honors its own contract; unsafe and trusted host boundaries still apply.

Ordinary System.Option/Result now implement the selected [member convention](union-convention.md)
in platform IL. Existing I/O/text/numeric APIs and native adapters have **not** been
migrated to them. Next: adapt host/native boundaries and their callers, then remove
bootstrap encodings with an explicit artifact compatibility break.

```sh
cargo run --locked -- run examples/ordinary_carrier.neoil
```

Output: `Success`, `42`, `Failure`, `7`, then `=> Void`. Tests also cover distinct empty
and Void-carrying records, nested erased records, independent copies, failed queries,
generic primitive normalization, pointer lifetime and depth limits.
