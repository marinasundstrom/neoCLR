# Explicit cloning

System.Clonable<T> is an ordinary generic interface implemented in the System library:

```text
.interface System.Clonable<T>
    .method instance byref Clone() -> T
    .end
.end
```

Value semantics remain the default. Assignment, argument passing and ordinary loads
use the existing value-copy contract; they do not invoke Clone or require Clonable.
Clonable supplies an explicit operation when the caller wants the implementing type
to produce a clone. Typically a type Widget implements Clonable<Widget>. T gives
the result its exact type without an erased object return or a cast.

The receiver is borrowed explicitly through T& or a managed interface view. This
avoids copying the receiver merely to request a clone. Current byref permissions
are writable; the contract requires implementations to preserve the source's
observable value, but the VM does not prove that requirement. A future readonly
capability can strengthen it with an explicit compatibility decision.

Clone should return an equivalent value under the type's documented semantics.
Each implementation must specify which resources it duplicates and which references
it intentionally shares. Copying a pointer alone does not clone its target or
establish ownership. Clonable does not mean that every reachable object is recursively
duplicated. Immutable backing storage may be shared where the lifetime contract
supports it. Clone is separate from move, retaining a shared owner, and destruction.

The interface does not provide recoverable errors. An operation with expected
duplication failure should expose a separate Result-returning API with its own error
contract. Runtime Faults remain possible as for other guest methods.

Implementations use ordinary IL and explicit conformance, with matching byref
receiver and return types. A method named Clone alone grants no interface membership.
There are no clone opcodes, automatic implementations, ownership hooks, generic
constraints or primitive Clone methods added by this slice.

The [sample](../examples/clonable.neoil) implements a generic Snapshot<T>, clones a
String-containing snapshot through a forwarded managed interface view, then changes
the clone while preserving the original. It needs no native String layout.

```sh
cargo run --locked -- verify examples/clonable.neoil
cargo run --locked -- run examples/clonable.neoil
cargo test --locked --test clonable
```

Output is `original`, `changed clone`, and `=> Void`, each on its own line. The
tests also exercise serialized artifacts, conformance rejection and the distinction
between an ordinary value copy and explicit Clone dispatch.

See the [lifecycle proposal](lifecycle.md) for the future destruction and resource
ownership contracts. Those mechanisms are not implemented by Clonable.
