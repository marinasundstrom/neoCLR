# Temporary explicit typed value storage

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
`Value` type encoding are additions to prototype JSON format 4. Older readers reject
them; no CLI binary opcode assignment or compatibility is claimed. Compared with CLR
boxing/casting, this contract has no reference identity, null sentinel, subtype
conversion or shared mutable box. The interpreter does allocate host heap storage
when packing; this is an explicit cost, not an allocation-free operation. Distinct instruction
spellings make those semantic differences explicit.

## Retirement decision

System.Value is a temporary interpreter facility, not a permanent platform type.
Remove it, its short alias, value.pack/value.is/value.unpack, and their dedicated
runtime handling once ordinary type storage and explicit references can support the
library payloads that currently depend on them. Do not preserve the mechanism under
a different name or make a future System.Object an implicit arbitrary-value box.

The intended foundation is typed values, native storage and explicit pointers.
Void* deliberately carries no concrete payload type or ownership policy. A library
using it must maintain its own tag or descriptor and lifetime contract. Higher-level
languages may enforce safe/unsafe distinctions or provide ownership abstractions;
neoCLR should retain intentional access to the underlying low-level operations.

Before removal, establish usable native representations and explicit lifetime/copy/
release contracts for current String, error and nested carrier payloads. Migrate
System.Option/Result, native bindings, host inputs and samples together, preserving
case extraction and typed Error behavior. Then remove the obsolete encodings with
an explicit artifact version break and update the opcode/service documentation.
The borrowed pointer sample proves only the native-layout subset, not this whole
migration. General inheritance or a full class library need not precede removal if
the required storage and ordinary type contracts can be supplied independently.

This decision does not remove the current implementation immediately or silently
expand Preview 1 into a complete object model. The sections below document the
existing temporary behavior, including its allocation costs.

## Slot model

Every local and field has a declared slot type. A normal slot stores that type's
complete value with the usual copy and initialization rules; it does not need a
common base type. `System.Value` is the explicit escape hatch for a slot that must
carry values of different, runtime-known types. Packing into it and unpacking from
it are visible operations, so the VM never silently boxes a value merely because a
slot is wide enough.

At a lower level, a native backend may represent a slot as an address and a layout
descriptor (`pointer + type metadata + size/alignment`). That is an implementation
of typed storage, not a new guest type and not an ownership promise. A pointer to a
slot remains an unmanaged `Ptr<T>` and follows the existing lifetime checks. Future
allocators or collectors may choose where such storage lives without changing the
metadata contract.

`System.Object` is reserved for a later common object API. It should not be used as
the backing representation for arbitrary slots, and introducing it does not make
all values reference-compatible. This keeps the low-level machine model explicit
while leaving higher-level languages free to provide ergonomic object semantics.

## Physical layouts and unions

Erased storage and overlapping storage solve different problems. `System.Value`
preserves one complete value together with its exact runtime type. A low-level type
may instead describe a fixed-size region whose alternatives share bytes, similar to
a C union, or describe a struct containing such regions. Reading an alternative then
requires an explicit discriminant or an unsafe operation supplied by the containing
type; it does not consult `System.Value` and does not create a hidden box.

This is a future metadata/layout capability. The current interpreter keeps record
fields disjoint and rejects native layout for `System.Value`. When overlapping
layouts are added, the metadata must state size, alignment, offsets and whether a
field is an overlay, while the allocator remains responsible for the storage
location and lifetime. Existing `Ptr<T>` operations are the natural low-level way
to address that region.

Packing uses normal storage conversion: `ldc.i4 257; value.pack Byte` stores Byte(1).
It matches Byte, not Int32. Extraction applies normal stack normalization, so unpacking
Byte puts Int32(1) on the stack. Generic operands are substituted normally. Void is a
valid complete payload. Packing System.Value itself creates another explicit layer.

There is no empty or null System.Value and no fabricated default payload. Locals still
require initialization. Copying a stored record copies its value, including nested
erased values; extracting and updating a copy does not mutate the original. Pointers
and Ref handles retain existing alias/lifetime contracts: erasure does not retain an
allocation, extend frame lifetime, or acquire ownership.

The Rust interpreter uses an owned tree internally. Every successful pack creates a
Box<Value> on the host heap. Copying an erased value recursively clones its owned
payload, including record fields, strings and nested erased values. Pointer and Ref
handles are copied as handles; their targets are not recursively copied or retained.
This does not create an addressable guest heap object. These allocations follow the
Rust host allocator; host allocation failure is not universally converted into a guest
Fault. No total memory quota or guest allocator-selection mechanism is implied.
Each pack limits payload traversal
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
in platform IL. I/O, text and arithmetic APIs and native adapters use ordinary carriers.
Format 4 removes bootstrap union encodings and requires source reassembly.

```sh
cargo run --locked -- run examples/ordinary_carrier.neoil
```

Output: `Success`, `42`, `Failure`, `7`, then `=> Void`. Tests also cover distinct empty
and Void-carrying records, nested erased records, independent copies, failed queries,
generic primitive normalization, pointer lifetime and depth limits.

An alternative [pointer-backed carrier experiment](pointer-carriers.md) stores a tag
and Void* and borrows caller-managed native storage. It demonstrates explicit aliasing
and release responsibilities. It does not replace System.Value for payloads without
a native layout, and copying that view does not copy or retain its target.
