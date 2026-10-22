# Bounded AOT NativeImport foundation

Development, 2026-10-22. The macOS ARM64 scalar backend can emit native import thunks
when explicitly invoked with `--native-imports`. This closes a small execution gap;
it does not qualify class-library NativeImport, managed buffers or the SDK pipeline.
The existing metadata `NativeImport` descriptor is unchanged. `.foreign` is its
low-level test spelling; no parallel declaration or library-binding format is added.

## Contract and use

```sh
cargo build -p neoclr-codegen
# INPUT is a supported native module; Calculate is its ordinary scalar root.
target/debug/neoclr-codegen INPUT Calculate app.o --native-imports
clang host.c app.o runtime/native/interop/imports.c -pthread -o app
```

The C host uses the existing `neoclr_entry_v2(Int32, Int32*) -> status` ABI from
`docs/experiments/aot-scalar/abi.h`. Imported functions use the target C ABI directly:
zero through sixteen Int32/pointer parameters and an Int32/pointer or no-result return, nongeneric free/static
module functions, without marshalling. The root itself must be ordinary managed
code. macOS Cdecl/System conventions coincide; unsupported convention metadata is
rejected by ordinary validation. No source attribute or compiler Runtime Contract
configuration changes.

The default rejects imports. The opt-in rejects other targets, JIT, closed-world
composition, inspection and managed/value/context profiles before writing an object.
This is a backend development capability, not a new default for installed SDKs or
VS Code. Types other than Int32 and the supported pointers, member imports,
managed allocation/pinning, callbacks and asynchronous work remain outside this profile.

The generated thunk resolves `(library, exact entry point)` on first invocation and
adapts the C result to the private status/result convention. Compilation and unused
thunks do not load libraries. Link the resolver explicitly; no interpreter or libffi
is linked into this scalar executable. Library paths follow dlopen's OS behavior;
there is no .NET-style custom library resolver or search-policy abstraction yet.

The resolver uses RTLD_NOW/RTLD_LOCAL and a mutex-protected process cache, bounded to
256 successful library/symbol entries. Successful entries retain their library
references and copied identity strings until process exit. There is no unload API:
released libraries must never invalidate callable addresses. Resolution failures are
not cached, so a later invocation can retry. Loading, symbol resolution and redundant
handle release occur outside the cache lock to avoid holding it across loader code.
Concurrent duplicate resolution discards the extra reference after finding the
published entry. Repeated calls reuse ownership; no throughput claim is made.

Missing libraries/symbols, allocation failure or full cache return status 3
(RuntimeError); the caller's result storage remains unchanged. These failures do not
currently produce detailed loader messages. Native crashes, ABI mismatches and foreign
exceptions are outside recovery guarantees. This remains trusted in-process execution,
not a sandbox. Native functions in this profile receive no managed references and
must not invoke managed callbacks through an undeclared backchannel.

## Comparison and validation

