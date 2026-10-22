# Bounded AOT NativeImport foundation

Development, 2026-10-22. The macOS ARM64 scalar backend can emit native import thunks
when explicitly invoked with `--native-imports`. This closes a small execution gap;
it does not qualify managed class-library bodies, buffers or the ordinary SDK AOT pipeline.
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
zero through sixteen Int32/native-sized-integer/pointer parameters and an Int32/native-sized-integer/pointer or no-result return, nongeneric free/static
module functions, without marshalling. The root itself must be ordinary managed
code. macOS Cdecl/System conventions coincide; unsupported convention metadata is
rejected by ordinary validation. No source attribute or compiler Runtime Contract
configuration changes.

The default rejects imports. The opt-in rejects other targets, managed JIT, closed-world
composition, inspection and managed/value/context profiles before writing an object.
This is a backend development capability, not a new default for installed SDKs or
VS Code. Types other than Int32, IntPtr/UIntPtr and the supported pointers, member imports,
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
metadata flag with a Void return (`-> void` in low-level fixtures). This maps
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


The current spelling is `-> void`, including structural `fn<void>` results.
`noresult` is rejected by the assembler. The internal `no_result` ABI metadata is
unchanged. Canonicalizing legacy named-Void returns remains pending; see
[the Void migration status](../../void-semantics.md).

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

## Raven source and explicit verification context (2026-10-22)

The scalar NativeImport profile now accepts trailing `--system`, `--module` and
`--object-root` inputs for metadata verification. Supply the matching seed/core library;
`--system` is required when a load context is supplied. The ordinary LoadedProgram
loader verifies identity, references, accessibility and bodies in that context. No
reference is stripped or treated as resolved merely because its code is unused.
Only the original application is emitted. Existing scalar admission still rejects
type definitions, external managed calls, unsupported values and marshalling; this
is not closed-world managed-library NativeImport support.

Use `@entry` for a Raven executable's metadata entry point. Source names may be lowered
to identity-qualified names; no new root-name guessing or metadata naming rule is added.

```sh
neoclr-codegen RavenAotImports.dll @entry app.o --native-imports \
  --system /path/to/bundle/lib/System.runtime.neox \
  --module /path/to/bundle/lib/System.Runtime.dll \
  --object-root /path/to/bundle/lib/System.Runtime.dll
```

The checked-in `raven/` consumer uses NativeImport attributes and raw `*int` parameters
and results, with `unit` for a no-result release helper. It allocates an Int32 in C,
reads 42, frees it through a Raven helper and checks the live allocation count. The
Rust integration test compiles Raven to PE/#Neo directly, emits an AOT object, links
the resolver/C host, and executes 32 calls. It also rejects missing source dependencies
without writing an object. The project and all generated artifacts are copied into a
temporary test directory; no hand-authored neoIL supplies the consumer's behavior.

```sh
NEOCLR_RAVEN_BUNDLE=/absolute/path/to/matching-bundle \
  cargo test -p neoclr-codegen --test native_imports raven_native_metadata \
  -- --ignored --nocapture
```

This opt-in test requires an external matching Raven compiler/bundle. It is ignored in
the default Cargo run with an explicit reason, not silently reported as passing. The
Rust test framework owns assertions because the bounded scalar profile cannot yet
execute the managed Raven test runner's Result/class contracts. The ordinary seven
native-import tests and 24 scalar regressions remain the default gate.

Qualification uses the local native-callback bundle with Raven `5966b5f84`, macOS ARM64,
matching Xcode clang/SDK and the dedicated interop-validation Cargo target. This is a
source-to-native ABI proof, not an installed SDK release. Compared with .NET Native AOT,
metadata dependencies are likewise validated at compilation, but this profile still
supports only a bounded scalar application and requires explicit native linking.

Final focused validation: the opt-in Raven test passes, as do all seven default import
checks, all 24 scalar regressions and all 24 website checks. The Raven test is separately
selected rather than counted as passed in the default run where it is ignored.

