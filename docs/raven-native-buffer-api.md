# Native buffers through Raven

The experimental target exposes the existing unsafe `System.Array<T>` value
descriptor, distinct from managed `T[]` arrays. `Allocate(length, initialValue)`
allocates native storage; `View(pointer, length)` borrows existing storage. Descriptor
copies share storage. The owner must call `Free()` exactly once after all views and
pointers have stopped being used. Garbage collection does not release this region.

[The saved sample](experiments/raven-target/samples/library-native-buffer.rvn) uses
allocation, zero-length allocation, indexing, Get/Set, GetElementAddress, public
Data/Length fields, pointer reads/writes, views and release. The current importer
admits primitive numeric and Boolean elements. Direct indirect-load/store IL is
currently limited to Int32; other admitted elements work through Get/Set/indexers.
Managed String elements are rejected before an executable is emitted. Arbitrary
record layouts and pointer arithmetic remain outside this importer subset.

Use Raven `unsafe func` or an unsafe context. Pointer syntax is `*int`; dereference
is `*pointer`. These are native pointers, not managed object references. Unlike
managed CLR arrays, this existing API is a low-level descriptor with public fields
and explicit lifetime; it offers native interop control at the cost of manual
ownership. It is not proposed as a replacement for ordinary managed arrays. The
runtime contract and CLR comparison remain in [arrays and pointers](arrays-and-pointers.md).

Prepare the collection profile using the [experiment instructions](experiments/raven-target/README.md)
and copy the sample into the saved project. `verify_project.py --collections` runs
its successful paths; `verify_native_buffer.py` checks negative length, bounds,
use-after-free and repeated release faults, plus managed-layout rejection.
`verify_editor.py --reflection` also checks native Array static completion.
The source compiler requires its target field-metadata and pointer-substitution
fixes; the older installed SDK/extension does not include them yet.
