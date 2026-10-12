# Source foreign import validation (2026-10-12)

This development slice compiles the actual Raven interop attribute source into a
System.Runtime reference, compiles extern consumers with the neoCLR backend, writes
binary native metadata, rereads it and runs it against a Rust native fixture.
It uses the Raven `codex/source-object-metadata-resolution` line, separately from
Raven main. See [validation.json](validation.json) for base revisions, input hashes,
commands and ten successful cases, including the expected missing-export fault.
The report identifies working-tree implementations rather than claiming its base
commits contain the new implementation.

Build the matching runtime and compiler probe first:

```sh
cargo build --locked --bin neoclr
dotnet build /path/to/Raven/tools/NeoClrMetadataProbe/NeoClrMetadataProbe.csproj -p:NeoClrMetadataProject=/path/to/neoclr/tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj -m:1
dotnet build docs/experiments/raven-target/Probe.csproj -p:RavenRoot=/path/to/Raven -m:1
dotnet docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll --reference-core target/foreign-api/NeoCLR.CoreProbe.dll
python3 docs/experiments/foreign-interop/validate.py --raven /path/to/Raven --core target/foreign-api/NeoCLR.CoreProbe.dll --output target/foreign-validation --report docs/experiments/foreign-interop/validation.json
```

On this host the command-line-tools SDK 27 stubs were incompatible with the selected
linker. The successful harness run set `SDKROOT` to Xcode's MacOSX26.2.sdk; no global
toolchain settings changed. The host was macOS ARM64. The core above is an explicit
reference-only bootstrap, not an installed native class-library bundle.

Additional focused validation: all 11 Rust pinvoke tests, metadata author/readback
and rejection checks (`NeoCLR.Metadata.Experimental.Tests --foreign-imports`),
and seven existing Raven extern parser/semantic/.NET emission tests passed.
The compiler probe includes rejected declarations alongside the ten emitted cases.
API reference source/fingerprint validation passes. No website build was required.

Qualified behavior includes scalar calls, default export names, static class imports,
UTF-8/UTF-16 and mixed per-slot encodings, Bool8/Bool32 parameters and results,
System ABI on this host, and deferred missing-export failure. Rust tests additionally
cover empty strings, embedded NULs and incompatible descriptor rejection.
This does not qualify Win32 stdcall, other hosts, JIT/AOT imports, installed bundles,
returned-string ownership, buffer lengths/layouts, callbacks or custom marshallers.

## Imported symbol follow-through (2026-10-12)

The actual attribute-source probe now reloads all ten emitted binary artifacts into
Raven and checks exactly one extern method plus a non-extern managed Main. Both
namespace functions and static class imports pass. The regression failed at ForeignAdd
before NativeMethodSymbol.IsExtern was changed to use the public immutable metadata
descriptor. Inspection passes with the deliberately nonexistent library name
`nonexistent-native-library`; this follow-through does not execute native code.

```sh
dotnet /path/to/Raven/tools/NeoClrMetadataProbe/bin/Debug/net10.0/NeoClrMetadataProbe.dll --native-foreign-source target/foreign-api/NeoCLR.CoreProbe.dll nonexistent-native-library target/foreign-symbol-validation runtime/raven/src/System/Runtime/InteropServices/NativeImportAttribute.rvn
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests/NeoCLR.Metadata.Experimental.Tests.csproj --no-restore /property:WarningLevel=0 -- --foreign-imports
```

Both commands passed. Metadata checks cover the unchanged string default, System
calling convention, Bool8 parameter/Bool32 return descriptors, defensive copying and
null descriptors for ordinary methods. Existing native execution evidence above is
reused because invocation/emission behavior did not change. This adds a host metadata
inspection member documented in the manual experimental metadata reference; guest
attribute signatures and their reference snapshot are unchanged.

The API snapshot check was attempted during this follow-through and reports stale
inputs from concurrent HTTP/TLS source and reference-declaration work (CoreDeclarations,
HttpBindings, SourceUnionReferences, SslDeclarations, Security/Ssl* and Web/Http*).
Those changes are outside this slice; the snapshot was not refreshed from a mismatched
reference assembly. None of the changed fingerprint inputs belongs to this follow-through.

## Internal-call identity follow-through (2026-10-12)

The same attribute-source probe additionally reloads an unbound runtime internal-call
declaration and verifies static, non-abstract, extern identity without resolving a
service. This failed before the InternalCall flag was included in IsExtern and passes
afterward; all ten foreign readback cases still pass. The metadata --foreign-imports
checks also pass, distinguishing zero implementation flags on managed/foreign methods
from InternalCall, verifying a null native descriptor on the runtime service and
rejecting attempts to attach a native import to it. No runtime dispatch code changed.
The API snapshot check was repeated and remains stale from the concurrent unrelated
source inputs noted above; guest reference signatures are unchanged by this slice.

## Structural native-address binding (feature branch, 2026-10-12)

Branch `codex/native-function-pointers` adds function.bind.native, producing the
existing structural Function from a pointer and NativeAbi contract. The independent
Raven compiler probe above is unchanged; the new instruction is authored in neoIL.
The [sample](../../../examples/foreign_function_pointer.neoil) obtains an Add address
from a real native library and invokes it with 19 and 23, yielding 42.

Focused reproduction from this branch:

```sh
CARGO_TARGET_DIR=target/ffi-validation cargo test --locked --test pinvoke --test function_objects -j 4
CARGO_TARGET_DIR=target/ffi-validation cargo test --locked --lib interop::tests::address_binding_retains_library_after_session_drop -j 4
CARGO_TARGET_DIR=target/ffi-backend-check cargo check --manifest-path tools/aot-poc/Cargo.toml --locked -j 4
python3 scripts/build-api-docs.py --check
```

This host uses the Xcode MacOSX26.2 SDK override described above. Private target
directories avoid contention with unrelated workspace checks. The integration cases
cover JSON and binary NEOX round trips, returned-address invocation, per-slot UTF-8
and Bool8 conversion, System ABI on this host, null/data-pointer rejection, trusted
execution, binding identity and the absence of a managed reflection target. The
unit check verifies library ownership survives session drop and releases with the
last binding. The backend check establishes compilation only, not native lowering.

No new guest attribute signature or reference assembly is required. Raven source
mapping, independent C# metadata authoring, native backend lowering, callback exports
and AOT sidecar generation remain open. Future exports should use the same signature
and ABI vocabulary; see [the contract direction](../../design/native-import-export-contract.md).

[Recorded results and source fingerprints](pointer-validation.json): 15 interop tests,
26 existing Function-object tests and the library-lifetime test pass on macOS ARM64.
The native backend compilation and API snapshot checks also pass.
