# Native libraries and foreign functions

**Development (2026-10-12):** Raven `extern` declarations can use neutral native-import
attributes, lowered to explicit runtime metadata. The interpreter supports C ABI
scalars, pointers, input text and explicitly sized booleans. JIT/AOT import lowering
and installed compiler-bundle qualification remain open. This is trusted native
execution, not a sandbox or a complete FFI.

## Source contract

The source API lives in `System.Runtime.InteropServices`:

```raven
import System.Runtime.InteropServices.*

[NativeImport("sample", EntryPoint: "neoclr_mixed_encoding",
    StringMarshalling: NativeStringMarshalling.Utf16)]
extern func Mixed([MarshalAs(NativeType.Utf8)] a: string, b: string) -> int;
```

This is the declaration shape exercised by the compiled `ForeignMixedEncoding`
consumer, using the actual fixture library path. The first argument overrides the
UTF-16 default; the second inherits it. [Reproduction and evidence](experiments/foreign-interop/README.md).

`extern` says the implementation is external and has no managed body. NativeImport
supplies a library and optional exact export name; omission of EntryPoint uses the
source function name. CallingConvention defaults to Cdecl. StringMarshalling defaults
to None, which rejects unannotated String parameters. Namespace functions and static
methods on nongeneric classes are supported; instance, generic, async and managed-body
imports are rejected. The compiler recognizes the resolved attribute identity from
its explicit core or registered System.Runtime dependency, not arbitrary short names.

A parameter's MarshalAs descriptor overrides the import's string default. Return
attributes use Raven's existing `[return: MarshalAs(...)]` syntax. The implemented
NativeType choices are:

| Descriptor | Managed type | Native representation | Direction |
| --- | --- | --- | --- |
| Utf8 | String | `const char*`, terminated UTF-8 | Input only |
| Utf16 | String | `const uint16_t*`, terminated native-endian UTF-16 | Input only |
| Bool8 | Boolean | One-byte integer boolean | Input/result |
| Bool32 | Boolean | Four-byte integer boolean | Input/result |

Boolean input becomes exactly 0 or 1; every nonzero native result becomes true.
Boolean without an explicit descriptor is rejected. These widths avoid an ambiguous
platform-dependent `Bool` default. Numeric scalar types and Ptr<T> use direct ABI
storage and reject incompatible conversion annotations. Void is return-only.
Unsupported combinations are diagnosed before output and also rejected by runtime
metadata validation; hand-authored artifacts cannot bypass the contract.

The [API reference](../api-docs/native-interop.md) covers constructors, properties,
enum values, diagnostics and limitations. Source attributes are consumed into native
import descriptors, rather than requiring runtime execution of their constructors.
General attribute reflection is not used to discover imports at startup.

## Calling conventions

NativeCallingConvention.Cdecl selects the platform C ABI. System follows the platform
system ABI: stdcall on 32-bit Windows, C elsewhere. It is a portable name rather than
a Windows default. The libffi call interface uses the selected ABI explicitly.
On the qualified macOS ARM64 host, these conventions share the platform ABI.
The 32-bit Windows distinction is implemented but not qualified by these local tests.
Arbitrary stdcall/fastcall/vectorcall selection, variadic functions and C++ ABI
binding are not supported. A C wrapper is needed for unsupported native interfaces.

Signed/unsigned 8/16/32/64-bit integers, native integers, Single, Double and pointers
retain their declared storage width. This does not mean C `long` has a portable
width: match the actual header/target. Char, records by value, unions, managed byrefs,
arrays, implicit output buffers and returned strings remain unsupported.

## Strings and ownership

Each String becomes a copied, call-owned terminated buffer. Empty strings have a
non-null terminator-only buffer. Embedded NULs fault before entering the foreign
function; library initializers may already have run. UTF-16 preserves supplementary
characters as surrogate pairs and is not the host-dependent C `wchar_t` type.

All argument buffers live together until the synchronous call returns. Native code
must not modify, free, retain or return addresses into them. They are released on
return or marshalling failure. Copying avoids exposing managed storage but allocates
per String; no throughput advantage over .NET is claimed. Nullable strings and
per-parameter length/ownership relationships require further contracts.

Pointers carry real native addresses. Tracked dangling pointers are rejected before
dispatch; tracked guest storage is exclusively borrowed through the call. Native code
can mutate initialized scalar heap.alloc storage. Initialize it first: foreign writes
do not infer guest initialization. Null and foreign pointers can be forwarded subject
to the native function's contract. Native code must not free guest allocations or
retain pointers past their lifetime, including localloc frame storage.