## Pointer conversions and defaults (2026-10-22)

The scalar NativeImport profile now handles `ptr.cast` and `ptr.null` for its existing
Void/Byte/Int32 pointee set. Casts preserve every address bit; they neither dereference
nor allocate storage. Null materializes a pointer-width zero. Unsupported target types,
integer-to-pointer casts and use without the native-import opt-in reject before output.
Ordinary verification still enforces exact pointee types at calls/stores/returns.

Raven `default(*int)` currently emits local-address initialization (`ldloca`/`initobj`)
followed by a local read. The profile admits exact local initialization for its Int32
and pointer storage slots. Lowering initializes the SSA local directly; it does not
expose a native address for that local. Flow analysis retains the local's identity and
rejects escaping addresses, mismatched initialization and conversion of a local address
to an unmanaged pointer. The ordinary verifier retains definite-assignment checks.
This is bounded local initialization, not general byref or arbitrary initobj support.

The Raven consumer now receives `*unit` (native void*), explicitly converts it to
`*int` for reading and implicitly erases the pointer type when releasing it. It also
passes `default(*int)` to the fixture's null-safe release function. C allocation,
reads and release still execute over 32 native calls; the native allocator owns storage.
This follows .NET unsafe pointer-conversion and value-initialization semantics without
adding a marshalling API, taking ownership or claiming a pointer cast establishes safety.
The release still lacks Ptr<T> projection, general pointer arithmetic/dereferencing,
managed-memory pinning and managed-library NativeImport AOT composition.

Validation of this follow-through: all ten native-import tests pass with the explicit
Raven opt-in enabled, all 24 scalar regressions pass, and all 24 website checks pass.
The pre-fix Raven test rejected ptr.cast; the intermediate run exposed ldloca/initobj,
which is now supported within the stated local-initialization boundary.

## Native-sized integer transport (2026-10-22)

IntPtr/UIntPtr parameters, results, locals and helper calls now preserve the target's
64-bit width in this macOS ARM64 profile. Raven spells them `nint`/`nuint`; a `size_t`
argument no longer needs an incorrect Int32 declaration. The external entry ABI remains
Int32. Native integer signedness is retained by flow analysis and ordinary verification.

The supported unchecked conversion operations are conv.i/conv.u, conv.i8/conv.u8 and
conv.i4, with Int64 constants/intermediate values admitted for those conversions.
Widening Int32 uses sign extension for signed opcodes and zero extension for unsigned
opcodes, matching the interpreter's existing numeric contract. Same-width conversions
preserve bits; conv.i4 retains the low 32 bits. This does not admit Int64/UInt64 function
signatures, native-width arithmetic, checked conversions, floating conversions or
pointer-to-integer conversion. Unsupported operations reject before output rather
than silently using Int32 arithmetic. Pointee-layout support remains unchanged.

The Raven C fixture now takes a real `size_t` allocation size, returns a value with
high unsigned bits set and a negative intptr_t outside the Int32 range, and checks
both after Raven local/helper transport. A separate Rust/C test checks nine ordered
native-sized arguments across register/stack locations, signed/unsigned widening and
return narrowing. This follows the C ABI/.NET native-sized integer model and reuses
the existing neoCLR conversion semantics; it adds no implicit ownership or buffer
length inference. NativeMemory's managed library APIs and full SDK AOT integration
remain separate work.

Validation: all 12 native-import tests pass with the explicit Raven bundle enabled,
all 24 scalar regressions pass, and all 24 website checks pass.

## Pointer and native-integer conditions (2026-10-22)

The bounded scalar profile now admits ceq, beq/bne.un and brtrue/brfalse for
supported pointers and native-sized integers through its existing full-width
comparison lowering. Ordinary metadata verification still requires compatible
operand types. This follows .NET/C address-equality semantics: the operation does
not dereference the target or validate its lifetime. Pointer ordering, arithmetic
and pointer/integer conversions remain excluded.