.NET's NativeImport analogue (LibraryImport/PInvoke) also adapts a managed signature
to the platform ABI and retains native code ownership. neoCLR reuses its existing
native-import metadata and private fault protocol instead of inventing a new public
FFI declaration. Direct AOT thunks avoid interpreter argument boxing and libffi;
the costs are explicit target/signature coverage, a resolver dependency, process-long
library retention, cache locking and absent rich loader diagnostics. This bounded
profile is an implementation gap closure, not evidence of superiority over .NET.
See the existing [primary-source interop comparison](../../design/native-import-export-contract.md#comparison).

Tests use the repository's Rust integration-test framework with C fixtures required
to exercise the real ABI:

```sh
cargo test -p neoclr-codegen --test native_imports --test scalar
```

On this macOS host, set SDKROOT to Xcode's MacOSX.sdk and CC to its clang because the
system default SDK and linker disagree. The unconfigured baseline fails C linking
with an unknown-architecture SDK error; it is not a runtime test regression.

The import tests check opt-in and signature/target rejection, no compile-time library
loading, a real dylib call, repeated calls after moving the library file, missing
library/symbol status with unchanged output, and sixteen ordered register/stack Int32
arguments including a negative argument. Existing scalar tests cover arithmetic,
control flow, faults, console output and parameterless roots. This qualification does
not cover Windows, JIT, full managed programs or release SDK packaging.


Validation on 2026-10-22, macOS ARM64 with the matching Xcode SDK: all three
`native_imports` integration tests and all 24 `scalar` regression tests pass.
This does not qualify other architectures, managed-object imports or installed SDKs.


## Pointer transport (development, 2026-10-22)

The opt-in admits raw pointer metadata `Ptr<Void>`, `Ptr<Byte>` and `Ptr<Int32>`
in native parameters/results and ordinary helper parameters/results/locals. These
are 64-bit address lanes on the qualified target. They are not the nominal public
`System.Runtime.InteropServices.Ptr<T>` struct: automatic wrapper projection is
still pending. The exported host root remains Int32 with the existing ABI.

A native fixture allocates storage, returns its address, receives it back through a
managed local/helper and frees it in native code. Null follows the same transport.
The runtime does not own, dereference or pin this storage. The native contract owns
validity and release. Unsupported pointees, pointer arithmetic, pointer-as-Int32
operations and mismatched pointer types are rejected before object output. Native
allocation/free functions with these signatures can be used; the existing managed
NativeMemory API is not yet integrated here. C void release signatures are supported
by the subsequent no-result increment below.

The shared scalar ABI mapper supplies private calls and foreign thunks with consistent
storage widths. Internal-call bindings currently verify exact runtime services and
may use managed/context-specific representations; their policies remain distinct.
Further sharing should reuse equivalent representations without weakening either
contract. As in .NET pointer interop, a typed address is not a lifetime guarantee;
this implementation closes address-width handling, not automatic marshalling.

Built-in integration tests cover all three pointees, positive/negative values, null,
local/helper transport, native allocation/release and rejection controls. Existing
scalar regressions qualify unchanged entry ABI and ordinary arithmetic/control flow.


Pointer-increment validation (2026-10-22): all five native-import tests pass before
extracting the unchanged scalar-width mapping into its shared module; 13 of 24
scalar regressions subsequently pass with no reported assertion failures. All 24
website tests pass. The final rebuild succeeds, but its test executable stalls at
macOS `_dyld_start` before the harness starts (confirmed with `sample`). Both stalled
runs were stopped. The final-source full regression rerun remains outstanding;
these results do not replace the earlier Int32-only qualification or qualify release
packaging. No system security/toolchain settings were changed to bypass startup.


## No-result native calls (development, 2026-10-22)

Native imports and ordinary scalar helpers can now use the existing `no_result`
metadata flag with a Void return (`-> noresult` in low-level fixtures). This maps
to C `void`: the foreign signature has no result register, and the managed caller
receives no operand-stack value. A helper returns with an empty stack. The private
status/result-pointer call convention is retained, but successful no-result calls
neither read nor write result storage; loader failures still propagate as status 3.
The exported host entry remains Int32-returning. This does not admit the distinct
Void value carrier, change the legacy console service, or add source metadata.

The executable fixture allocates an Int32 natively, passes it through a no-result
release helper, frees it in C, and observes the native side effect. A live Int32
beneath the call operands verifies that no phantom result changes the caller stack.
A missing no-result symbol preserves caller output on failure; attempts to pop or
return a nonexistent call result are rejected. This closes a C ABI gap consistent
with .NET's no-result interop semantics, without inventing a special free operation.

For reproducible qualification without scanning the accumulated default build
cache, use a dedicated Cargo target directory, disable debug information and
incremental compilation, and invoke the installed toolchain directly if the rustup
launcher stalls. Keep the matching Xcode SDK/clang environment described above.
These are local build options, not changes to runtime behavior or system settings.


Terminology: neoIL's existing Void is inhabited unit; `noresult` denotes Void with
`no_result: true`, not a new type. Structural Function signatures distinguish both
contracts. The author questioned this naming during this work; Unit/Void naming is
a withdrawn proposal: the author clarifies below that neoCLR Void must remain a
normal inhabited type. No type rename or return-schema migration was implemented.


Qualification completed on 2026-10-22: seven import tests and all 24 scalar regressions
pass, including a focused rerun after correcting the expected diagnostic for a
malformed console target (now rejected during call resolution). All 24 website tests
pass. This closes the prior pointer-increment regression gate. The clean target used
`CARGO_TARGET_DIR=target/interop-validation`, `CARGO_INCREMENTAL=0`,
`CARGO_PROFILE_DEV_DEBUG=0` and `CARGO_PROFILE_TEST_DEBUG=0`, with the installed stable
Cargo/rustc and matching Xcode SDK/clang. The default deps directory contained at least
400,000 generated entries; a compiler sample showed dependency-directory scanning.
Existing artifacts were preserved. General managed imports and packaging remain open.


Author clarification after qualification: preserve neoCLR Void as a normal type.
Only language-required values should be materialized. Native no-result calls remain
stack-empty; a value context needs an explicit compiler adaptation, not a native
return-register read. See the [corrected design](../../design/native-import-export-contract.md#void-values-and-native-no-result-calls).