Returned pointers recover current guest allocation provenance where possible. Other
addresses remain untracked: they can be forwarded to native APIs, but guest loads and
heap.free reject them. A foreign allocation must be released by its matching foreign
API. Pointer-field provenance and foreign heap ownership are not fully integrated.

## Validation, loading and failure

Loading checks the descriptor and supported signatures without loading libraries or
executing their initializers. The first executed import loads the compiled native
library and resolves the exact symbol. Successful resolution is cached by library
and symbol; prepared calls retain library lifetime. Signatures and marshalling remain
per declaration, and the libffi interface is constructed per call. Eager startup
resolution is a possible later policy, not current behavior.

Missing libraries or exports produce instruction-located terminal Faults. An existing
symbol's true ABI generally cannot be verified from dynamic-library metadata. A wrong
signature may corrupt memory or crash rather than produce a managed failure. The
binding author must match the header and native lifetime/threading contract. Native
code must not unwind across the boundary. Guest instruction budgets cannot interrupt
an in-progress native call, and native side effects are immediate.

Extensionless library names gain the host prefix/suffix, preserving directories:
`./sample` selects libsample.dylib, libsample.so or sample.dll. Explicit extensions
are literal. Relative paths use the working directory; bare names follow OS loader
search rules. .NET probing, assembly-relative resolution and custom resolvers are
not implemented. Libraries remain loaded for the execution's lifetime; this does
not extend the lifetime of separately allocated foreign memory.

Safe Rust run/run_with_library reject executed imports. Embedders opt into unsafe
run_with_native and uphold its ABI/memory contract. CLI interpreter execution enables
trusted imports for the selected program, like executing native code.

## Low-level metadata and compatibility

The canonical neoIL spelling is `.foreign`, with `.pinvoke` retained as an alias:

```text
.method static Length(String text) -> UIntPtr
    .foreign "sample" "neoclr_utf8_len" cdecl utf8
.end
```

`system` can replace `cdecl`; an optional `utf8`/`utf16` sets the string default.
Per-slot descriptors are currently emitted by Raven/the metadata API, not a separate
neoIL annotation syntax. Calls remain ordinary signature-based calls.

The existing `pinvoke` metadata record stores library, entry_point,
calling_convention (Cdecl/System), optional string_marshalling (Utf8/Utf16),
parameter_marshalling (canonical zero-based decimal-string keys) and return_marshalling.
Per-slot values are Utf8/Utf16/Bool8/Bool32. Decimal-string keys work in both JSON
and the native binary container. No implicit parameter insertion occurs.

Existing artifacts without the optional fields keep their meaning. Older runtimes
cannot read newly extended descriptors; use matching compiler/runtime builds. Rust
NativeImport literals need the new optional/default fields. The independent metadata
API exposes NativeImportDefinition and SetNativeImport, preserves descriptors when
reading native artifacts, and rejects bodies, runtime flags and entry-point imports.
Introspection.MethodInfo.NativeImport exposes the immutable descriptor without
resolving native libraries; Raven retains IsExtern when importing native artifacts.
CLI projections remain reference-only throwing stubs; executable CLI writing rejects
these imports. No partial or host-dependent DllImport marshalling substitute is emitted.