The Raven consumer checks for failed allocation before Read, exercises a deliberately
rejected allocation size and validates pointer identity. A native fixture compares
opaque sentinel addresses with identical low 32 bits but different high bits,
checks null/non-null branches and signed/unsigned native-width zero/equality tests.
The sentinels are never dereferenced. Rejection controls retain pointer ordering,
mixed pointer/integer and incompatible pointee comparisons outside the profile.
This is backend acceptance of existing Raven output; no Runtime Contract or metadata
encoding changes. No public library API or new allocation is introduced.

Validation: 13 native-import tests (including the explicitly enabled Raven consumer),
24 scalar regressions and 24 website checks pass. Expanded rejection controls also
pass in a focused rerun. The same local Raven 5966b5f84 compiler overlay is used;
this does not qualify a released SDK or add managed AOT integration.

## Direct native scalar memory access (2026-10-22)

The opt-in macOS ARM64 scalar profile admits Byte/Int32 ldobj/stobj and the
corresponding ldind.u1/ldind.i4/stind.i1/stind.i4 operations on unmanaged pointers.
Byte loads zero-extend to the evaluation stack's Int32 representation; stores use
the storage width. Managed local addresses remain confined to default initialization,
and unsupported pointees, managed references and unaligned prefixes remain rejected.

Null addresses and misaligned Int32 accesses return RuntimeError (status 3) before
touching memory, preserving the host output on failure. These checks do not establish
bounds, initialization, mutability or lifetime. Other invalid addresses can crash the
process. This is trusted native execution, like unsafe pointer access at a .NET native
boundary, not the interpreter's tracked-allocation safety model. Explicit alignment faults
retain neoCLR's checked memory-access convention; .NET unsafe access can instead
follow target-specific unaligned behavior. The cost is a null check and, for Int32,
an alignment check per access; this slice makes no throughput claim. No managed storage
pinning, native Slice borrowing or ownership inference is introduced.

The Raven consumer reads and updates a C allocation directly, then checks the write
through C before releasing it. Rust framework/C fixtures also cover unsigned byte
loads, signed Int32 storage and null/misaligned reads and writes. This uses the
existing metadata memory operations, not a new public API or bytecode encoding.

Validation: all 14 native-import tests pass across the 13-test runtime run and the
explicit Raven consumer rerun with the repaired compiler/SDK overlay. All 24 scalar
regressions, 22 Raven portable/.NET pointer tests, the SDK pointer/local/byref/out
contract group and 24 website tests pass. The four new compiler cases and SDK pointer
access test reproduced their respective rejection before the fixes. The initial
compiler overlay used the wrong host framework; rebuilding it for the bundle's
net10.0 host corrected that setup error. This is a local paired overlay, not an
installed-SDK or release qualification.

Compiler pairing: Raven `3fc2b6d0e` with the metadata validator in this slice.

## Shared scalar JIT import execution (2026-10-22)

`neoclr-codegen --jit-scalar input.neoil Calculate 4 --native-imports` now uses the
same scalar validation, import thunk generation and C resolver as AOT on macOS ARM64.
The codegen build links `runtime/native/interop/imports.c` through the existing locked
cc dependency on that target; AOT consumers still link the same adapter explicitly.
The JIT binds only the resolver's exact private symbol. Compilation/finalization do
not load foreign libraries; resolution happens on invocation. The shared cache owns
successful foreign libraries for process lifetime, independently of each JIT code
owner. Generated callable addresses do not escape the synchronous execution scope.

The default JIT rejects imports. Other targets, managed JIT, callbacks and managed
class-library composition remain excluded. The scalar JIT CLI does not yet accept
Raven's seed/core verification context; the source consumer remains AOT-qualified.
This is a development probe, not ordinary SDK/runtime JIT NativeImport completion.
The Int32 entry ABI, private status/result convention, bounds/lifetime responsibilities
and scalar signature restrictions are unchanged. No new public import declaration,
Runtime Contract setting or bytecode encoding is introduced.

