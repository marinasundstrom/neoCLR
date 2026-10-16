# Interface variance (development)

These are the class library's annotated declarations. RavenDoc preserves `in`/`out`
in generated signatures from native metadata. This reference collects the eleven
annotations and their conversion limits; no type or member is excluded.

| Namespace | Declaration |
| --- | --- |
| System.Collections | `interface Iterator<out T>` |
| System.Collections | `interface Iterable<out T>` |
| System.Collections | `interface Collection<out T>` |
| System.Collections | `interface Sequence<out T>` |
| System.Collections | `interface AsyncIterator<out T>` |
| System.Collections | `interface AsyncIterable<out T>` |
| System.Collections | `interface Comparer<in T>` |
| System.Collections | `interface EqualityComparer<in T>` |
| System | `interface ConvertibleInto<out T>` |
| System | `interface ComparableTo<in T>` |
| System | `interface EquatableTo<in T>` |

Covariant interfaces produce T; contravariant interfaces consume T. Differing
arguments require supported reference conversions; identical value arguments remain
valid. Conversions preserve the receiver and do not allocate adapters or box elements.
Concrete classes, mutable collections, array storage, Option/Result and Sliceable
remain invariant. A class may still expose a wider read-only interface through its
explicit implementations.

Inspect a declaration through [GenericParameterTypeInfo.Variance](reflection.md#generic-parameter-variance-development).
Native open-definition discovery supports explicitly retained type catalogs; open
member/relationship queries and intrinsic String variance materialization remain
implementation gaps. Annotations do not imply support for those missing operations.

Variance also follows explicitly declared local-to-imported class ancestry. The
separately compiled Raven consumer validates producer/consumer and nested views in
interpreter/AOT/JIT, preserving receiver/payload identity and inherited-field writes.
Reverse and unrelated-class conversions reject; this does not enable generic class bases.

Interface-valued reference arguments may widen to Object in the declared variance
direction. Native lowering preserves the verified Object-root role through private
renaming; producer and consumer paths are qualified with actual identity checks in
interpreter/AOT/JIT. This does not admit value boxing or intrinsic String widening.

Array references are also supported as variant arguments: Producer<int[]> can
widen to Producer<Object>, and Consumer<Object> can narrow to Consumer<int[]>.
The calls retain the same array; element storage remains invariant. Both directions
are qualified in Raven interpreter, AOT and managed JIT.
