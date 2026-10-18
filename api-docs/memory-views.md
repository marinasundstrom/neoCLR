# Slice and ReadOnlySlice

Development APIs authored in Raven (`System.Runtime`). The interpreter provides
checked zero-copy storage; native JIT/AOT lowering remains unavailable. Array
range syntax returns Slice<T> and shares storage; use ToArray() for an explicit
copy. Memory/ReadOnlyMemory have internal runtime test coverage but are not
public Raven APIs yet.

## Slice<T>

A borrowed writable contiguous view. `T[]` converts implicitly to `Slice<T>`.
Copying the view shares its storage; converting does not copy elements.

- `Length: int`: number of elements; `IsEmpty: bool`: whether Length is zero.
- `self[index: int]: T` and `self[index: Index]: T`: read or replace one element.
- `self[range: Range]: Slice<T>`: a receiver-relative zero-copy window.
- `Slice(start: int)`, `Slice(start: int, length: int)`, `Slice(range: Range)`:
  explicit window operations supplied by `System.SliceOperations<T>` extensions.
- `ToArray(): T[]`: an independent shallow copy. This operation allocates storage.

## ReadOnlySlice<T>

A borrowed readonly contiguous view. `T[]` and `Slice<T>` convert implicitly to
`ReadOnlySlice<T>` without copying. There is no reverse conversion.

- `Length: int` and `IsEmpty: bool` have the same meaning as on Slice.
- `self[index: int]: T` and `self[index: Index]: T` are getter-only.
- `self[range: Range]: ReadOnlySlice<T>` returns a readonly zero-copy window.
- `Slice(start: int)`, `Slice(start: int, length: int)`, `Slice(range: Range)`
  return ReadOnlySlice windows.
- `ToArray(): T[]` creates an independent shallow copy.

Readonly restricts this access path. Another writable alias may change the
underlying elements, and those changes remain visible through readonly views.
Element reads return values, not writable element references.

## Bounds, lifetime and errors

Start/count arguments are relative to the view. Negative or excessive indexes,
reversed ranges and out-of-bounds windows fault; bounds are never clamped. A
zero-length window at Length is valid. Null arrays convert to empty views. Default
views are empty. Reading an uninitialized array element faults.

Slices cannot be stored in ordinary fields, arrays or boxed objects, used as
ordinary generic arguments, captured by functions, or retained across suspension.
The interpreter conservatively checks resident values, including dead locals.
Returning a view borrowed from caller or heap storage is supported synchronously;
returning a view of callee-owned frame storage faults. Views trace their backing
owners while live. No raw-pointer ABI or native performance guarantee is implied.

## String.Utf8Bytes

`String.Utf8Bytes: ReadOnlySlice<byte>` shares the immutable UTF-8 representation.
Its offsets count bytes and may split a scalar or grapheme. Ordinary String
indexing and slicing continue to use grapheme coordinates. Decoding is required
to interpret a byte window as text.

## Example

```raven
func Sum(data: ReadOnlySlice<int>) -> int {
    var total = 0
    var index = 0
    while index < data.Length {
        total += data[index]
        index += 1
    }
    return total
}

let data = [10, 20, 30]
let view: Slice<int> = data
view[1] = 42
let total = Sum(data)
let copy = view.ToArray()
```

The [validation consumer](https://github.com/marinasundstrom/neoCLR/blob/main/docs/validation/memory-views-20261019/consumer.rvn)
exercises conversions, writable/readonly aliases, nested ranges, explicit copying
and UTF-8 bytes. No public AsSlice conversion helpers are required.