Native intent is an ABI descriptor independent of CLI attributes. Raven's ordinary
.NET extern/DllImport path remains unchanged. No new Runtime Contract setting is
required; the neoCLR backend and explicit registered core/library identities own
admission. The current compiler work is on Raven's separate neoCLR line, not main.
See [compiler integration](raven-cli-bridge.md#foreign-import-attributes-2026-10-12).

## Runtime internal calls and implementation options

`extern` means the source declaration supplies no body. Its provider is identified
separately: `MethodImpl(MethodImplOptions.InternalCall)` selects a runtime-owned
service; `NativeImport` selects a foreign library/export. A declaration cannot select
both. Runtime services use their registered managed signature and runtime conventions,
not the public foreign-function marshaller. The current Raven service admission is
restricted to internal extern namespace functions in neoCLR.Runtime.

Metadata tooling can inspect GetMethodImplementationFlags() and NativeImport separately.
Raven reports IsExtern for imported internal calls and foreign imports alike. Reading
either kind does not resolve a runtime service or load a foreign library. The native
wire format still accepts only zero or InternalCall implementation flags; this does
not add NoInlining, NoOptimization or other MethodImpl options. There is no redundant
implementation-kind enum or change to Runtime Contract configuration.

## Research and alternatives

Author direction: implement platform invocation/FFI, learn from .NET without treating
it as a limit, keep the interface platform-neutral, retain extern plus attributes,
and describe calling convention and individual value marshalling explicitly.
Primary sources retrieved 2026-10-12:

- [.NET interop guidance](https://learn.microsoft.com/en-us/dotnet/standard/native-interop/best-practices)
  recommends exact signatures, explicit encoding and lifetime-aware handles. Its
  boolean examples show why native widths matter. We retain its attribute-based
  ergonomics while requiring explicit Bool8/Bool32 representations.
- [LibraryImport generator compatibility](https://github.com/dotnet/runtime/blob/main/docs/design/libraries/LibraryImportGenerator/Compatibility.md)
  separates source-generated marshalling from DllImport's runtime mechanism.
  neoCLR places the import contract in metadata and interpreter dispatch; it does
  not need a source generator or reproduce platform-dependent ANSI defaults.
- [Rust CString](https://doc.rust-lang.org/std/ffi/struct.CString.html) inspires owned
  terminated input and interior-NUL rejection. [Rust's FFI guidance](https://doc.rust-lang.org/stable/nomicon/ffi.html)
  supplies a useful separation between C/system ABI, pointer ownership and unwinding.
- [.NET issue 92361](https://github.com/dotnet/runtime/issues/92361) requests access to
  the already-encoded UTF-8 byte count for pointer-plus-length APIs. This remains a
  request in the inspected page, not a shipped facility or proof that neoCLR solves
  it. A future bounded-buffer descriptor must express units, direction and ownership.
- [Silk.NET's marshalling account](https://dotnet.github.io/Silk.NET/blog/nov-2020/silktouch-invokes-marshalling/)
  describes generated pointer/string adapters and explicit lengths. This historical
  experience supports keeping ergonomic wrappers in libraries/generators over a small
  runtime ABI. It does not establish current implementation or performance parity.

Pointer-only imports avoid marshaller allocations but repeat conversion/lifetime
logic in every binding. Automatically converting every bool/string is convenient
but hides ABI mismatches. Explicit defaults plus per-slot overrides provide a
small enforceable contract across frontends. Costs are descriptor/API complexity,
copied strings and a deliberate embedded-NUL rejection policy. No .NET execution
comparison or pinned CoreCLR implementation audit is claimed by this slice.

## Validation and next gates

The [focused evidence](experiments/foreign-interop/README.md) covers actual Raven
attribute source, native metadata round trips, positive/negative compiler cases,
real native calls, mixed encoding overrides, both boolean widths/results, System
ABI, static class imports and lazy missing-export failure. Rust's eleven P/Invoke
integration tests retain scalar/pointer/trust checks. Local execution is macOS ARM64;
Windows/Linux qualification and installed bundles remain open.

Next: shared JIT/AOT import lowering; bounded pointer-plus-length input/output buffers;
explicit owned handles and release functions; errno/GetLastError capture; verified
record layout; retained callbacks with thread entry and teardown.
The feature branch adds the bounded callable experiment described below. Native libraries
that retain pointers need explicit ownership, not an annotation that extends a temporary
buffer accidentally. These are gates, not implemented capabilities.

## Structural native Function bindings (feature branch)

The `codex/native-function-pointers` runtime experiment adds the low-level instruction
`function.bind.native fn<...> = {"calling_convention":"Cdecl"}`. It consumes a native
pointer and produces an ordinary structural Function value. ABI JSON accepts the same
string default and parameter/result marshalling descriptors as named imports. No
delegate or source-level Raven mapping API is introduced in this slice.

The binding retains all libraries already loaded by its native execution session.
External addresses still require caller-owned lifetime guarantees; retaining those
libraries cannot prove that an arbitrary address belongs to one of them. Binding and
invocation require trusted native execution. Null and tracked guest-data pointers are
rejected. Signature/ABI mismatch with actual machine code remains the caller's risk.

Clones share binding identity; independently mapped native addresses create distinct
bindings. Shape identity remains structural. ToString identifies a native Function;
the transitional managed Function property faults because a raw address has no managed
MethodInfo. Direct scheduler/worker submission requires a managed wrapper.
Call-scoped callbacks are described below. Exports, JIT/AOT lowering and independent
C# metadata authoring remain unimplemented.

See [the executable example](../examples/foreign_function_pointer.neoil) and the
[author-directed AOT sidecar design](design/native-import-export-contract.md).

## Callable parameters and results (feature branch)

The same interpreter branch now maps structural Function values in both directions.
A parameter has `callback_parameters["index"]: NativeAbi`; a Function result has
`function_return: NativeAbi`. Each callable has an independent convention and
conversion map. The shape remains the declared Function type. These optional fields
are shared by NativeImport and NativeAbi; existing descriptors omit them unchanged.
New descriptors require a matching runtime. This adds fields to Rust struct literals.

The [tested neoIL example](../examples/foreign_callable.neoil) passes a managed
`fn<Int32,Int32,Int32>` to a native function and calls a returned `fn<Int32>`:

```text
.function Apply(fn<Int32,Int32,Int32>) -> Int32
.foreign "./target/native/neoclr_sample" "neoclr_apply_binary" cdecl
.callback 0 = {"calling_convention":"Cdecl"}
.end
.function GetFunction() -> fn<Int32>
.foreign "./target/native/neoclr_sample" "neoclr_get_constant" cdecl
.functionreturn {"calling_convention":"Cdecl"}
.end
```

The callback adapter roots the Function and its captured receiver through the native
call. libffi generates a temporary entry stub; entry rejoins the same invocation,
heap, instruction budget and services without holding the graph or library lock.
Callbacks may invoke ordinary managed functions and make nested native calls.
They do not implicitly drain the enclosing scheduler. Callback entry consumes the
remaining managed frame budget, including its adapter frame, and has a further
32-entry native reentry cap to bound host-stack recursion.

This first callback contract is **synchronous, call-scoped and on the calling thread**.
Native code must finish all uses before returning, and must not store, free or return
the callback pointer. The runtime rejects other-thread entry without running guest
code. Waiting for managed suspension is unsupported. A managed fault or caught Rust
panic produces a zero/false native result (or void); later callbacks in that same
native call also return zero without guest execution. After native code returns,
the first callback fault propagates to the managed caller. Native side effects are
not rolled back, and a native routine that never returns cannot deliver that fault.
A native API needing an immediate error indication needs a matching status protocol.

Callback signatures currently support the numeric scalar types, explicitly mapped
Boolean parameters/results, and Void or no-result returns. Strings, pointers,
records, byrefs and nested callable values within a callback signature are rejected.
A callback parameter must contain a non-null Function with the exact structural
signature. Its ABI cannot also have an ordinary parameter conversion. Unknown,
noncanonical or out-of-range indices and incompatible mappings fail before entry.

A returned native callable is automatically wrapped; users do not manually convert
its pointer. It retains the returning binding's library owners and uses its own ABI
when invoked. A null pointer, a tracked guest data address, or one of the current
call's temporary callback entry pointers is rejected. External dependencies and
native-owned callable state still require a trusted lifetime guarantee. Retaining
a library does not prove ownership of arbitrary executable addresses. A returned
Function can itself be passed as a callable parameter, using an adapter with the
parameter's ABI. ABI descriptor nesting is bounded to eight levels.

The author's `Register` and `GetFunction` declarations are illustrative extern
functions, not a proposed registration API. Their names imply no callback lifetime.
If a foreign function retains a callback beyond the call, that use is not supported
by the current call-scoped contract. Supporting it will require explicit ownership,
release and in-flight-call coordination, plus thread attachment where applicable.
Returning a callable that captures native state likewise needs a future context
and release contract. No retained-callback attribute spelling is being standardized
by this experiment. Raven attributes/compiler emission, C# metadata authoring,
JIT/AOT adapters and sidecar exports remain follow-through.

Compared with .NET's delegate-to-function-pointer marshalling, this adapts neoCLR's
existing structural Function directly, preserving captured state without introducing
a delegate declaration. It retains the distinction between automatic call-duration
rooting and a longer-lived callback registration. The cost is a per-call executable
stub and interpreter entry; no performance advantage is claimed. Raw pointers alone
would avoid this adapter but could not carry a managed captured receiver safely.
Static-only unmanaged exports remain a useful separate AOT path, not a substitute
for captured callbacks. See the [primary comparison sources](design/native-import-export-contract.md#comparison).
The closure storage, widened integer returns and raw entry mechanics were checked
against libffi 5.2.0 / libffi-sys 4.2.2, including its bundled libffi manual, on
2026-10-12. Other-platform and executable-memory-policy qualification remains open.
