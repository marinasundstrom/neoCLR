# Value implementations of nominal interfaces

The runtime now admits ordinary interface contracts backed by value methods with
managed-byref receivers. `box T` copies a non-reference value into GC-owned storage
and produces a System.Object reference. `castclass Interface` checks the complete
implementation contract; interface calls automatically pass the stored payload to
the value method. Interface aliases share that allocation, while the source value
remains independent. Returned interface references survive the producing frame.

This follows the CLR [box instruction](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.box?view=net-10.0)
and ECMA-335's automatic unboxing of value receivers during virtual calls
([standard](https://ecma-international.org/publications-and-standards/standards/ecma-335/),
consulted 2026-09-13). It restores expected value-to-interface lifetime and copy
behavior at the cost of allocation. Borrowing a stack value would avoid the copy but
would impose a different lifetime and mutation contract. Direct calls still use
values/byrefs; no universal boxing of generic payloads is introduced.

The current instruction requires an ordinary System.Object class declaration.
It does not impose Object ancestry on every type. This is a bounded interpreter
implementation: generic reference-type box no-ops, nullable boxing, unbox/unbox.any,
general Object virtual methods and constrained-call allocation optimizations remain
future work. Boxed Int32 has a bounded Object.Equals/GetHashCode intrinsic, described
in the [Object review](object-model-review.md#boxed-int32-equality-and-hash--2026-09-24). The typed verifier currently requires an explicit interface cast after box;
the Raven importer can normalize an implicit CLI assignment with that checked cast.

String retains its existing intrinsic representation. Casting it to an implemented
interface materializes an immutable GC-owned string handle; casting back extracts
the string payload. This is an interpreter representation cost, not value-type
boxing in the language. It does not establish canonical string reference identity.

Box allocations and string interface handles obey heap limits and GC roots. Native
pointers and managed byrefs cannot be boxed. Interface dispatch preserves readonly
implementation receivers; parameter, result and output contracts remain exact.
Legacy explicitly borrowed interface contracts keep their receiver requirements.
Tests in `tests/boxed_interfaces.rs` cover copy independence, alias mutation,
escape, collection, limits and invalid operations. Raven API projection is a
subsequent slice, not evidence supplied by these runtime tests alone.
