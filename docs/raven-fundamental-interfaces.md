# Fundamental interfaces through Raven

The collection target profile now exposes EquatableTo<T>, ComparableTo<T>, ConvertibleInto<T>, Clonable<T>
and Closable<E>, alongside Disposable and the collection interfaces. These keep
neoCLR's existing names and invariant type parameters. Closable.Close returns
Result<Void,E>; Disposable.Dispose is an ordinary no-result call. This is explicit
resource cleanup, not automatic destructor or unwind behavior.

Ordinary source assignment converts implementing values to an interface:

```raven
var value = 42
let order: ComparableTo<int> = value
value = 100
Console.WriteLine(order.CompareTo(42)) // 0: interface owns the copied value
```

Raven emits CLI boxing; the importer retains the box and inserts a checked interface
cast at the typed runtime boundary. Class interface conversions preserve the original
reference. The runtime implementation and CLR allocation/copy tradeoff are documented
in [boxed interface values](boxed-interface-values.md). Primitive direct calls still
use their existing receiver ABI. Metadata marks interface implementations as final
virtual slots; this does not introduce overriding into these value types.

The profile retains all existing ComparableTo implementations, Int32/String/Date/Time
EquatableTo implementations, and restores Type's EquatableTo contract after its class
migration. [The saved sample](experiments/raven-target/samples/library-value-interfaces.rvn)
checks value-copy independence, calls through parameters, String and Type equality,
Date ordering/equality, and numeric/Boolean interfaces. The signature probe checks
all five contracts, including Clonable and Closable results and invariance. Completion
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

ConvertibleInto<T>.Convert() returns T through ordinary interface dispatch. The
contract has no built-in implementations and supplies no implicit conversion or
return-directed overload rule. See [migration and design](common-interfaces.md#directional-interface-names-development-2026-09-25).


## Development Number integration — 2026-09-27

Number inherits ComparableTo<Self>, with static abstract Zero/One and binary
arithmetic operators, implemented by eight fixed-width integers, Single and Double.
Concrete Parse is separate; no parsing interface is selected. See
[numeric contracts](design/numeric-contracts.md) and the
[focused consumer](experiments/numeric-contracts/README.md).

Raven general fixes were validated independently on a main-based branch, then
integrated into main and its neoCLR branch: authored static declarations and
constrained dispatch, inherited constraint members, and semantic constraint-kind
classification during target-metadata emission. Runtime Contract configuration is
unchanged. The target reference projects exact static abstract CLI slots; source
implementations must match. Placeholder reference bodies are never runtime code.

The bridge closes selected static application numeric algorithms before normal
checked import, limits specialization to the ten primitive types and 128 copies,
and rejects other constraints. Generated helpers are internal and have no invented
source metadata token; diagnostic maps retain the original method token. This is
not generic runtime static dispatch, nor complete interface default/accessibility
support. The [runtime tracker](tracking/runtime-language.md#interfaces-as-a-platform-capability)
owns broader author-directed capabilities. Archived Neo profiles retain their
legacy parsing boundary; new standard-union parsing belongs to the Raven library.
