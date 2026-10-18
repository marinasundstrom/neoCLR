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
| `Slice(range: Range): Slice<T>` | Borrowed zero-copy window; invalid bounds fault. |
| `self[range: Range]: Slice<T>` | Borrowed zero-copy window; invalid bounds fault. |
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

Development (2026-10-19): Array.Slice and array range syntax return
[Slice<T>](memory-views.md). Writes through the view affect the array. Use
`array[range].ToArray()` for an independent shallow copy. Bounds are checked
against Length; empty windows at valid endpoints succeed and invalid bounds fault.
`T[]` range syntax uses RuntimeHelpers.GetSlice. This intentionally changes the
previous copy contract; callers relying on independent storage must add ToArray.

Sliceable<TSlice> has been removed: its Result-based contract cannot carry a
borrowed slice under the current generic lifetime rules. ArrayList's existing
range operation still returns owned copies without that interface; growing
collections need a separate invalidation/ownership design before exposing borrowed
storage.
The new view path is currently interpreter-only; JIT/AOT lowering is pending.
