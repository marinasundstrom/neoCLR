# Native interop: first P/Invoke slice

A free function or static method can declare a native import:

```text
.type Native
    .method static Add(a: Int32, b: Int32) -> Int32
        .pinvoke "./target/native/neoclr_sample" "neoclr_add" cdecl
    .end
.end
```

The directive takes two JSON-quoted strings and the cdecl keyword. Call sites use
ordinary signature-based call instructions. Imports have no IL body or locals,
cannot be instance methods or entry points, and cannot also use InternalCall.

Metadata stores an optional pinvoke record containing library, entry_point, and
calling_convention (Cdecl). It is a prototype equivalent of the CLI ImplMap concept,
not a MethodImpl/InternalCall flag. A future CLI binary writer should map this to
ModuleRef/ImplMap and the method's PInvokeImpl attribute. Existing format-3 modules
that omit the optional record continue to load. General custom attributes remain
unimplemented; no DllImport source-attribute parser is claimed.

## Supported ABI

The platform C calling convention is supported through libffi. Signed and unsigned
8/16/32/64-bit integers, native integers, Single, Double, and Ptr<T> can be arguments
or returns. Void is supported as a return only; it produces neoCLR's inhabited Void
value after a native void call. Normal signature storage conversion happens before
marshalling, so Byte arguments occupy a byte and Single arguments use binary32.
Results normalize back to the evaluation-stack category after the call.

Boolean, Char, String, Error, unions, Ref, and records passed by value are rejected.
Use explicit numeric storage types or pointers for this subset. There is no implicit
encoding conversion, string termination, Boolean-width convention, struct marshalling,
variadic call, callback, last-error capture, or ownership transfer. Other calling
conventions are not supported yet. Exact C export spelling is required.

A pointer carries its actual native address. Tracked dangling pointers are rejected
before dispatch. Null and externally obtained pointers can be forwarded to native
code, whose declared contract decides whether they are valid arguments. Native code
can write initialized scalar storage allocated by heap.alloc; subsequent guest loads
observe those bytes. Initialize the storage before passing it: this slice cannot
infer which previously uninitialized bytes a foreign function wrote.

Returned addresses are associated with a current live guest allocation where one
matches. Other addresses stay untracked and can be passed to later native calls,
but guest ldobj/ldind and heap.free still reject untracked non-null pointers. Foreign
allocation and deallocation must remain with the matching foreign API. Native code
must not free guest allocations or retain their pointers beyond the agreed lifetime.
Pointer-field writes do not have full provenance/initialization integration yet;
this slice demonstrates scalar memory mutation, not a complete foreign heap model.

## Loading and execution trust

Libraries are loaded on the first executed import and cached for that execution.
Assembling, loading JSON metadata, and CLI check validate signatures without loading
native libraries or running their initializers. Missing libraries and symbols become
instruction-located Faults. No host-function registry name implicitly enables P/Invoke.

An extensionless library name gains the platform prefix/suffix, preserving its directory:
`./target/native/neoclr_sample` selects libneoclr_sample.dylib on macOS,
libneoclr_sample.so on Linux, and neoclr_sample.dll on Windows. Explicit extensions
are used literally. Relative paths resolve against the process working directory;
bare names use the operating system loader's search rules. No full .NET probing or
assembly-relative resolution is implemented.

Execution retains loaded libraries in native_libraries so returned addresses into
library storage do not immediately lose their backing library. Keeping that execution
alive does not extend the lifetime of arbitrary foreign allocations. Explicit native
memory contracts still apply, including on execution failure and teardown.

Rust's safe run/run_with_library APIs reject an executed native import. Embedders
use unsafe run_with_native(module, library, limits) for trusted native execution and
must uphold its ABI and memory-safety contract. The CLI run command enables this
path for the selected program, like running a native executable. This boundary is
not a sandbox: malformed native signatures, invalid pointer use, native exceptions
or unwinding, and foreign faults cannot be made safe by metadata validation. Native
side effects happen immediately, and a call cannot be interrupted by the guest
instruction budget. Native code must not unwind across the C ABI.

## Build and sample

The runtime uses libloading and the libffi crate. The default build compiles vendored
libffi and therefore needs a native C toolchain: compiler and make/shell tools on Unix,
or the supported MSVC tools on Windows. The packaged libffi sources include configure.
No installed .NET runtime is needed. The sample library is Rust compiled to a C ABI,
not a second neoCLR module.

From the repository root:

```sh
cargo run --locked --example build_native
cargo run --locked -- run examples/pinvoke.neoil
```

The sample allocates and initializes an Int32, calls native addition and a pointer
setter, reads the value through a returned pointer, prints 42, and frees the guest
allocation. The integration tests build temporary native libraries and exercise all
supported scalar signatures, mixed argument classes, void returns, pointer mutation,
foreign pointer forwarding, loader failures, and metadata-only validation. Linux,
macOS, and Windows run these through the CI matrix; local validation is macOS ARM64.
