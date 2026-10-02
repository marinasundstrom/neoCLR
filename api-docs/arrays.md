# Array

An array shape `T[]` has an element type and fixed length. Array values can be
passed as Object. Its shape has no nominal declaration name; the library's
[Array&lt;T&gt; member contract](xref:System.Array`1) supplies the operations below.
The named library contract and an individual array shape are distinct descriptors.

## Members

| Member | Contract |
| --- | --- |
| `Length: int` | Number of element slots; does not change after creation. |
| `Count: int` | The same number of slots through the sequence contract. |
| `self[index: int]: T` | Reads or replaces an element. An index outside the array bounds fails. |
| `Empty: T[]` | Static library property returning an empty array of the element type. |
| `ForEach(action: (T) -> Void)` | Invokes the callback once per element, in index order. |
| `GetIterator(): Iterator<T>` | Traverses elements through the collection iterator contract. |

Follow the [generated Array reference](xref:System.Array`1) for member signatures
and links, and [collections and queries](/features/collections/) for applicable
sequence extensions. Structural shape does not prohibit extension members.
The [array guide](/features/arrays/) contains executable examples and the current
storage, mutation and Object conversion limits.

[All structural families](structural-types.md)
