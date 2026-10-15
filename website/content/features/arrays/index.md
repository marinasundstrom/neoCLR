# Managed arrays

An array is fixed-size managed storage. Assigning it shares the same array; changing an element is visible through its aliases. neoCLR keeps these familiar .NET behaviors while giving arrays a generic API shape and invariant element types.

Arrays have a structural runtime shape and a nominal `Array<T>` representation.
Both describe the same intrinsic array; `Array<T>` does not wrap or copy it.
[Structural type families](../../docs/structural-types.html) explain how this differs
from tuples, unions, intersections and functions.

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

Comparison references: [Microsoft’s C# array guide](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/arrays) and [.NET 10 System.Array reference](https://learn.microsoft.com/en-us/dotnet/api/system.array?view=net-10.0). See the [neoCLR implementation contract](https://github.com/marinasundstrom/neoCLR/blob/main/docs/generic-managed-arrays.md) for metadata and compiler details.

<a id="ownership"></a>

## Managed storage, checked by the runtime
The runtime owns array storage and traces live references through the managed heap. Keeping an array reachable keeps its reference elements reachable too. Byte and integer elements are values; object elements retain references. The ordinary reference-array path supports default null reference slots; dispatch through a null element faults. This does not settle the broader nullability proposal.

Bounds and element compatibility are runtime responsibilities, not just source-language diagnostics. Direct IL tests exercise invalid indices, incompatible elements and references surviving collection. The generic array shape reuses those checks. There is no separate user-allocated object behind Array&lt;T&gt;.

A byte array is a useful I/O buffer, but being managed does not make it safe for a native operation to retain an untracked address. Pending I/O still needs explicit rooting, mutation and cancellation rules. A read-only view does not prove exclusive ownership.

<a id="limits"></a>

## Supported operations and limits

The API includes Length, Count, indexed access, iteration, Empty and ForEach, with the runtime’s supported element types. The generic shape is not a promise that every possible element signature or .NET Array helper is admitted. Direct array loops use indexed access; Iterable consumers use the declared interface contract.

Bounds failures are terminal, so validate user-supplied ranges before accessing elements when invalid input should produce a recoverable Result. Array size and allocation are also constrained by runtime budgets.

Span/Memory APIs, general interface variance and immutable or frozen array families are not introduced by this implementation.

<a id="direction"></a>

## Buffer experiments and planned work

Socket and HTTP operations already exercise byte buffers, partial transfers and retention through garbage collection. Future buffer views or asynchronous streams need explicit aliasing and lifetime contracts; they are not implied by the current array API.

[Platform roadmap →](../../proposals/#http-poc) · [Collection capabilities →](../collections/)

<a id="feedback"></a>

## Questions and contributions

Include the required access and lifetime when reporting an array issue. This distinguishes a missing library operation from a missing runtime guarantee.

[Share an example on GitHub ↗](https://github.com/marinasundstrom/neoCLR/issues)

Questions, sample programs and documentation corrections are welcome. See [how to contribute](../../#feedback) for ways to participate.


## Development: Index and Range

The matching development compiler and native libraries support `values[^1]`,
from-end assignment and half-open slices such as `values[1..^1]` and `values[..]`.
Each array slice makes an independent shallow copy; referenced elements remain shared.
`^0` describes the end boundary, so it can end a slice but cannot select an element.

Indexing and slicing syntax are convenience APIs that may fault on invalid bounds;
callers must establish valid input. `values.Get(index)` returns an `Option<T>` for
an integer or Index. `values.Slice(range)` returns `Result<T[], InvalidRangeError>`.
Index and Range describe positions and boundaries; applying them determines validity.

The development Raven compiler now uses `..` for an exclusive end and `..=` for an
inclusive end. Existing half-open slices retain `..`; the former `..<` spelling is
removed. Native inclusive Range values now use checked Index conversion, rejecting
unrepresentable ends before arithmetic while retaining the half-open Range library
contract. The native compiler work is qualified on macOS ARM64.

The array contract is tested in interpreter and macOS native AOT; the Windows JIT
framework checks also cover array and string Index/Range operations. String range support is available in the matching development libraries, using
grapheme boundaries. [Sequences and lists](../collections/) also support indexing
and slicing. Borrowed views remain future work.

[Array API reference →](../../docs/api/System/Array%601/index.html)


Development compiler note: signed int/long range patterns in conditions and match
arms now use the same exclusive `..` and inclusive `..=` endpoints on the native
target. Checked inclusive Range construction is also supported. Bounds for a long subject currently require long literals, such as
`1L..=3L`.


Development signed int/long range loops and comprehensions now stop before
advancement would overflow, including an inclusive maximum endpoint. Other numeric
stepping domains retain their existing behavior. Nominal safe-slicing capabilities
and explicit Interval progressions remain design work, not published APIs.
