# Sequence indexing and slicing

Development API in System.Collections. Import `System.*` and `System.Collections.*`.

| Member | Contract |
| --- | --- |
| `SequenceAccess<T>.self[index: Index]: T` | Read-only extension indexer; resolves against Count and uses the integer indexer. Invalid bounds fault. |
| `SequenceAccess<T>.self[range: Range]: T[]` | Read-only extension indexer returning an independent shallow copy. Invalid bounds or premature exhaustion fault. |
| `Get(index: int): Optional<T>` | None for negative or past-end indices, Some(element) otherwise. |
| `Get(index: Index): Optional<T>` | Resolves against Count, then returns the same validated result. ^0 is past the final element. |
| `Slice(range: Range): Result<T[], InvalidRangeError>` | Validates half-open bounds, including empty end slices. Error for invalid bounds or an iterator ending before the requested range. |
| `List<T>.self[index: Index]: T` | Read/write instance contract. Invalid bounds fault. |
| `List<T>.self[range: Range]: T[]` | Read-only instance contract returning an independent shallow copy. Invalid bounds fault. |
| `ArrayList<T>.self[index: Index]: T` | Implements List directional read/write against Count. |
| `ArrayList<T>.self[range: Range]: T[]` | Implements List copying directly from backing storage, bounded by Count rather than Capacity. |

Instance members take precedence over extensions. Sequence.Slice delegates to a
List's range indexer when available. Otherwise it uses one iterator, skips the
prefix without reading Current, and copies the requested elements. The fallback
acquires no iterator for empty or invalid ranges. Iterators are disposed on normal
completion and premature exhaustion. Provider faults are terminal; these APIs do
not catch them or promise a concurrent snapshot.

Elements retain ordinary value/reference semantics: structs are copied, class
references share identity. Copying the array does not deep-copy referenced objects.
Range results are copies, not views. Custom List implementations must implement the
new indexers and uphold this contract. Safe alternatives are the Get/Slice methods.

[Collection overview](/features/collections/) · [SequenceAccess reference](xref:System.Collections.SequenceAccess)
