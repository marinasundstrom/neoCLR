# Array

An array shape `T[]` has an element type and fixed length. Array values can be
passed as Object. Its shape has no nominal declaration name; the library's
[Array&lt;T&gt; member contract](xref:System.Array`1) supplies the operations below.
The named library contract and an individual array shape are distinct descriptors.
`T[]` and ``Array<T>`` represent the same intrinsic array contract: the nominal
spelling does not add a wrapper or copy. Arrays therefore combine runtime structural
shape with a nominal API representation.

## Members

| Member | Contract |
| --- | --- |
| `Length: int` | Number of element slots; does not change after creation. |
| `Count: int` | The same number of slots through the sequence contract. |
| `self[index: int]: T` | Reads or replaces an element. An index outside the array bounds fails. |
| `Slice(range: Range): Result<T[], InvalidRangeError>` | Safe shallow copy; invalid bounds return Error. |
| `self[range: Range]: T[]` | Shallow copy using the same validation; invalid bounds fault. |
| `Empty: T[]` | Static library property returning an empty array of the element type. |
| `ForEach(action: (T) -> Void)` | Invokes the callback once per element, in index order. |
| `GetIterator(): Iterator<T>` | Traverses elements through the collection iterator contract. |

Follow the [generated Array reference](xref:System.Array`1) for member signatures
and links, and [collections and queries](/features/collections/) for applicable
sequence extensions. Structural shape does not prohibit extension members.
The [array guide](/features/arrays/) contains executable examples and the current
storage, mutation and Object conversion limits.

[All structural families](structural-types.md)

<a id="nominal-slicing-development"></a>

## Nominal slicing

Array and ArrayList explicitly implement [Sliceable&lt;TSlice&gt;](xref:System.Sliceable`1)
with `TSlice = T[]`. The safe method accepts empty ranges at valid boundaries,
resolves from-end boundaries against Length (Array) or Count (ArrayList), and
rejects reversed/out-of-bounds ranges without clamping. Result allocation follows
validation. Copies have independent element slots but retain shared referenced
elements. The interface does not imply enumeration or a range indexer.

Declared range indexers delegate to the safe method and fault on Error. Intrinsic
T[] range syntax uses RuntimeHelpers.GetSubArray and the same Range validator.
