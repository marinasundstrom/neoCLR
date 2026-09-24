# Managed arrays

An array is fixed-size managed storage. Assigning it shares the same array; changing an element is visible through its aliases. neoCLR keeps these familiar .NET behaviors while giving arrays a generic API shape and invariant element types.

**Preview 9 implementation.** This example is included in the matching Preview 9 toolchain and sample bundle. The broader API remains open to revision.

<a id="example"></a>

## Array aliasing and copying

The array holds three readings. A second name changes the same storage, and a Sequence view sees that correction. ToList then copies the integer elements into a separate collection, preserving those readings while the original array changes.

```raven
{{ARRAY_TOUR}}
```

`let` prevents rebinding the local name; it does not freeze the array. A Sequence exposes reading, but other aliases can still replace elements. The snapshot here contains integers: copying references into a new collection would still share the referenced objects.

### Expected output

```text
{{ARRAY_OUTPUT}}
```

[Download the complete sample →](../../samples/library-array-tour.rvn) · [Expected output →](../../samples/library-array-tour.expected.txt) · [Run with the preview toolchain →](../../try/#development)

<a id="dotnet"></a>

## Compared with .NET arrays
| Concern | .NET / C# | neoCLR / Raven target |
| --- | --- | --- |
| Storage and assignment | Arrays are reference types. Assignment shares storage; each instance has fixed dimensions. | Ordinary T[] is a managed reference. Assignment aliases the same fixed-size array. |
| Element access | Zero-based vectors have checked indices. Invalid indexing throws `IndexOutOfRangeException`. | Zero-based vectors have runtime bounds checks. Invalid indexing causes a terminal Fault, not a catchable guest exception. |
| API shape | T[] derives from nongeneric System.Array; vector arrays also expose generic collection interfaces. | T[] and System.Array&lt;T&gt; resolve to the same intrinsic array. The generic spelling does not wrap or copy it. |
| Element-type conversion | Reference arrays allow covariance, with runtime store checks. Value-element arrays do not have that covariance. | Mutable arrays are invariant. An array of derived elements cannot become an array of base elements. |
| Replacement versus growth | Vectors implement IList&lt;T&gt;, although fixed-size arrays reject growth operations. | MutableSequence&lt;T&gt; permits replacement. Arrays do not implement the growable List&lt;T&gt; contract. |
| Helpers | Array.Empty&lt;T&gt;() and static Array.ForEach(array, action). | Array&lt;T&gt;.Empty and instance array.ForEach(action). Empty-array identity caching is not promised. |
| Dimensions | Vectors, rectangular multidimensional arrays and jagged arrays. | The documented Raven subset is vectors and nested arrays; rectangular multidimensional arrays are outside this subset. |

The generic shape makes the element type and ordinary members available together. Invariance avoids covariant array-store failures, but programs relying on .NET reference-array conversions need adaptation. Separating replacement from growth gives a more precise contract, at the cost of a different interface vocabulary. These are deliberate tradeoffs, not a performance claim or full .NET compatibility.

Baseline checked September 23, 2026: [Microsoft’s C# array guide](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/arrays) and [.NET 10 System.Array reference](https://learn.microsoft.com/en-us/dotnet/api/system.array?view=net-10.0). See the [neoCLR implementation contract](https://github.com/marinasundstrom/neoCLR/blob/main/docs/generic-managed-arrays.md) for metadata and compiler details.

<a id="ownership"></a>

## Managed storage, checked by the runtime
The runtime owns array storage and traces live references through the managed heap. Keeping an array reachable keeps its reference elements reachable too. Byte and integer elements are values; object elements retain references. The ordinary reference-array path supports default null reference slots; dispatch through a null element faults. This does not settle the broader nullability proposal.

Bounds and element compatibility are runtime responsibilities, not just source-language diagnostics. Direct IL tests exercise invalid indices, incompatible elements and references surviving collection. The generic array shape reuses those checks. There is no separate user-allocated object behind Array&lt;T&gt;.

A byte array is a useful I/O buffer, but being managed does not make it safe for a native operation to retain an untracked address. Pending I/O still needs explicit rooting, mutation and cancellation rules. A read-only view does not prove exclusive ownership.

<a id="limits"></a>

## Supported operations and limits

The API includes Length, Count, indexed access, iteration, Empty and ForEach, with the runtime’s supported element types. The generic shape is not a promise that every possible element signature or .NET Array helper is admitted. Direct array loops use indexed access; Iterable consumers use the declared interface contract.

Bounds failures are terminal, so validate user-supplied ranges before accessing elements when invalid input should produce a recoverable Result. Array size and allocation are also constrained by runtime budgets.

Span/Memory APIs, general interface variance and immutable or frozen array families are not introduced by this implementation. Historical Neo owned-array experiments are separate from the ordinary Raven T[] behavior described here.

<a id="direction"></a>

## Buffer experiments and planned work

The platform roadmap starts with byte copy, then text/JSON transformation and controlled delayed operations before sockets. These cases test range arithmetic, overlap, partial transfers and retention through garbage collection. They can reshape provisional APIs before the HTTP application milestone.

[Platform roadmap →](../../proposals/#http-poc) · [Collection capabilities →](../collections/)

<a id="feedback"></a>

## Questions and contributions

Include the required access and lifetime when reporting an array issue. This distinguishes a missing library operation from a missing runtime guarantee.

[Share an example on GitHub ↗](https://github.com/marinasundstrom/neoCLR/issues)

Questions, sample programs and documentation corrections are welcome. See [how to contribute](../../#feedback) for ways to participate.