The Rust integration test invokes C allocation/release and high-bit native-integer
returns through the JIT, with direct pointer reads/writes. Thirty-two repeated calls
check successful reuse without outstanding allocations; null access and missing
symbols return RuntimeError, and absence of the explicit flag rejects before execution.
This shares .NET-style platform-ABI binding across compilation modes while retaining
neoCLR's explicit trusted-native opt-in and limited profile; full cross-engine parity
and export-name design remain later work.

Validation: all 15 native-import tests (including the explicitly enabled Raven AOT
consumer), 24 scalar regressions and 24 website checks pass. The AOT consumer uses
Raven `3fc2b6d0e` and the metadata SDK from neoCLR `417e7a451` in a local bundle overlay.
Building the codegen host on macOS ARM64 now also compiles the shared C resolver;
use a matching C toolchain/SDK. Interpreter-only builds are unaffected.

## Raven metadata contexts in scalar JIT (2026-10-22)

The scalar JIT now accepts `@entry` and the same explicit trailing `--system`,
`--module` and `--object-root` inputs as AOT when `--native-imports` is enabled.
Path-option parsing, Object-root selection and LoadedProgram construction are shared
with the existing AOT/closed-world driver. Full dependency verification is retained;
dependency bodies do not become compiled callees in this scalar profile. Missing
seed/core references, duplicate options and malformed trailing pairs reject before
execution. Object roots still must identify exactly one explicit compiled dependency.

The existing Raven allocation/read/write/release consumer is compiled once and run
32 times under scalar JIT and 32 times as linked AOT code, preserving the result 42.
It also covers null/default pointers, unit/no-result release, high-bit native sizes
and negative offsets. The explicit Rust-framework integration test continues to
require the matching local Raven bundle; built-in Raven test-runner managed types
remain outside this profile. No source/compiler/metadata encoding or Runtime Contract
change is required. This closes a driver verification-context gap, not full managed
JIT/AOT NativeImport qualification or the subsequent export-name design milestone.

Validation: all 15 native-import tests (Raven opt-in enabled), 24 scalar regressions,
two closed-world runtime-context tests and 24 website checks pass. The paired local
bundle remains Raven `3fc2b6d0e` plus the metadata validator from neoCLR `417e7a451`;
no rebuilt release package or wider native export contract is claimed.

## Three-engine transport baseline (2026-10-22)

`raven/Transport.rvn` now provides a shared source contract executed 32 times in
each of the interpreter, scalar JIT and linked AOT modes. C owns allocation, reading
and release. Raven checks a null return, casts/forwards the pointer, calls no-result
release (including null release), and forwards high-bit unsigned sizes and negative
signed offsets through locals/helpers. Each run returns 42 and the C live-allocation
counter returns to zero. All engines consume the same Raven-produced metadata;
the harness recognizes NEOX and PE/#Neo by content, not by a .dll filename suffix.
Interpreter execution requires the explicit trusted-native API; the safe API rejects.

The original `Main.rvn` remains a separate JIT/AOT direct pointer read/write proof.
Interpreter pointer storage only permits dereference of runtime-tracked allocations;
a foreign returned pointer does not acquire bounds, initialization or lifetime from
its declared pointee type. Direct access to untracked foreign storage still rejects.
That engine difference is an implementation/ownership-contract gap, not a proposed
permanent split in language semantics. Do not remove the interpreter check merely
to make the unsafe AOT example pass. Full foreign-buffer borrowing, managed-library
integration, pinning, callbacks and broader scalar ABI coverage remain open before
claiming complete three-engine NativeImport support or moving to export-name design.

Both consumers use the repository Rust test framework because the bounded scalar
backend cannot execute the managed built-in Raven runner. C fixtures exercise the
actual ABI, and the harness compiles the Raven source once per consumer and cleans
its temporary files. This is qualification of existing behavior, not new runtime
capability or release-package certification.

Validation: both focused Raven source integration tests pass with the explicit bundle
opt-in, and all 24 website checks pass. The unchanged scalar backend regressions
retain their preceding evidence. The compiler/SDK overlay revisions remain as above.
