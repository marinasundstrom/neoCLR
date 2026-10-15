# Interface variance (development)

These are the class library's exact annotated declarations. The current RavenDoc
signature renderer omits `in`/`out` in its heading; the annotations are retained in
native metadata and the generated type remarks. This manual reference supplies the
missing signature qualifiers without excluding any type or member from the browser.

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
Native intrinsic String materialization and native open-definition reflection
remain documented implementation gaps. The annotations do not imply support for
those missing operations.
