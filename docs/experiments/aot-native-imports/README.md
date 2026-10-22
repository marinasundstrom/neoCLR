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
zero through sixteen Int32 parameters and an Int32 result, nongeneric free/static
module functions, without marshalling. The root itself must be ordinary managed
code. macOS Cdecl/System conventions coincide; unsupported convention metadata is
rejected by ordinary validation. No source attribute or compiler Runtime Contract
configuration changes.

The default rejects imports. The opt-in rejects other targets, JIT, closed-world
composition, inspection and managed/value/context profiles before writing an object.
This is a backend development capability, not a new default for installed SDKs or
VS Code. Types other than Int32, no-result calls, member imports, pointers, native
allocation, pinning, callbacks and asynchronous work remain outside this profile.

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
