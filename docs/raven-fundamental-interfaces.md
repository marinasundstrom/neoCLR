# Fundamental interfaces through Raven

The collection target profile now exposes Equatable<T>, Comparable<T>, Clonable<T>
and Closable<E>, alongside Disposable and the collection interfaces. These keep
neoCLR's existing names and invariant type parameters. Closable.Close returns
Result<Void,E>; Disposable.Dispose is an ordinary no-result call. This is explicit
resource cleanup, not automatic destructor or unwind behavior.

Ordinary source assignment converts implementing values to an interface:

```raven
var value = 42
let order: Comparable<int> = value
value = 100
Console.WriteLine(order.CompareTo(42)) // 0: interface owns the copied value
```

Raven emits CLI boxing; the importer retains the box and inserts a checked interface
cast at the typed runtime boundary. Class interface conversions preserve the original
reference. The runtime implementation and CLR allocation/copy tradeoff are documented
in [boxed interface values](boxed-interface-values.md). Primitive direct calls still
use their existing receiver ABI. Metadata marks interface implementations as final
virtual slots; this does not introduce overriding into these value types.

The profile retains all existing Comparable implementations, Int32/String/Date/Time
Equatable implementations, and restores Type's Equatable contract after its class
migration. [The saved sample](experiments/raven-target/samples/library-value-interfaces.rvn)
checks value-copy independence, calls through parameters, String and Type equality,
Date ordering/equality, and numeric/Boolean interfaces. The signature probe checks
all four contracts, including Clonable and Closable results and invariance. Completion
checks include each interface's member.

Clonable and Closable have no concrete implementations in the existing library.
Their metadata and call imports are available; a saved-source demonstration that
defines a new implementing application class still needs broader application-type
import. Do not claim that the declarations alone demonstrate such an implementation.
Disposed iterator behavior remains covered by the existing collection checks.

This follows the .NET interface-call model, with names and result-based Close adapted
to neoCLR. It does not add variance, arbitrary application type import, constrained
calls, default-interface-body import, or deterministic fault unwinding. The Neo
frontend and original borrowed interface profile are unchanged.
