# Pointer-backed carriers and explicit references

Removing the class/struct allocation distinction does not remove references. T
represents a value; T* represents an address. Copying T copies its contents. Copying
T* aliases its target without retaining or releasing it. A pointer field gives an
ordinary value shared access to storage without making its type a reference type.

## Void pointers

Void* is already supported by the general pointer type and instructions. `ptr.cast Void`
converts a typed pointer to Void*, and `ptr.cast T` converts it back to T*. These are
explicit static-type erasures and reinterpretations. They preserve the address and
interpreter allocation tracking; they do not capture a dynamic type, validate the
chosen reinterpretation, acquire ownership, or extend the allocation lifetime.

A Void* occupies one native pointer slot regardless of the pointee size. To read a
payload as T, cast to T* and use `ldobj T`. The normal bounds, alignment, initialization
and allocation-lifetime checks still apply. Those checks do not prove that T was the
originally stored type. This is a low-level trusted contract, not safe dynamic casting.
For arbitrary heterogeneous storage, a separate descriptor would be needed to remember
the concrete type. A closed union can instead use its tag and generic arguments.

Void remains an inhabited, zero-sized type in neoCLR. Consequently `ldobj Void` is
permitted on a valid Void* and yields Void; it does not reveal or read an arbitrary
payload. Null and stale pointers still Fault on that operation. This slice does not
change that existing rule to adopt C's incomplete void type.

## A borrowed carrier

[PointerResult<T,E>](../examples/pointer_union.neoil) is an ordinary sample type with
two private fields: a Byte tag and a Void* payload. FromOk(T*) and FromError(E*) select
the tag and erase the pointer's static type. IsOk is a property; GetOk/GetError check
the tag before casting and loading a value copy. Equal T and E types remain distinct
cases because the tag, rather than the pointer type, selects the alternative.

The carrier is a **borrowed view**. Its factories allocate nothing, and it has no
Release method. The caller owns the allocation and must keep it live for every read.
The same representation borrows heap storage or frame-local storage. It has a native
sequential layout and can itself be stored in native memory. No new instruction,
special union encoding, System.Value packing, or implicit ownership is involved.

The sample demonstrates:

1. Allocate and initialize an Int32 on the heap, then create an Ok view.
2. Copy the view and extract an independent Int32 snapshot.
3. Update the allocation: the snapshot stays 42 and the alias reads 7.
4. Free the original allocation exactly once, through a Void* cast.
5. Create an Error view over frame-local storage and read 11 before returning.

Both heap views become dangling after step 4. Copying a view never transfers ownership
or creates another obligation to free. A returned view into a callee's frame is also
dangling. The interpreter catches subsequent accesses to those tracked allocations;
a future native backend must specify how it provides or restricts these checks.

```sh
cargo run --locked -- verify examples/pointer_union.neoil
cargo run --locked -- run examples/pointer_union.neoil
cargo test --locked --test pointer_unions
```

Expected program output is 42, 7, 11 on separate lines, followed by `=> Void`.
The tests also assemble and load a JSON round trip, reject mismatched case extraction,
check expired heap/frame payloads, and exercise native carrier storage and Byte/Void.

## Limits and next decisions

This is a storage experiment, not a replacement for the public System.Result API.
Payload loads require a supported native layout. Numbers, pointers, Void and records
composed of native-layout fields work. String, Error, System.Value and the current
System.Option/Result carriers do not yet have native payload layouts. Their pointer
types can be represented, but allocating or loading those pointees is unsupported.
Erasing a pointer cannot remove this limitation.

For Void payloads this experiment uses existing zero-sized native allocations, which
still have a lifetime and require release when heap-allocated. A specialized unit case
could instead omit allocation and have its accessor synthesize Void after checking the
tag; null alone must not select a case. No such specialization is implemented here.

An owning carrier is a separate abstraction. Before adding it, specify explicit move,
clone and release contracts, behavior of aliases after release, and whether nested
resources are borrowed or owned. `heap.free` releases one allocation; it does not walk
payload pointers or run destructors. Ordinary value copying cannot silently acquire
reference counts or deep-copy pointer targets. Ref<T> or another ownership abstraction
may implement a different contract later.

The existing System.Value carrier has different semantics: explicit packing owns a
host value tree and copies owned payloads. It remains necessary for current general
library payloads. Pointer-backed carriers demonstrate the lower-level option and its
manual lifetime responsibilities, not an allocation-free implementation of that tree.

System.Value is scheduled for [removal](value-storage.md#retirement-decision) once
ordinary type storage and explicit reference/lifetime contracts cover these remaining
payloads. This experiment is a step toward that migration, not a permanent two-track
choice between pointer carriers and a built-in erased-value container.
