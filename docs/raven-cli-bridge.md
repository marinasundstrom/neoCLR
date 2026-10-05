> 2026-10-02 metadata-branch update: codex/extended-cli-metadata now integrates
> structural Functions from codex/structural-types at the author's direction. The
> nominal baseline descriptions below are historical. Use the regenerated Function
> bundle for this branch; direct native metadata callback emission remains in progress.

# Raven CLI bridge

## Self integration on nominal main (2026-09-30)

Self is integrated independently of `codex/structural-types`. Raven projects must
select `RavenTargetPlatform=NeoCLR` and configure the fieldless
`System.Runtime.CompilerServices.Self` marker in `NeoCLR.CoreProbe`. The shared
props do this. Raven's .NET target rejects that configuration with RAVT003.

The importer replaces the marker with native Self and admits bounded Number and
Clonable generic dispatch, including borrowed receivers. See
[self contracts](self-types.md) and [the consumer](experiments/native-self/README.md)
for inheritance and current importer restrictions. A real neoCLR metadata layer
must replace marker recognition with native contract identity/signatures; it must
preserve conformance ownership, substitution and unsupported-use diagnostics.
Compiler binding/diagnostics belong to Raven; CLI translation is owned by
`docs/experiments/raven-target` and native validation/dispatch by `src/self_types.rs`.

The nominal delegate ABI still uses Func with an inhabited Void result. This is
required even without structural Functions; Raven retains that bridge encoding.
Structural identity, Function introspection and structural assignability remain on
feature branches in both repositories. Do not use a Function-branch bundle as
acceptance evidence for this integration. General core, unit, tuple, iteration,
propagation and typeof encodings remain described in Raven's
`docs/compiler/neoclr-cli-bridge.md`; this CLI bridge is not a native metadata loader.

## Native metadata exploration (2026-09-30)

The [extended CLI metadata design](design/extended-cli-metadata.md) starts on
`codex/extended-cli-metadata` from main. It includes structural type identity,
synthesized members, explicit capabilities and a staged Raven importer/emitter path.
The [standalone codec and bounded PE probe](experiments/extended-cli-metadata/README.md)
now transport #Neo for inspection. .NET and Cecil read conventional metadata unchanged,
but Cecil rewriting strips the stream. Do not route native metadata through an ordinary
Cecil rewrite without preservation/remapping support. Current marker/carrier behavior
and Runtime Contract configuration remain as described above. No native runtime-loaded
artifact or Raven compiler bundle is validated by this experiment.

The [reader/writer architecture](design/extended-cli-metadata.md#reader-and-writer-support-on-net-and-neoclr)
plans a .NET-hosted metadata library for Raven and corresponding support on neoCLR,
including a guest-accessible library. Format codecs, semantic resolution and Raven
symbol/emission adapters remain separate; the current bridge is not that library.

The [provisional recognition contract](experiments/extended-cli-metadata/README.md#marked-artifact-recognition-2026-09-30)
requires an explicit expected-extended input profile, a metadata-root marker and a
matching stream digest. Future Raven native-metadata loading must reject failures
without falling back to carrier/ordinary CLI interpretation; ordinary .NET targeting
remains separate. This is a Python probe contract, not an implemented Raven loader.

## Independent metadata consumer, stage 1 (2026-09-30)

Raven's `codex/metadata-consumer` at `7e18edb66` adds an opt-in
`tools/NeoClrMetadataProbe` consumer of the independent metadata library. The frontend
uses the existing .NET provider and default runtime contract to bind primitive Int32
source and an API-produced PE library. A compiler-side adapter (now consuming shared lowered bodies) emits
the application as native format-5 JSON through the metadata API. The dependency is
also emitted natively by that API. neoCLR loads/verifies both and returns 42.
The application does not go through the existing CLI import bridge.

This is a staged bootstrap, not a new Runtime Contract option or a completed native
ICompilationEmitter/ISemanticDataLoader. The PE dependency and host-core identity are
temporary inputs to the .NET frontend. Native top-level functions have no artificial
user type. Only required Int32 values, returns, primitive unchecked/unlifted arithmetic,
local calls and the explicit static dependency are supported. Other constructs and
unresolved methods are rejected; attributes, defaults, debug data and structural
contracts are not silently advertised as preserved. Parameter names/source mappings
are not yet emitted by the native subset.

Ownership remains Raven symbols/operations/adapters -> separate metadata model/format
library -> native runtime loader/verifier/VM. The next stages replace the .NET metadata
bootstrap with native symbol loading, integrate target diagnostics/configuration, and
expand ordinary format support using this executable consumer. Shared operations and
missing-Param-row fixes are general Raven corrections, already fast-forwarded to local
main as `1ea0ca263` and `d7040e21d`; the consumer remains experimental.

Validation: the .NET 10 consumer runs the native output to 42 and rejects unsupported
division plus an unresolved method. Raven's .NET 11 operations/default-parameter filter
passes 87 tests; two invocation and one missing-parameter-row regression failed before
the fixes. Compiler builds pass for .NET 10/11. See
[hash evidence](experiments/extended-cli-metadata/raven-compiler-validation.json) and
[reproduction](experiments/extended-cli-metadata/README.md#raven-compiler-consumer-stage-1).
No full native target, structural execution, NanoFramework or release is claimed.

### Read-only call imports — 2026-09-30

The independent host library now provides `AssemblyBuilder.ImportReference` and
`MethodBuilder.Call(ImportedMethodReference)`. This follows the existing Cecil-style
import direction: a compiler consumes immutable definitions and imports a scoped
reference into its output. Compared with CLR reflection/Reflection.Emit, this needs
no loaded runtime assembly or executable method handle. Compared with full Cecil
imports it deliberately supports only unsigned, top-level-owner static Int32/void
signatures; no generic substitution, access checking or arbitrary IL translation.
The cost is an explicit host assertion of the dependency core contract and a bounded
format-5 naming agreement. Private reference-only nodes reuse both writers' existing
call encodings without retaining producer bodies. A future general signature model
should replace these bounded nodes as supported compiler cases require it.

Raven's feature-branch probe now receives only the read-only dependency snapshot;
it does not receive its builder graph. The primitive .NET Runtime Contract remains
the binding bootstrap, and ordinary .NET code generation is unchanged. Native
semantic-data loading and production emitter installation remain the next integration
stages; structural support stays later. C# contract tests pass (22 groups); both the
native global-function gate and Raven probe verify/run to 42. Missing dependency,
wrong revision, unsupported operation and binding diagnostics remain checked.

### Compiler-owned native adapter checkpoint — 2026-09-30

The working operations consumer is now the optional `Raven.CodeAnalysis.NeoClr`
project, with `NeoClrCompilationEmitter.Emit`, immutable output/core/dependency options
and a success/diagnostic result. The probe is a C# caller of this reusable adapter.
The metadata API remains a separate project with no Raven dependency. The new project
is opt-in through NeoClrMetadataProject; ordinary .NET behavior/default builds and
`Compilation.Emit` composition are unchanged.

Calls use explicit compiler-reference-to-snapshot bindings and resolved assembly-symbol
identity instead of a simple-name selection. Host snapshot consistency and the primitive
core assertion remain explicit responsibilities until a native provider owns them.
NEOMETA001 carries source locations; NEOMETA002 rejects incompatible configuration;
NEOMETA003 reports writer limits/invalid graphs. Original binding diagnostics survive.
All validation precedes output writes; host I/O failures propagate and can partially
write. Supported source/format-5 encoding remains the existing static Int32 subset.

C# consumer checks cover expression spans, unchanged rejected output, original error
identities, invalid/duplicate/unregistered/core bindings, writer limits, multi-tree
rejection, repeated output and stream ownership/failure. The emitted application still
verifies/runs in neoCLR with result 42. Hash evidence includes the new adapter binary.
Native symbol loading and production target registration remain pending. Structural
support remains later; this does not change the runtime bridge's platform capabilities.

### Multi-file native adapter checkpoint — 2026-09-30

The optional compiler-owned emitter now accepts multiple source trees. It collects
all supported top-level declarations before emitting bodies and retains each body's
own semantic model. Like the ordinary .NET compiler, valid cross-file calls bind
independently of file order; native metadata token/declaration order still follows
input order. No shared binding or .NET emitter change was required. Macro trees and
unsupported constructs remain excluded under the existing explicit bootstrap contract.

The new two-file regression first failed with the adapter's NEOMETA002 single-tree
restriction. After the refactor, Helper.rvn/Main.rvn and the reversed input order
both verify/run to 42 in neoCLR. A division expression in the later helper file
produces NEOMETA001 with that file's source location and leaves output unchanged.
The original one-file and adapter contract checks still pass. Validation evidence
includes hashes for all three applications. Native symbol loading and production
registration remain separate next steps; the metadata library stays independent.

The tested adapter source is committed on Raven `codex/metadata-consumer` at
`03705e88b` (adapter extraction `dc84dd07b`). The native runtime and independent
metadata library remain on neoCLR `codex/extended-cli-metadata`; the paired binary
and output hashes are in `docs/experiments/extended-cli-metadata/raven-compiler-validation.json`.

### Native dependency input through a reference projection — 2026-09-30

The independent metadata project now reads its bounded native format-5 declaration
contract with `NativeAssemblyDefinition.ReadAssembly`. It checks manifest/name/origin
consistency, duplicate and unsupported declaration fields, owner/signature contracts
and resource bounds. Bodies remain opaque: successful metadata reading does not imply
successful native verification or execution. General native schemas, arbitrary types,
structural metadata and executable rewriting remain unsupported.

The snapshot can create a reference-only PE using an explicit core identity. The
projection preserves supported callable/type declarations and marks the assembly with
ReferenceAssemblyAttribute; placeholder bodies throw. It omits the native entry point
and implementation dependency references. These signatures need only primitive types.
This is the temporary native-input bridge, following the .NET separation of compilation
contracts and executable implementations, not a new executable CLI representation of
native code. The cost is an extra PE and the existing .NET symbol provider. Native
ISemanticDataLoader/symbol construction should ultimately consume native declarations
directly; the projection must then be retired, not made a permanent platform rule.
Primary comparison: [Microsoft reference assemblies](https://learn.microsoft.com/en-us/dotnet/standard/assembly/reference-assemblies).

The Raven probe now emits only the original native dependency from its producer,
reads that native artifact, and creates MetadataProbeLibrary.reference.dll from the
reader. It binds that reference with the existing .NET primitive Runtime Contract,
imports the read-only callable contract, and emits native applications. The original
native dependency (never the reference PE) is supplied to neoCLR. The one-file and
both two-file orders verify/run to 42; diagnostic/stream checks still pass. The report
records the reference projection hash alongside the native dependency/application hashes.

Ownership: the metadata project owns reading/projection; Raven owns compiler binding
and emission; the host supplies matching explicit core identities and native runtime
dependencies. Ordinary .NET defaults and existing CLI targets are unchanged. The
metadata project stays separate, and all work remains on the existing feature branches.
C# contracts pass (23 groups), including malformed inputs, projection ownership,
reference marking and rejection of execution loading by .NET. No production native
semantic loader or target registration is claimed.

### Raven library-to-application native case — 2026-09-30

Both sides of the integration case now originate in Raven source. The optional adapter
accepts library output without an entry point and public nongeneric static classes in
the global namespace containing public static Int32 methods. Ordinary Raven default
public method accessibility is accepted. Nonpublic members/types/library globals and
additional type contracts are rejected; the public-only metadata writer must not
silently widen a library's visibility. Console top-level functions remain native
functions outside types. Broader visibility/namespace/type support is still pending.

The producer declares MathLibrary.Twice overloads and a Multiply helper. Raven emits
the library as native format 5; the independent metadata API reads it and projects
reference-only declarations. A separate Raven application binds the one-argument
overload, emits a native external call, and neoCLR executes the original library's
local helper call. The one-file and both multi-file input orders return 42. No producer
builder graph or hand-authored native dependency body is used in the case.

The new case first failed with NEOMETA002 because the adapter accepted only console
output. Focused C# checks now confirm entry-less library output, overload/local-call
execution, source-located visibility/type rejection with unchanged output, and native
missing-dependency/wrong-revision errors. Existing diagnostic/stream and multi-file
checks pass. The .NET primitive Runtime Contract and reference-only input bridge remain
explicit; no default .NET behavior, general binder, metadata library API or runtime
format change was needed. The next replacement remains a native semantic provider and
production target composition, with further supported constructs driven by real cases.

This checkpoint uses Raven `codex/metadata-consumer` source commit `907fb0596`
and the independent metadata API on neoCLR `codex/extended-cli-metadata`
(`cfd7ed50` reader/projection implementation). Matching binary and artifact hashes
are retained in `docs/experiments/extended-cli-metadata/raven-compiler-validation.json`.

### Transitive native runtime acceptance — 2026-09-30

The end-to-end chain now consists entirely of Raven-compiled native assemblies:
application -> MetadataProbeLibrary -> ArithmeticDependency. The outer library imports
the inner library's native declarations through the temporary reference projection.
The application references only the outer projection; Arithmetic is absent from its
symbol lookup and the outer reference PE's AssemblyRef rows, since all public
signatures use primitives. Like .NET reference assemblies, implementation dependencies
remain outside that compile-time signature surface. They are still required at runtime.

`NativeAssemblyDefinition.References` now exposes the exact direct native identities
in manifest order as an owned read-only list. It does not resolve dependencies or build
a transitive closure. The host explicitly supplies both native dependencies to neoCLR.
The existing runtime reference validator (`src/references.rs` in neoCLR), loader,
verifier and VM accept the emitted chain: all three application variants return 42,
and reversed runtime module order also returns 42. Missing direct/transitive modules
and wrong direct/transitive revisions fail verification with the expected diagnostics.
No runtime implementation change was necessary for this supported format-5 graph.

This is actual loading and execution of the emitted native format, not a claim based
on PE readability or reader roundtrips. Reference-only PEs are compiler input only.
The author reiterated runtime loading as a required acceptance gate. Direct PE/#Neo
loading and structural NEOX semantics are still unimplemented, and the .NET primitive
binding bootstrap remains temporary. Neither limitation is hidden by this test.

Validation: 24 C# metadata contract groups pass, including direct identity/list
ownership and rejection checks; Raven adapter/library/multi-file contracts pass; the
runtime checks above pass. Reports include both native dependencies and both reference
projection hashes. Compiler integration remains on codex/metadata-consumer and the
independent metadata/runtime checkout on codex/extended-cli-metadata.

## Direct PE/#Neo runtime bridge — 2026-09-30

The optional Raven adapter now offers `EmitMetadataAssembly`, backed by the separate
.NET metadata project. It requires the existing explicit .NET primitive bootstrap,
output/core identities and registered dependency snapshots; no Runtime Contract
configuration or default target behavior changes. Runtime-native declarations and
bodies occupy required section 256/schema 1. CLI declarations remain a reference-only
compiler projection; neoCLR reads the native section directly from the same PE files.
Owners remain Raven for semantic mapping/diagnostics, the metadata project for encoding
and projections, and neoCLR for admission/linking/verification/execution.

This replaces separate PE/JSON packaging in the end-to-end case, but not JSON encoding
inside the section. Native symbol loading, binary payload encoding and wider semantic
coverage will replace the remaining bridge in stages. No performance gain is claimed.
Both module orders return 42; malformed/unsupported containers and dependency failures
are checked. [Format and validation](design/extended-cli-metadata.md#direct-runtime-container-checkpoint--2026-09-30).
Feature branches and hashes are retained in the compiler validation report; this is
not published/main runtime support or a guest Introspection loader.

The Hello World acceptance slice adds explicit `NeoClrEmitOptions.ConsoleReference`
in Raven. Only the exact registered assembly symbol's System.Console.WriteLine with
a non-null string literal is mapped. The independent API's `WriteConsoleLine` emits
ldstr/call/pop; the pop consumes the existing native System method's Void value.
Direct Main output and Main calling Greet both print Hello World and exit zero through
PE/#Neo. Int32 entry/helper results remain the bounded source contract. General
Console/string signature import and native platform-call contracts remain future
replacement work; the ordinary .NET target is unchanged.

## Binary execution encoding — 2026-09-30

Raven's opt-in EmitMetadataAssembly now chooses execution schema 2, bounded CBOR,
through the independent WriteBinary API. The .NET primitive/Console reference contracts
and reference-only CLI projection are unchanged. This replaces JSON at the native
loading boundary: neoCLR preserves PE bytes and directly decodes the binary payload
to its module model. Compiler-host emission still uses JSON as an intermediate.
Schema 1 remains supported; older schema-1-only runtimes reject required schema 2.
Both versions are feature-branch experiments and are not production target contracts.

Owners remain Raven (semantic mapping), the metadata library (encoding/projection)
and neoCLR (admission/linking/verification/execution). Structural schemas and guest
Introspection APIs remain absent. Indexed native tables may replace this provisional
object encoding. [Profile, compatibility and evidence](design/extended-cli-metadata.md#binary-native-execution-profile--2026-09-30).


## Existing class-library transport baseline — 2026-09-30

NativeModuleContainer now translates the current assembled System JSON into standalone
binary NEOX, preserving native values and bypassing the restricted CLI projection.
Both Hello examples and the generic module chain execute using translated System;
all 641 IL functions verify. The independent metadata project owns transport encoding;
neoCLR owns admission and execution. No Raven Runtime Contract setting, semantic
mapping or production target registration changes in this slice. These envelopes
cannot be loaded as .NET compiler references. Direct Raven source compilation and
broader symbol projection/native import remain the next compiler work. Translated
artifacts provide a regression baseline, not independent correctness proof.


## Existing Raven application translation experiment — 2026-09-30

The legacy bridge's sample outputs are now inputs to the independent binary translation
experiment. Compiler Runtime Contract configuration and the CLI projection remain the
existing collection-profile contracts; the native JSON values are preserved through
NEOX. The matching 417-type/4,090-function System profile exceeds current binary bounds,
so application comparisons explicitly retain JSON System. This is not the same artifact
as the smaller bundled System translated previously. No compiler target capabilities
are broadened by this test. [Evidence and limitations](experiments/extended-cli-metadata/raven-sample-translation.md).


## Larger native library profile — 2026-09-30

Standalone schema 3 now transports the complete matching Raven collection System and
UInt64 instruction operands. FloatingMath, OptionPositional and ValueCopy execute with
binary application and binary System artifacts. Compiler Runtime Contracts, the legacy
CLI bridge's semantic mappings and .NET defaults do not change. The metadata library
owns the versioned encoding; neoCLR owns admission/execution. No CLI projection or native
compiler symbol provider is added. PE/#Neo remains on schema 1/2; the translator's new
schema-3 default requires the matching runtime feature branch.
[Contract and tested consumer](design/extended-cli-metadata.md#library-execution-profile-3--2026-09-30).


## Direct neoil assembly output — 2026-09-30

The runtime assembler now accepts `--format neox`, letting the existing bridge's neoil
output become a native assembly without a JSON translation step. Compiler Runtime
Contracts and semantic/emission mappings are unchanged; the assembler preserves the
root module and verifies the explicit dependency set. The independent metadata API
remains separate and checks the produced values in C# consumer tests. Full matching
System, FloatingMath, OptionPositional and ValueCopy pass. This removes an artifact
conversion stage; it does not add direct Raven native emission or compiler symbol import.
[Validation and commands](experiments/extended-cli-metadata/direct-assembly.md).

### Unit-returning native helpers and library methods — 2026-09-30

The opt-in `NeoClrCompilationEmitter` now accepts Unit/no-result functions and public
static methods with required Int32 parameters. Statement calls, explicit bare returns
and implicit fall-through returns emit through the independent Cecil-style metadata
API. An Int32 entry point can call a Unit helper without a synthetic source return value.
Imported method matching includes the result contract as well as parameter count.

Configuration remains the explicit .NET primitive binding bootstrap (`TargetPlatform.DotNet`),
unsigned output/core identities and registered `NeoClrMetadataDependency` bindings;
Console literal output additionally requires `ConsoleReference`. No Runtime Contract
setting, ordinary .NET emission or production target registration changes. Native intent
is a no-result function; the temporary CLI reference projection represents it as `void`,
which Raven binds as Unit. The native PE/#Neo schema-2 section owns execution, and the
projection contains reference-only bodies. The compiler adapter owns mapping and
validation; the separate metadata library owns encoding. A native symbol provider and
broader native backend will replace the projection/bootstrap.

C# consumer checks compile and execute four Hello variants (Int32 and Unit helpers,
explicit and implicit returns), then emit a Raven library and reload its projection into
Raven. A second compilation calls its Unit overload and returns its Int32 overload;
neoCLR verifies both assemblies and prints exactly one Hello World line with exit zero.
Rejected discarded Int32 calls, named arguments, Unit entry points and unsupported
Console mappings leave the output stream unchanged. This is bounded linear-body support:
entry points still return Int32, and generic/instance methods, general result types,
control flow and complete runtime class-library compilation remain pending.

Validation used Raven `3d6c8dbf1` on `codex/metadata-consumer`, the separate metadata
library from neoCLR `217a59a0` on `codex/extended-cli-metadata`, and the release runtime
built at `116be40e`. Raven's `tools/NeoClrMetadataProbe/validation.json` records the
compiler, adapter, metadata library, runtime and artifact hashes. The complete probe
passed, including the pre-existing multi-file and transitive-dependency checks.

### Namespaced native library types — 2026-09-30

The opt-in adapter now traverses block-scoped, nested and file-scoped namespace
members. Public nongeneric static classes retain their bound namespace and metadata
name in the independent metadata API. As with .NET type identity, two types with the
same short name in different namespaces remain distinct; imports affect Raven lookup,
not native ownership. No format change or new namespace table is needed for this slice.

The C# consumer emits `Example.First.Math` and `Example.Second.Math`, with a call
between them, in both source-file orders. It reads the resulting CLI projection,
checks both namespaces, binds an independent Raven application using imported and
qualified names, and verifies/runs each application against its binary library in
neoCLR to 42. Namespace-scoped free functions and nested types are still rejected with
source diagnostics and unchanged output: the current native function model lacks a
namespace contract, so flattening those functions would lose ownership.

Configuration remains `TargetPlatform.DotNet` for the primitive semantic bootstrap,
with explicit core/output identities and dependency bindings. Runtime Contract options
and default .NET emission are unchanged. The compiler adapter owns symbol-to-metadata
mapping, the independent library owns PE/#Neo encoding, and neoCLR loads the native
schema-2 section. CLI declarations remain a temporary reference-only representation;
a native symbol provider will replace that projection. Generic/instance members,
namespace-owned native functions and complete runtime class-library compilation remain
pending. This extends the earlier global-namespace-only compiler checkpoint.

Validation: Raven `0c13e890b` on `codex/metadata-consumer`, metadata API at neoCLR
`d075cb44` and runtime built at `116be40e`. The full C# probe and native verification/
execution pass; Raven `tools/NeoClrMetadataProbe/validation.json` records artifact hashes.

### Opt-in native compiler command — 2026-09-30

Build `src/Raven.Compiler` with `-p:NeoClrMetadataProject=/absolute/path/to/neoclr/tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj`
to enable `rvnc neoclr`. The metadata library remains an independent project and is
not a dependency of ordinary compiler builds. Without that property the command
reports how to enable it; default .NET emission is unchanged.

```sh
dotnet rvnc.dll neoclr --library -o Library.dll Library.rvn
dotnet rvnc.dll neoclr --reference Library.dll -o App.dll Main.rvn Helper.rvn
neoclr verify App.dll --module Library.dll
neoclr run App.dll --module Library.dll
```

The command compiles supplied source files through the bounded native adapter and
writes PE/#Neo schema-2 assemblies directly. Global source functions become native
assembly-owned functions; classes retain namespaces. Repeated `--reference` options
load native API-produced PE assemblies, validate their native declarations, and expose
their CLI reference-only projections to Raven. No CLI-to-neoil application translation
is involved. Binding/encoding failures create no destination, existing outputs are
refused, and unknown options fail. Successful output creation uses CreateNew; a later
I/O failure may leave a partial file. Output identity is its filename without extension,
version 1.0.0.0, unsigned. Without `-o`, the first source's extension becomes `.dll`.

This is a source-file command, not project/MSBuild/publish integration. It generates no
PDB, runtimeconfig or apphost and does not execute under .NET. Explicit dependency
paths are required; there is no automatic dependency resolution. C# process checks
compile a namespaced library and a two-file application, exercise native assembly-owned
Unit function calls and Console literal output, and verify/run in neoCLR to 42.
Failures cover unsupported source, ordinary CLI references, invalid options and existing
output preservation. The full earlier metadata probe remains green.

**Symbol-loading boundary:** Host .NET primitive/Console/System.Runtime references
remain a temporary semantic bootstrap (`TargetPlatform.DotNet`). The command configures
that core identity and Console contract explicitly, without changing Runtime Contract
options. It does not yet accept the translated standalone System assembly as a core
reference: that file has broad format-5 declarations and no CLI projection, beyond the
writer-shaped declaration reader's subset. The author's required next loader acceptance
is to obtain Raven symbols from that translated System/System.Runtime metadata itself,
then compile against and execute with the matching runtime assembly. Implement a native
symbol provider or faithful broader projection; do not replace those symbols with host
.NET equivalents or silently omit unsupported declarations. Emission and symbol loading
are parallel integration requirements; full format adaptation is not a prerequisite for
making the supported native compiler path usable.

Validation: Raven `22b18a642` on `codex/metadata-consumer`, independent metadata API
at neoCLR `9c95b10c`, release runtime built at `116be40e`. The C# probe with
`--driver /path/to/rvnc.dll` passed all prior checks plus native command process cases.
Raven's checked-in `tools/NeoClrMetadataProbe/validation.json` records matching tool
hashes. A separate net10.0 default compiler build had no native metadata dependency,
reported the disabled command and compiled an ordinary .NET application successfully.

### Translated System callable import — 2026-09-30

Raven can now bind an explicitly selected static Int32 callable read from the translated
standalone System assembly, then emit its original native call identity. The independent
metadata library inventories the binary native module; `CreateStaticInt32ReferenceAssembly`
builds a deliberately partial reference-only view. The existing DotNetSemanticDataLoader
and reflection-backed PE symbols import that view. This reuses semantic import rather
than introducing a second binder or substituting handwritten host declarations.

```sh
dotnet rvnc.dll neoclr --system-symbols System.neox \
  --system-method System.Math.Min/2 -o App.dll App.rvn
neoclr verify App.dll --system System.neox
neoclr run App.dll --system System.neox --show-result
```

The tested source is `func Main() -> int { return System.Math.Min(42, 99) }`.
The symbol test checks that the bound method belongs to the generated native view's
assembly, not the host .NET implementation. Native execution returns 42 using the same
translated collection System assembly (417 types, 4,090 functions). Both compiler API
and actual rvnc process paths are covered. Generic-arity name collisions remain in the
inventory; public visibility comes from native visibility and available origin metadata.
Private methods, internal owners, missing selections and unsupported signatures fail.
Native Result-returning `System.Math.Abs/1` is rejected rather than changed to .NET Abs.

Each `--system-method` explicitly selects qualified native name plus Int32 parameter
count; it can be repeated. No unselected methods are claimed to be imported. The static
view preserves callable ownership/name/signature but projects owners as static classes;
instance shape, fields, properties, generic contracts, parameter names and attributes
remain unprojected. Primitive binding still uses the host core (`TargetPlatform.DotNet`);
this is **not complete System/Core import**. In this mode host Console/System.Runtime
facade references and literal Console mapping are disabled, because host forwarders can
win name lookup. No Runtime Contract option changes. General namespace/type collisions
still require native core composition, not lookup-order assumptions.

`NeoClrEmitOptions.SystemSymbols` accepts an explicit `NeoClrSystemSymbols` binding
(reference, projection assembly name, native inventory, selected functions). The exact
reference must be registered; invalid selections produce NEOMETA002 without output.
Native method selection additionally matches the bound assembly, owner, name, Int32
parameters and result. Emission uses `MethodBuilder.Call(NativeFunctionDefinition)`;
this bootstrap only supports the implicit `System` module. The caller must supply the
matching System artifact to runtime verify/run: no System revision/image digest is yet
encoded. General dependency identity, richer signatures and full core import remain
required follow-up work. Projection files created by the command are temporary and
removed after compilation. The metadata library remains separate from Raven.

Reproduction through the C# consumer:
`NeoClrMetadataProbe --system-symbols <runtime> <rvnc.dll> <System.neox> <fresh-output>`.
The output includes source, reference projection, native assemblies and validation hashes.

**Architecture direction:** ISemanticDataLoader and ICompilationEmitter are existing
composition boundaries, but DotNetSemanticDataLoader and PE symbols still depend on
MetadataLoadContext, Assembly/Type/MethodBase. This bridge adapts native metadata into
that importer. Extract common declaration/signature inputs for shared symbol construction
as coverage grows; the projection is not a permanent native representation. The native
operation emitter already avoids Reflection.Emit. The general .NET code generator still
uses ILGenerator and builders throughout, so removing that dependency requires a common
body/instruction abstraction and backend writers, not just replacing one emitter class.

Validation: Raven `b5d38d985` on `codex/metadata-consumer`, the metadata library in
this neoCLR feature-branch slice, and release runtime built at `116be40e`.
[System symbol evidence](experiments/extended-cli-metadata/system-symbol-validation.json)
records the translated System, runtime, compiler and native output hashes. Thirty C#
metadata contract groups, the complete existing Raven integration probe and the new
System API/driver tests pass. The host API documentation snapshot check passes; no
runtime code or guest API snapshot changed.

### Unit entry points — 2026-09-30

Native Raven output now accepts parameterless Unit entry points as well as Int32
entry points, whether assembly-owned functions or public static class methods.
For example, `func Main() { System.Console.WriteLine("Hello World") }` emits through
the independent metadata API, loads/verifies in neoCLR, prints one line and exits zero.
A Unit entry point may call a Unit helper, return explicitly, or have an empty body.
This supersedes the earlier Int32-only entry restriction; generic/instance/argumented
entry points and broader source constructs remain outside the bounded native adapter.

Native format 5 already represents no-result functions with `returns: Void` and
`no_result: true`. The writer and declaration reader now admit that existing contract
for an entry point; no new transport schema or runtime opcode is needed. Ordinary CLI
output uses void and a managed entry token, matching the familiar CLR no-result entry
contract. PE/#Neo keeps its CLI view reference-only and the native entry authoritative.
No inhabited Void value or artificial integer return is inserted into the source body.

Configuration remains explicit native emission with the current primitive binding
bootstrap and optional Console/System-symbol contracts. Runtime Contract settings and
default .NET compiler emission are unchanged. Tests cover metadata roundtrips, actual
.NET invocation of ordinary CLI output, native API and compiler-command execution,
empty bodies, global/static entry ownership, and rejection of extra return-stack values,
parameterized entries and foreign entry methods. The .NET metadata model owns the
entry contract, Raven maps Unit to no result, and neoCLR's existing loader/VM executes it.
Full System symbol import and richer metadata/backend coverage remain the main follow-up.

Validation: Raven `c4faf8c7e` on `codex/metadata-consumer`, the accompanying independent
metadata-library update, and release neoCLR runtime built at `116be40e`. The complete
compiler integration probe (including rvnc Unit entry execution), 31 C# metadata contract
groups and the API snapshot check pass. [Evidence](experiments/extended-cli-metadata/unit-entry-validation.json)
records the tested runtime/compiler/library and output hashes. No guest API snapshot or
runtime source change was needed.

### Opcode-based metadata emission — 2026-09-30

Raven's bounded native operation emitter now writes its linear bodies through the
independent metadata library's `MethodBuilder.Emit` overloads. Supported logical
opcodes are Ldc_I4, Ldarg, Add, Sub, Mul, Call and Ret. Integer operands and typed
builder/imported/native call operands use separate overloads; unsupported opcode/operand
pairs fail before mutation. Native System calls retain their selected native identity.
Existing LoadConstant/LoadArgument/arithmetic/Call/Return helpers delegate to the same
path, with unchanged stack, ownership, backend and resource validation.

This provides low-level construction for the currently implemented subset, not arbitrary
CLI bytes or all neoIL opcodes. Console literal emission remains its explicit native
convenience operation; there is no general string operand yet. Branches, locals,
exception regions, public instruction objects and ILProcessor-like body insertion remain
future work. The underlying body representation is still internal. Enum numeric values
are not serialized opcode values and do not define an on-disk ABI.

The .NET comparison is typed Emit overload ergonomics without taking a dependency on
Reflection.Emit. The independent metadata writer still chooses native/CLI encoding and
validates the complete body. This creates a compiler-facing emission surface that can
grow toward backend reuse; it does not yet replace Raven's general .NET code generator.
Target/runtime configuration and existing temporary reference projections are unchanged.
C# checks compare helper/Emit artifacts, execute ordinary CLI output to 42, reject bad
operands without body mutation, and cover imported/native calls. Raven's existing native
compiler/runtime cases and selected translated-System calls validate its actual use.

Validation: Raven `1dcd9071c` on `codex/metadata-consumer` with the accompanying
metadata API slice, release neoCLR runtime built at `116be40e`, and the translated
collection System. All 32 C# metadata groups pass. The
[compiler integration](experiments/extended-cli-metadata/opcode-compiler-validation.json)
and [translated-System integration](experiments/extended-cli-metadata/opcode-system-validation.json)
record passing runtime results and artifact/tool hashes. The host API documentation
snapshot check passes; runtime code and guest reference metadata are unchanged.

### Shared emission pipeline — 2026-09-30

Native emission now enters the ordinary `Compilation.Emit` pipeline. Select it with
`new EmitOptions().WithBackend(new NeoClrEmissionBackend(nativeOptions))`; the
`rvnc neoclr` command and existing `NeoClrCompilationEmitter` convenience APIs use
that same path. The compiler owns setup, declarations, semantic diagnostics, macro
preparation and resolved Runtime Contract validation. The backend returns only its
own diagnostics and creates fresh metadata builders on every call. No global backend
registration or dependency from the core compiler to the independent metadata library
is introduced. Clearing the backend restores the selected target's default emitter.

`ICompilationEmissionBackend` is the artifact boundary, not a replacement for the
semantic target. Native configuration still requires the .NET primitive bootstrap,
explicit reference projections and matching native dependency artifacts. Binary PE/#Neo
is the default native artifact; JSON interchange remains available explicitly. Native
PDB output and `EmitOptions.TargetCoreLibraryIdentity` rewriting reject with NEOMETA002
before either stream is changed; configure native core identity in `NeoClrEmitOptions`.
The native source/signature subset and reference-only CLI bodies are unchanged.
Macro preparation belongs to the common pipeline; native macro support is not established.

Compared with the previous .NET-only emission entry point, this lets independently
packaged backends reuse compiler validation without depending on Reflection.Emit.
It does not make the existing .NET `TypeBuilder`, method handles or IL operands portable:
those remain the next extraction boundary. Share declaration/body lowering where
semantics agree, with backend-owned type/method handles and explicit capabilities
where representations differ (notably assembly-owned native functions versus CLI
carrier types). Do not turn unsupported native shapes into silent CLI fallbacks.
The cost is an explicit backend API whose implementations must validate artifact options
and keep per-emission mutable state private.

Focused C# tests cover backend selection, preserved option copies, compiler/contract
failure before backend entry and default .NET PE emission. The native executable probe
covers shared-pipeline/wrapper equivalence, unchanged output on unsupported artifact
options, Hello World, helper calls, dependencies and translated System.Math.Min.
The general backend API is a shared-line integration candidate; it remains on the
consumer branch with the native adapter pending reconciliation with Raven main.

### Shared linear-body lowering and backend method builders — 2026-09-30

The native emitter's existing operation lowering now lives in the core compiler as
`LinearMethodBody`. It produces an immutable plan containing compiler symbols and
logical constant/argument/arithmetic/call/return instructions. It carries source syntax
for diagnostics but no Reflection.Emit or neoCLR metadata handles. `ILinearMethodBuilder`
is an internal compiler implementation contract; the optional native assembly receives
friend access, without a dependency from the core compiler to the metadata library.

Two adapters consume that same plan. The .NET adapter uses the existing method builder,
IL-builder factory and runtime-symbol resolver. The native adapter uses the independent
metadata library's typed Emit overloads and explicit local/dependency/System mappings.
Console literal calls retain distinct backend policy: .NET calls the bound CLI method;
neoCLR requires the explicit registered Console reference and emits its native mapping.
Imported CLI Unit results are discarded only when the actual CLI signature returns a
value; native no-result encoding is unchanged.

Normal .NET emission uses the shared body path for nongeneric static Int32 source
methods in release mode without requested PDB output. Complete lowering happens before
opening a builder, so unsupported bodies use the existing general generator without
partial IL. Debug/PDB, generic, synthesized and other methods retain their prior path.
This is incremental codegen reuse, not a new independent .NET compiler. The current
native Int32/Unit source subset is unchanged and still reports unsupported constructs.

Compared with Reflection.Emit's direct Type/MethodInfo/OpCode use, the common plan lets
both producers reuse language-level body decisions while keeping handle resolution and
encoding in adapters. Its cost is a small per-body plan allocation; no performance gain
is claimed. Type declarations, fields, method signatures, generic constraints, locals,
branches, exception regions and debug positions still need equivalent abstractions.
The established .NET declaration generator remains responsible for type/method creation,
attributes, type completion and PE writing. Broader metadata builder unification remains
open; this slice abstracts executable method bodies, not the complete TypeBuilder API.

Validation includes C# .NET release/debug execution, unchecked Int32 arithmetic, calls,
source-located rejection with general-generator fallback, and release PDB sequence points.
The executable probe emits one compilation through both backends: both print Shared Hello
and return 42 after a helper call. Existing native Hello/Unit/library/reference failures,
namespace cases and compiler-command execution pass. In the additional translated-System
run, the direct API case binds the selected projection and executes to 42, but the driver
binds the colliding host System.Math and rejects emission with NEOMETA001 (unregistered
System.Private.CoreLib dependency). Earlier driver runs passed; selection is not a reliable
contract. The author explicitly defers metadata loading to a future slice. No assembly-name
trick, reordered-reference workaround or silent native call substitution is added here.
The bounded Hello/helper case is the acceptance target for this codegen slice.
The shared implementation is a general shared-line candidate; reconcile it separately
from the native adapter when integrating the consumer branch into main.

### Shared Unit functions and entry points — 2026-09-30

The existing shared linear-body path now also serves eligible release-mode .NET
assembly-level functions and Unit-returning static methods. A Unit method enters this
path only when its already-created CLI method signature returns `System.Void`; a
value-bearing Unit representation stays on the general generator. This reuses existing
.NET declaration/signature construction rather than adding a second builder hierarchy.
The same immutable plan handles helper calls, explicit/implicit return and empty bodies.
Debug/PDB, captures, generic methods and unsupported bodies retain the established path.

The end-to-end probe emits each compilation through both backends. The Unit function
and static-method cases print Shared Hello and exit zero; an empty Unit entry also runs
on both runtimes. The Int32 helper case still returns 42. C# tests additionally verify
void signatures, release/debug behavior, assembly-function arithmetic and preserved PDB
sequence points. Native format/API and Runtime Contract configuration are unchanged.
Compared with CLI void, native no-result remains a backend representation choice; the
common lowering does not manufacture a Unit value or erase a value-bearing CLI result.
Metadata loading and the recorded optional System-driver collision remain deferred.

### Shared callable declarations — 2026-09-30

The supported Int32/Unit signature is now represented once in the compiler by
`Int32CallableSignature`. Declaration and body validation use that same description.
`ICallableDefinitionBuilder<TMethod>` returns a backend-owned method handle without
requiring native builders to inherit from Reflection.Emit's MethodInfo/MethodBuilder.
Both contracts are internal implementation details, not public metadata APIs.

The .NET adapter defines eligible static ordinary methods and functions on the existing
Reflection.Emit TypeBuilder. It retains the existing emitted name and method attributes,
resolves Int32/void through the compiler's target-aware resolver, and leaves parameter
names, custom attributes and other declaration bookkeeping with MethodGenerator.
Generic, captured, extension, extern and richer signatures keep the established path.
The native adapter defines either a method on a metadata TypeBuilder or an assembly-owned
function. Existing visibility/capability checks and metadata-library bounds remain in
force; unsupported metadata is not silently discarded. No native format change occurs.

Compared with the direct .NET builder calls, this extracts the common callable signature
while preserving differing ownership and handle models. CLI functions still use their
existing carrier types; native functions remain assembly-owned. The native subset still
lacks parameter-name/custom-attribute support, while the .NET path preserves it. Full type
construction, generics and richer signatures are subsequent work, not hidden behind this
small contract. No loader redesign or reference-selection workaround is included.

Validation covers public/private method flags, parameter names and types, Int32 and void
results, executable calls, generic fallback, selected System.Runtime core references
without host CoreLib leakage, and PDB preservation. Existing same-compilation .NET/native
Hello/helper cases and rvnc runtime checks validate the two concrete declaration adapters.
This compiler refactor remains a general shared-line candidate pending consumer-branch
reconciliation; native policy stays in the optional adapter.

## Compiler-lowered native bodies — 2026-10-01

The shared linear instruction planner now consumes `BoundTreeView.Lowered`, replacing
its source `IOperation` traversal. Both eligible release .NET methods and the optional
native backend use the existing compiler Lowerer before backend instruction encoding.
Implicit Int32 returns now work without a second return-rewriting implementation;
simple named calls whose lowered arguments fit the subset also work. Static qualified
calls treat a bound type receiver as a qualifier, not a runtime value.

Runtime Contract selection, semantic binding and the temporary CLI reference projection
are unchanged. .NET retains its carrier types and Reflection.Emit adapter; neoCLR emits
assembly-owned functions through the separate metadata API and loads PE/#Neo directly.
The format and runtime need no changes for this slice. Native debug output, general
signatures, locals/control flow and synthesized bodies remain unsupported; .NET debug,
PDB and unsupported bodies keep general codegen. Console permission/identity checks
remain target-owned. Metadata importer work and the known System driver collision stay
deferred. This internal refactor is pending shared-line reconciliation.

Validation: 19 focused C# compiler tests cover execution, implicit returns, named Unit
calls, fallback, PDB preservation and selected core identity. The metadata probe covers
same-compilation .NET/native Hello/helper/Unit execution and implicit Int32 returns,
plus native verification/loading, dependency failures, diagnostics and rvnc behavior.

[Recorded binary/runtime probe evidence](experiments/extended-cli-metadata/lowered-bodies-validation.json).

Tested Raven revision: `9660933cb` on `codex/metadata-consumer`; runtime remains the
previously tested metadata runtime build (its SHA-256 is recorded in the evidence).
The native integration remains on `codex/extended-cli-metadata`, not main.

## Callable identity resolution — 2026-10-01

The bounded .NET and neoCLR emitters now use a shared `CallableReferenceTable<THandle>`.
Compiler method symbols are the target-neutral identities; symbol equality preserves
owners, overloads and assemblies instead of relying on names. Each emission owns its
table and backend handles. Native definitions are registered before bodies; unresolved
references go through the existing explicit dependency/System import policy. Failed
resolution is not cached. Native local, imported CLI-projection and selected native
function handles are encoded inside the native backend, without casting between them.

The .NET adapter resolves through the existing metadata-proxy/runtime resolver and
caches only within its CodeGenerator. This preserves the CLI carrier representation of
free functions; native functions retain assembly ownership. Runtime Contract selection,
binding, metadata format and runtime loading are unchanged. No reference is shared
across emissions or compilations. Broader Reflection-dependent mappings, type/field
references and declaration traversal are still pending; this is the first callable
reference boundary, not completion of the general backend refactor.

Compared with the previous native per-call declaration-list search, the table centralizes
symbol identity and separates call encoding from common resolution bookkeeping. The cost
is a per-emission dictionary and backend handle objects; no execution-speed claim or
benchmark is implied. Focused C# runtime tests and the dual-runtime probe cover repeated
and forward calls, overloads and identical names across assembly functions and types.
Existing checks retain core identity, PDB/fallback, dependency diagnostics and failed-output
contracts. Native symbol loading and its known driver collision remain deferred.

Validation: 20 focused C# tests passed; [runtime and driver evidence](experiments/extended-cli-metadata/callable-identities-validation.json).

Tested Raven revision: `d76a85701` on `codex/metadata-consumer`; native metadata integration remains on `codex/extended-cli-metadata`. The runtime binary hash is recorded in the evidence; no runtime or format change was needed.

## Shared source callable plans — 2026-10-01

`SourceCallablePlan` now carries the supported source method symbol, declaration/body,
logical owner, metadata name and Int32/Unit signature. Both .NET method declaration and
bounded body emission consume that plan; native codegen consumes it for definition and
body emission too. An assembly-level function has no logical type owner even though its
CLI symbol may belong to a compiler-generated carrier. The .NET adapter retains its
chosen emitted name, owner, attributes and target-aware type resolver.

Native emission now collects and validates all supported source declarations first,
then creates type/callable definitions, registers references, and emits bodies. Empty
static types remain declarations even when they have no callable plans. The native
syntax/capability validator remains adapter-owned; it still rejects unsupported attributes,
visibility, namespace functions and richer type contracts rather than dropping metadata.

Compared with the previous inline builder creation, the plan separates source semantics
from backend lifetime/ownership and gives both adapters one callable definition/body
contract. The cost is an additional immutable source-plan object per eligible callable.
This does not unify the general .NET declaration traversal: synthesized methods,
accessors, state machines and unsupported signatures retain their established path.
General type/field handles and traversal remain later work. Runtime Contract and semantic
binding selection, binary format and the native runtime are unchanged. The public
metadata API remains a separate project, with no new public API in this slice.

Focused C# coverage checks source ownership and executable .NET output. The native probe
inspects actual metadata ownership, preserves an empty static type, and verifies/runs
the mixed assembly-function/type-method program. Existing overload, forward-call,
core-reference, diagnostic, PDB and rvnc cases remain part of focused validation.

Validation: 21 focused C# tests passed; [native metadata/runtime and driver evidence](experiments/extended-cli-metadata/source-plans-validation.json).

Tested Raven revision: `0f555a52b` on `codex/metadata-consumer`. Native integration remains on `codex/extended-cli-metadata`; the unchanged runtime binary is identified by SHA-256 in the evidence.

## Shared static type plans — 2026-10-01

Both backends now consume `SourceStaticTypePlan` through a typed type-definition
builder contract for top-level public nongeneric static classes. The plan retains the
compiler symbol for owner lookup and a shared namespace/metadata-name mapping. The
native adapter stores these plans during declaration collection and creates native type
handles afterward. Empty static types still get definitions. Callable owner lookup uses
symbol equality, not a display name.

The .NET adapter creates its TypeBuilder with the existing TypeGenerator flags; base
resolution, custom attributes, members and completion remain in the existing generator.
Generic, nested, nonpublic and instance types keep the general .NET construction path.
Native syntax/capability validation still rejects unsupported contracts; the plan does
not silently approximate bases, interfaces or attributes. Runtime Contract selection,
reference binding, metadata format and runtime loading are unchanged.

Compared with direct Reflection.Emit and native AddType calls, this shares declaration
identity while keeping backend handles and type completion distinct. It costs a small
source plan/adapter allocation and remains a bounded type-definition contract, not a
general type-system or type-reference abstraction. The earlier paused prototype has been
adapted to retain symbol identity and preserve the .NET generator's computed attributes.
General field/type references and declaration traversal remain pending; metadata loading
is still deferred.

Focused C# tests check two same-named classes in distinct namespaces, public/abstract/
sealed flags, base types and executable calls, plus generic/nested/instance fallback.
The native probe retains empty types, namespaced library cases in both file orders,
assembly/type ownership, and actual runtime/driver loading and execution.

Validation: 23 focused C# tests passed; [native runtime and driver evidence](experiments/extended-cli-metadata/static-type-plans-validation.json).

Tested Raven revision: `11fd389eb` on `codex/metadata-consumer`. Native integration remains on `codex/extended-cli-metadata`; the unchanged runtime binary is identified by SHA-256 in the evidence.

## Int32 locals and assignment — 2026-10-01

The shared lowered-body path now admits initialized Int32 local declarations, local
reads and standalone local assignments. Local symbol identity maps to slots before
backend emission; the .NET adapter creates locals with the selected target's Int32 type,
and the native adapter uses method-owned metadata local slots. No host type substitution
or source-operation rewrite is involved. The same Main/helper program with immutable
and mutable locals returns 42 on .NET and from a PE/#Neo assembly loaded by neoCLR.

The independent metadata library adds typed LocalDefinition handles and raw Ldloc/Stloc
operands, CLI local signatures and native format-5 local lists. Writes check ownership,
slot bounds, stack balance and stores before loads for the current linear body subset.
ClearBody retains declarations but resets initialization through revalidation. Older
producer artifacts with no locals remain readable; older experimental host readers may
reject newly written local lists. Runtime format-5 and binary transport schemas do not
change: neoCLR already implements these locals and instructions. Reference-only CLI
projections continue to omit executable body details.

Compared with .NET's general local/IL surface this is deliberately bounded to Int32 and
initialized declarations; arbitrary types, address-taking, control flow and debug local
scopes remain later work. The extra slot/initialization bookkeeping makes compiler and
assembler misuse fail at write time. Runtime Contract selection and the temporary symbol
loader are unchanged; native metadata loading continues directly in the runtime. See
the metadata API manual for the new public contract. General .NET debug/fallback emission
remains intact. No performance claim or runtime optimization is included.

Validation: 25 compiler tests, 33 metadata API groups and the API snapshot check passed; [runtime/driver evidence](experiments/extended-cli-metadata/int32-locals-validation.json).

Tested Raven revision: `5975edf02` on `codex/metadata-consumer`; metadata API changes remain on `codex/extended-cli-metadata`. Runtime binary unchanged; see the evidence hash.

## Shared comparisons and control flow — 2026-10-01

The shared body planner now consumes lowered labels/gotos and emits bound if statements
through the same backend-neutral instruction plan. Signed Int32 equality/less/greater
comparisons, Boolean constants, forward/backward branches and nested blocks support
ordinary if/else and while-loop consumers. Local symbol identities remain distinct
across lexical scopes. Disposal-bearing scopes, arbitrary Boolean/value signatures,
other comparison operators and exception regions remain outside the admitted subset.
The existing .NET general/debug/PDB path still handles unsupported bodies.

The independent writer adds method-owned BranchLabel handles and typed branch/Boolean
Emit overloads. A worklist validates stack types and definitely assigned locals across
joins and cycles; unmarked targets, incompatible stacks, path-dependent uninitialized
loads, reachable fallthrough and unreachable executable instructions reject before output.
CLI byte offsets and native instruction indices are computed separately, after native
Console expansion. Labels remain symbolic in the compiler and public writer API.

This reuses existing runtime format-5 operations; no runtime or binary schema change is
required. Compared with .NET/Reflection.Emit's broad branch surface, the metadata API
checks a bounded typed graph while retaining separate representations (native comparison
results are Boolean values). It costs graph state/initialization analysis during writing,
not a new runtime translation layer. Source syntax is not reparsed to implement loops;
the compiler Lowerer still owns loop rewriting. The historical LinearMethodBody name
now denotes this bounded body planner, including control flow, pending naming cleanup.
Runtime Contract selection, native symbol-loading deferral and explicit dependency
policy remain unchanged.

Validation: 27 compiler tests, 34 metadata API groups and API snapshot validation passed; [native runtime/driver evidence](experiments/extended-cli-metadata/control-flow-validation.json).

Tested Raven revision: `3c31e9c3b` on `codex/metadata-consumer`; metadata branch `codex/extended-cli-metadata`. Runtime remains unchanged, with its hash recorded in the evidence.

## Negated comparisons and loop exits — 2026-10-01

The shared body plan adds `!`, `!=`, `<=` and `>=` for the admitted Boolean/Int32
conditions. Negation emits Boolean false plus equality on native bodies; .NET uses its
CLI Boolean stack representation. The metadata writer now accepts Ceq for matching
Boolean operands as well as Int32, while rejecting mixed operand types. This matches
the existing runtime's typed equality rather than conflating native Boolean with Int32.
The signature/local subset remains Int32 with Unit/no-result methods.

A consumer combines a constant-true loop, comparisons, negation, continue and break;
loop exits reuse the labels/gotos produced by the existing Lowerer. No source-level
loop rewrite or runtime change is added. Both runtimes execute the same source and
return 42. Short-circuit expressions, general Boolean signatures/locals and exception
regions remain separate capability work. Runtime Contract and reference loading remain
unchanged, and unsupported .NET bodies retain general emission.

Validation: 29 compiler tests, 34 metadata API groups and the API snapshot check passed; [native runtime/driver evidence](experiments/extended-cli-metadata/loop-exits-validation.json).

Tested Raven revision: `6573d8998` on `codex/metadata-consumer`; metadata branch `codex/extended-cli-metadata`. Runtime unchanged; evidence includes its hash.

## Primitive callable signatures — 2026-10-01

The shared callable contract now carries ordered Int32/Boolean parameter types and
Int32/Boolean/no-result return types. .NET resolves each through its selected core;
neoCLR maps them to the independent metadata API's immutable primitive signatures.
Overload resolution/import matching uses parameter types, not just parameter count.
Runtime Contract selection and ordinary .NET defaults are unchanged.

Compared with CLR Boolean signatures, native metadata preserves the same source type
identity while validating Boolean evaluation-stack values distinctly from Int32.
No implicit Boolean/integer conversion is introduced. Native entrypoints remain
parameterless Int32/Unit. Locals and selected System inventory imports remain Int32-only.
The CLI declaration projection remains a temporary semantic-loader bridge: it carries
primitive declarations but no executable native body. Native semantic import, broader
types/conversions, fields/instances and complete target composition remain pending.

Validation: 31 focused C# compiler tests, 35 independent C# metadata contract groups,
and the native probe cover same-source execution on both runtimes plus separately
compiled Boolean library imports and same-name/same-arity Boolean/Int32 overloads.
The binary assemblies are verified and run by neoCLR. General changes remain shared-line
candidates on the consumer branch until independently integrated.

Tested Raven consumer revision: `85077a3c8` on `codex/metadata-consumer`.
[Recorded executable evidence](experiments/extended-cli-metadata/primitive-signatures-validation.json).
The metadata API remains on `codex/extended-cli-metadata`; these are not main-line release claims.

## Typed primitive locals — 2026-10-01

The shared lowered-body plan now carries each local's primitive type. .NET resolves
its selected core Int32/Boolean type; neoCLR declares the matching typed metadata
slot. Boolean predicate results can be stored, reassigned, loaded and compared for
equality/inequality. Both backends share source lowering and instruction planning.

The existing Runtime Contract and CLI symbol projection remain unchanged. Compared
with CLI's integer evaluation-stack representation, the native writer enforces a
separate Boolean stack type; stores must match their declared local type. The cost is
explicit type validation. No implicit conversion, uninitialized local, disposal,
nonprimitive local or new System inventory contract is introduced. The .NET general
fallback remains in place. Native metadata/backend replacement of the temporary
symbol projection is still pending.

Validation adds a predicate-local program on both runtimes, C# Release/Debug coverage,
and metadata contracts for reflected CLI local types, native projection and invalid
cross-type stores. All 35 metadata groups and 33 focused compiler tests pass.

This consumer also exposed the assignment parser bypassing logical negation on its
right-hand side. General fix `762bebad0` restores full expression parsing and retains
right-associative chains; 480 parser/assignment tests pass. This fix is independently
validated for the shared line, not a native representation workaround.

Tested Raven consumer revision `5a1da7135` on `codex/metadata-consumer`;
[executable evidence](experiments/extended-cli-metadata/boolean-locals-validation.json).

## Short-circuit Boolean expressions — 2026-10-01

The shared body planner emits built-in Boolean &&/|| with symbolic branches and a
Boolean stack value at the join. Operands are evaluated left to right; the right side
is skipped when the left determines the result. Nested expressions, local assignment
and value returns reuse this plan on both backends. Overloaded operators, nullable
logic and general conversions remain outside this subset.

Runtime Contract configuration and primitive CLI projection are unchanged. This
matches ordinary CLR Boolean short-circuit behavior; neoCLR uses its existing branch
instructions and distinct Boolean stack type. No native schema or runtime change is
required. The temporary projection is still owned by the metadata library and used
by the compiler's .NET semantic provider; native semantic import remains deferred.

Validation: 35 focused compiler tests, including Release/Debug skipped-operand cases,
and the dual-runtime native probe. Console side effects prove that precisely two of
five possible helper calls execute, while the program returns 42. Existing 35 metadata
contract groups cover the unchanged branch/Boolean encoding and validation.

Tested Raven consumer revision `5b42f19bd` on `codex/metadata-consumer`;
[executable evidence](experiments/extended-cli-metadata/short-circuit-validation.json).
No Rust runtime or metadata encoding change was required for this slice.

## Statement-call result handling — 2026-10-01

The shared body planner now permits Int32/Boolean-returning calls in statement
position. It emits the call followed by a stack discard, preserving argument/call
side effects. No-result Unit calls emit no discard. The .NET adapter still handles
an imported inhabited Unit representation according to the actual CLI signature;
the native Console literal mapping retains its existing explicit Void-value discard.

Compared with CLR pop, the native metadata API's Pop has the same stack effect but
participates in native typed-flow validation. Empty-stack discards reject before
writing. The compiler shares result-use planning; each backend owns instruction
encoding. No native schema, runtime implementation, Runtime Contract configuration
or temporary reference-projection change is required. This removes a bounded emitter
restriction, not a language rule. Nonprimitive results remain outside the shared
subset; native semantic import and broader target composition remain pending.

Validation: 37 focused compiler tests, 35 C# metadata contract groups, and the binary
native probe cover local Int32/Boolean statement calls, no-result calls, imported
Int32 calls, preserved side-effect order and rejected pop underflow. Both .NET and
neoCLR execute the same source and return 42.

Tested Raven consumer revision `91d2075c1` on `codex/metadata-consumer`;
[executable evidence](experiments/extended-cli-metadata/discarded-results-validation.json).
The independent metadata API remains on `codex/extended-cli-metadata`.

## Int64 primitives and signed conversions — 2026-10-01

The shared signature/body contract now includes Int64 parameters, results, constants
and locals. Int32→Int64 widening sign-extends; Int64→Int32 narrowing retains the low
32 bits. Existing compiler-bound numeric conversions select these operations; unsigned,
floating-point, checked and user-defined conversion support is not implied. Matching
Int64 arithmetic/comparisons reuse the shared operators. Mixed source arithmetic relies
on the binder's explicit operand conversions, not native stack reinterpretation.

.NET uses its selected core types and ordinary CLI integer opcodes. neoCLR uses the
independent metadata library's typed signatures, locals and existing native integer
operations. The metadata validator now tracks explicit primitive stack types rather
than Boolean tags; this costs a wider internal tag but preserves width at calls, local
stores and control-flow joins. No runtime or native schema change was required.

Runtime Contract configuration and the temporary CLI declaration projection are
unchanged. The metadata API owns the projection and preserves Int64 declarations;
Raven still binds them through the .NET semantic provider. Native semantic import is
pending. Entrypoints remain Int32/Unit and the selected System inventory stays Int32-only.
Older experimental host readers may reject Int64 declarations.

Validation: 41 focused C# compiler tests including integral-cast regressions, 36 C#
metadata contract groups, and the dual-runtime probe. Cases cover signed widening,
low-bit narrowing, long locals/arithmetic, extrema, an imported Int64 helper from a
separately compiled native library, and rejection of Boolean conversions/mixed widths.

Tested Raven consumer revision `ab3c262ce` on `codex/metadata-consumer`;
[executable evidence](experiments/extended-cli-metadata/int64-conversions-validation.json).
Runtime behavior reuses the existing signed conversion implementation in `src/numeric.rs`
and its typed verifier contracts in `src/verifier.rs`; no Rust change was required.

## Signed unary integer operations — 2026-10-01

The shared body planner now handles built-in unary +, - and ~ for Int32/Int64.
Unary + evaluates its operand unchanged; negation and bitwise complement preserve
width. Logical Boolean ! remains separate. .NET uses its existing neg/not semantics;
neoCLR emits the corresponding native operations through the independent metadata API.
Negating the minimum signed value wraps to itself on both targets, matching the
existing general .NET emitter and native numeric implementation.

This slice changes no Runtime Contract configuration, symbol projection or metadata
schema. The writer checks integer operand type/stack presence before encoding. Checked,
unsigned and floating-point unary support is not implied. Native semantic metadata
import, broader types and full target composition remain pending. The CLI reference
projection stays temporary and metadata-library-owned.

Validation: 43 focused C# compiler tests and 37 independent metadata contract groups,
plus the native binary-assembly probe. Release/Debug cases cover both widths, extrema,
identity and complement. The same source executes on .NET and neoCLR, checks wrapping
at both signed minima and returns 42. Writer tests reject Boolean operands and underflow.

Tested Raven consumer revision `0f2743fb2` on `codex/metadata-consumer`;
[executable evidence](experiments/extended-cli-metadata/unary-integers-validation.json).
This reuses `src/numeric.rs` unary behavior and `src/verifier.rs` operand checks.
The independent metadata API remains on `codex/extended-cli-metadata`.

## Shared primitive type contract — 2026-10-01

Callable signatures and local declarations now carry EmissionPrimitiveType rather
than passing semantic SpecialType values to backend builders. Shared classification
admits Int32, Int64 and Boolean values and explicitly distinguishes NoResult. Unit/void
are normalized only in return position; Unit parameters/locals, nullable and other
unsupported types are not silently converted into no-result or primitive values.

Both declaration and body builders use IEmissionTypeMapper<TType>. The .NET mapper
resolves every type through the caller's selected-core resolver; it never substitutes
host typeof handles. The native mapper lives independently of the callable builder
and maps to the separate metadata library's PrimitiveType contract. Native local
emission no longer depends on the callable declaration builder for type mapping.

This is a bounded type boundary, not general nominal/array/generic type support.
Compared with passing SpecialType through each builder, the benefit is one shared
value/no-result admission rule and explicit target-owned representation mapping. The
cost is a small internal type vocabulary and mapper implementation per backend. CLR
Reflection.Emit handles remain in its adapters; native metadata handles remain in
its adapters. Ordinary .NET behavior and Runtime Contract configuration are unchanged.
The temporary CLI declaration projection and deferred semantic import are unchanged.

Validation: 44 focused C# compiler tests include rejection of unsupported signature
shapes and selected-core inspection of Int32/Int64/Boolean locals and callable
signatures. The existing native probe exercises both mappers by executing all supported
primitive cases on .NET and binary assemblies loaded by neoCLR. The metadata format
and API did not change; the prior 37 metadata contract groups remain applicable.

Tested Raven consumer revision `649eeb851` on `codex/metadata-consumer`;
[executable evidence](experiments/extended-cli-metadata/primitive-type-boundary-validation.json).
The native metadata API remains a separate project on `codex/extended-cli-metadata`;
no API or runtime change was needed for this compiler boundary.

## Partial static declarations — 2026-10-01

Raven `codex/metadata-consumer` at `c34f65e6f` coalesces public nongeneric partial
static classes by the existing binder's symbol identity. All declaration parts are
validated and all members collected before metadata builders/bodies are produced.
Empty parts do not add types; unsupported members in any part reject output with
NEOMETA001 at their source location and leave the output stream unchanged.

Like .NET, native metadata erases the source-only partial boundary into one type.
This reuses existing native format-5 ownership and the temporary CLI reference
projection; the benefit is multi-file source organization without a new encoding,
with the cost of explicit identity coalescing and per-part validation. The independent
metadata API and runtime loader are unchanged. Runtime Contract configuration and
binding semantics are unchanged; native emission remains opt-in, using hosted
primitive binding. General instance/generic types, partial methods and the future
native symbol provider remain outside this slice.

Validation: four focused .NET static/partial declaration tests passed. The C# probe
executes cross-part overloads and an empty part in both file orders on .NET and binary
neoCLR, returning 42; it checks one projected type with three methods and rejects an
unsupported property in either order. The existing integration probe also passes.
This run did not repeat the optional compiler-driver checks.
[Executable evidence](experiments/extended-cli-metadata/partial-static-types-validation.json).

## String signatures and computed console output — 2026-10-01

Tested Raven `codex/metadata-consumer` revision: `86d4fed0f`.

The development metadata API and Raven shared body path now carry String parameters,
results, initialized locals, assignments, literals, call results and joins. A helper
from a separately compiled native library can return text to Console.WriteLine.
Source binding/lowering still belongs to Raven; backend mappers retain selected-core
.NET types and native built-in String identities. No Runtime Contract option changes.
Ordinary .NET remains default; native emission is still explicit and primitive
binding still uses the .NET bootstrap, with native declarations imported through
temporary CLI reference projections. General symbol-loader work remains deferred.

The independent metadata library encodes CLI String signatures and ldstr user-string
tokens, or native String signatures and UTF-8 ldstr. Existing runtime String verification
and execution load the binary payload directly; format/schema versions are unchanged.
String calls/returns/local stores and branch joins validate exact stack types.
Entrypoints remain Int32/no-result. Null literals/nullable signatures, equality,
concatenation and instance members remain outside the supported producer subset.

The Console policy checks the registered reference and one-string WriteLine overload.
The shared plan emits its argument normally; the native adapter calls the new
WriteConsoleLine() operation, which consumes String and discards bundled System's
inhabited Void. This remains a bootstrap library mapping owned by the compiler adapter
and metadata writer; eventual native core symbol import/call resolution replaces it.
Reference projection bodies still throw and do not translate native execution.

Compared with .NET's [ldstr instruction](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.ldstr?view=net-10.0)
and [UTF-16 text model](https://learn.microsoft.com/en-us/dotnet/standard/base-types/character-encoding-introduction)
(primary documentation reviewed 2026-10-01), this producer contract accepts Unicode
scalar text encodable in UTF-8 and rejects unpaired surrogate code units. It retains
the existing 64 KiB UTF-8 literal limit and aggregate image limits. The alternative
of replacement encoding would silently change text; preserving arbitrary UTF-16
would require a new native text representation. Reusing the current runtime avoids
that format change at the cost of a narrower literal domain than .NET. No interning,
allocation or performance equivalence is claimed; equality remains separately scoped.

Validation: 38 C# metadata contract groups and 41 focused Raven tests cover Unicode,
empty/NUL literals, UTF-8 size boundaries, malformed operands, projection/import,
selected-core .NET signatures/locals and Debug/Release execution. The integration
probe exercises computed text and cross-assembly String overloads in binary native
assemblies; native runtime verification and execution must pass.

The updated rvnc driver case also passes: it compiles a String helper library and
prints its returned Unicode text from the binary native application.
[Executable evidence](experiments/extended-cli-metadata/string-values-validation.json).
Older bounded metadata readers reject the newly admitted String declarations; use
the matching metadata library and compiler adapter. Existing artifacts remain readable.

## Metadata argument stores and compiler boundary — 2026-10-01

Raven `codex/metadata-consumer` records the matching architecture/spec clarification
at `a9ed7b24b`; its String integration remains at `86d4fed0f`.

The independent metadata library exposes OpCode.Starg through Emit(OpCode, int)
and StoreArgument(int), validating slot bounds and exact stack type before output.
Stores replace only the callee's by-value slot, leaving the caller's variable intact.
Parameters start initialized. Ref/out/in and receiver stores remain outside this
bounded writer API.

This is not a new Raven source feature. Investigation found a stale specification
paragraph permitting var on ordinary parameters, but the binder and diagnostic
regressions reject it. The paragraph was corrected; ordinary Raven source parameters
remain immutable. The proposed shared source-level store path was withdrawn before
commit rather than introducing an unrelated language change. Current compiler
adapters do not expose this operation; it is available to metadata producers and
future compiler-generated bodies through a deliberate capability boundary.

This matches the ordinary [CLI starg](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.starg?view=net-10.0)
slot operation (Microsoft .NET 10 documentation reviewed 2026-10-01), using its
UInt16 operand for CLI output and the existing native starg for runtime execution.
Reusing those instructions avoids introducing a new format concept. Native stack
checking keeps Boolean distinct from Int32; this bounded API admits no small integer
or floating-point storage conversions. No Runtime Contract setting, native schema
or semantic importer change is involved. General object/field support remains open.

The author reaffirmed extended CLI compatibility while this slice was underway.
Standard metadata/IL remains the baseline; today's throwing CLI reference projection
plus separate native execution payload is still a bridge, not completed executable
.NET/neoCLR interchange. See the [compatibility boundary](design/extended-cli-metadata.md#compatibility-baseline-reaffirmed--2026-10-01).

The author also asked to keep codegen performance in view for a later revisit.
The shared plan's allocations and fallback work need measurement; no codegen speedup
is claimed. A later phase/allocation benchmark should separate collection, planning,
reference resolution and serialization from end-to-end emission, comparing supported
.NET/native cases without weakening diagnostics or output contracts.

Validation: 39 metadata C# contract groups pass, covering all four types, the last
slot, invalid indices, underflow and type mismatch. A direct API producer's binary
assembly verifies and executes in neoCLR with result 42 and reassigned Unicode text.
Three existing Raven diagnostic tests confirm ordinary var/val parameter rejection.
[Native evidence](experiments/extended-cli-metadata/argument-stores-validation.json).

The author further directs a target-neutral shared codegen abstraction with selectively
exposed instruction/metadata categories. Keep compiler-owned typed references and
logical operations separate from Reflection.Emit/native metadata handles; target
capabilities govern extra categories. The current bounded plan implements only part
of that architecture. General types/fields and explicit capability composition remain
open, rather than silently assuming the intersection or union of both target formats.

## Selective body capability admission — 2026-10-01

Tested Raven `codex/metadata-consumer` revision: `3fb50ed79`.

Raven now owns an immutable capability contract for logical instructions and built-in
value/no-result types. Each adapter lists admitted operations/types explicitly; new
planner operations are not implicitly enabled on every backend. Shared signature,
call, local, expression and instruction checks use the selected profile, while target
handles and encodings remain inside adapters. All native bodies now pass these
checks before assembly/type/method builder allocation. Later dependency and writer
validation still runs before output is committed.

Signed Int32/Int64 division demonstrates selective admission. The shared planner
understands it and .NET emits standard [CLI div](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.div?view=net-10.0)
(Microsoft .NET 10 documentation reviewed 2026-10-01). The native profile rejects the
instruction at its source expression with NEOMETA001 because the independent metadata
writer has not exposed it. This is a producer capability limit, not a restriction on
neoCLR's existing runtime or a new metadata extension. .NET runtime tests preserve
signed truncation and divide-by-zero/overflow faults in Debug and Release.

Compared with a shared planner limited to the intersection of both backends, explicit
profiles allow one adapter to support more while preserving diagnostics in the other.
The cost is maintaining admitted sets and an instruction admission pass. Profiles
are copied once and reused; native preflight retains body plans for the assembly.
Allocation/time tradeoffs are not measured yet; no speedup is claimed.

No Runtime Contract option, semantic-loader, metadata-library API, native schema or
runtime change is introduced. Ordinary .NET and its unsupported-body/Debug fallback
remain default. The native backend override and hosted primitive/projection binding
remain temporary. General nominal types, fields, metadata category capabilities and
full target composition remain open. The current reference projection/native payload
bridge still does not establish full extended-CLI executable compatibility.

Validation: 48 focused Raven tests pass, including restricted capability profiles,
immutable configuration, selected-core types and both .NET generator paths. The native
probe checks the specific division-capability diagnostic, source location and unchanged
output, alongside existing binary load/execute and compiler-driver acceptance cases.

[Binary runtime and driver evidence](experiments/extended-cli-metadata/emission-capabilities-validation.json).
The metadata API and runtime implementation are unchanged in this slice; their prior
39 C# contract groups and native argument-store evidence remain applicable.

## Declaration-category profiles — 2026-10-01

Raven's backend-owned capability profiles now explicitly admit logical assembly
functions, static methods and static types, independently of instruction/type admission.
Omitted categories admit nothing. Production source-plan collection checks the profile
before declaration builders; callable body admission also checks the category. The
profiles are immutable snapshots shared by declaration and body adapters.

These logical categories preserve the existing difference between native assembly
functions and .NET carrier methods. Compared with placing physical CLI method ownership
in the common contract, they keep representation in the backend at the cost of explicit
category admission. Existing API/type/method encodings and runtime behavior are reused;
no new format extension, Runtime Contract option or loader change is introduced. The
independent metadata project is unchanged. Visibility and general nominal/field/member
categories remain open, as does full extended-CLI executable compatibility.

Validation: 53 focused Raven tests pass, including category isolation and copied
configuration, .NET declaration metadata/runtime checks and existing instruction/type
capability checks. Native binary/driver acceptance is recorded with the matching
compiler revision below. This is feature-branch development, not published support.

Raven revision `e5b462e74`: [binary runtime and compiler-driver evidence](experiments/extended-cli-metadata/declaration-capabilities-validation.json).

## Internal static helpers on both targets — 2026-10-01

The shared Raven static type plan now carries Public/Internal accessibility, admitted
explicitly by each backend profile. Both .NET emission and neoCLR emission accept
internal static helpers. The independent metadata library adds TypeVisibility,
AddType(namespace, name, visibility) and TypeBuilder.Visibility. The two-argument
API retains public behavior. No Runtime Contract configuration change is needed.

This adopts the conventional top-level CLI Public/NotPublic distinction rather than
inventing a new extension ([.NET TypeAttributes reference](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.typeattributes?view=net-10.0),
consulted 2026-10-01). Native output reuses the runtime's internal type access checks
and origin.publicly_visible flag; the reference projection preserves NotPublic.
Compared with keeping all producer types public, this supports implementation helpers
without exposing them to consumers. It costs an explicit visibility mapping in each
adapter and consistency validation in the bounded reader. No execution speedup is
claimed; the shared contract contains no Reflection.Emit or metadata builder handles.

Public artifacts keep their existing omitted visibility default. Old bounded readers
reject new internal declarations; the updated reader accepts both and rejects
inconsistent visibility/origin flags. Methods remain public static and primitive-only;
private/nested types, nonpublic methods and friend assemblies are not added. The
compiler backend owns source admission, the independent metadata project owns encoding,
and the runtime owns access enforcement. CLI reference bodies still throw; #Neo remains
the executable payload. Native metadata/semantic loading and broader backend coverage
remain subsequent work, not completed by this bridge.

External-access validation also exposed a shared Raven binder gap: qualified type
expressions skipped accessibility checks even for ordinary .NET metadata. The fix
uses existing accessibility policy at both qualified type-expression and namespace
receiver paths. Its regression fails against the previous compiler and is independent
of neoCLR. Treat this as a general candidate for the shared compiler line, not a
permanent experiment; the current integration/evidence remain on feature branches.

Validation: 40 C# metadata contract groups pass (including CLI execution, visibility
projection and malformed declarations); the guest API snapshot check passes. Raven
`ee07a991e` passes 90 focused codegen/accessibility tests and the full binary runtime
and rvnc probe. The paired executable runs an internal helper on both targets and a
separate native consumer through a public facade. A raw API-produced external call
bypassing source checks fails native verification with `type access denied` in both
library file orders. [Recorded binary/driver evidence](experiments/extended-cli-metadata/internal-types-validation.json).

## Signed division on both emission targets — 2026-10-01

Raven's native capability profile now admits the existing shared Divide operation;
the independent metadata API adds OpCode.Div and MethodBuilder.Divide. Validation
requires matching Int32/Int64 stack operands before output. Standard CLI div and the
runtime's existing native div truncate toward zero and fault for zero or minimum/-1.
This follows the [.NET div contract](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.div?view=net-10.0)
(consulted 2026-10-01), rather than introducing a runtime helper or new instruction.
It expands ordinary expression coverage at the cost of one explicit adapter mapping;
no performance change is claimed. Unsigned/floating coverage remains pending.

No Runtime Contract setting or runtime implementation changes. The old native
rejection is superseded; custom restricted-profile tests continue to verify selective
admission. Unsupported shift expressions retain source diagnostics and unchanged
output. The compiler owns operation planning, the separate metadata project owns
encoding, and native runtime verification/execution owns arithmetic faults. The
CLI reference projection/#Neo bridge and deferred semantic importer remain unchanged.

Validation: 41 C# metadata contract groups and API snapshot check pass. Raven
`71a4e6124` passes the complete binary runtime/driver probe, including signed quotient
results and four Int32/Int64 zero/overflow execution cases on both .NET and neoCLR.
The unchanged shared planner's 36 focused tests pass as the targeted baseline.
[Binary evidence](experiments/extended-cli-metadata/division-validation.json).

## Shared signed remainder — 2026-10-01

The shared lowered-body planner now models signed remainder for Int32/Int64. Both
backend profiles admit it and map to CLI/native rem; the separate metadata library
adds OpCode.Rem and MethodBuilder.Remainder with matching-width stack validation.
No binder/language semantics or Runtime Contract options change. Source '%' already
worked through the general .NET generator; it now shares the bounded path and native
emission. Debug/general fallback remains available on .NET.

This follows the [.NET rem instruction contract](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.rem?view=net-10.0)
(consulted 2026-10-01): ordinary remainder keeps the dividend's sign. Zero divisors
fault. The existing native runtime also faults on minimum/-1, as does the tested CLR;
.NET documents that overflow edge as platform-sensitive, so universal host equivalence
is not claimed. Reusing rem avoids a division/multiplication expansion and extra
intermediates, but performance is not benchmarked. Unsigned/floating operands and
exception-region emission remain outside this bounded producer. Metadata signatures,
the temporary reference projection and #Neo execution transport are unchanged.

Validation: 42 C# metadata groups, API snapshot check, 38 focused Raven shared-body/
capability tests and the complete binary runtime/rvnc probe pass. Raven `2e03b8dd0`
runs both quotient and remainder result programs, plus eight Int32/Int64 zero/overflow
fault programs against .NET and neoCLR. Native fault cases pass binary verification
before failing execution. [Recorded evidence](experiments/extended-cli-metadata/remainder-validation.json).

## Shared integer bitwise operations — 2026-10-01

The shared Raven planner now admits Int32/Int64 AND, OR and XOR in both backend
profiles. The independent metadata writer adds And/Or/Xor and the corresponding
BitwiseAnd/BitwiseOr/BitwiseXor helpers. Both consume matching integer widths; the
writer rejects invalid stack shapes before producing output. No Runtime Contract
setting or runtime implementation change is needed. Boolean/enum bitwise support
remains outside the bounded producer, without changing ordinary .NET behavior.

The baseline is conventional CLI and/or/xor ([.NET opcode definitions](https://github.com/dotnet/runtime/blob/main/src/coreclr/inc/opcode.def),
consulted 2026-10-01). Reusing those instructions preserves fixed-width sign bits
and avoids helper calls or new metadata categories. The cost is explicit capability
and adapter mappings; no performance improvement is asserted without measurement.
The native payload/reference projection bridge and deferred importer remain unchanged.

Validation: 43 C# metadata groups and the API snapshot check pass. Raven `9885caf6e`
passes 41 focused shared-body/capability tests and the complete binary native/rvnc
probe, including all three operators at both widths with negative and wide values.
[Binary evidence](experiments/extended-cli-metadata/bitwise-validation.json).

## Shared integer shifts — 2026-10-01

Raven's shared planner and both capability profiles now admit left and signed-right
shifts of Int32/Int64 by an Int32 count. The metadata API adds Shl/Shr and
ShiftLeft/ShiftRight; its stack validator preserves the value width while consuming
the narrower count. Existing CLI/native encodings are reused, without a Runtime
Contract option or runtime change. The semantic intent is existing Raven shift
behavior; this slice changes emission coverage, not binding or language policy.

[CLI shift rules](https://download.microsoft.com/download/7/3/3/733ad403-90b2-4064-a81e-01035a7fe13c/ms%20partition%20iii.pdf)
(Partition III, shl/shr; consulted 2026-10-01) leave out-of-range counts unspecified.
neoCLR's existing runtime masks counts to the width. Preserve this difference rather
than silently adding masks to ordinary .NET output. A caller can request portable
masking explicitly with AND. In-range counts share discarded-bit/sign-extension
behavior. Compared with synthesizing shifts using arithmetic or helper calls, direct
instructions preserve the runtime contract and keep mapping small; no performance
claim is made. Unsigned/native-sized shifts and portable count policy remain open.
The native payload/reference bridge and deferred metadata importer remain unchanged.

Validation: 44 C# metadata groups and API snapshot check pass. Raven `7f8d91b35`
passes 45 focused shared-body/capability tests and the full binary native/rvnc probe.
The paired program covers sign extension, lost high bits, zero and 31/63-bit counts,
and Int64 values with Int32 counts. Unsupported floating conversion validates precise
source diagnostics and preserved output after shifts become supported.
[Binary evidence](experiments/extended-cli-metadata/shifts-validation.json).

## Metadata method visibility foundation — 2026-10-01

The separate host metadata project now preserves public/internal/private static method
access in CLI output, native definitions and reference projections. Existing overloads
remain public. This adopts standard [CLI MethodAttributes access](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.methodattributes?view=net-10.0)
(consulted 2026-10-01); native output uses its existing visibility and matching origin
member_access fields. It enables implementation helpers without making them public,
at the cost of explicit access mapping and reader consistency checks. No new runtime
instruction or schema is introduced; no performance change is claimed.

The raw writer deliberately does not enforce call accessibility. Native verification
checks resolved declaring type/assembly identity, including references created directly
by the API. Assembly-level functions remain public in this bounded producer; protected,
friend and inheritance-dependent access remain outside scope. Public encoding stays
unchanged; older bounded readers reject new nonpublic rows. The native reference
projection still contains throwing bodies; #Neo remains executable payload.

Validation: 45 C# contract groups and API snapshot check pass. A directly produced
binary runs legal same-type private and same-assembly internal calls to 42. Native
verification rejects external private/internal calls and same-assembly private calls
from another owner. [Independent API/runtime evidence](experiments/extended-cli-metadata/method-visibility-api-validation.json).
The foundation commit was followed by the Raven integration below; no Runtime Contract
configuration changes are required.


## Raven static method access integration — 2026-10-01

Raven `7fa72b67b` on `codex/metadata-consumer` consumes the separate metadata API at
`5ccc41e8` on `codex/extended-cli-metadata`. Shared callable plans carry source access
and backend capabilities explicitly admit public/internal/private static methods.
The native declaration adapter maps access; .NET retains its established MethodAttributes
and carrier policy. This enables hidden implementation helpers on both targets using
CLR-style visibility, with explicit mapping as the maintenance cost. No new runtime
instruction, format version or Runtime Contract configuration is introduced.

The compiler owns source admission and diagnostics; the metadata library owns CLI/native
serialization; the existing runtime verifier owns resolved access enforcement. PE/#Neo
still carries a throwing CLI reference projection plus native execution payload. The
existing Raven semantic importer consumes that projection; replacing it with native
metadata loading is deferred. Assembly-function access remains the bounded public
encoding, with nonpublic library functions rejected. Protected and instance native
methods remain unsupported. No performance claim or per-call cache redesign is made.

Validation: 83 focused Raven C# tests and the complete binary runtime/rvnc probe pass.
Private same-owner and internal cross-owner helper calls execute to 42 on both targets.
Separate libraries in both source orders retain access; Raven rejects external private/
internal calls and native verification rejects raw API callers independently.
[Recorded Raven/runtime evidence](experiments/extended-cli-metadata/method-visibility-raven-validation.json)
includes artifact hashes and runtime identity. The independent metadata foundation's
45 C# contract groups and direct binary access tests remain applicable. These are
feature-branch revisions, not main or published support.


## Expression-bodied Raven callables — 2026-10-01

Raven `912cdff51` and `c70956b91` on `codex/metadata-consumer` extend shared callable
plans and native admission to arrow bodies. The original bound arrow block passes
through the existing compiler Lowerer, as in ordinary .NET emission. This preserves
return conversions and Unit statement semantics without backend-specific rewriting.
The tradeoff is the existing bounded primitive/body capability subset; general .NET
and Debug fallback remain. No new Runtime Contract configuration, metadata API, native
instruction or schema is introduced. Compiler lowering owns source semantics; the
independent writer and runtime consume existing encodings. PE/#Neo reference projection
and eventual native symbol-loading replacement remain unchanged.

Validation: 48 focused Raven C# tests and the complete binary runtime/rvnc probe pass
against metadata `5ccc41e8`. Paired .NET/native cases cover Int32/Int64/Boolean/String,
implicit widening, Unit entry/helper calls and Unicode console output. Separate-library
methods execute in both source orders; unsupported arrow conversions retain exact
source spans and unchanged output. [Recorded evidence](experiments/extended-cli-metadata/expression-body-raven-validation.json).
This is feature-branch development; async/generic/instance native emission remains deferred.


## Eager Boolean operation foundation — 2026-10-01

Native and/or/xor now accept two exact Boolean operands and produce Boolean, while
retaining integer behavior. Mixed Boolean/integer operands and Boolean arithmetic
remain invalid. This follows the truth tables and eager evaluation expected by
[Raven's existing Boolean operator binding and .NET Boolean operators](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/operators/boolean-logical-operators)
(consulted 2026-10-01). CLI uses integer stack operations for Boolean bitwise logic;
neoCLR keeps its distinct Boolean stack value. Reusing opcode names avoids synthetic
branches or Boolean/integer coercion, at the cost of typed verifier/runtime dispatch.
No new schema, opcode number or Runtime Contract option is introduced. Older runtimes
reject this operand category; producer integration requires this runtime revision.
Nullable Boolean, enum operators and short-circuit changes are outside this slice.
Metadata and compiler admission follow separately. Rust truth-table tests exercise
all twelve combinations and verify/run rejection of mixed types and Boolean addition.


The independent metadata API now admits these exact Boolean operands through
`Emit(OpCode.And/Or/Xor)` and `BitwiseAnd/Or/Xor`. Typed validation preserves a Boolean
result and rejects mixed operands and mismatched returns. Native output requires
runtime `fa25609d` or later on this feature branch. CLI output retains standard
encodings and executes on .NET. All 45 C# metadata contract groups and the guest API
snapshot check pass, including raw/helper truth tables and native reference projection.
Raven source admission was completed in the integration below.


## Raven eager Boolean integration — 2026-10-01

Raven `88495dbd7` on `codex/metadata-consumer` consumes metadata `83200ad6` and native
runtime `fa25609d` on `codex/extended-cli-metadata`. Built-in Boolean &, | and ^ now
pass through the shared body planner and existing instruction adapters. Ordinary
.NET remains the default; no Runtime Contract setting or binder semantics change.
The planner evaluates left then right; && and || retain separate short-circuit
lowering. Enum/nullable Boolean and user-defined operator contracts remain deferred.
PE/#Neo reference projection and the future native metadata importer boundary remain.

Validation: all 45 C# metadata contract groups, 52 focused Raven C# tests, the 8-test
Rust integer suite (with the two new verifier cases rerun after explicit verification
assertions), and the API snapshot check pass. The complete binary runtime/rvnc probe
executes all twelve Boolean truth-table cases and printed left/right markers for
all three eager operators on both targets. Existing short-circuit and integer bitwise
cases pass against the rebuilt runtime. [Artifact/runtime evidence](experiments/extended-cli-metadata/boolean-bitwise-raven-validation.json).
Older native runtimes reject Boolean operands; these feature-branch revisions must
be deployed together. No throughput or allocation improvement is claimed.


## Primitive conditional values — 2026-10-01

Raven `a9e6d2d7b` on `codex/metadata-consumer` now emits primitive value-producing
if/else through shared branch instructions. Matching Int32/Int64/Boolean/String
branches leave one value at the join; only the selected branch executes. This uses
ordinary .NET conditional control flow and existing neoCLR branch/stack validation,
with no new Runtime Contract option, metadata format, writer API or runtime code.
Compiler binding still owns source expression context and conversions. The initial
slice permits single-expression branch blocks; Unit/missing-else values, nonprimitive
joins and multi-statement value blocks remain outside this bounded plan. .NET keeps
its general fallback. CLI reference projection remains temporary pending native
symbol loading. No performance improvement is claimed.

Validation: 32 existing shared-body/block-expression C# tests plus both new Release/
Debug conditional tests pass. The full binary runtime/rvnc probe passes against
metadata `83200ad6` and runtime `fa25609d`, covering both alternatives, nested values,
all four primitive types and skipped faulting/side-effecting branches.
[Recorded evidence](experiments/extended-cli-metadata/conditional-values-raven-validation.json).


## Local computation in value blocks — 2026-10-01

Raven `bccfd3507` extends the shared plan with initialized locals, local assignments
and calls before a value block's trailing primitive expression. Existing statement
emission handles these operations and discards unused call results. Symbol-based
local identities keep same-named locals in different branches distinct. .NET and
neoCLR execute only the selected branch, including updates to outer local storage.
This reuses existing CLI/native local and branch operations; no metadata API/schema,
runtime code or Runtime Contract configuration changes. Disposal and prefix control
flow remain bounded-plan limitations; the ordinary .NET fallback remains available.
Native symbol loading remains deferred and the CLI reference projection is unchanged.

Validation: 52 focused Raven C# tests and the full binary runtime/rvnc probe pass.
Both branches execute with distinct locals and outer assignments; prefix loops reject
before output writes. [Recorded evidence](experiments/extended-cli-metadata/value-block-raven-validation.json).
No performance improvement is claimed; metadata/runtime evidence from unchanged
components remains applicable.


## Internal value-block control flow — 2026-10-01

Raven `53e4519d0` extends value-block prefixes with existing statement if/loop emission,
including internal break/continue. Preflight traverses lowered statement blocks and
discarded block expressions, rejecting returns and jumps whose target is outside the
value block. This keeps partially evaluated enclosing-expression operands intact.
Pure Unit statements have no emitted effect. Disposal remains unsupported. Existing
.NET fallback, source semantics, Runtime Contract options and CLI/native encodings
remain unchanged. This adds compiler planning work without a performance claim;
nonlocal exits require a future enclosing-expression stack contract. The metadata
project remains separate, with no API/schema/runtime change in this slice. Native
symbol loading remains deferred; the temporary CLI reference projection is unchanged.

Validation: 55 focused Raven C# tests and the full binary runtime/rvnc probe pass.
Both .NET paths and neoCLR preserve an earlier arithmetic operand across the value
block's loop, internal break/continue and assignments. A conditional return is
rejected before returning a shared plan or modifying native output.
[Recorded evidence](experiments/extended-cli-metadata/value-block-control-flow-raven-validation.json).


## Assembly-function visibility foundation — 2026-10-01

The separate metadata API adds a Public/Internal visibility overload for AddFunction.
Existing overloads remain Public. Functions remain ownerless in native metadata and
CLI globals in the reference projection. CLI uses ordinary Public/Assembly access
flags (the same access mapping researched for method visibility above); native
verification uses existing resolved module identity. This provides hidden assembly
implementation functions without introducing synthetic native owner types, at the
cost of explicit producer admission and access mapping. Private ownerless functions
remain unsupported because native private access requires declaring-type identity.
No runtime instruction, schema or Runtime Contract configuration changes. Older
bounded readers reject internal ownerless rows; reader/writer updates must be paired.
The native importer replacement and temporary CLI reference projection remain.

Validation: 46 C# metadata groups and the API snapshot check pass. API-produced binary
assemblies execute same-module internal calls, an internal entry and a public facade
to 42. Native verification rejects an external internal-function call.
[API/runtime evidence](experiments/extended-cli-metadata/function-visibility-api-validation.json).
Raven `11b922327` now uses shared function-access capabilities and preserves explicit
public/internal and default internal source access. No Runtime Contract configuration
changes. Ordinary .NET remains the default; native emission is opt-in. Previously
widened default functions are now internal, so external raw calls can be rejected.
Direct source import of projected CLI globals remains deferred; a public static facade
exercises cross-assembly use without changing native ownership.
Validation: 51 focused Raven C# tests and the full binary/rvnc probe pass, including both
file orders, reference access flags, facade execution and denied external internal calls.
[Compiler/runtime evidence](experiments/extended-cli-metadata/function-visibility-raven-validation.json).
The runtime binary remains built from `fa25609d`; no runtime implementation changes.

## Class-library emission acceptance — 2026-10-01

The author prioritizes compiling actual Raven runtime-library source before broader
metadata loading, then using a broad consumer to drive missing codegen/metadata.
`NeoClrMetadataProbe --class-library-emission <runtime/raven/src> <fresh-output>`
now records source hashes, exact selected source, diagnostic phase and emitted byte
count. It uses the existing host-core primitive bootstrap; no Runtime Contract or
production target configuration changes. Ordinary .NET emission is unaffected.

The first run attempts unchanged Math, UnicodeScalar and GC files. They stop in
binding because native Result/error/RuntimeServices dependencies are absent. This
is not evidence that their bodies or metadata are supported. Selecting the original
Int32 Min/Max/Sign declarations with their System.Math namespace, excluding unrelated
imports/declarations, binds successfully and stops at NEOMETA001: native function
namespace metadata is missing. All failures leave the output empty. No runtime
execution or completed class-library assembly is claimed. The probe reports current
outcomes rather than asserting that unsupported features must remain unsupported.

Next: preserve namespace identity for native ownerless functions and define its CLI
projection explicitly, then emit and execute the selected real Math declarations.
Use neoCLR's order-collections application as the broader acceptance case: it spans
constructors/properties, generic collections/interfaces, arrays/iteration,
lambdas/delegates, Option/Result/patterns and shared reference identity. Compile its
actual library dependency sources as coverage grows; do not substitute fake library
contracts. JSON is a complementary UTF-8/inheritance case; neither sample covers
all language/runtime features. Metadata importer expansion remains deferred.

[Recorded emission inventory](experiments/extended-cli-metadata/class-library-validation.json).

## Function namespace metadata foundation — 2026-10-01

The independent metadata API and runtime now retain native ownerless function
namespaces. No Runtime Contract setting changes. The temporary CLI projection uses
encoded global names; native source lookup and regular .NET semantic-loader support
are distinct. Compiler integration is the next slice. [Contract, alternatives,
compatibility and validation](design/extended-cli-metadata.md#ownerless-function-namespaces--2026-10-01).

## Namespaced functions and real Math source — 2026-10-01

Raven now admits block/file namespace functions through a distinct shared target
capability, preserving the full semantic namespace and simple name. Both bounded
backend profiles opt in; ordinary .NET remains the default. Native functions retain
no type owner. No Runtime Contract setting changes. The native adapter requires the
independent metadata/runtime namespace slice `e8611966`; its CLI reference projection
uses reversible encoded global names. Direct source import of these projected globals
is still deferred, as is the general native metadata importer.

`NeoClrMetadataProbe --class-library-runtime <runtime/raven/src> <fresh-output> <neoclr>`
selects the original integer Min/Max/Sign declarations and their System.Math namespace.
It excludes unrelated declarations/imports without rewriting function signatures or
bodies. The selected library now emits successfully. A separate Main source exercises
11 boundary cases (Int32 endpoints, equality and all Sign branches), in both file
orders, through ordinary CLI execution and binary native verification/execution to 42.
The probe also asserts exact native System.Math namespace and null owners; it writes
source hashes, selected source and runtime hash. This uses host core primitives and
is not a full System build. Whole Math/UnicodeScalar/GC files still stop in binding
on absent native library dependencies. The order-collections consumer is the next
acceptance expansion; its constructors/properties, generics and delegates exceed the
current static primitive producer.

Validation: 53 focused Raven C# tests and the full native/rvnc integration probe pass.
[Real-source execution evidence](experiments/extended-cli-metadata/class-library-runtime-validation.json).

## Order consumer frontier and shared property identity — 2026-10-01

The broad acceptance seed is neoCLR's
`docs/experiments/raven-target/samples/application-order-collections.rvn`.
`NeoClrMetadataProbe --consumer-emission <sample.rvn> <fresh-output>` now inventories
the unchanged full source and its exact global Order declaration. It records original
and selected hashes, selected source, diagnostic phase/count (first 32 messages),
and actual semantic members. It uses host-core references only; it does not replace
native collection/LINQ/union dependencies with stubs. Full-source binding errors are
not assertions about emission coverage. No Runtime Contract setting or target default
changes, and this inventory does not claim native object execution.

The isolated Order declaration binds with zero errors and reaches the native
nonstatic-class gate. It contains two instance properties, two backing fields, four
accessors and a constructor. Repeated binding exposed a general accessor/backing-field
identity bug, now fixed in the shared member binder and independently validated by
ordinary .NET execution. The producer must consume those canonical symbols, not
filter duplicate names as a backend workaround.

Next implementation sequence: shared nominal type/receiver references and nonstatic
type definitions; primitive instance fields and constructor/accessor method contracts;
property-to-accessor associations; then allocation, constructor calls and instance
field access. Use the selected real Order declaration plus creation/mutation/aliasing
checks on both targets. Preserve ordinary CLI Field/Property/MethodSemantics concepts
where applicable; add explicit native mappings behind target capabilities. Do not
strip source properties into an ad hoc field-only contract. Generic collection and
union/delegate coverage follows that first object case; broad native symbol importing
remains deferred.

Validation: 49 focused property/binding C# tests pass, including the independently failing identity regression and .NET execution. The unchanged consumer inventory now enumerates exactly nine Order members.

[Consumer frontier evidence](experiments/extended-cli-metadata/order-consumer-validation.json).

## Root class and field producer foundation — 2026-10-01

The separate metadata API now creates nonstatic root classes with primitive mutable
instance fields. It writes ordinary CLI TypeDef/Field signatures and access flags,
and the matching existing native reference-type/field layout with field origin tokens.
The read-only snapshot exposes owned fields, signatures and type flags. This follows
[ECMA-335 sixth edition](https://www.ecma-international.org/wp-content/uploads/ECMA-335_6th_edition_june_2012.pdf)
II.22.15 (Field), II.22.37 (TypeDef) and II.23.2.4 (FieldSig), reusing the existing
native field/access design rather than adding a new runtime storage model.

CLI classes inherit System.Object; the bounded native class is a root with no explicit
base. Static AddType behavior is unchanged. The API does not synthesize a constructor
or flatten properties into fields. Benefits are faithful primitive instance layout
and independently inspectable ownership/access; costs are additional field rows,
origin metadata and matching-reader requirements. No speedup is claimed. This remains
the PE/#Neo native-payload/reference-projection bridge, not CLI-body execution.
No Runtime Contract or runtime implementation changes; the tested binary is from
`e8611966`. Native reader/writer and raw field snapshots now have matching bounds.

Validation: 48 C# metadata groups; CLI reflection and owned field snapshots preserve
class flags, multiple-type field ranges and access; malformed origins and invalid
field declarations reject. API-produced binary class/field metadata loads/verifies
in neoCLR and its independent primitive entry returns 42. This is a metadata-load
gate, not yet an object allocation/mutation test. Instance calls and field bodies
are next, followed by property associations and Raven's real Order declaration.


## Root object execution foundation — 2026-10-01

The independent metadata producer now supports root constructors and nonvirtual
instance methods with primitive declared signatures, typed receivers, allocation,
duplication and primitive field loads/stores. The API-produced Order fixture executes
constructor branches, private field access, mutation and reference aliasing. Both CLI
execution and direct binary neoCLR execution return 42; the native image verifies.
This extends the declaration evidence above without claiming Raven Order emission.

Ordinary CLI HasThis, constructor flags and newobj/dup/ldfld/stfld encodings are retained.
The CLI backend initializes System.Object before the declared root constructor body;
native root construction uses the existing runtime contract without a base call.
The metadata library owns that bounded initialization policy. Chaining/inheritance,
property associations and nominal signatures/locals remain unsupported. Field slots
are preassigned to avoid scanning the layout per emitted field instruction; no
performance claim is made. Runtime Contract configuration is unchanged. The runtime
binary remains e8611966, with no new Rust implementation required.

The PE/#Neo bridge still carries executable native bodies and throwing CLI reference
projections. A future native metadata/backend representation replaces that transport;
this slice does not change the broader import boundary or ordinary Raven .NET behavior.
Next integrate property associations and shared receiver/type contracts for the real
Order source, followed by generic/delegate/union consumer coverage. The compiler itself
is unchanged in this slice.

Validation: 49 C# metadata test groups, reference projection and CLI constructor/default
field behavior, invalid receivers and receiver-store rejection, and native binary
verify/run. [Recorded evidence](experiments/extended-cli-metadata/instance-object-validation.json).


## Primitive property associations — 2026-10-01

The metadata API now associates non-indexed primitive properties with existing static
or instance methods, including read-only/write-only properties and private accessors.
Ordinary CLI Property/PropertyMap/MethodSemantics tables preserve signature, accessor
identity and visibility. This follows ECMA-335 II.22.28, II.22.35, II.22.36 and II.23.2.5;
the existing native Property/FunctionRef representation carries the same association
and origin tokens. It adds no storage or dispatch behavior. Reuse of accessors and
same-name properties is restricted by this bounded producer. The benefit is preserving
source property contracts instead of reducing them to fields; costs are extra metadata
and reader validation. No runtime implementation or Runtime Contract changes are needed.

The metadata library owns the mapping and throwing CLI reference projection. Native
bodies still execute from #Neo; replacement with a native metadata/backend remains
future work. Existing property-free output is unchanged; property-bearing output needs
the matching reader. Raven's real Order source is still blocked at nonstatic type
emission; this API fixture does not claim compiler integration.

Validation: 50 C# metadata groups; CLI property read/write and static property execution,
read-only/write-only projection, private setter preservation and malformed accessor/
origin rejection. The binary Order fixture with Number/Pending properties verifies
and executes in neoCLR e8611966, returning 42. See
[property evidence](experiments/extended-cli-metadata/property-validation.json).


The subsequent read-only slice exposes owned Property rows and getter/setter/Other
links through the Cecil-like snapshot. Accessor identity is shared with the module's
method inventory; signatures remain copied and opaque when outside the primitive
helper. It rejects associations to another declaring type and bounds property and
MethodSemantics counts. 51 C# groups pass, including input/signature mutation isolation,
empty type ranges, native projection and an intentionally corrupted CLI accessor.
No runtime or Raven compiler behavior changes in this snapshot slice.


## Shared root and instance declaration contracts — 2026-10-01

Raven now shares root/static type categories and nonvirtual instance/static callable
categories, with backend-owned mapping. Ordinary .NET Release methods use shared
primitive signatures and receiver-aware argument slots; Debug/general fallback remains.
The native type/method adapters map AddClass/AddInstanceMethod, while native source
admission stays closed until complete constructor/member bodies are available. No
Runtime Contract setting changes or structural Function experiments are involved.

Compared with CLR metadata, the common contract separates logical source categories
from TypeAttributes/HasThis and native builder handles; .NET preserves those physical
encodings in its adapter. The cost is explicit capability checks and a currently
bounded root-class shape. This is groundwork for complete Order emission, not evidence
that the unchanged Order source now runs natively. PE/#Neo transport remains temporary;
constructor/field/property body plans and nominal locals are next. 56 focused Raven
C# tests pass (54 baseline), including Release/Debug instance execution and existing
Order symbol stability. The existing native emission probe passes; it checks supported static/assembly-function regressions, not native instance source coverage. Raven revision: `8036b3404`.


## Unchanged Raven Order source executes — 2026-10-01

Raven now admits the actual Order declaration: an ordinary root class with explicit
primitive constructor and mutable primitive auto-properties. Shared receiver, field
load/store, accessor-call and allocation instructions reuse existing compiler-lowered
bodies, including synthesized accessor bodies. Canonical symbols map to metadata
fields/methods/properties; private backing storage and accessor associations survive
CLI reference projection. .NET retains ordinary Field/Property/MethodSemantics and
constructor base initialization; native roots reuse existing runtime allocation and
instance-body semantics. Synthesized .NET debugger annotations are not projected.

This advances the source gate recorded above. No Runtime Contract configuration or
runtime source changes were needed; the tested runtime remains e8611966. The compiler
owns source admission/lowering and adapter handles; the separate metadata API owns
encoding and reference projection. PE/#Neo remains temporary native-payload transport,
not CLR IL execution. User attributes, initializers, implicit constructors, explicit
field declarations and nominal signatures/locals remain unsupported in this source path.
General nullable receivers are not admitted; tested receivers are constructed objects
or self. Full native symbol importing remains deferred.

The exact Order source plus a separate Main passes Boolean and Int32-boundary cases on
.NET and native binaries in both source orders, returning 42. C# projection checks
retain two properties, two backing fields and five methods. 52 focused Raven tests pass
(51 baseline), with Release/Debug property mutation and shared field capability checks.
The native executable probe also rejects implicit constructors, initializers and object
locals without output. Existing supported native emission probes pass. The complete
order-collections program still has 49 binding errors from missing native dependencies.
Next: object locals and aliasing/mutation, not a claim of complete consumer support.
[Runtime evidence](experiments/extended-cli-metadata/order-runtime-validation.json).


## Nominal local producer foundation — 2026-10-01

The independent metadata API now declares locals of an owned root class. Ordinary
CLI CLASS/TypeDef local signatures and existing native Named types preserve exact
class identity and aliasing, with definite-store checks and no boxing or primitive
sentinel. LocalDefinition.Type becomes nullable and ClassType carries nominal identity;
host consumers must handle that development API change. 52 C# metadata groups and a
direct binary runtime alias/mutation case pass on the existing e8611966 runtime,
returning 42 on .NET and neoCLR. No Runtime Contract or Rust change is involved.
The bounded reader validates local class ownership but reference projections continue
to omit body locals. This fills the library gap for Raven's next nominal-local slice;
null, external-class locals and inheritance conversions remain unsupported.


## Raven object locals and aliasing — 2026-10-01

Raven's shared local plan now carries primitive kinds or nominal compiler symbols,
with explicit root-local admission independent of declaration capabilities. .NET maps
the symbol to its existing CLR type; neoCLR maps it to an owned metadata class handle.
Ordinary CLASS local signatures and native Named types preserve identity; no boxing,
Object erasure or Void placeholder is introduced. The matching metadata API (212b422b)
is required, including nullable LocalDefinition.Type and nominal ClassType.

The unchanged Order plus separate Main now mutates Number/Pending through one local
and reads the changes through another alias. Both source orders verify/run on .NET
and binary neoCLR, returning 42; property/member metadata checks remain. 54 focused
compiler tests and the existing native emission regression probes pass. Nullable locals
reject without output. Nominal signatures, nullable/external/generic local types,
implicit constructors and initializers remain outside this bounded source path. No
Runtime Contract or runtime source change; runtime remains e8611966. Native symbol
loading and replacement of the temporary PE/#Neo bridge are still deferred.
[Updated evidence](experiments/extended-cli-metadata/order-runtime-validation.json).


## Ordinary source instance calls — 2026-10-01

Raven now lowers ordinary nonvirtual source instance calls through the shared receiver
and call-reference contracts. Receiver evaluation precedes arguments; nested calls and
argument side effects preserve source order. Private self calls and no-result mutating
methods reuse existing metadata/runtime signatures. No Runtime Contract or runtime/
metadata schema changes were needed. Native receivers remain constructed objects, self
or owned locals; virtual/nullable/imported instance calls and nominal signatures are
still unsupported in this bounded path.

An independent Counter helper added to the unchanged Order consumer checks nested
private calls, no-result mutation and argument order. It is test code, not a substitute
for native library dependencies. Both source orders run on .NET and neoCLR to 42.
56 focused Raven C# tests pass (54 baseline), plus the existing native emission probe.
[Updated evidence](experiments/extended-cli-metadata/order-runtime-validation.json).


## Raven private primitive storage — 2026-10-01

Raven now emits `private var` primitive storage without an initializer as the existing
private instance field definition, preserving its field-only implementation. It does
not generate a property or accessor methods. Shared body lowering also accepts qualified
field reads such as `self.Number`; unqualified reads and mutations use the same canonical
field symbol. This matches ordinary CLI field representation and adds no Runtime Contract
option, metadata API or native schema. The compiler owns source classification and shared
lowering; backend adapters own field handles and the existing binary execution bridge.

The unchanged Order declaration still executes with a separate consumer. Its Counter
helper now uses private storage, private nested calls and mutation. Both source orders
verify/run to 42 on .NET and binary neoCLR. Metadata checks assert one Counter field,
zero properties and seven real methods. 58 focused C# compiler tests pass; private storage
initializers reject without writing output. See the [updated executable evidence](experiments/extended-cli-metadata/order-runtime-validation.json).
Readonly storage, explicit field declarations, initializers and nominal signatures remain
unsupported by this bounded native producer. The full native metadata/backend replacement
and full consumer dependency coverage remain open.


## Accessible val setters in Raven — 2026-10-01

Raven commit `935598201` corrects a shared binding issue exposed while integrating
computed properties: public read-only `val` semantics no longer block assignment,
compound assignment or increment/decrement through an accessible explicit private
setter inside its owner. Outside writes remain rejected and the semantic property's
public mutability stays false. This restores Raven's documented property contract;
it adds no native semantics, Runtime Contract option or metadata encoding. The native
backend will consume the same bound setter calls as .NET. 60 focused property binding,
execution and regression tests passed; explicit native accessor emission is a separate
slice. This general fix is intended for Raven's shared line, not a permanent target fork.


## Raven computed properties and explicit accessors — 2026-10-01

The native compiler producer now accepts primitive computed property expressions and
implemented get/set block or arrow accessors on bounded root classes. Shared callable
plans feed existing body lowering and field/call adapters on .NET and neoCLR. Optional
backing fields preserve Raven's `field` semantics; computed-only properties acquire no
storage. Property associations and accessor access remain standard CLI reference rows
with matching native execution metadata. This is coverage of the existing format,
not a new metadata category, Runtime Contract switch or schema. It builds on the existing
root/property design comparison and the shared setter-binding correction above.

The executable Gauge helper covers computed-only getters, block accessors, a private
setter on `val`, setter branching and `field` reads/writes. Both source orders verify/run
to 42 on .NET and binary neoCLR alongside unchanged Order source. Projection assertions
check two Gauge fields, four properties and nine methods, including private/getter-only
accessor shapes. Independent C# Release/Debug tests check behavior and metadata; the
60-test focused emission set passes. The [saved evidence](experiments/extended-cli-metadata/order-runtime-validation.json)
includes the consumer and runtime hashes. Explicit accessor lists without bodies still
reject before output; initializers, init-only accessors, indexers, virtual dispatch,
nominal signatures and full native metadata/backend replacement remain future slices.


## Raven expression-bodied constructors — 2026-10-01

Native Raven emission now accepts explicit root constructors with arrow bodies. It
reuses existing source callable plans and compiler lowering for assignment and Unit
helper-call expressions. Overload signatures and the new-object contract are unchanged;
no metadata schema, Runtime Contract option or runtime code change is needed. This
matches the existing CLI constructor representation rather than introducing a native
constructor variant. The .NET general constructor generator retains its base initialization
responsibility; this change only broadens native declaration admission.

The expanded Order executable consumer tests both overloads, helper-driven initialization,
and two mutating argument calls whose order and evaluation count affect the result. Both
source orders verify/run to 42 on .NET and binary neoCLR. Two focused C# Release/Debug
tests check shared planner admission, overload metadata and execution; the 10-test baseline
passed. Explicit base chaining rejects before output along with the earlier unsupported
contracts. [Evidence](experiments/extended-cli-metadata/order-runtime-validation.json).
Implicit constructors, chaining and field/property initializers still need a shared
initialization contract. Nominal signatures and full native metadata/backend replacement
remain separate work.


## Default constructors and primitive initialization — 2026-10-01

Raven's shared FieldInitializationPlan now supplies canonical bound field assignments to
both the .NET constructor generator and native constructor lowering. Native default root
constructors use an empty body after these assignments; explicit constructors run them
before their bodies. Mutable private storage and primitive property initializers now work
without a metadata/runtime change or Runtime Contract option. This restores ordinary
constructor initialization behavior through the existing CLI/native field representation.
Chaining, primary constructors, readonly storage and lifecycle initialization blocks remain
explicitly outside this native slice. 44 focused C# tests pass; implicit and explicit
initialization execute in both source orders to 42 on .NET and binary neoCLR.
[Evidence](experiments/extended-cli-metadata/order-runtime-validation.json).


## Owned nominal signature producer foundation — 2026-10-01

The independent metadata library now accepts owned root-class parameter/result types in
functions, static/instance methods and constructor parameters. MethodSignature carries
immutable SignatureType values, preserving exact TypeBuilder identity. Existing primitive
construction remains available through PrimitiveMethodSignature; the public builder and
signature-property API types have widened and compiled consumers must be rebuilt.
CLI output uses CLASS TypeDef signatures; native output uses existing Named records, with
no new runtime schema or Runtime Contract option. The declaration reader remaps those
references into each reference assembly. Imports remain primitive-only; foreign class
signatures reject before mutation and external nominal calls reject before writing.

53 C# metadata groups pass, including nominal CLI execution, raw signature projection
comparison, rejection of unknown named types and wrong-class arguments/results. The
API-generated binary loads, verifies and runs to 42 in neoCLR with factory/identity and
self-return calls plus mutation through a nominal parameter. See the
[API contract](../api-docs/experimental-metadata.md#nominal-signatures) and
[design comparison](design/extended-cli-metadata.md#owned-nominal-callable-signatures--2026-10-01).
Raven's shared logical signature mapping is the next consumer slice; this foundation
does not introduce a general semantic importer or external nominal type contract.


## Raven owned nominal signature integration — 2026-10-01

Raven now consumes metadata commit `b2333489` through shared CallableSignature and
EmissionType values carrying compiler identities. Explicit adapter capabilities admit
root-class signatures; backend maps resolve .NET types or owned metadata TypeBuilders.
All native class definitions exist before method signatures are materialized. Factory
results, nominal arguments, self-return, constructor parameters and overload distinction
now use the same body plan as primitive calls. The native reader carries those signatures
through the CLI reference projection. There is no Runtime Contract option, native opcode
or schema change; the existing Named type records load directly in the runtime.

63 focused C# compiler tests pass, including Release/Debug execution and a profile that
denies nominal signatures. The expanded unchanged-Order consumer tests returned aliases,
mutation, discarded nominal results, nominal overloads and constructor arguments; both
source orders verify/run to 42 on .NET and binary neoCLR. The existing primitive/import
regression probe passes after updating its obsolete default-root-constructor rejection
to reject abstract classes. [Saved evidence](experiments/extended-cli-metadata/order-runtime-validation.json)
records consumer/source/runtime hashes. The direct metadata producer separately passes
53 groups and a binary runtime case. API snapshot validation passes.

The requested bounded nominal-signature/default-constructor/primitive-initialization
slices are complete. Cross-assembly nominal importing, nullability, generic/structural
signatures, nominal fields/properties, readonly storage and constructor chaining remain
separate contracts. General metadata importing is still deferred; the broad consumer's
49 native collection/LINQ/union binding errors are not hidden with substitute libraries.


## Raven explicit primitive fields — 2026-10-01

Native Raven declaration collection now accepts explicit mutable primitive instance
fields on bounded root classes. It preserves public/internal/private access, canonical
field identity and the shared initialization/load/store paths, producing field rows with
no synthetic property/accessor rows. This fills a compiler collection gap in the existing
CLI/native field contract; the independent metadata API and runtime require no changes.
No Runtime Contract configuration is added. Ordinary private storage still uses idiomatic
`private var`; explicit fields express intentional field identity or a public field API.

Two C# Release/Debug tests pass after a seven-test constructor/reference-field baseline.
The expanded Order consumer initializes public Int32, internal Int64 and private Boolean
fields, mutates through an alias and reads the original instance. Both source orders
verify/run to 42 on .NET and binary neoCLR; projection checks preserve access and the
absence of properties. Static and nominal field rejection leaves output untouched.
Readonly, by-reference and attributed field declarations remain later contracts.
[Evidence](experiments/extended-cli-metadata/order-runtime-validation.json).


### Owned nominal field storage — 2026-10-01

Raven's shared field load/store plan now accepts the same owned root-class logical
types as callable signatures. The native adapter maps explicit mutable fields and
private `var` storage to independent metadata field handles; nominal property rows
remain outside this slice. No Runtime Contract configuration changes are required.
Ordinary .NET emission remains the default and uses the existing field definitions.

The independent API writes ordinary CLI CLASS signatures and existing native Named
field records. The current binary #Neo executable/reference-projection bridge remains
in place; native extended-CLI codegen will replace that bridge, not the logical source
field identity contract. External imports, nullable contracts, readonly enforcement,
static fields and generics remain deferred. Default allocation is not a new source
non-null initialization guarantee.

Validation: 54 metadata C# contract groups; API-produced PE verifies/runs with result
42; four Raven C# Release/Debug field tests; unchanged Order plus a separate Holder
consumer verifies/runs with result 42 on .NET and binary neoCLR in both file orders.
The consumer checks private nominal initializers, replacement and mutation through
an alias stored in a field. Five incomplete-contract fixtures still reject without
output, now including nullable nominal fields instead of supported nominal fields.
Runtime remains revision e8611966 on codex/extended-cli-metadata; compiler work is on
codex/metadata-consumer. See the refreshed
[record](experiments/extended-cli-metadata/order-runtime-validation.json) for hashes.
The broad collection/LINQ/union consumer and native symbol imports remain incomplete.


### Owned nominal property emission — 2026-10-01

Raven's neoCLR adapter now maps owned root-class property types through SignatureType,
reusing existing accessor plans and field mapping. Auto-properties with initializers,
computed getters, block/arrow accessors and private setters retain property associations
and exact class identity. Ordinary .NET remains the default; no new Runtime Contract
option is required. The independent host API now exposes SignatureType through
AddProperty/PropertyType, requiring development consumers to rebuild. CLI output uses
ordinary CLASS property signatures; native property/accessor records use existing Named
types. This extends the current #Neo/reference-projection bridge without a binary schema
change. Eventual native extended-CLI emission replaces that bridge encoding.

The .NET C# regression exposed a general Raven binding defect: a reused auto-property
field retained a provisional BoundErrorExpression from before a forward constructor was
bound. The binder now refreshes the field initializer while preserving canonical identity.
This is a shared semantic/emission correction, independently validated on .NET, rather
than a native-only workaround. It restores normal constructor-initializer behavior.

Validation: 55 metadata contract groups, API-produced PE verification/execution (42),
14 focused Raven C# tests in Release/Debug, and the Order consumer's native/CLI runs (42)
in both source orders. Coverage includes property replacement, alias mutation, forward
constructor initializers, accessor visibility and projected nominal signatures. Six
unsupported fixtures reject before writing, including nullable nominal properties.
[Evidence](experiments/extended-cli-metadata/order-runtime-validation.json) records hashes.
Runtime remains e8611966 on codex/extended-cli-metadata, with compiler work on
codex/metadata-consumer. Readonly/static storage, indexed/generic/nullable properties,
external nominal imports, constructor chaining and the full broad consumer remain open.


### Explicit root base initialization — 2026-10-01

Raven now admits explicit `base()` on bounded root constructors only when semantic
binding identifies parameterless System.Object construction and no source/bound
arguments. Compared with CLR initialization this preserves the existing root contract:
.NET emits its normal bound base call, the independent CLI writer supplies its ordinary
Object constructor prologue, and a native root has no base. No metadata format, runtime
binary or Runtime Contract configuration changes are needed. This does not add
user-defined base calls, inheritance or general constructor delegation; unresolved and
unsupported initializers reject before output.

The executable case also exposed private stored-property rebinding that recreated
backing fields. Raven now preserves canonical identity for all stored properties, so
initializer completion updates the field used by emission. This general compiler fix
is independently validated on .NET rather than limited to the native bridge.
Sixteen C# constructor/property/field tests pass in Release/Debug. Side-effecting field
initializers and block/arrow constructor bodies run in the expected order. The Order
consumer verifies/runs to 42 on both runtimes in both source orders; six rejection
fixtures remain, now including a user-defined base call instead of root `base()`.
[Updated evidence](experiments/extended-cli-metadata/order-runtime-validation.json)
records the unchanged e8611966 runtime hash. Work remains on the metadata and compiler
feature branches; this is not a claim of general inheritance support on main.


### Readonly instance storage — 2026-10-01

The independent API now models primitive and owned nominal readonly fields via
AddField(isReadOnly: true)/FieldBuilder.IsReadOnly, using standard CLI InitOnly and
existing native field_readonly flags. The writer rejects stores outside the declaring
constructor; projections preserve flags. The runtime now checks those flags on direct
stores and restricts managed field addresses outside the declaring constructor to
readonly access. Reading the address and mutating a referenced object remain legal.
This is shallow storage protection, not deep immutability or an unsafe-memory sandbox;
raw unmanaged pointers remain outside the managed guarantee.

Raven passes canonical field mutability into this API and admits private val storage
and stored val properties through the existing shared initialization plan. Ordinary
.NET remains the default. No Runtime Contract option or binary schema version changes,
but this is a semantic compatibility change: rebuild host consumers for AddField's new
optional parameter and use the updated runtime. The earlier e8611966 runtime does not
enforce these flags during execution. The tested runtime is the readonly-field slice on
codex/extended-cli-metadata; the exact tested binary SHA is recorded in the refreshed
[Order evidence](experiments/extended-cli-metadata/order-runtime-validation.json).
The origin-array encoding is a temporary bridge; native field semantics must replace
it when the extended-CLI backend replaces the bridge.

Validation: 56 metadata C# contract groups, eight Raven constructor C# tests in
Release/Debug, and 82 focused Rust tests covering constructors, access, managed references,
metadata origins and reflection. API-produced binaries return 42 for valid construction;
illegal direct and indirect stores fail both verify and run. The Order consumer returns
42 on .NET and neoCLR in both source orders and preserves InitOnly/getter-only metadata.
Static/literal storage, explicit readonly field syntax in the bounded Raven collector,
generic/nullable fields and external nominal imports remain separate work.

## Shared vector emission — 2026-10-01

Raven now emits one-dimensional zero-based arrays of admitted primitives and owned
root classes through the shared logical type/instruction plan. Both target adapters
explicitly admit vectors; signature, local, field and property mappings preserve exact
element identity. Literal allocation (including empty literals), Int32 indexing, element
assignment and Length are supported. Evaluation is receiver/index/value order; aliases
retain the same array and contained objects. The unchanged Order declaration and its
three-element batch expression from the broad consumer execute in a separate array
consumer on .NET and binary neoCLR, in both source orders, returning 42.

Ordinary .NET remains the default and no Runtime Contract option is added. The native
adapter consumes the independent metadata API on codex/extended-cli-metadata (8e2a56ed);
compiler work remains on codex/metadata-consumer. CLI uses SZARRAY and standard typed
array instructions; the temporary native execution payload uses ArrayRef/newarr/ldelem/
stelem/ldlen. Length normalizes to Int32 explicitly, matching the source API. A later
native metadata backend replaces payload serialization, not this shared logical plan.
The host core is still the binding bootstrap; this does not implement general native
metadata imports or compile the complete collections application/System library.

Fixed-length type contracts, nested/multidimensional arrays, spread/comprehension expansion, covariance, imported
nominal elements, spans and element addresses remain outside this bounded shared path.
.NET keeps its general fallback. Array iteration is covered below. Backend primitive
type tokens are cached per output, and literals allocate directly without intermediate
collections; no performance measurement or speedup is claimed. C# shared-plan tests
cover Release/Debug execution and capability rejection; the executable ArrayChecks
probe verifies native binaries and storage projections in both source orders.

### Shared array iteration

Ordinary rank-one array `for` loops with an exact element local now lower into existing
bound locals, Length/index operations and branches, before either backend. Collection
evaluation occurs once. Nested loops, labeled continue/break, ordinary break/continue
and empty arrays retain their source semantics. Loops left for general .NET codegen
keep ownership of their unlabeled transfers, even inside a lowered vector loop.
There is no backend-specific language rewrite or additional metadata category.
Discard/converting iteration, generic enumerators and multidimensional arrays remain
outside this bounded native path; .NET uses its existing general path where needed.

Compared with the previous .NET generator-owned array loop, this shares lowering and
control flow across targets at the cost of temporary bound nodes and locals during
compilation. Runtime iteration remains indexed with no enumerator allocation. Performance
has not been benchmarked. C# tests independently validate .NET Release/Debug, target
admission, mixed enumerator/vector nesting and existing range/async/iterator loops
(39 tests passed). An additional fixed-length signature rejection check also passes;
this prevents silently erasing shape metadata. The expanded Order-array probe verifies and runs binary neoCLR
output and .NET output in both source orders (42), including side-effecting collection
calls and labeled continue. `array-runtime-validation.json` records source and runtime
SHA-256 identities. The full collections application and System build remain incomplete.

## Shared indexed property emission — 2026-10-01

Raven now admits implemented root-class indexers through an explicit IndexerAccessor
capability. Both .NET and neoCLR share getter/setter body planning, including declaration
arrow getters, and receiver/index/value evaluation. Calls use ordinary accessor methods;
the native adapter associates their existing signatures with indexed Property rows in
the independent metadata API. Overloaded index types, multiple indices and read-only
indexers preserve signatures/accessor associations in native CLI reference projections.

Ordinary .NET remains the default, with general fallback for unsupported shapes; no
Runtime Contract option changes. The metadata API on codex/extended-cli-metadata
(00752f25) uses standard CLI indexed Property signatures and existing native property
parameter lists. Native execution still uses the temporary #Neo payload plus CLI
projection; eventual native metadata/backend replacement must retain these associations.
The tested native runtime is identified by SHA-256 in indexer-runtime-validation.json.
No runtime opcode or indexed introspection GetValue/SetValue API was added.

The bounded native collector rejects interface/virtual/static indexers, ref/default/
variadic index parameters and unsupported element types. Imported symbol loading and
DefaultMemberAttribute synthesis for other CLI compilers remain separate contracts.
The .NET backend retains its ordinary DefaultMemberAttribute behavior. No parameter
boxing or intermediate array is introduced by shared indexed calls; performance has
not been benchmarked. Eleven focused C# tests pass, covering shared planning and
Release/Debug execution plus existing struct/imported-interface fallback. The native
probe verifies and runs overloads, two-index properties and read-only getters on both
runtimes in both source orders (42).

### Indexed Order collection acceptance

The expanded separate consumer retains the original Order declaration/batch expression
and wraps the array in a concrete OrderBuffer. Its indexers return and replace objects;
mutations remain visible through the original array and aliases. Nominal and array
index parameters, overloads and multi-index properties preserve their associations.
Side-effecting receiver/index/value calls execute once in order. Both file orders
verify/run on .NET and neoCLR to 42. An out-of-range indexed read verifies successfully
and propagates IndexOutOfRange on both runtimes. Unsupported fixed/nested-array index
signatures reject without writing output. C# Release/Debug tests also check two-index
assignment order independently. Evidence fields now name the actual indexer checks,
replacing copied array-probe labels from the initial snapshot. This is a bounded
collection consumer, not the generic ArrayList/HashMap implementation or full sample.

Generic producer milestone: the host metadata API now emits owned unconstrained generic
static calls using CLI MethodSpec and native generic arguments. Forwarded method
parameters and generic array factories verify and execute from a binary PE/#Neo
container (42). Raven source integration is the next slice; generic type/constraint/import
support is not implied. The native loader admits generic static class methods while
retaining its generic instance-method restriction.

Raven shared generic source emission now connects this producer to unconstrained
owned functions and static methods, including forwarded method parameters, locals,
vector arguments and owned Order references. The `--generic-runtime` probe returns 42
on .NET and binary neoCLR in both source orders. Generic capabilities are explicit;
there is no new Runtime Contract switch. .NET generic declaration registration remains
in its existing adapter path; shared Release bodies use those registered handles.
Native imported generics, generic types, instance generics and constraints remain outside
this milestone. Development branches: Raven `codex/metadata-consumer`, neoCLR
`codex/extended-cli-metadata` (producer/runtime commit `2916fc3f`).

Expanded generic acceptance now covers inferred/explicit calls, recursive forwarding,
multiple type parameters and overloads, generic array creation/iteration, conditional
values and object identity. Both source orders return 42 on .NET/native; five binding-valid
unsupported forms reject with source diagnostics and no output. [Recorded evidence](experiments/extended-cli-metadata/generic-runtime-validation.json)
includes the tested runtime and consumer hashes. This is a bounded milestone; native
generic types, constraints, instance methods and imported generic symbols are still open.

### Generic class receivers (2026-10-01 development)

The experimental metadata producer and native verifier now admit ordinary nonvirtual
generic instance methods on owned root classes. CLI uses instance GenericParam/MVAR
signatures and MethodSpec calls; native calls carry instance=true alongside generic
arguments. Receiver slot zero is independent from method generic parameter zero.
The C# producer binary verifies/runs (42) with receiver mutation and forwarded generic
object identity; missing receivers reject before output. Generic constructors,
by-reference class receivers and virtual generic dispatch remain unsupported. This
continues the PE/#Neo execution bridge and its CLI projection rather than introducing
a new metadata category. Raven integration follows in a separate slice.

Raven now consumes this receiver contract through shared callable signatures and its
existing receiver-first body planner. An explicit instance-generic capability controls
admission independently from static generics; no Runtime Contract option is introduced.
.NET retains generic declaration registration in its adapter and native code uses
owned generic call references. Release/Debug C# cases and the Order binary consumer
pass (42) on both runtimes/source orders. Generic type owners, constraints and virtual
generic dispatch remain outside this step.

Expanded receiver acceptance: no-result generic copy/reverse methods, recursive
instance calls, receiver/argument evaluation order and separate receiver state execute
on both targets (42). [Evidence](experiments/extended-cli-metadata/generic-runtime-validation.json)
records the consumer and runtime hashes. Runtime checks also preserve generic receiver
and argument roots during forced collection; API validation rejects wrong/missing
receivers and generic constructor projections.

### Generic defaults (2026-10-01 development producer)

The host metadata API now emits typed local addresses/initialization and LoadDefault.
CLI ldloca/initobj/ldloc and native equivalents preserve generic primitive/reference
zero/null defaults without a new bridge encoding. Flow validation tracks exact local
address provenance and definite assignment; general byref signatures remain excluded.
The C# default binary verifies/runs 42. Native instructions and generic substitution
already support these operations, so no runtime semantic change is needed. Raven
source lowering is the next integration slice.

Default emission reuses the established [managed initialization](managed-initialization.md)
and [intrinsic String defaults](string-default-storage.md) contracts and their .NET
comparison. A new default opcode or special generic runtime intrinsic would duplicate
existing typed storage semantics; scratch-local lowering is simpler and preserves
backend independence, at the cost of one local and three instructions per default site.
No performance improvement is claimed; scratch reuse is a possible later optimization.

Raven now lowers supported default values through a shared DefaultValue operation with
backend-owned scratch locals. Generic numeric array clearing and typed primitive/
reference defaults pass the expanded binary consumer on both source orders and runtimes.
Clearing an Order vector and dereferencing an element raises a null-reference fault on
both. C# Release/Debug tests validate defaults and capability denial. Reference-loading,
nullable-source signatures and general byref contracts remain deferred; this is direct
emission using the existing .NET binding bootstrap. [Evidence](experiments/extended-cli-metadata/generic-runtime-validation.json).

Static generic owner producer slice (2026-10-01): independent declaring-type and method
parameter scopes now emit ordinary CLI VAR/MVAR, constructed TypeSpec/MemberRef and
MethodSpec calls. Native open/constructed owner records already support this behavior;
C# producer binaries verify/run 42 without a runtime change. Reference projection
retains type/method arities. This follows the existing [generic metadata contract](generic-metadata.md)
rather than flattening owner parameters into method parameters, which would lose type
identity. Generic instance type layouts and fields remain the next larger boundary;
Raven source integration follows this API slice.

Raven static generic owner integration now verifies/runs 42 on both targets and in
both source orders. Explicit capabilities preserve independent owner/method arguments;
the .NET resolver also fixes source method calls that previously left their declaring
type open. Tested matching producer: `0da5a3b0`, receiver runtime `6a7a0dd2` or later
on `codex/extended-cli-metadata`; Raven `codex/metadata-consumer`. No Runtime Contract
configuration change. Generic owner arrays/defaults and alias mutation pass; generic
instance layouts/fields, constraints, imported owners and full class-library compilation
remain open. See [binary evidence](experiments/extended-cli-metadata/generic-runtime-validation.json).

The final owner consumer also permutes two owner arguments and forwards a caller
method parameter into a callee owner while forwarding an owner parameter back into
its result. Both source orders execute 42 on CLI/native. Native reader contract tests
reject out-of-range VAR, closed/noncanonical declaration owners and arity/name mismatch.

Generic instance class producer slice (2026-10-01): AddGenericClass and constructed
class signatures now support instance constructors, owner-typed fields and calls on
exact constructed receivers. CLI uses standard GENERICINST/VAR/member references;
native uses the existing generic record/reference contract, preserving field layout
and type identity. API-produced binaries verify/run 42 without runtime changes.
Raven integration follows. External constructed fields, generic properties, constraints
and full class-library emission remain open. This reuses the [generic metadata design](generic-metadata.md)
rather than creating a second generic storage representation.

Generic instance storage integration now passes: Raven shares generic class value,
constructor and field-body planning; native emission binds owner and method arguments
independently. Primitive, Order and nested Box<Box<int>> consumers verify/run 42 on
.NET/neoCLR in both source orders. Matching producer: `62bf5931` or later; runtime
`6a7a0dd2` or later on `codex/extended-cli-metadata`, Raven `codex/metadata-consumer`.
No Runtime Contract configuration change. Generic properties and external constructed
field references are explicit remaining bridge limits, followed by constraints/imports
and broader class-library acceptance. [Evidence](experiments/extended-cli-metadata/generic-runtime-validation.json)
records the consumer and runtime hashes; this does not establish full-library emission.

Generic property producer slice (2026-10-01): generic static/instance property values
and index parameters now preserve VAR scope and accessor associations through standard
CLI Property/MethodSemantics and existing native Constructed owners. Reader validation
requires exact canonical open owner identity/arguments. API reflection execution and
native binary verification/run (42) pass; no native schema or Runtime Contract change.
Raven accessor integration follows; external constructed fields and constraints/imports
remain separate work. This extends the existing [generic metadata contract](generic-metadata.md).

Generic property/indexer integration is now verified through Raven: setter/getter calls,
Order alias mutation and independent generic key/value parameters run 42 on .NET and
neoCLR in both source orders. Matching producer/reader: `dbe03b1a` or later on
`codex/extended-cli-metadata`; Raven `codex/metadata-consumer`; receiver runtime
`6a7a0dd2` or later. No Runtime Contract change. Fifteen focused C# tests pass;
[recorded evidence](experiments/extended-cli-metadata/generic-runtime-validation.json)
includes consumer/runtime hashes. External constructed fields, constraints and generic
imports remain open, along with broader class-library acceptance.

Constructed-field producer slice: typed field references now bind generic owners and
validate exact receiver/value types before emitting standard CLI Field MemberRefs or
existing native field operations. No schema/Runtime Contract change. Raven integration
follows; the author directs generic type constraints immediately after this slice.
Constraints must preserve CLR/native semantics rather than translating unlike flags.

Author-directed type-constraint slice (2026-10-01): owned nongeneric class bounds now
use CLI GenericParamConstraint and existing native TypeBound, preserving meaning rather
than mapping CLR flags to native notvoid/notreference. Producer/reference projection and
binary runtime verification/execution pass; invalid concrete native arguments reject.
Raven integration follows. The author requests the other constraint categories next;
class/struct/new/nullability and interface/dependent bounds require distinct contracts.
This reuses the existing [generic contract research](generic-metadata.md).


Special type constraints and assessment (2026-10-01): Raven now preserves `class`,
`struct` and `new()` type-parameter requirements through shared capability admission
and the independent producer. Owned nominal bounds and constructed-field access are
also integrated. The broad generic consumer verifies/runs 42 on CLI/native in both
source orders; this is still selected-source coverage, not the full application.
Ordinary .NET struct metadata now includes its implied DefaultConstructor flag.

CLI GenericParam flags follow [.NET's documented categories](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.genericparameterattributes?view=net-10.0).
Native ReferenceType/ValueType/DefaultConstructor are new distinct constraint kinds,
rather than aliases for notvoid/notreference: String is a reference; managed arrays
are references; pointers/byrefs and Void are not admitted as these value arguments.
Native value records satisfy value/default-constructor requirements; reference classes
need a public zero-argument instance constructor and must not be abstract for new().
This preserves useful concrete-instantiation checks at the cost of a runtime schema
extension. Existing payloads retain their meaning, but new kinds require the matching
`codex/extended-cli-metadata` runtime. Raven is on `codex/metadata-consumer`; no Runtime
Contract option change. The [evidence](experiments/extended-cli-metadata/generic-runtime-validation.json)
records the tested runtime hash. No release/main support is implied.

The producer owns metadata identity/validation; Raven owns source constraints and
capability admission; the runtime checks concrete substitutions. Native executable
metadata plus CLI reference projection remains a temporary bridge. Metadata importer,
new T() codegen, open constrained dispatch, method bounds, interface/dependent bounds,
notnull and byref-like contracts remain outside this producer slice. Notnull has no
corresponding CLI special flag and must not be silently erased. The planned native
metadata/backend replacement must preserve those distinctions.
See the [state assessment](experiments/extended-cli-metadata/state-assessment-2026-10-01.md).


Whole-source Language acceptance (2026-10-01): the complete unchanged
runtime/raven/src/System/Globalization/Language.rvn now executes through Raven's shared
accessor-call lowering. Static computed properties map to ordinary static calls and
existing CLI Property/MethodSemantics/native associations. The independent metadata
API and runtime need no change. Static setters and constructed generic static getters
also execute; static storage remains a diagnosed unsupported capability. Runtime
Contract configuration and explicit console bridge are unchanged. Both file orders
print und/sv/he and return 42 on .NET/native; five focused compiler tests pass.
[Evidence](experiments/extended-cli-metadata/whole-library-runtime-validation.json)
records hashes and the host-core bootstrap. This does not establish full-library
compilation or native symbol import.

The [updated source inventory](experiments/extended-cli-metadata/class-library-validation.json)
shows Comparer<T>/EqualityComparer<T> bind but need interface declaration emission;
ArrayList still fails binding on native dependencies. That identifies the next bounded
contract without claiming collection support. Per the author's clarification, .NET
behavior and CLI instruction semantics are the default for supported facilities.
Unsupported exceptions/other features limit coverage; the temporary #Neo storage
bridge does not justify a different instruction set. Native backend/metadata replacement
must preserve these semantics and only deliberately chosen extensions.


Interface declaration slice (2026-10-01): unchanged Comparer<T> and
EqualityComparer<T> now compile through an explicit bounded interface declaration
plan and capability categories. The producer preserves ordinary CLI interface/type
and public abstract virtual new-slot method flags, with RVA zero and no fake bodies.
Native metadata reuses Interface identity and bodyless method contracts; native
record is_abstract storage is not reused for interface type identity. No runtime or
instruction-set change. [API contract](../api-docs/experimental-metadata.md#interface-declarations-development).

The matching experimental reader projects the same declarations. The producer owns
metadata flags/identity; Raven owns source admission. No Runtime Contract configuration
change; host-core bootstrap remains. Generic variance, inherited interfaces, properties,
implementations, interface value signatures and dispatch remain subsequent slices.
The two complete source files verify/load in both file orders on .NET/native; an
independent entry returns 42. This is metadata-loading evidence, not dispatch evidence.
[Consumer evidence](experiments/extended-cli-metadata/interface-library-runtime-validation.json).
68 API test groups and 13 focused compiler tests pass; API-produced interface binaries
also load/verify/run the independent entry. Both feature branches remain experimental.


Iterator declaration extension: the same producer/adapter now preserves nongeneric
inherited interface edges and abstract property associations. Complete unchanged
Disposable and Iterator<T> sources join the comparer test in both file orders.
CLI uses InterfaceImpl/Property/MethodSemantics and bodyless SpecialName accessors;
native metadata uses existing implements/property records. The reader validates cycles,
identity and accessor scope, and accepts canonical empty field-origin arrays for
property-only types. No runtime or instruction change. 68 API groups and 13 focused
compiler tests pass, plus native binary verification. The entry remains independent;
interface calls/implementations, generic base instantiations and interface-valued
signatures remain next. Runtime Contract options and host-core bootstrap are unchanged.


Interface reference extension (2026-10-01): unchanged Iterable<T> now emits beside
Comparer, EqualityComparer, Disposable and Iterator. Owned interface identities and
constructions share CLI CLASS/GenericInst and native Named/Constructed signatures.
The shared compiler descriptor is nominal rather than class-specific, with an explicit
interface-signature capability; nullable reference annotations retain binder semantics
and map to the same reference storage. Parameters, results, locals, defaults and arrays
execute on .NET and binary neoCLR in both source orders (42). The host-core bootstrap
and Runtime Contract configuration are unchanged. No runtime or ISA changes are needed.
Interface invocation/implementation remains the next author-directed acceptance gate;
generic interface inheritance remains deferred. Seven focused C# interface tests and
68 metadata API test groups pass. The evidence exercises null/default reference flow,
not dynamic dispatch.


## Owned interface dispatch — 2026-10-01

Raven now admits nongeneric root classes implementing owned nongeneric interfaces,
including inherited contracts. Shared InterfaceImplementation and InterfaceCall
capabilities gate admission; implicit reference conversions retain normal binder
semantics. Interface method/property calls use ordinary CLI callvirt (0x6f), and the
native payload uses existing callvirt/implements contracts. The separate metadata API
owns implementation validation, InterfaceImpl rows and public virtual/final/new-slot
CLI implementation flags. Native concrete methods use existing implicit implementation
lookup. No runtime code or instruction-set changes are required. Runtime Contract
configuration and the host-core bootstrap remain unchanged.

The C# producer rejects missing/incompatible implementations, foreign/generic dispatch
targets and unrelated receivers; direct calls to abstract contracts remain invalid.
CLI and native execution select two implementations through an inherited contract (42),
and null receiver calls fault. Raven additionally executes interface method/property
calls and interface-array alias updates in both source orders on .NET and neoCLR (42).
[Evidence](experiments/extended-cli-metadata/interface-dispatch-validation.json). Twelve focused C# compiler tests and 69 metadata API groups
pass. Owned nongeneric implicit implementations are the bounded target contract;
generic interface dispatch, explicit/default methods, external imports and class virtual
overrides remain future adapter work, not alternate platform semantics. Both feature
branches remain experimental. Native symbol loading remains a separate compiler gate.


The post-dispatch [readiness assessment](experiments/extended-cli-metadata/readiness-assessment-2026-10-01.md)
compares the direct binary backend and existing CLI bridge. In particular,
CompilationOptions.NeoCLR remains rejected by the direct backend's bootstrap gate even
though twelve unchanged samples bind and emit through ordinary CLI. Reference snapshots
for consumers do not supply RuntimeServices/CheckedStorage for compiling System itself.
Three legacy-bridge samples run; order-collections produces invalid Option constructor
CLI and is correctly rejected before execution. These are explicit integration gaps,
not changes to runtime contracts or acceptance claims for the native backend.

## Direct primitive-vector imports (2026-10-01)

On `codex/extended-cli-metadata` with Raven `codex/metadata-consumer`, a separately
emitted Raven library returns arrays to a native-profile application. Static nongeneric
Int32/Int64/Boolean/String vector signatures use ordinary CLI SZARRAY in the declaration
projection and existing native array identities in the executable payload. Both binaries
load directly into neoCLR; neither body goes through the CLI-to-JSON importer.

Runtime Contract configuration remains `CompilationOptions.NeoCLR` with the matching
NeoCLR.CoreProbe reference and explicit core/dependency bindings. Raven still loads
symbols through projected CLI declarations; arbitrary nominal/generic imports, nested
vectors and native semantic metadata loading remain unsupported. The projection loses
native semantic categories and is not an executable replacement for the original image.
The independent metadata API owns signature encoding; Raven's target adapter owns
mapping and capability checks. Native semantic-data loading must eventually replace
this declaration bridge. No default .NET binding or emission path changes.

The C# `--vector-library-runtime <neoclr-root> <fresh-output> <runtime>` probe in Raven
checks four element overloads, alias mutation, void calls, iteration, missing bindings
and mismatched signatures. See [validation](experiments/extended-cli-metadata/vector-library-validation.json)
for tested core/runtime identities. Imported generic collections remain open.

## Imported generic-method boundary (2026-10-01)

The metadata/Raven feature branches now admit unconstrained static generic methods
on nongeneric owners with primitive/vector arguments. Native intent is ordinary generic
substitution; the temporary input projection uses standard CLI MVAR/GenericParam and
output CLI uses MethodSpec/MemberRef. Native binaries retain the existing generic call
encoding and load directly, with no body translation or runtime format change.

Configuration still requires CompilationOptions.NeoCLR, the matching declaration
core and explicit dependency/core bindings. The metadata API owns decoding, scope and
constraint rejection; Raven's adapter owns exact symbol/signature matching and concrete
argument admission. Generic parameter display names are positional in the imported API;
nominal identities, generic owners, constraints and caller-generic forwarding remain
outside this imported subset. A future native semantic loader must replace the projected
CLI input without treating these temporary limits as platform rules.

The C# `--generic-library-runtime` probe emits both binaries, verifies/runs them (42),
checks arity overloads, array aliases, void calls and missing dependency/declaration
rejection. [Evidence](experiments/extended-cli-metadata/generic-library-validation.json)
pins branches and tested runtime/core hashes. .NET shared codegen was not changed.

## External signature prerequisite (2026-10-01)

The metadata API on codex/extended-cli-metadata can now import public unconstrained
reference-type declarations and form external generic signatures using consumer-owned
type arguments. CLI keeps ordinary TypeRef/TypeSpec shape; native Named/Constructed
references retain manifest scope and survive the declaration projection. Binary runtime
loading is tested with a separate library and Box<consumer Order> signatures.

Raven's target adapter is unchanged in this slice. Its collections Register signature
still fails admission. Follow-through must map external symbols to these references,
then import constructors/instance members. The translated System artifact uses its
original native names, whereas this API currently targets dependencies produced with
its format-5 naming contract. An explicit mapping from System's retained origins is
required; do not guess names or treat the synthetic CLI core identity as executable
native identity. Native semantic metadata should eventually replace that bridge.
Runtime Contract configuration remains unchanged. See
[validation](experiments/extended-cli-metadata/imported-type-validation.json).

## Raven external reference signatures (2026-10-01)

Raven's neoCLR adapter now opts into imported public top-level class/interface
signatures through an explicit shared capability. The default .NET portable admission
is unchanged. The adapter resolves symbols against registered dependency snapshots,
checks definition name/arity, and uses the independent metadata API's external type
references. Repeated definitions are cached for the emission. Standard CLI TypeRef,
GENERICINST and TypeSpec shape and native dependency scope are preserved.

Configuration remains CompilationOptions.NeoCLR, a matching projected declaration
core and explicit NeoClrMetadataDependency registrations. This supports signatures,
nullable reference locals/defaults, interface arrays and owned method-generic forwarding
with external constructions. The C# --external-signature-runtime probe produces a Raven
library and consumer; neoCLR verifies both and returns 42. Missing bindings or missing
snapshot types reject without modifying the output. No Runtime Contract change.

This is tested on codex/metadata-consumer with codex/extended-cli-metadata (7f084f4c),
not a published runtime capability. The unchanged collections sample now passes
Register declaration admission and stops at PendingOrder's Option<Order> return
contract; its ordinary CLI control still emits. Imported value/union types, external
constructors/member calls and translated-System origin-to-native identity mapping
remain open. Synthetic CLI core identity is not native implementation identity.
The temporary CLI symbol source must eventually be replaced by native metadata loading.

See [probe evidence](experiments/extended-cli-metadata/external-signature-validation.json) and [unchanged-sample gate](experiments/extended-cli-metadata/collections-after-external-signatures.json).

## Consumer-scoped imported generic calls (2026-10-01)

The next bounded integration removes the primitive-only instantiation gate for imported
unconstrained static generic methods. Arguments are validated in the consuming assembly,
while the definition retains its dependency identity. Caller method and declaring-type
parameters are checked at emission; foreign output types and out-of-scope parameters
reject. This preserves normal .NET MethodSpec/MemberRef semantics and introduces no
metadata format, runtime opcode or Runtime Contract change.

Raven maps these arguments through its existing target type mapper. The C# native probe
now executes consumer-owned Order arguments with alias mutation, caller method/owner
parameter forwarding and an external Box<int> construction. Both emitted binaries verify
and execution returns 42. The metadata C# checks additionally execute the consumer-owned
argument on the CLR. The CLI projection remains the temporary symbol source; explicit
core/dependency configuration is unchanged. These feature-branch results do not advance
the collections Option<Order> gate: imported value/union signatures, generic declaring
type members and translated-System native identities remain open. Native loading of
compiler symbols remains future work.

## Imported nominal method signatures (2026-10-01)

The feature-branch metadata API and Raven adapter now import static methods on
nongeneric owners whose signatures contain dependency-local reference types,
constructions, vectors and unconstrained method parameters. Raven compares the fully
remapped signature against the bound symbol rather than matching module-local tokens.
The metadata API owns bounded decoding, exact snapshot identity and reference validation;
Raven owns explicit dependency selection and overload matching. Ordinary .NET defaults
and Runtime Contract configuration remain unchanged.

The native intent is ordinary cross-assembly calls. CLI declaration input and emitted
TypeRef/MemberRef/MethodSpec are a temporary bridge; no native encoding or opcode change
was required. CompilationOptions.NeoCLR still requires matching core and registered
NeoClrMetadataDependency snapshots. The native implementation dependency must use the
producer's format-5 identity naming; translated System still needs an origin-based map.

The C# --external-signature-runtime probe now emits a factory returning Box<Order>,
forwards it and retrieves a consumer-owned payload while preserving aliases. Interface
vectors, nominal overload matching and missing-method rejection also pass. Both binaries
verify and execution returns 42; 74 metadata contract test groups pass, including CLR
factory/reader execution and malformed signature rejection. The existing generic-library
probe still verifies/runs (42). Tested branches are codex/metadata-consumer and
codex/extended-cli-metadata; evidence pins core/runtime hashes.

The unchanged collections sample still stops at PendingOrder's Option<Order> result,
which is a value-type union in this CLI bridge. Do not admit it as a reference type.
Next work requires value-type category preservation in signatures/native projection,
then appropriate union operations and imported instance/generic-owner members. The
current projection also omits nullable annotations; the factory boundary above uses
nonnull references. Native semantic loading remains future work. Primitive-only reader
recognizers/MemberReference resolution remain narrower than this producer import API.

## Value-type category prerequisite (2026-10-01)

The independent producer on codex/extended-cli-metadata now distinguishes owned value
definitions from reference classes. CLI uses sealed sequential System.ValueType bases
and VALUETYPE/GENERICINST signatures; native output uses its existing non-reference
sealed type contract. C# tests prove default/field-read/array/generic-argument flow on
CLR and direct neoCLR binaries, including value-category preservation in projection.
The runtime format and Raven Runtime Contract configuration are unchanged.

This slice changes no Raven admission rules. The unchanged collections gate remains
Option<Order>, a value-type union in the CLI declaration bridge. Owned value declarations
currently support only primitive fields/static methods. Payload VAR fields, addressed
field mutation, instance members and imported value categories are separate required
steps. The metadata API owns category-preserving encoding/projection; Raven will need
explicit value-type capabilities and union lowering rather than class-based admission.
The future native semantic loader must replace CLI input without carrying forward
these temporary producer limits. [Evidence](experiments/extended-cli-metadata/value-type-validation.json).

### Value payloads and addressed locals (2026-10-01)

The producer now supports declaring-type parameter fields and field loads/stores
through initialized local addresses of the exact value-type owner. Default initialization
remains explicit; partial field stores do not establish definite assignment. CLR and
native tests confirm independent scalar/nested payload copies, array copies and retained
reference payload aliases. The native writer emits stfld followed by pop for addressed
value stores because the existing runtime returns inhabited Void; CLI output uses plain
stfld. Expanded native branch offsets are validated by the executable checks. This is
an explicit temporary lowering owned by the native writer; a future native instruction
contract aligned with CLI's no-result store would remove the extra pop. No runtime
format/opcode or Raven Runtime Contract change was made. Value imports and union
operations remain open; the unchanged collections gate is not claimed complete.


### Metadata definition migration checkpoint (2026-10-01)

On `codex/extended-cli-metadata`, assembly/type/field definitions now support bounded
direct construction; method builders expose shared method declarations. Assembly-level
function definitions can also be attached directly and emitted using body helpers. Raven's
`codex/metadata-consumer` adapter still uses the builder facade and requires no compiler
or Runtime Contract changes. CLI/native encodings and ordinary .NET behavior remain
unchanged. Native intent remains assembly-owned functions and CLI-compatible categories;
this authoring migration introduces no new bridge encoding or information loss.
The existing ownerless function projection remains temporary. Body definitions and
loaded editing are pending, as are the broader collections Option<Order> imports.

Validation: all 76 C# metadata contract groups, manual struct CLR/native execution (42),
and rebuilt Raven external-signature library/application execution (42). Runtime
SHA256 `193B7F995EE4FC92A086439FB1FDBDAFC3A0C58139F0CAF30CB4A2D8640A432D`;
Raven compiler revision `55a29312f`. The metadata library owns graph migration; Raven
owns target mapping, and neoCLR owns native loading/execution.

The next definition slice admits direct static type-method declarations and routes
builder-created type methods through the same collection. The CLI/CIL contract, Runtime
Contract configuration and Raven adapter mapping are unchanged; direct instance method
construction and canonical bodies remain pending. The manual entry/function/type-method
call chain verifies and returns 42 on neoCLR.

Direct instance-method/root-constructor definitions now reuse the existing reference-class
body and encoding paths. The manual object creation/readonly initialization/instance-call
case returns 42 on CLR and neoCLR. Runtime Contract, Raven target admission and the
CLI/native bridge representations are unchanged; this does not close imported union
or instance-member support in the compiler.

Direct nongeneric interface and abstract-method definitions now dispatch through the
existing interface implementation helpers (CLR/neoCLR: 42). This authoring change retains
CLI InterfaceImpl/abstract-method/callvirt and existing native encodings; Runtime Contract
and Raven admission are unchanged. Relationship definitions and canonical bodies remain
pending. The rebuilt Raven external-signature probe continues to verify/run (42).

Authored InterfaceImplementation edges now back interface inheritance/implementation
helpers. Native inherited dispatch returns 42; CLI/native encodings, Runtime Contract
and Raven target admission remain unchanged. Loaded relationship decoding is pending.

Method definitions now own instruction/local/label storage. Existing Raven body helpers
and CLI/native writers use that same storage; body clearing and inherited-dispatch
execution remain valid (42). Runtime Contract, emission admission and encodings are
unchanged. Arbitrary instruction editing and loaded body decoding remain pending.

Property declarations/accessor associations now belong to definitions and are shared
by builder facades. CLR reflection and native execution return 42; Raven’s rebuilt probe
passes without Runtime Contract or admission changes. Definitions stay Cecil-like; the
author clarified that generation builders should follow Reflection.Emit-style patterns
without being a replacement API. CLI/native property encoding remains unchanged.

Direct generic type declarations now share parameter names and special/nominal constraint
storage with builders. Constructed value-constrained calls return 42; Runtime Contract,
Raven admission and CLI/native generic encodings are unchanged. The author specified
builders → definitions → metadata → PE, with reverse reader boundaries. Extracting the
combined metadata/PE writer and editable reader materialization remains future work.


Integration priority reset (2026-10-01): broad metadata architecture work is deferred
behind the unchanged collections case. The fresh probe against metadata e13c8634 and
Raven 80edf8fbe still stops at PendingOrder's Option<Order> return signature, while CLI
control emission succeeds. Runtime Contract and native admission have not changed.
Imported value categories, member/union operations and native System identity mapping
remain open. Future metadata changes should serve those measured blockers.


### Imported value signature integration (2026-10-01)

Raven now opts into external value signatures separately from external reference signatures.
Binding and Runtime Contract configuration are unchanged; the neoCLR adapter maps exact
public external struct identities and validates metadata/symbol category agreement.
Ordinary .NET shared admission remains at its defaults. Supported operations include
imported value return/parameter/local/forwarding via static methods on nongeneric owners.
Imported constructors/instance/generic-owner calls, unions and translated-System identity
mapping remain open. Reference-only admission is not used for value unions.

The temporary format-5 manifest value_type_references annotation preserves the CLI
VALUETYPE category absent from bare Named/Constructed references. The metadata library
writes/projects it; neoCLR validates it against loaded definitions. Older strict runtimes
reject images containing the field; existing images without it remain supported. Native
indexed signature metadata should eventually replace this projection annotation.

Validation: 77 C# metadata groups; CLR/native separate value library/consumer execution
(42) and runtime category-mismatch rejection; a Raven value consumer verifies/runs (42).
The unchanged collections sample now rejects a lowered BoundInvocationExpression instead
of PendingOrder's Option<Order> declaration. No broad compiler/union support is claimed.

## Imported constructed dispatch (development, 2026-10-02)

Raven now opts into external instance-call admission through a separate backend
capability. Public nonvirtual/final class members and nongeneric abstract interface
members can use unconstrained invariant reference owners with type arguments.
The compiler resolves the open method against an explicit dependency snapshot and
checks receiver category, dispatch kind and full parameter/result signature before
constructing a consumer-scoped reference. Missing dependencies still reject without
writing output. Runtime Contract and ordinary .NET defaults are unchanged.

Compared with CLR, the metadata API uses the same VAR substitution, constructed
TypeSpec/MemberRef shape and callvirt null-checking/dispatch semantics. The existing
native generic interface dispatcher executes the resulting binary assemblies; no
new instruction or alternate runtime lookup convention is needed. The supported
producer subset now includes nongeneric root classes implementing closed generic
interfaces. Definitions own the relationship, builders append it, and native readers
preserve it in the CLI projection. This expands the host library's supported subset;
older experimental readers cannot project these relationships.

`--imported-interface-runtime` in Raven proves constructed interface and final class
calls against a metadata-produced library (42). The C# metadata tests independently
execute CLR/native dispatch (42), reject incorrect owner/arity/opcode contracts and
exercise native null-receiver failure. The unchanged collections sample advances
past MutableMap.TryAdd and ArrayList.Add to BoundPropagateExpression; no complete
collections execution is claimed. Owners are the shared callable/body admission
layer, neoCLR adapter, independent metadata library and existing native dispatcher.

The CLI reference projection remains a temporary symbol-loading bridge. Native
symbol import should eventually preserve these contracts directly. Cross-dependency
TypeRefs, imported constructors/value-instance methods, instance generic methods,
extensible class virtual slots and generic interface inheritance remain outside this
slice. They are implementation limits, not neoCLR language rules.

## Concrete case lowering checkpoint (2026-10-02)

The unchanged collections sample's apparent propagation rejection was caused by a
shared lowering exception in a later `Option<Order>(None())` expression. A concrete
case target must construct the case directly, matching the existing .NET fallback
emitter. Shared lowering now honors that target; binding and Runtime Propagation,
Self and Unit configuration remain unchanged. The fix is independently committed
as `dcc77ef5f` on the main-based `codex/compiler-fixes-from-neoclr` branch. All 25
focused imported-case/propagation tests pass on both branches using .NET 11.

The CLI bridge control still emits 7168 bytes. Native emission now reaches the
synthesized uninitialized `out` local and rejects `only initialized value locals`.
Managed local addresses, byref signatures, imported value receivers and protocol
failure paths must be implemented as explicit shared/backend contracts before this
propagation protocol can execute. Do not replace out parameters with unrelated value
calls or silently initialize them to bypass the capability check. Existing CLR/CIL
byref and address semantics remain the baseline. No new metadata encoding or runtime
support is claimed by this checkpoint, and the complete application has not executed.

## Structural experiment branch

The source experiment is `codex/structural-types`, based on the Self-integrated main.
Its Function work is now integrated into `codex/extended-cli-metadata` for direct codegen development.
Its existing structural Function implementation and API are retained, independently
of main's nominal delegates. Function identity/metadata work is still experimental;
Self support is inherited from main. Use this branch's matching generated core,
bridge and native library, not a nominal main bundle.

### Native static Function producer (2026-10-02)

Raven's metadata adapter now explicitly enables shared Function signature/local and
bind/invoke capabilities. A static method-group callback compiles directly to binary
PE and executes to 42; Hello World, arrays, interfaces and ref/out controls also pass.
Compiler binding still uses the matching CLI reference snapshot and unchanged Runtime
Contracts. Func/Action carry signatures in CLI metadata; native code uses structural
Function identity and binding. The compiler owns logical lowering, the independent
metadata API owns encoding, and neoCLR owns invocation. Ordinary .NET emission retains
its existing path. Lambda synthesis, captured receivers, generic/imported binding
and inhabited-Void callbacks remain pending in this producer profile; these are not
permanent native Function restrictions. Runtime integration remains on the feature
branch `codex/extended-cli-metadata`.

The subsequent compiler slice supports noncapturing synchronous lambdas through
internal assembly-function definitions and the shared lowered body planner. The
direct lambda consumer returns 42. Capturing, async, iterator and generic lambda
targets remain producer gaps; .NET closure generation and Runtime Contracts are unchanged.

Static extension calls are admitted through Raven's explicit
`AllowsLoweredExtensionCalls` capability after shared lowering supplies the receiver
as an argument. This changes no binding or Runtime Contract configuration and keeps
ordinary .NET defaults. The unchanged collections sample passes `Single` admission
and next reports unsupported union-pattern emission; it is not yet executable through
the direct backend.

Raven now admits checked union-case/member conditional branches and concrete value
overrides in its native profile. Pattern extraction follows the existing .NET
TryGet/getter behavior and branches before exposing payload bindings. Generated match
failure uses the existing terminal-failure contract; .NET retains its exception body.
Runtime Contracts and binding remain unchanged. 34 focused compiler tests pass,
including imported-case extraction on .NET; direct collections emission next needs
reference conversions. Native execution of that broad sample is still pending.

Raven's native adapter now enables logical reference conversion using metadata
`castclass`. Ordinary .NET retains its existing path. Union-case storage matching
uses exact assembly/physical-name/type-argument identity when semantic carrier views
differ, notably for nongeneric None. Binding and Runtime Contracts are unchanged.
Seven direct native profile controls pass; the unchanged collections sample next
requires iterator-loop lowering. This still does not claim broad sample completion.

The portable reference-iterator helper now consumes compiler-selected GetEnumerator,
MoveNext and Current members through ordinary bound calls, locals and branches. The
native adapter opts in; .NET retains its existing generator and Runtime Contracts.
The unchanged collections application completes body planning and now needs explicit
linkage between the matching CLI reference snapshot and translated native System.


### Generic Function invocation access (2026-10-02 development)

On `codex/extended-cli-metadata`, native Function invocation now follows the existing
nominal generic-owner access rule: substituting a caller-owned internal type does not
revoke a generic library's permission to invoke a supplied callback. As with CLR generic
delegate use, this does not grant access to members of that internal type. Open signature
and operand checks still reject a library explicitly naming a foreign internal type;
target access is checked at binding. The two-module native regression returns 42 and
checks the negative case. The unchanged Raven collections application exposed this at
`System.Linq.Operators.Single<Order>`; no source visibility change was needed.

### Unchanged collections end-to-end acceptance (2026-10-02 development)

The `codex/extended-cli-metadata` and Raven `codex/metadata-consumer` branches now compile
`application-order-collections.rvn` directly to a 69,632-byte PE/#Neo binary assembly,
load it with translated native System, verify and execute it with exit 0 and exact
[expected output](experiments/raven-target/samples/application-order-collections.expected.txt).
The [bundle-hashed evidence](experiments/extended-cli-metadata/collections-end-to-end-2026-10-02.json)
records source, declaration-reference, implementation and runtime hashes. This supersedes
the earlier collection blockers without claiming complete backend/class-library support.

Configuration uses `CompilationOptions.NeoCLR`, the explicit CoreProbe Self marker,
and a `NeoClrMetadataDependency(reference, definition, coreLibrary, nativeImplementation)`.
The optional `NativeImplementation` property carries the matching translated binary
inventory. Raven registers it with the independent metadata API before imports; the API
validates selected declaration contracts and owns the CLI/native identity mapping.
Default .NET compilation and dependencies without this option retain their paths.
The output application body is generated directly by the metadata backend, not translated
from CLI or executed as JSON. The existing library translation remains a temporary
implementation dependency. CLI input still supplies symbols; native semantic importing,
native emission of the complete runtime library, closures and broader Function emission
remain future work. Bridge losses include unsupported semantic metadata beyond the
selected CLI shapes; the binding is not an arbitrary native library symbol loader.

Compared with CLR's single declaration/implementation assembly identity, this bridge
explicitly pairs the CLI snapshot with a native implementation module. It allows testing
native application codegen against the existing library, at the cost of matched-bundle
maintenance and a restricted signature/name convention. The eventual native metadata
loader/backend should consume one native declaration/implementation contract directly.
The metadata library owns that replacement; Raven owns target capabilities and symbol
mapping; the runtime owns admission, access, specialization and execution.

Run Raven's `NeoClrMetadataProbe --readiness-linked-sample <neo-root> <fresh-output>
<runtime> application-order-collections <matching-System.neox>`. The selected sample's
`.expected.txt` is now an assertion: emission, verification or execution failure, nonzero
exit, stderr or different stdout fails the command after saving diagnostics. The System
binary used here is assembled from the matching translated `System.neoil` with
`neoclr assemble <System.neoil> <System.neox> --format neox`.

Validation: 91 C# metadata contract groups; a C# CLI/native linkage consumer (42); six
runtime metadata-container tests including the C# fixture and malformed scopes; 44
Function/object tests (including the reproduced internal-type regression); matching API
snapshot check; and the [seven native profile controls](experiments/extended-cli-metadata/native-controls-after-linkage-2026-10-02.json). No website build or performance claim.

### Collection contract source compilation (2026-10-02 development)

The next bounded source-emission slice compiles the unchanged Disposable, Iterator<T>,
Iterable<T> and Collection<T> sources with an executable same-assembly consumer.
Collection's `Iterable<T>` base preserves its owner argument through metadata and the
PE projection. A nongeneric consumer interface closes the hierarchy with Int32; a class
implements it and inherited Count dispatch returns 42. Both source orders verify and
execute on neoCLR with both the host bootstrap and real neoCLR target configuration.
The host-profile CLR controls also return 42. [Evidence](experiments/extended-cli-metadata/collection-contracts-2026-10-02.json).

The shared plan now exposes constructed interface inheritance through an explicit
capability enabled by the CLR and native adapters. The metadata API owns positional
substitution and cycle/implementation validation. Native emission uses the existing
constructed callvirt/InterfaceImpl-equivalent contract; no runtime changes are required.
Configuration remains CompilationOptions.NeoCLR and the matching CoreProbe/Self marker
for the target run. The source declarations and consumer are in one assembly; this is
not yet a separately bootstrapped class library. The CLI snapshot still supplies core
symbols. No library implementation is substituted by a stub or rewritten for this probe.

Relative to CLR, the emitted relationships use the same invariant generic interface
model and CLI InterfaceImpl TypeSpecs. Supported class implementations remain nongeneric,
without variance, default interface bodies or explicit MethodImpl entries. The benefit
is reusing source/semantic contracts and existing dispatch; the cost is a bounded producer
subset and transitive validation work. Native symbol importing remains the eventual
replacement for the CLI reference bridge, not a prerequisite for this emission slice.

Run Raven's `NeoClrMetadataProbe --collection-contract-runtime <neo-root> <fresh-output>
<runtime>`. Validation: eight shared-interface C# tests, 92 metadata groups, a C#-produced
three-level generic dispatch fixture on CLR/native, the unchanged-source probe, and the
collections application still matching its established output. Next: Sequence<T>'s
interface indexer; ArrayList source also needs the implementation seed's RuntimeServices/
CheckedStorage declarations. A fresh inventory's whole-library diagnostics still reflect
that unsupported bootstrap, not a count of independent compiler defects.


### Sequence interface indexers (2026-10-02, development)

Raven's `InterfaceIndexer` capability admits supported public bodyless instance indexers
in the shared plan. CLR and native profiles opt in. Native codegen reuses the existing
metadata property/accessor definitions and constructed interface calls; the property
signature retains index parameters. This follows CLI behavior without a format or
runtime instruction change. Native declaration mapping belongs to Raven, encoding to
the metadata library, and execution to neoCLR.

The unchanged Sequence<T> source joins Disposable, Iterator<T>, Iterable<T> and
Collection<T>. A concrete same-assembly consumer calls inherited Count (40) and indexer
(2), returning 42. Both source orders run on native with host bootstrap and
CompilationOptions.NeoCLR plus the matching CoreProbe Self contract; host-profile CLR
controls also return 42. Projection checks preserve the getter-only property association.
[Source and bundle hashes](experiments/extended-cli-metadata/sequence-contracts-2026-10-02.json).

Two general Raven defects were isolated in commit d360b964f: source member/interface
views could cache incomplete cross-file declarations, and bodyless interface indexers
did not infer abstract accessors. Independent .NET source-order tests exercise these
fixes; completed declaration caches remain in use. No performance claim is made.
Target admission and the expanded C# probe are in Raven commit 67787fa13 on
`codex/metadata-consumer`; runtime/metadata remain on `codex/extended-cli-metadata`.
29 focused interface declaration/completion/symbol tests pass, including shared
getter/setter mutation and capability rejection. The unchanged broad collections sample
still verifies and executes with exact expected stdout, exit 0 and empty stderr
([regression evidence](experiments/extended-cli-metadata/collections-after-sequence-2026-10-02.json)).

This advances source emission, not full System bootstrap: implementing classes remain
nongeneric in the native producer, and native semantic importing remains future work.
Core symbols still come from the matching CLI snapshot; the broad sample still binds
translated System. The eventual replacement loads native declarations directly and
compiles System implementation sources. Next is the implementation seed needed for
ArrayList's RuntimeServices/CheckedStorage dependencies. No public metadata API changed
in this slice; its existing property/accessor and generic interface contracts suffice.


### Generic implementation source checkpoint (2026-10-02, development)

The metadata producer now admits root-class implementations of owned interfaces with
owner type parameters. Required methods and receiver conformance substitute the actual
arguments through inherited contracts. This follows CLR InterfaceImpl/TypeSpec and
callvirt behavior, reusing existing neoCLR runtime dispatch; there is no new instruction
or encoded category. Static/value implementation owners, variance, MethodImpl and default
interface bodies remain unsupported. The [host API reference](../api-docs/experimental-metadata.md#generic-class-interface-implementations-development-2026-10-02)
documents builders, definitions, errors and the remaining loaded-CLI reader limitation.

Raven commit `0406a6d4d` on `codex/metadata-consumer` exposes
`AllowsConstructedInterfaceImplementations` as an opt-in shared capability;
CLR/native profiles enable it. Its adapter resolves constructed interfaces through their
original definitions and preserves owner arguments. Open root classes use the existing
nonsealed metadata shape. Unchanged Sequence hierarchy sources run with generic provider
and iterator classes: constructors store T, and inherited Count/indexer/iterator dispatch
returns 42 on CLR and neoCLR in both source orders. The native target uses
CompilationOptions.NeoCLR and the matched CoreProbe Self contract. [Source and bundle evidence](experiments/extended-cli-metadata/generic-collection-contracts-2026-10-02.json).

The C# metadata fixture independently executes transitive generic dispatch and rejects
mismatched implementations, incompatible constructed receivers and invalid argument scopes
([evidence](experiments/extended-cli-metadata/generic-interface-implementation-2026-10-02.json)).
All 93 metadata groups and 32 focused Raven interface tests pass. The existing broad application still returns its exact
expected stdout, exit 0 and empty stderr
([regression evidence](experiments/extended-cli-metadata/collections-after-generic-implementation-2026-10-02.json)).

The bridge already has an authoring seed distinct from the consumer core. Generate it
with `raven-target/Probe --reference-library-core <seed.dll>`, then run Raven's
`NeoClrMetadataProbe --library-source <neo-root> <fresh-output> <seed.dll> <System.neox>`.
This binds unchanged ArrayList plus its interface hierarchy with zero binding errors;
the nonexecuted CLI control emits 6144 bytes. Imported Option declarations bind explicitly
to translated System. Native output remains empty with a CheckedStorage.Reserve<T>
diagnostic ([complete report and hashes](experiments/extended-cli-metadata/array-list-authoring-seed-2026-10-02.json)).
The reference bodies are never executed. Consumer references continue to omit these
implementation-only RuntimeServices/CheckedStorage declarations.

Next is a native producer mapping for checked uninitialized reservation. The existing
translator uses array.reserve; ordinary CLR newarr initializes every element and cannot
replace it without changing read-before-write behavior. Raven owns target recognition,
the metadata library must preserve the operation, and neoCLR already owns its checked
execution semantics. This boundary is not evidence that ArrayList or all System sources
now execute through the native producer. Native semantic importing remains future work;
CLI authoring declarations and translated System still supply the temporary bootstrap.

### Native checked reservation (2026-10-02, development)

Raven commit `f1b85fa14` on `codex/metadata-consumer` adds optional
`NeoClrEmitOptions.BootstrapReference`, which binds the exact registered
implementation core reference for CheckedStorage.Reserve<T>. Null disables mapping.
The native adapter validates the static owner's public generic method signature and
preserves actual element arguments, including caller generic parameters. Shared body
lowering still emits a normal call; target recognition stays in the native adapter.
Ordinary .NET emission is unchanged, and consumer references still omit the intrinsic.
The experimental constructor adds an optional argument; source calls remain valid, but
compiled host consumers must rebuild against the matching compiler package.

The metadata API now provides `MethodBuilder.ReserveArray(SignatureType)` and
`Emit(OpCode.ReserveArray, SignatureType)`. Native writing retains existing array.reserve
through PE/#Neo; executable CLI writing rejects it because newarr's default initialization
is not equivalent. Declaration-only CLI projection remains available. The
[API contract](../api-docs/experimental-metadata.md#checked-uninitialized-reservation-development-2026-10-02)
and [existing .NET comparison](reserved-array-capacity.md) describe errors, read checks,
tracked-state cost and lack of a performance claim. Runtime encoding and behavior are
unchanged; this closes a producer gap.

Validation: all 94 metadata groups pass; C# metadata and Raven-generated generic helpers
return 42 from a written slot and fault on an unread slot. Raven tests reject default
intrinsic mapping and an unregistered reference without output. Reproduce with metadata
`--reserved-array-runtime <runtime> <output>` and Raven
`--reserved-storage-runtime <seed.dll> <fresh-output> <runtime>`. The seed is generated
by the existing raven-target `--reference-library-core` mode. Runtime configuration is
CompilationOptions.NeoCLR with the matching CoreProbe Self marker. See
[metadata evidence](experiments/extended-cli-metadata/reserved-array-metadata-2026-10-02.json)
and [Raven evidence](experiments/extended-cli-metadata/reserved-array-raven-2026-10-02.json).

The unchanged ArrayList source now passes Reserve and stops at importing System.Fail
from the CLI namespace container; it still does not emit or execute as a complete native
implementation ([report](experiments/extended-cli-metadata/array-list-after-reservation-2026-10-02.json)).
The broader collections application remains executable with exact output
([regression evidence](experiments/extended-cli-metadata/collections-after-reservation-2026-10-02.json)).
Next is namespace-function dependency mapping. The CLI seed and translated System remain
explicit temporary dependencies; native metadata importing remains their eventual
replacement. Compiler support on its feature branch does not imply runtime publication
or a main-branch merge.

## ArrayList source execution (2026-10-02 development)

The unchanged ArrayList implementation and its source interface hierarchy now compile
to native PE/#Neo and execute on neoCLR. The consumer exercises growth, indexed writes,
copy independence, iteration, FindAll, Exists, TrueForAll, Find/FindLast and their index
variants, including absent Option results. Negative capacity and invalid indexing reach
the expected System.Fail faults. The broad collections application retains exact output.

Native namespace functions are temporarily imported through public abstract sealed CLI
containers marked by the exact configured core's parameterless TopLevelAttribute. The
metadata binding validates the public static method signature and maps namespace/name
to an ownerless native function. CLI writing retains the original container MemberRef.
Unmarked containers, wrong core scopes and mismatched signatures are rejected. Raw CLI
global-function imports and general custom-attribute editing are outside this slice.
Native semantic importing will replace this scoped CLI marker inspection.

Raven recognizes Length on the explicitly configured RuntimeIterationContract array
shape; ordinary .NET System.Array lowering remains supported. A separately isolated
shared fix preserves BoundRequiredResultExpression values, needed by match-as-value.
The library-authoring seed and BootstrapReference remain explicit; translated System
still supplies dependencies such as Option and Fail. No seed stubs execute.

Validation: 95 C# metadata groups, focused compiler array/default checks and native
source consumers. [Source hashes, consumers and runtime results](experiments/extended-cli-metadata/array-list-source-2026-10-02/execution.json)
identify the tested feature-branch bundle. Full class-library compilation and native
symbol loading remain future work; next assess comparer implementations and HashMap.

### Callback comparer source checkpoint (2026-10-02)

Unchanged Comparer, EqualityComparer, FunctionComparer and FunctionEqualityComparer
compile, verify and execute through native PE (42). Raven's explicit field declaration
path now uses its existing target capability profile when admitting/mapping types.
Generic stored callbacks use structural Function metadata and existing runtime storage
and interface dispatch; the CLI representation remains core Func/Action signatures.
This requires no runtime or metadata encoding change. Noncapturing callbacks are tested;
capturing closures remain unsupported. The explicit implementation seed and translated
System dependency are still required. Native importing will replace the CLI signatures.
[Consumer/source hashes](experiments/extended-cli-metadata/comparer-source-2026-10-02.json)
and [runtime results](experiments/extended-cli-metadata/comparer-execution-2026-10-02.json).

### HashMap source checkpoint (2026-10-02)

Thirteen unchanged library source units, including HashMap, ArrayList, source interfaces
and FunctionEqualityComparer, now compile together and execute native PE (42). The test
forces all keys into one hash bucket, grows beyond initial capacity, rejects duplicates,
updates/inserts through MutableMap, checks independent Keys snapshots and absent/present
Option results through inherited Map dispatch. The policy-constructor case verifies
equivalent distinct keys using callback equality and hashing.

Raven's shared interface planner now propagates the selected target capabilities to
method return/parameter and property types, admitting the explicitly supported imported
Option<V> signature. This isolated general correction leaves ordinary .NET shared
capabilities unchanged; CLR imported reference/value interface contracts are independently
checked. The bridge still uses CLI seed signatures and translated System dependencies;
native semantic importing will replace that representation. No metadata/runtime changes
are needed for this slice. Capturing closures, full System source compilation and native
symbol importing remain open.

[Source and consumer evidence](experiments/extended-cli-metadata/hashmap-source-2026-10-02.json)
and [native execution](experiments/extended-cli-metadata/hashmap-execution-2026-10-02.json)
include source, seed, System and runtime hashes. All 14 focused Raven interface tests
pass, and the ArrayList source success/failure consumers still pass after this change.
This is feature-branch evidence only.

### Source collections with reference payloads (2026-10-02)

The HashMap source runtime probe now also uses internal Order objects. Growth, map
lookup, filtering, shared mutation visibility, replacement independence and iterator reads
verify and execute (42), alongside the numeric collision/policy consumer. This reuses
existing generic reference storage, Function and interface-dispatch support without
compiler, metadata or runtime encoding changes.

The unchanged broad application was separately assessed with source collections and
translated queries. Single cannot bind because the translated extension expects its
own nominal Iterable identity. Including unchanged Operators.rvn resolves that lookup,
but both its iteration and the application hit RAVT001: the configured iteration assembly
is the seed, while source Iterable/Iterator now shadow those metadata names. Array shape
still belongs to the seed. A coherent source bootstrap core/iteration contract is next;
do not treat these distinct nominal types as interchangeable. Source query declaration
emission and the remainder of the file are not yet validated. Native semantic importing
remains deferred, and the previously working translated-System application is unaffected.

[Reference consumer](experiments/extended-cli-metadata/source-reference-2026-10-02/reference-identity.json),
[execution](experiments/extended-cli-metadata/source-reference-2026-10-02/execution.json),
[translated query assessment](experiments/extended-cli-metadata/source-reference-2026-10-02/translated-queries.json)
and [source query assessment](experiments/extended-cli-metadata/source-reference-2026-10-02/source-queries.json)
record consumers, diagnostics and matching source/bundle hashes.

### Native importer priority (2026-10-02)

The author now directs native semantic importing before further CLI bootstrap expansion.
The [existing metadata architecture and implementation alignment](design/extended-cli-metadata.md#direct-native-semantic-import-implementation-alignment-2026-10-02)
govern this work. Materialize native declarations into the shared definition model,
then adapt Raven's semantic loader and preserve native identities into emission.
Projection APIs remain existing compatibility/test tools; no new projection constitutes
native import. Current runtime contract configuration and shipped behavior are unchanged.
The importer and native System contract resolution remain unimplemented.

### Direct native reader foundation (2026-10-02)

The metadata library now reads primitive nongeneric namespace functions from native
PE/#Neo directly into shared definitions, including scoped references and entry identity.
The compiler can inspect these via TryGetSignature without a CLI projection or reflection.
Existing exact dependency resolution accepts the snapshots. Both container encodings
and compatibility tests pass (96 groups); loaded bodies remain opaque and unsupported
nominal/generic declarations fail explicitly. Raven still uses the existing CLI importer:
connecting native definitions to compiler symbols and preserving their identity into
codegen is next. Existing Runtime Contract settings and .NET behavior are unchanged.

### First direct native semantic imports (2026-10-02)

Raven's explicit NeoClrMetadataReference.ReadAssembly now consumes the existing native
function definition reader directly. Native namespace/method symbols participate in
GetSymbolInfo/GetTypeInfo, overload resolution and accessibility without reflection
objects or nominal container types. An explicit CLI primitive core remains required;
Runtime Contract configuration is unchanged and full native System import is pending.

The compiler owns symbols per compilation, with provider-owned exact assembly identity
for equality and dependency matching. Native dependencies must be registered explicitly;
missing/version-mismatched, duplicate and wrong-target configurations produce RAVT003.
No filesystem probing, package resolution or reflection API is introduced. Positional
parameters have implicit ordinal display names because this profile lacks source names.
Direct dependencies are tested; full cyclic dependency qualification remains future work.

Default CLI emission rejects native semantic references; native calls currently reach
the missing adapter diagnostic, both without output. Next connect native definition
identity to the metadata emitter and run a cross-assembly call. The 67 focused .NET
target and symbol-equality regressions pass. The C# semantic probe checks both reference
orders, namespace overloads, semantic types, stable lookup, compilation isolation,
visibility, invalid arguments, exact version identity and dependency rejection.
[Recorded semantic evidence](experiments/extended-cli-metadata/native-symbols-2026-10-02.json).
This is symbol-loading evidence, not a new runtime execution claim.

### Direct native call execution checkpoint (2026-10-02)

The first native semantic provider now feeds actual cross-assembly emission. Raven
imports the exact MethodDefinition already owned by the bound native symbol through
AssemblyBuilder.ImportReference. The host supplies the registered native reference,
its exact Definition and matching explicit core in NeoClrMetadataDependency; mismatched
snapshots or translated implementation mappings are rejected without output.

Both an API-authored native overload library and a Raven-authored namespace-function
library are read directly, consumed by Raven and executed in neoCLR, returning
Int32(42). The primitive core remains the explicit CLI bootstrap. This follows the
shared definitions/reader/writer direction; neither a reflection facade nor a native
library-to-CLI symbol projection is involved. Compared with the CLI import path, native
snapshot consistency uses a cached SHA-256 of the owned image because native manifests
have no MVID. This conservatively rejects byte-different snapshots under one identity,
including equivalent containers encoded differently; it is not a persistent assembly ID.

C# metadata checks cover repeated identical-snapshot imports, namespace identity,
conflicting snapshots and incompatible core contracts: 96 metadata contract groups
and 22 Raven target-emission tests pass; the API snapshot check passes. See
[native call runtime evidence](experiments/extended-cli-metadata/native-calls-2026-10-02.json).
Broader nominal/generic importing, native System symbols and source bootstrap remain
open. Existing CLI loading/emission continues through its current provider/backend.

### Direct native static-type consumer (2026-10-02)

The shared definition reader now materializes fieldless nongeneric top-level static
classes and their primitive methods, retaining native type origin tokens, namespace,
visibility and canonical method ownership. Raven supplies compilation-owned nominal
symbols, normal type lookup and primitive overload resolution. The emitter imports the
bound native method definition through the same route as namespace functions; there is
no synthetic CLI dependency or reflection representation.

The C# native-symbol runtime probe adds a Raven-produced static class library and a
consumer using both Boolean and Int32 overloads. The explicit CLI primitive core and
explicit matching native emission bindings remain required. Instance types, nominal
signatures, fields, properties, nested declarations and generics remain unsupported by
this direct-reader profile, even where existing writer/runtime paths support them.
This is the first nominal ownership slice, not full class-library metadata loading.

Validation: three native consumers return 42; all 96 metadata contract groups and
39 Raven accessibility tests pass. Evidence is recorded in
[native static-type runtime results](experiments/extended-cli-metadata/native-static-types-2026-10-02.json).
A .NET-only regression reproduced an identifier-expression access-check omission;
Raven's general binder fix is isolated in e3afed13c for independent integration.
No merge to main is claimed.

### Direct native instance-class checkpoint (2026-10-02)

The direct reader now admits fieldless nongeneric top-level instance classes alongside
static types/functions. It preserves class flags, instance receivers and constructor
attributes in the existing definitions; Raven uses these for named-type/constructor
symbols and the existing imported allocation/call emit path. The Raven class consumer
constructs Calculator, stores an alias and invokes its primitive Add method, returning
42 in neoCLR. Private constructor and instance method calls require RAV0500 diagnostics.
Static signature helpers now reject loaded native instance methods; TryGetSignature
continues to report their primitive explicit parameter/result contract.

This matches the CLI ownership/constructor/receiver model for the admitted subset.
The explicit CLI primitive core remains. No new metadata encoding or runtime bridge is
introduced. Fields/properties, interfaces, value/nested/generic types and nominal
parameter/result signatures are pending in the direct reader. Added exploratory
reference-equality/inequality expressions hit the portable lowerer's BoundBinaryExpression
limit (NEOMETA001); that separate codegen gap remains open and is not a passing sample.

Validation: all 96 metadata contract groups and three runtime consumers pass; the API
snapshot check passes. [Instance-class evidence](experiments/extended-cli-metadata/native-instance-types-2026-10-02.json).

### Direct native primitive storage checkpoint (2026-10-02)

The native read model now materializes primitive instance fields into the existing
FieldDefinition collection, preserving canonical owner/token identity, visibility and
readonly flags. The compiler adds native field symbols with lazy core primitive mapping.
No CLI blob or reflection field is fabricated: GetSignature rejects native fields and
TryGetPrimitiveType supplies the supported logical type. Authored FieldType remains a
separate existing contract until broader loaded signature materialization is implemented.

The Raven library's Calculator constructor writes its private Int32 storage; Add reads
it after the consumer constructs Calculator(20), stores a local alias and calls Add(22).
The native runtime returns 42. Public field lookup/type binding and private field
access diagnostics are also checked. Direct imported field emission still reports
NEOMETA001 without output because the external field operand adapter is not implemented.
This is distinct from the working cross-assembly method calls into stateful objects.

Compared with CLI field metadata, names, owner/origin tokens and FieldAttributes retain
the same model; only raw CLI signature access is unavailable for native input. Nominal
field/method signatures, properties and wider type categories remain pending. Native
format and runtime code are unchanged; the explicit CLI primitive core remains required.

Validation: all 96 metadata contract groups, three runtime consumers and the API snapshot
check pass. [Primitive-field evidence](experiments/extended-cli-metadata/native-primitive-fields-2026-10-02.json).

### Direct native field access checkpoint (2026-10-02)

The imported field operand gap is closed for public primitive instance fields on public
nongeneric top-level reference classes. AssemblyBuilder imports an immutable typed
reference from the exact definition snapshot. The CLI writer emits a field MemberRef;
the native writer uses the validated native field ordinal. CLI snapshots cannot supply
native ordinals and reject native writing. No translated-layout guesses or new native
opcode encoding are introduced. Readonly stores, wrong receivers and foreign output
ownership reject before an assembly is returned.

Raven resolves the bound native field through its explicit dependency binding. Its
consumer writes Visible through an alias, reads through the original reference and
calls the stateful method (42). NativeFieldConsumer also performs a direct constructor/
field-load round trip (42). A private field preceding Visible tests ordinal preservation.
The explicit CLI primitive core remains; nominal signatures, generic/value owners,
translated field layouts and reference-comparison lowering remain pending.

Validation: 97 metadata contract groups (including actual CLR execution), four neoCLR
consumers and the API snapshot check pass. [Field-operand evidence](experiments/extended-cli-metadata/native-field-operands-2026-10-02.json).

### Direct native local class signatures (2026-10-02 development)

Native function/method/constructor signatures can now refer to another supported class
in the same native assembly. The metadata library owns immutable nominal references;
Raven maps them to the same compilation-owned types returned by namespace/type lookup
and imports output operands through the metadata API. The existing CLI primitive core
and explicit native dependency/core bindings remain required; Runtime Contract settings
are unchanged. No CLI projection of this dependency is generated, and no shared binder
or .NET loading/emission behavior changes.

The Raven source library exposes a factory, namespace/static/instance identity calls and
a constructor accepting a class. Its native consumer preserves aliases and returns 42;
all four runtime consumers pass. C# metadata checks pass 98 groups, including CLR execution
of the corresponding imported nominal signatures. Ordinary .NET regression evidence from
the preceding shared-layer slices is reused, not claimed as rerun here.

Local nongeneric root classes only: native signature dependencies on other assemblies,
nominal fields, generic/interface/value types and full System import remain pending.
The metadata library owns future native shape/resolution support; Raven's target provider
owns symbol mapping. Existing translated System runtime input and CLI core bootstrap are
still temporary and require native core/reference loading and full source emission to
replace them.

### Direct native local class fields (2026-10-02 development)

Native fields may now refer to another supported class in the same module, including
forward/cyclic declaration references. The metadata reader publishes immutable nominal
field signatures; Raven lazily maps them to canonical native type symbols and the
existing field import contract creates output-owned operands. A source-library consumer
replaces a stored object, mutates the replacement and proves original-object independence
in neoCLR (42). This follows ordinary CLR nominal field/alias behavior; PE/#Neo encoding
and runtime instructions are unchanged.

99 C# metadata groups pass, including .NET execution from native snapshot imports,
wrong-type and readonly-store rejection. All four native runtime consumers return 42.
No shared binder or .NET provider changes were made. Existing CLI primitive core and
explicit dependency bindings remain required; Runtime Contract configuration is unchanged.
No CLI projection is used for these native library symbols. External signature types,
generic/value/interface/array field profiles and full System import remain pending.
The independent metadata library owns those reader/import extensions; Raven owns symbol
mapping. Native core loading/source emission remain the replacement for bootstrap inputs.

### Explicit native signature dependencies (2026-10-02 development)

Native method/constructor/field signatures now refer to supported classes in explicitly
supplied native dependencies. The metadata library retains immutable assembly-scoped
TypeReferences and resolves them through IAssemblyResolver; Raven maps the resolved
definition to its canonical compilation-owned symbol. Emission supplies the exact
registered dependency snapshots to the new resolver-taking import overloads. Missing,
wrong-version or missing-type dependencies diagnose; no CLI projection or reflection
loading is used for these native references. Runtime Contract settings are unchanged.

A Raven-produced payload library, holder library and consumer now compile and execute
as three native assemblies (42), including constructor parameters, nominal method
results/arguments and class-valued field replacement. All five runtime consumers and
100 metadata C# groups pass. Equivalent imported signatures execute on .NET too.
Existing .NET provider behavior and CLI cross-dependency decoding remain unchanged.
No PE/#Neo schema or runtime instruction change is required.

The CLI primitive core and translated System remain explicit bootstrap inputs. Generic,
value/interface/array signatures, type forwarding and full System native import remain
open; the independent metadata library owns reader/import support and Raven owns symbol
mapping. Native core loading and source compilation remain their bootstrap replacement.

### Direct native array signatures (2026-10-02 development)

Native field/method/constructor signatures now admit one-dimensional zero-based arrays
of supported primitives and local or explicitly resolved external classes. The metadata
library owns immutable element shapes and recursive output import; Raven shares cached
signature-to-symbol mapping between methods and fields. Array symbols preserve canonical
element identity. Existing explicit dependency/core bindings and Runtime Contract settings
are unchanged. No CLI projection is used to load these native library symbols.

The Raven-produced payload/holder/consumer case now stores an external-class array in a
field, replaces an element through an alias, and passes a primitive array across libraries;
neoCLR returns 42. All five runtime consumers and 101 metadata C# groups pass, including
.NET execution of equivalent imported vectors and wrong-element-type rejection.
No shared binder or .NET provider change is made, and the format/opcodes are unchanged.

Jagged/multidimensional arrays, covariance, generic/value/interface elements, broader
properties and full native System import remain outside this slice. The explicit CLI
primitive core and translated System bootstrap still require native core loading and
source compilation for their eventual replacement.

### Direct native non-indexed properties (2026-10-02 development)

Native properties now load into canonical metadata definitions and Raven property
symbols, with associated getter/setter methods and lazily resolved primitive, class
and vector signatures. Static/read-only/write-only accessors retain visibility and
identity. Emission imports the existing method operands; no property format or opcode
changes are required. Property signature admission uses explicit NeoCLR capabilities;
shared static property lowering allows external owners through the external-reference
capability. A separate general binder fix rejects inaccessible setter writes, proved
with an ordinary C#/.NET fixture (isolated Raven commit 23161cffb; 76 focused
property/accessibility tests pass). Runtime Contract configuration is unchanged.

The Raven payload/holder/consumer case reads/writes native nominal and array properties,
reads a static property, preserves aliases and returns 42 in neoCLR. All five native
consumers and 102 C# metadata groups pass. Read-only/private setter writes diagnose.
The native dependencies are not projected to CLI. The primitive CLI core and translated
System remain explicit bootstrap inputs; the metadata library owns broader reader
support, Raven owns symbol mapping, and native core loading/source compilation remain
their eventual replacement. Indexers, generic/value/interface owners and full native
System loading remain open.

### Direct native indexed properties (2026-10-02 development)

The independent metadata reader now retains indexed-property signatures, and Raven
imports them with canonical getter/setter symbols and cached index parameters. Source
indexers use explicit NeoCLR signature capabilities; emission imports the existing
accessor method operands. No metadata schema, opcode or Runtime Contract changes are
needed. Libraries are read directly from native definitions without CLI projection.

The payload/holder/consumer case now replaces and reads an external-class array element
through an imported indexer. An overloaded String indexer reads the nominal property;
private-setter and wrong-index-type assignments diagnose. All five native consumers
return 42. The CLI primitive core and translated System remain explicit bootstrap
inputs. Setter-only indexers can be inspected but source access remains a binder gap;
full native System importing and broader owner categories remain pending.

Validation: 102 metadata groups and 75 focused .NET indexer/accessibility tests pass.
General Raven fixes are isolated as 9ee5aad97 (indexer access) and 2df6f3f6d
(qualified type-name crash in shared emission). Neither is merged into main here.
See [hashed evidence](experiments/extended-cli-metadata/native-indexers-2026-10-02.json).

### Setter-only native indexers (2026-10-02 development)

Raven now assigns through setter-only indexers loaded directly from native definitions.
The shared binder uses the property parameter contract, already provided by the native
reader, and emits the existing setter call. No metadata schema, runtime instruction,
Runtime Contract or bootstrap configuration changes are required. Reading a setter-only
indexer and compound assignment remain diagnostics because a getter is required.

The payload/holder/consumer case includes a Boolean setter-only indexer with a nominal
external value type, verifies its parameter/accessor identity in both reference orders,
and replaces an object through it before reading via a separate indexer. All five native
consumers return 42. The independent shared compiler correction is Raven e07477274,
with 79 passing .NET indexer/accessibility tests. The unchanged metadata library retains
its prior 102-group evidence. CLI primitive core and translated System remain explicit
bootstrap inputs; native interfaces/generics/value owners and full System import remain
pending. No native library is projected to CLI for symbol loading.

### Direct native interface import (2026-10-02 development)

Native nongeneric interfaces, local inheritance and root-class implementations now
load into canonical definitions and Raven symbols. Interface method abstract/virtual
flags and inherited properties drive the existing callvirt emission path. The metadata
importer validates reference-to-interface conversions against exact native relationships;
no CLI projection, runtime opcode or Runtime Contract change is involved.

A native library exposes two implementations through factories returning a derived
interface. Its separately compiled consumer invokes inherited method/property contracts
and neoCLR returns 42. All six native consumers and 103 metadata groups pass. Local
relationships are supported; external implementation edges, generic/value owners and
full System loading remain pending. The CLI primitive core and translated System remain
explicit bootstrap inputs, to be replaced by native core loading/source compilation.

### External interface storage validation (2026-10-02 development)

The native interface case now spans a contract/implementation library, a storage library
and its consumer. The storage library imports interface-valued fields, arrays and
constructor arguments directly from native metadata; its method dispatches through a
stored interface. The consumer replaces values through a shared array and confirms
original-reference independence (42). Unrelated native classes do not convert to the
interface. Both reference orders preserve canonical external interface symbols.

All six runtime consumers and 103 C# metadata groups pass, including CLR field/array
store execution using imports from native definitions. This is additional validation of
existing shared paths: no new encoding, runtime opcode, compiler policy or Runtime
Contract change. CLI core/translated System bootstrap dependencies remain explicit.
Generic/value owners and external implementation edges remain pending.

Resolved shared diagnostic gap (2026-10-02): incompatible expression-bodied returns
were independently reproduced against .NET references. Raven now binds the complete
arrow body during diagnostics, reusing existing return conversion validation. The 106
focused .NET tests pass; the native probe requires RAV1503 before emission and an empty
output on rejection. All six native consumers still execute with result 42. No Runtime
Contract, encoding or runtime changes were required. Generic/value import and the CLI
core/translated System bootstrap limitations remain open.

### Native generic definition groundwork (2026-10-02)

The metadata library now reads and imports unconstrained static generic functions and
methods on nongeneric owners directly. C# consumers execute imported generic identity
on CLR and native static/namespace generic calls on neoCLR (42, both containers).
An initial Raven rejection boundary kept the profile explicit during the library
expansion. Raven now owns native method type parameters and reuses shared constructed
methods, inference and generic call emission. Its seventh native consumer builds the
library from Raven, imports it directly, and executes namespace/static generic calls,
forwarding, overloads, vector mutation and reference aliases (42). C# checks cover
parameter/vector identity, compilation isolation, both reference orders and invalid
arguments. All seven runtime consumers pass.
No Runtime Contract or temporary CLI encoding changes. Generic owners, constraints,
instance methods and complete native System import remain subsequent work.

### Native generic root class import (2026-10-02 development)

Raven now reads unconstrained generic root classes directly from native definitions.
Owner-scoped parameter names, fields, method signatures and properties map into shared
constructed-type/member substitution. The generic library consumer constructs Box<int>
and Box<Item>, invokes constructors/methods/getters and preserves object/vector aliases
(42). All seven runtime consumers and 105 C# metadata groups pass; standalone metadata
consumers execute imported Box<int> on CLR and neoCLR in both native schemas.

No Runtime Contract, metadata encoding or runtime changes were required. Constructed
nominal signatures, constraints, generic interface inheritance and direct imported
constructed-field emission remain pending. CLI core/translated System bootstrap remains.

### Local closed constructed signatures (2026-10-02 development)

Direct native signatures now retain local Box<int>-style constructions in an immutable
ReferencedGenericType model. Raven resolves these through its existing module signature
cache and shared type construction. CreateBox/EchoBox native namespace functions now
carry constructed parameters/results across the library boundary; all seven consumers
execute (42). C# metadata checks and CLR/native factory consumers pass (105 groups).
No Runtime Contract, format or runtime change. Open/external generic constructions and
constraints remain unsupported, and the CLI core/translated System bootstrap remains.

### Open local constructed signatures (2026-10-02 development)

Raven now consumes Box<T>-style native signatures whose arguments refer to method or
owner parameters. Recursive signature mapping uses the appropriate method/owner cache,
retaining module caching for closed signatures. OpenBox/OpenBoxes generic namespace
functions and Box<TItem>.Same compile and execute; the seven native consumers return 42.
C# metadata tests pass (105 groups), including CLR/native instantiated open-signature
calls. No Runtime Contract, CLI bridge or runtime encoding change. External generic
constructions, constraints and full native System/bootstrap remain pending.

### External generic construction integration (2026-10-02 development)

A Raven-produced native bridge now forwards Box<T> signatures owned by another native
library, including closed results and vectors. Exact resolver identity and the original
definition survive both reference orders; missing dependencies diagnose. The consumer
loads both libraries and executes (42); all seven consumers and 106 C# metadata groups
pass. Existing Raven semantic/emission paths needed no implementation changes. No
Runtime Contract, encoding or bootstrap change; constraints and inheritance remain pending.

Observed follow-up: qualified generic namespace calls GenericBridge.Forward(...) and
GenericBridge.ForwardArray(...) reported RAV0234. Imported unqualified calls execute.
Reproduce independently before attributing this to a general binder defect or changing
shared compiler behavior; this is not a metadata-format restriction.


### Qualified native namespace functions resolved (2026-10-02)

Raven shared lookup now includes directly namespace-owned static methods alongside
CLI-container promotions, and binds these functions without a synthetic type receiver.
Three positive .NET controls passed before the fix: this was a provider-neutral
ownership contract gap exposed by native symbols, not a demonstrated .NET inference bug.
The separate shared fix is Raven `0633ba18d`; native probe coverage is `36f67a097`.
All seven consumers execute with exit 42, including three-assembly qualified inferred
and explicit generic forwarding. An incompatible explicit argument diagnoses. The
156-test focused .NET run and four qualified controls pass (three overlap).
See [evidence](experiments/extended-cli-metadata/native-qualified-functions-2026-10-02.json).
The explicit CLI primitive core and translated System bootstrap remain required.
No metadata schema, instruction or runtime change was needed; reuse the unchanged
106-group metadata evidence. Constraints and full native System import remain pending.


### Import/emission boundary direction (2026-10-02)

The author requires independent importer and emitter paths through Raven's semantic
symbol model. Current native emission still reaches reader definitions and resolvers;
that temporary coupling must be removed incrementally. Raven compiler interfaces are
separate from the metadata library's proposed IILGenerator body-authoring API. See
[the boundary and migration direction](design/extended-cli-metadata.md#independent-compiler-and-library-generation-boundaries-2026-10-02).
No APIs have moved yet. Explicit Runtime Contract configuration and CLI primitive/
translated System bootstrap remain unchanged; seven executing consumers establish
current behavior, not completion of the planned separation.


### Symbol-only namespace-function emission (2026-10-02)

Raven `2197fc2a2` and metadata `19a164d0` now author primitive/method-generic/vector
namespace calls from semantic symbols and captured assembly/artifact values. The
backend does not read the method definition or use the metadata resolver on this path.
Host snapshot checks remain, and no runtime format change is necessary. Nominal
signatures and type-owned methods/fields still use loaded definitions; host setup and
lazy symbol loading are not yet independent of readers. The library generator remains
planned. Explicit Runtime Contract selection, CLI primitive core and translated System
requirements remain unchanged. See [107 C# groups and seven runtime consumers](experiments/extended-cli-metadata/symbol-only-functions-2026-10-02.json).


### Symbol-owned root-class identity (2026-10-02)

Raven `690e81b3c` authors native public top-level root-class references from symbols
and exact artifact values, including invariant unconstrained generic definitions.
No input type-row search occurs on this path. Interface/inheritance, nested/value
profiles and member references remain reader-backed; their conversion/dispatch data
is not discarded. Explicit Runtime Contract and bootstrap requirements are unchanged.
All seven consumers execute; the metadata generic-owner check executes with the new
reference on CLR and both native containers. See [evidence](experiments/extended-cli-metadata/symbol-only-types-2026-10-02.json).


### Symbol-owned nominal namespace calls (2026-10-02)

Raven `269c60b07` reconstructs namespace-function signatures carrying external root
classes, including open/closed constructions and vectors, from symbols. The writer
recognizes these authored calls during graph validation. The three-assembly generic
forwarding case and all seven runtime consumers execute (42); 107 C# groups pass.
Type-owned methods/constructors/fields and richer type profiles remain reader-backed,
as do host setup and lazy symbol materialization. Explicit Runtime Contract selection,
CLI primitive core and translated System bootstrap are unchanged. See
[evidence](experiments/extended-cli-metadata/symbol-only-nominal-calls-2026-10-02.json).


### Symbol-owned root-class members (2026-10-02)

Raven `a9066f559` authors public nonvirtual root-class methods and constructors from
symbols, retaining owner generic parameter ordinals and existing construction semantics.
All seven native consumers execute (42). The metadata generic-owner consumer authors
constructor/Get/Set/Same references and executes on CLR and both native containers.
All 107 C# groups and API snapshot validation pass. Fields, virtual/interface/value/
nested profiles, host setup and semantic materialization remain reader-backed. Explicit
Runtime Contract, CLI primitive core and translated System bootstrap requirements are
unchanged. See [evidence](experiments/extended-cli-metadata/symbol-only-members-2026-10-02.json).


### Explicit field linkage values (2026-10-02)

Raven `43e565589` copies native field ordinals during import into a compiler-owned
optional layout contract. Supported nongeneric root-class fields then author references
from symbols, without consulting reader definitions. Ordinals include private slots
and are scoped by exact owner/artifact identity. This is target-specific linkage data,
not a requirement for ordinary .NET field lookup. The native format remains unchanged.
All seven native consumers and 107 metadata C# groups pass; readonly stores reject.
Interface/dispatch and richer layout profiles, host setup and semantic materialization
remain reader-backed. Runtime Contract and bootstrap requirements are unchanged. See
[evidence](experiments/extended-cli-metadata/symbol-only-fields-2026-10-02.json).


### Symbol-owned interface graph and dispatch (2026-10-02)

Raven `6a28042f3` authors nongeneric interface identities, direct interface edges and
abstract virtual member references from symbols. Transitive assignability is derived
by the output graph, not reader queries on this path. Seven native consumers execute
(42), including interface inheritance and storage aliases; 107 C# groups pass. Generic
interfaces, broader inheritance/virtual profiles, unsupported members, host setup and
semantic materialization remain outside the migration. Runtime Contract and bootstrap
requirements are unchanged. See [evidence](experiments/extended-cli-metadata/symbol-only-interfaces-2026-10-02.json).


### Independent library generator (2026-10-02)

Raven `47189b531` emits method bodies through the metadata library's IILGenerator in
the NeoCLR adapter. Shared Raven compiler interfaces remain independent. Builder and
attached authored-definition access return one stable generator over the canonical
body; internal operations currently delegate to the existing builder engine. Legacy
builder instruction methods remain available, and loaded/insertion editing is pending.
All 108 C# groups and seven native consumers pass; generator-based generic-owner code
executes on CLR and both native containers. Runtime Contract and bootstrap requirements
are unchanged. See [evidence](experiments/extended-cli-metadata/body-generator-2026-10-02.json).


### Generator owns body authoring (2026-10-02)

The 80 legacy builder body operations now forward to MethodILGenerator, which owns
appends, helpers, operand checks and local/label creation. Definition-owned storage
and declaration-scoped handles are unchanged; writer-side graph/flow validation stays
on the established path. Raven's adapter and shared contracts need no further change.
108 C# groups and seven native consumers pass; the generic-owner sample executes
on CLR and both native containers (42). Runtime Contract and bootstrap requirements
remain unchanged. See [evidence](experiments/extended-cli-metadata/body-engine-2026-10-02.json).


### Symbol-owned static containers (2026-10-02)

Raven `d32b2dcb4` admits static classes as declaration owners when reconstructing native
method references from symbols; value signatures remain separate and exclude static
classes. Seven native consumers execute (42), and 108 metadata C# groups pass. The
generic-static contract is authored from values and executes on CLR and both native
containers. Translated CLI and remaining virtual/profile bindings, host setup and lazy
materialization remain reader-backed. Runtime Contract/bootstrap requirements are
unchanged. See [evidence](experiments/extended-cli-metadata/symbol-only-static-owners-2026-10-02.json).


### Native callable fallback removed (2026-10-02)

Raven `4a3af6271` now requires supported native callable facts in symbols, without
falling back to reader definitions. Incomplete contracts diagnose. The unused method
definition property is removed. All seven consumers execute (42), including direct
concrete interface-implementation calls. Native concrete implementations do not carry
the CLI projection's final/virtual flags; no metadata change is made. CLI compatibility,
type/field/host paths and lazy symbol materialization remain separate pending work.
Runtime Contract/bootstrap requirements are unchanged. See [evidence](experiments/extended-cli-metadata/no-native-callable-fallback-2026-10-02.json).


### Native reference fallbacks removed (2026-10-02)

Raven `6baabdceb` removes native type/field fallback and the emitter-native resolver.
Alongside the earlier callable change, supported native references now require complete
symbol contracts; unsupported contracts diagnose instead of consulting definitions.
All seven consumers execute (42), with diagnostic checks retained. The metadata/runtime
are unchanged and prior 108-group library evidence is reused. Host-native input binding,
translated CLI lookup and lazy semantic loading remain separate pending concerns.
Runtime Contract/bootstrap requirements are unchanged. See [evidence](experiments/extended-cli-metadata/no-native-reference-fallback-2026-10-02.json).


### Native host dependency binding checkpoint (2026-10-02)

Raven `1d4cb92fc` adds `NeoClrMetadataDependency(NeoClrMetadataReference,
AssemblyIdentity)` for native emission configuration. The exact registered compiler
reference provides captured artifact identity/digest; the explicit second argument
specifies the primitive core contract. No separately supplied AssemblyDefinition or
image roundtrip is required. Output references continue to be authored from symbols.

The legacy snapshot constructor remains for CLI/translated compatibility and old native
callers. Its snapshot accessor `Definition` throws InvalidOperationException for the
new native binding. Duplicate identities/symbols, wrong core, unregistered references
and mismatched legacy native snapshots diagnose NEOMETA002 without output. All seven
native probe consumers now use the new overload, compile and execute with result 42.
The compiler reference still owns lazy semantic reader state. Runtime Contract,
explicit CLI primitive core, translated System bootstrap and native format are unchanged.
This is a Raven backend host API change, not a metadata-library or shared .NET API change.
Existing 108-group C# metadata validation remains applicable to the unchanged library.


### Closed generic field storage (2026-10-02)

Raven 73e6555b1 and the metadata authoring API now support public instance fields whose
storage is a closed generic reference class or a vector thereof, on nongeneric root
owners. The test first reproduced NEOMETA001 from the native field contract, then
passed after recursively admitting closed arguments. A second Raven-built library
holds Box<int>/Box<int>[] from the generic library; its consumer checks replacement
and aliasing across both dependencies. All seven native consumers execute (42).

The C# metadata tests pass 108/108 groups and execute the equivalent scalar/vector
field accesses on .NET and both native containers. Compared with .NET, the generic
field signature and CLI MemberRef already represent this storage; neoCLR still uses
its explicit ordinal layout contract. The change removes an authoring restriction
without adding encoding or runtime machinery. The tradeoff remains caller-asserted
layout accuracy when authoring from symbols, covered here by runtime execution.

Open parameters, fields on generic declaring owners, static/inherited fields and
unsupported value profiles remain outside this API path. Compiler symbols still own
semantic information; emission does not reopen importer definitions. Runtime Contract,
explicit primitive core and translated System bootstrap remain unchanged. Full generic
collection interface import and full class-library consumption remain follow-up work.


Generic-owner fields (2026-10-02): native public instance fields now support constructed
unconstrained root-class owners. Raven reads open field type/layout facts from compiler
symbols and binds consumer type arguments through ImportedConstructedFieldReference.
The metadata library preserves the open CLI MemberRef signature with a constructed
TypeSpec parent; stack validation uses the substituted type, while native emission keeps
the existing ordinal. IILGenerator owns instruction authoring. No importer definition
is reused by emission. Open caller parameters retain their scope; invalid arity, foreign
arguments, unconstructed field operands and out-of-scope arguments reject. Seven Raven
consumers execute (42), including generic forwarding and nominal mutation; 108/108 C#
metadata groups pass, with .NET and both native containers executing the field case.
Runtime Contract/core/System bootstrap and instruction encoding remain unchanged.


### Generic native interface imports (2026-10-02)

The native reader now retains constructed same-assembly interface relationships and
their type arguments, including open owner parameters. Raven maps these into existing
constructed symbols, preserving parameter owner identity, inherited interfaces and
invariant argument checking. Its emitter authors generic interface identities and
conversion edges from symbols, then binds interface calls to constructed references.
The loader is not consulted by emission. Metadata consumers may also import generic
interface methods and generic-owner fields from loaded native definitions.

NativeGenericConsumer now dispatches through MutableValue<int> -> Value<int> and
Value<Item>, including generic forwarding and class-to-inherited-interface conversion.
Both reference orders and incompatible argument diagnostics pass; all seven consumers
execute (42). NativeGenericOwnerChecks validates immutable reader relationships,
cycles/scope rejection, authored and reader-import dispatch, and field substitution.
All 108 C# groups pass; equivalent field/dispatch code executes on .NET and both native
containers. CLI TypeSpec/MemberRef and native dispatch/slot encodings are unchanged.

The supported interfaces are public, top-level, unconstrained and invariant. Native
relationship declarations currently resolve within their defining assembly. Emitting a
new class that implements an external interface or a new interface inheriting an
external interface remains a separate capability; this slice consumes already declared
relationships. Variance, constrained/value/nested profiles and instance generic methods
remain outside this native reader profile. Explicit primitive core, translated System
bootstrap and Runtime Contract configuration remain unchanged. This does not claim full
class-library import or remove the lazy semantic reader lifetime.


Resolution-view direction (2026-10-02, proposed):
[metadata resolution contexts and views](design/metadata-resolution-views.md) will provide
pure metadata dependency navigation and constructed signatures to importers. Raven
symbols remain the only importer/emitter boundary. No Runtime Contract, bootstrap,
encoding or implemented behavior changes in this design checkpoint.


Metadata facade checkpoint (2026-10-02): the C# Introspection namespace now owns a fixed
MetadataLoadContext and canonical assembly/module/nominal views. Raven removes its
private dependency resolver, reuses one context per immutable compilation, and maps
resolved metadata identity/token to symbols. Signature/member projection remains pending.
Runtime Contract, primitive core, System bootstrap, emission ownership and encoding are
unchanged. See [the facade design](design/metadata-resolution-views.md) and host API manual for current scope.


Constructed/field facade checkpoint (2026-10-02): the C# Introspection model now has
canonical primitive, vector, owner-parameter and constructed-type views plus declared
FieldInfo views. Definitions remain open; constructed owners substitute field signatures
simultaneously, preserving caller parameter scope and declaration identity. Recursive
nominal fields resolve without eagerly expanding members. Foreign/Void/bare-generic
arguments, wrong arity and unsupported method-parameter scopes reject explicitly.

Raven now consumes facade field types and closed signature projections, caching symbol
mapping by canonical view identity. This preserves array identity across fields, methods
and constructors; the existing integration assertion caught and verified that boundary.
Open method-signature adaptation remains in Raven until method/parameter views exist.
No emitter dependency on the context, Runtime Contract change, bootstrap change or
runtime/metadata encoding change is introduced. The runtime model informs names and
semantics but its guest implementation is unchanged. No performance claim is made.


Method/parameter facade checkpoint (2026-10-02): MethodInfo, ParameterInfo and
MethodGenericParameterTypeInfo now project namespace functions and declared methods,
including methods viewed on constructed owners. Owner and method argument scopes are
separate and substitution is simultaneous. Method/parameter identities remain canonical
within the context; no invocation or runtime loading is introduced.

Raven now builds native return/parameter symbols from these views and maps scoped
parameter identities back to the declaring compiler symbols. Its recursive signature
walkers and type/method generic-signature caches have been removed; the view-to-symbol
cache preserves signature identity. Language binding and special constructor return
semantics stay in Raven. This changes no Runtime Contract, primitive core/System
bootstrap, emission contract or metadata/runtime encoding. Properties and interface
relationship views remain next; generic method construction is not yet a facade API.

109 C# groups pass, including mixed owner/method scopes, generic function vectors,
constructed-owner returns, canonical method identity and invalid/foreign scopes.
All seven Raven native consumers compile and execute (42).


Property/interface facade checkpoint (2026-10-03): nominal and constructed views now
expose declared properties and directly declared interface relationships. Property
result/index types and interface arguments use the facade's simultaneous owner-scope
substitution; accessors share canonical method views. Indexed metadata excludes the
setter value parameter, including setter-only properties. Missing dependencies fail
explicitly, and CLI interface materialization remains unsupported rather than empty.

Raven consumes projected property types and direct interface views. Accessibility,
accessor association, parameter symbols and inherited-interface traversal remain compiler
responsibilities for this slice. Compared with .NET Reflection's property inspection,
this API has no get/set invocation or visibility filtering: it reports declaration data
and index types only. GetDeclaredInterfaces intentionally promises direct edges, not
Reflection's transitive GetInterfaces behavior. This keeps substitution reusable without
silently choosing compiler member-lookup or inheritance policies.

No Runtime Contract configuration, CLI primitive-core/translated-System bootstrap,
emission contract, guest API or serialized/runtime format changes. Native external
interface declarations and CLI relationship decoding remain separate gaps. Generic
method construction, constructor-specific views and bounded transitive metadata traversal
remain follow-up work. No performance claim is made.

Validation: 109/109 C# groups, all seven Raven consumers (42), and the API snapshot
check pass. See [property/interface evidence](experiments/extended-cli-metadata/introspection-properties-interfaces-2026-10-03.json)
for the tested Raven revision and artifact/runtime hashes.


Interface closure checkpoint (2026-10-03): GetInterfaces on nominal/constructed metadata
views now returns distinct direct and inherited interfaces, with composed owner argument
substitution. Iterative depth-first traversal follows metadata order; identity includes
constructed arguments. Cyclic declaration paths reject, even when arguments differ.
Traversal is bounded to 4,096 distinct views and 65,536 visited edges, with cached
read-only results per owner. This replaces Raven native AllInterfaces recursion;
Raven retains language symbol substitution and binding policy.

The .NET 10 baseline is Type.GetInterfaces (Microsoft Learn, retrieved 2026-10-03):
https://learn.microsoft.com/en-us/dotnet/api/system.type.getinterfaces?view=net-10.0
It includes inherited interfaces and substitutes constructed arguments. We use those
semantics for the supported native root-class/interface profile; our explicit DFS order
and traversal bounds are metadata-library policy, not claims of exact CLR ordering.
Compared with leaving recursion in each consumer, this centralizes metadata traversal
and bounds at the cost of retaining per-owner closure results. General base classes,
CLI relationship decoding and constrained parameter queries remain unsupported.
No Runtime Contract, emitter, bootstrap, guest API or encoding changes.

Validation: 109 C# groups, API snapshot checks and all seven native runtime consumers (42)
pass. See [recorded evidence](experiments/extended-cli-metadata/introspection-interface-closure-2026-10-03.json).


Generic method inspection checkpoint (2026-10-03): MethodInfo.MakeGenericMethod now
returns a canonical metadata view whose owner and method argument scopes are applied
simultaneously. GetGenericMethodDefinition retains the same open/constructed declaring
owner. Inputs are copied; caller-scoped parameters keep their original identity.
Namespace functions and vectors use the same projection. These views cannot invoke or
emit code; Raven continues its own inference and compiler-symbol construction.

The comparison baseline is .NET 10 MethodInfo.MakeGenericMethod (Microsoft Learn,
retrieved 2026-10-03):
https://learn.microsoft.com/en-us/dotnet/api/system.reflection.methodinfo.makegenericmethod?view=net-10.0
We follow definition/construction separation and allow supplied arguments that themselves
contain parameters. This is a bounded signature-inspection API, not .NET execution or
constraint validation: constrained CLI signatures already reject at the reader boundary;
wider constraint projection remains pending. A C# CLR comparison checks the same mixed
owner/method primitive substitution. Central projection avoids consumers reimplementing
that substitution, at the cost of retaining context-owned constructed-method views.
No Runtime Contract, emitter, bootstrap, guest API or encoding changes are introduced.

Importer assessment: native signature decoding, generic scope projection and interface
closure now live in the facade. Raven still reads declaration attributes, generic parameter
declarations, storage ordinals and accessor associations from reader definitions to create
language symbols. Those are adaptation sites, not emitter dependencies. The next bounded
facade work is constructor/member classification and accessibility metadata; do not move
Raven overload resolution, inference or language-specific visibility rules into this
library. General class inheritance, external native interface declarations, CLI relationship
decoding, full parameter metadata and constraint views remain explicit gaps.

Validation: 109/109 C# groups including a CLR signature comparison, API snapshot checks,
and all seven native runtime consumers (42) pass. See [evidence](experiments/extended-cli-metadata/introspection-method-construction-2026-10-03.json).


## Direct native compiler command (2026-10-03)

`rvnc neoclr --core-reference NeoCLR.CoreProbe.dll --reference Library.dll -o App.dll App.rvn`
now imports API-produced native assemblies using NeoClrMetadataReference and artifact-only
emission bindings. It selects CompilationOptions.NeoCLR and its explicit primitive/runtime
contracts. Native references never fall back to CLI projection. The core reference is
required when --reference is supplied; this is an experimental command compatibility change.
Without native references/core selection, the existing host primitive bootstrap remains
available. --system-symbols/--system-method is labelled legacy and remains an explicit
partial callable projection, not a native library import fallback.

Paired driver acceptance now uses `--dual-driver <rvnc.dll> <neoclr> <NeoCLR.CoreProbe.dll>
<fresh-output>`. Both .NET and NeoCLR Hello/helper and separate generic library consumers
execute (42); the consumer has no library source. Duplicate identities, missing transitive
dependencies, ordinary CLI references, malformed input and unsupported source reject without
publishing output. Host .NET runtime execution uses an explicitly hashed runtimeconfig.
The wider source-library/collections gate remains open; runtime encoding is unchanged.


Declaration-fact checkpoint (2026-10-03): metadata views now expose declared accessibility,
nominal abstract/sealed/static flags, instance/type-initializer classification, and
constructor enumeration on open/constructed owners. Constructor views share the callable
cache and substitute owner arguments. Raven maps supported metadata visibility to its own
accessibility and no longer decodes those type/method/field attribute bits itself.

This follows the existing CLI attribute contract rather than defining new access rules.
GetConstructors includes non-public instance constructors and type initializers explicitly;
GetMethods continues to exclude constructors. No invocation or implicit visibility filtering.
Existing canonical property accessor associations are retained. Supported native parameter
signatures remain by-value; byref/out, wider constrained/nested/value profiles still reject
at the reader boundary instead of losing their modes. This slice does not broaden encoding,
Runtime Contracts, the bootstrap, or runtime behavior.

Validation: 109 C# groups, API snapshot check, seven native consumers and paired driver
acceptance pass. See [evidence](experiments/extended-cli-metadata/introspection-declarations-2026-10-03.json).


External-interface gate baseline (2026-10-03): the paired harness now also supports
--dual-driver-external / --dual-driver-external-inventory with the same driver/runtime/core/
output arguments. Contracts, Box<T> implementation and consumer compile in separate
invocations; source is removed before downstream compilation. The .NET split executes 42.
Native Hello passes, but native implementation emission rejects in SourceTypePlan before
encoding because the interface identity is external. This is a failing future acceptance
case, not a claim that the external-interface slice is complete.

Required changes span SourceTypePlan/SourceInterfacePlan capability admission, symbol-only
external relationship authoring in the NeoCLR adapter, metadata relationship validation/
round-trip, and runtime dependency/dispatch verification. Current writer/reader relationship
attachment requires owned definitions. Do not simply remove admission checks or duplicate
contracts into the implementation assembly. No compiler/runtime behavior changes in this
baseline test slice.

### Native Self metadata contract slice (2026-10-03)

The host metadata API now retains `SignatureType.Self` and scoped `SelfTypeInfo` for
bodyless instance-interface declarations and associated properties. Runtime Contract
configuration is unchanged: Raven still needs its explicitly configured primitive core
and Self transport marker for the existing CLI bridge. No native Raven symbol or emitter
mapping is added by this slice. Native readers consume SelfType directly; reference-only
PE projection scopes the fieldless value-type marker to the supplied core identity.
Executable CLI emission rejects this native-only signature. This preserves contract
information without promising CLR execution of Self. Next adapt Raven symbols from
facade facts, then author native implementing-type substitution and typed calls from
symbols; keep importer objects out of emission. See the host API reference for the bounds.


### Checked-storage driver step (2026-10-03)

`rvnc neoclr --bootstrap-intrinsics --core-reference <core>` now passes the exact
registered primitive bootstrap to the existing target intrinsic contract. The flag is
opt-in, requires an explicit core and may be specified only once. It does not authorize
unrelated native dependencies or add an importer/emitter backchannel. `CheckedStorage`
must have the already validated public static `Reserve<T>(int) -> T[]` signature; it
emits the existing native array.reserve operation. Ordinary .NET defaults are unchanged.
Unlike CLR newarr, native reservation tracks uninitialized elements and faults on reads
before writes; the reference-only bootstrap body is never executable code.

The new `--reference-storage-core` generator mode adds only this declaration to the
primitive core, without competing source collection/union declarations. It is still a
bootstrap reference, not a retained System implementation or an executable .NET adapter.
No native format/runtime changes are needed. The C# API's existing BootstrapReference
validation and checked-storage tests remain the owning contracts.

`scripts/check-source-storage.py` first runs the existing dual-target source-iteration
acceptance. It then compiles a native generic helper library through rvnc, deletes its
source, imports its artifact into separate consumers without intrinsic opt-in, and verifies
execution. Mutation through an array alias returns 42; an unread reserved slot faults.
Disabled intrinsics, missing explicit core and repeated flags diagnose without publishing.
[Executable evidence](experiments/extended-cli-metadata/source-storage-2026-10-03.json).

Reproduce with the normal bridge build, selecting `--reference-storage-core /tmp/storage.dll`,
then `python3 scripts/check-source-storage.py --raven-root <Raven> --core /tmp/storage.dll
--output <fresh-directory>`.

The [unchanged-source inventory](experiments/extended-cli-metadata/arraylist-inventory-2026-10-03.json)
compiles Option and Propagatable alongside the seven collection contracts on .NET. Native
emission rejects the propagation interface's out-parameter signatures. Including ArrayList
also exposes missing System.Fail binding and minimal-bootstrap callback accessibility on
native; .NET lacks the runtime storage/Fail service adapters. These are separate dependency
and target-contract gaps. There is no ArrayList consumer or broad application success yet.
Next admit and preserve supported ref/out interface parameter modes through the native
metadata facade and compiler, then rerun the unchanged sources before widening emission.


### Native ref/out interface step (2026-10-03)

The existing native `ByRef` signatures and `out_parameters` indices now survive declaration
materialization and metadata-only introspection. ParameterInfo exposes element type plus
an explicit Value/Ref/Out mode. Raven imports those facts into parameter symbols and authors
member references from them. The portable interface plan admits writable ref/out only when
the target's managed-reference capability allows it; readonly variants remain unsupported.
Complete external contracts retain output indices when substituting generic owner arguments.
No runtime/schema change, importer-object emission dependency or new Runtime Contract option
is introduced. Ordinary .NET Reflection/Emit remains in place. Native semantics match the
existing CLR ref/out calling contract: out must be assigned before normal return; ref input
must already be initialized. CLI In metadata is not silently treated as writable ref.

A C# driver fixture compiles contracts, implementation and consumer independently, removes
library sources, then executes generic inherited interface out assignment and ref mutation
on both targets (42). Incompatible ref/out implementations reject without output publication.
Reproduce with NeoClrMetadataProbe `--dual-driver-parameter-modes <rvnc.dll> <neoclr>
<CoreProbe.dll> <fresh-output>`. C# metadata tests cover both containers, definition/builder
parity, mode conflicts, readonly rejection and open/constructed signature substitution.
See [evidence](experiments/extended-cli-metadata/parameter-modes-2026-10-03.json).

The unchanged source Propagatable interface now emits with the source-owned collection
contracts. Option no longer fails interface admission; it now reaches the native source
union/declaration emission rejection. This is the next source-library gate, alongside the
previously recorded Fail/callback bootstrap dependencies for ArrayList. No source stubs or
manual union carriers replace the runtime library, and broad application completion remains open.

## Union payload foundation (development, 2026-10-03)

The metadata producer now accepts direct nominal and constructed fields in owned
value types, including a `Payload<T>` embedded in a `Carrier<T>`. Builder calls and
manually attached field definitions share validation. Writing rejects recursive inline
storage and limits owned layout traversal to depth 64 and 4096 visited constructions;
references and vectors terminate inline traversal. Native input validates the same
owned layouts. External dependency layouts still require explicit dependency resolution
and runtime linking; this check does not load dependencies implicitly.

`AssemblyDefinition.ReadNativeAssembly` now materializes unconstrained top-level
value declarations, signatures, fields and supported constructors/methods. `IsValueType`
reflects the native category; sealed/sequential flags are preserved without inventing
a CLI `System.ValueType` dependency. ImportReference overloads retain the explicit
matching output core requirement. Nested declarations, constrained owners and generic
instance methods remain outside this native snapshot profile. Native snapshots remain
immutable and preserve their original bytes on Write.

This matches CLR inline value storage and copy semantics: an executable tag/payload
fixture returns 42 on both runtimes after mutating an independent copy. Direct native
imports of nongeneric/generic value constructors and methods also execute on both.
The fixture uses an Int32 tag and is a metadata contract test, not a replacement for
Raven union lowering or proof that source Option compiles.

Raven maps the introspection value category to Struct and its semantic ValueType base;
generic field substitution remains in introspection. It does not reopen imported
metadata in emission. Source value declarations, nested union cases, the Byte tag,
synthesized members and symbol-authored external value operands remain the next
compiler work. Runtime-library union sources are unchanged. No format version change,
CLI projection fallback, runtime implementation change or performance claim is needed.

## Source value declarations for union emission (2026-10-03)

The native adapter now opts into an explicit portable `ValueType` declaration category.
Top-level ordinary structs, including unconstrained generic owners, map to metadata
value definitions. Reference classes retain their existing path. Nested declarations,
value-interface implementations, ref structs and constrained value owners remain outside
this bounded source profile; no source unions are rewritten into classes or manual carriers.

Shared body planning preserves an addressed receiver for member access and takes a value
copy when `self` is used as an expression. Synthesized parameterless struct constructors
zero-initialize fields before declared initializers. The adapter uses the existing metadata
ILGenerator and value-type builder contracts. There is no metadata format/runtime change,
new bootstrap selection or importer-to-emitter dependency. Ordinary .NET emission retains
its existing Reflection/Emit path; its portable profile does not opt into this new category.

The C# `NeoClrMetadataProbe --source-value-driver <rvnc.dll> <neoclr> <core.dll> <fresh-dir>`
checks ordinary driver compilation and execution on both targets: generic inline payloads,
explicit/default constructors, accessors, self copies, local field mutation and independent
copies return 42 with empty stdout/stderr. Unsupported value-interface implementation
rejects with NEOMETA001 and no output. This is same-compilation source value coverage,
not separate native library consumption or source Option completion. Source and artifact
hashes plus command results are recorded by the harness.

Remaining union dependencies include nested case/companion declarations, the Byte tag,
synthesized union methods/relationships and symbol-authored external value references.
The source library and broad application gates remain open. No independent binder fix
was needed here; shared changes add an explicitly selected emission capability.

## Native nested case metadata (development, 2026-10-03)

`AssemblyDefinition.ReadNativeAssembly` now retains supported nested class/value
ownership beneath nongeneric declaring types, including generic nested value cases.
Local signature lookup keys include the declaring token; external TypeRef rows use
nested TypeRef scopes. Same-named cases beneath different companions remain distinct.
Nested public/internal accessibility, canonical declaring views and constructor/member
signatures survive direct native reading without a CLI projection. ImportReference
accepts these scoped native definitions with the existing explicit matching core policy.

Raven publishes nested symbols as members of their declaring type, preserving their
namespace and containing-type identities; namespace member and simple-name lookup do
not flatten them. Generic nested field substitution stays in introspection. Emission
continues to reject native nested operands outside its symbol-authored capability profile.

This follows CLI NestedClass/TypeRef identity semantics using existing native relationship
encoding; no schema change or implicit dependency loading is required. C# checks cover
same-named local/external payloads, multiple nesting levels, internal visibility, missing
dependencies and cyclic owners. Direct native nested generic/nongeneric constructor
imports execute on CLR and NeoCLR (42). Native semantic probes retain owner/field identity.

This is a reader/importer prerequisite for union cases, not source union completion.
Nested types that capture generic enclosing parameters remain unsupported. Source union
declaration collection, nested definition emission, Byte discriminators and complete
synthesized union contracts remain pending. Existing immutable snapshot behavior is unchanged.

## Nested source declaration emission (2026-10-03)

The NeoCLR adapter now opts into the compiler-owned `NestedType` declaration capability.
It collects nested class/struct declarations in owner-first order and calls the metadata
library's existing nested builders with empty child namespaces and lexical ownership.
Supported children are nongeneric root classes and unconstrained generic/nongeneric
values under nongeneric supported owners. Static children, generic enclosing-type
capture and generic nested reference classes remain explicit unsupported categories.
The ordinary .NET Reflection/Emit backend does not opt into the new portable category.

Nested lookup may provide a substituted accessor under a nongeneric owner. The native
callable resolver now reuses its original source declaration before considering external
references. This uses compiler symbols only, with no name-based member matching or
importer access. It is part of the new native emission capability; no independently
reproduced .NET regression or binder fix is claimed.

`NeoClrMetadataProbe --nested-value-driver <rvnc.dll> <neoclr> <core.dll> <fresh-dir>`
compiles equivalent sources through ordinary commands on both targets and executes 42
with empty output. It checks two same-named payloads with different owners, a generic
nested value, nested class construction, zero default storage and independent value
copies. It reads the emitted native snapshot to verify enclosing identities and rejects
generic enclosing owners and value-interface implementations without publishing output.
The explicit CoreProbe bootstrap is unchanged; no metadata schema or runtime change is
required. Existing library IILGenerator remains behind Raven's emitter boundary.

Generated union declaration collection, Byte discriminators, full synthesized union
contracts and symbol-authored external value/case operands remain pending. This test
contains ordinary nested declarations; it does not claim that unchanged Option or the
broad class-library consumer compiles yet.

## Byte discriminator prerequisite (2026-10-03)

The native adapter opts into compiler-owned Byte signatures and the ConvertByte
operation. Introspection's Byte view maps to the semantic System.Byte; emitted
references use those symbols, never importer handles. Byte fields, parameters,
returns and literals retain unsigned 8-bit identity. Numeric narrowing selects the
metadata library's `IILGenerator.Emit(OpCode.Conv_U1)`; widening uses existing integer
operations. The .NET portable adapter does not opt into Byte and retains its ordinary
Reflection/Emit path. No Runtime Contract or bootstrap selection changes are required.

This follows CLI unsigned small-integer storage with Int32 evaluation-stack values.
The metadata writer normalizes stack categories without equating `ref Byte` and
`ref Int32`. Existing native Byte storage and `conv.u1` implement truncation and zero
extension; no native format or runtime code change is needed. This does not add
floating-point narrowing, overflow-checked conversions or complete source-union support.

`NeoClrMetadataProbe --byte-discriminator-driver <rvnc.dll> <neoclr> <core.dll> <fresh-dir>`
checks ordinary source struct tags, byte literals, Int32/Int64 narrowing and widening,
then builds a byte-returning library, deletes its source and executes a separate
consumer using the emitted reference on each target. All programs return 42 with empty
output; -1 → 255 and 256 → 0 boundary assertions execute. Native library references use
the direct metadata path, with the explicit existing CLI primitive bootstrap retained.
The metadata C# checks additionally cover array/local truncation and exact byref identity.

Validation: 116 metadata test groups, native metadata fixture execution (42), paired
driver/source-absent import execution (42), and eight focused Raven emission/operator
regressions. Source union collection, synthesized union members and external value/case
operands remain pending. The pre-existing API snapshot regeneration blocker remains
recorded in neoCLR's api-docs/README.md; manual host API/XML documentation is updated.

## Source union declaration graph (2026-10-03)

The portable planning layer now discovers the complete bound union declaration graph:
carrier, optional nongeneric companion, case types, fields, properties and generated
methods. Accessor methods are deduplicated by symbol identity. Discovery includes unused
members rather than depending on application calls. Metadata owners are recorded
separately from semantic owners: generic union cases belong to the existing nongeneric
companion, including constructed case signatures. The native type adapter consumes that
physical owner without changing the language symbol's containing type.

Generated union callables with no source body may use the union declaration as a
syntax/model anchor. Their body still comes from Compilation.TryGetSynthesizedMethodBody;
the target adapter does not synthesize alternative language semantics. Ordinary .NET
Reflection/Emit remains unchanged and no Runtime Contract or bootstrap setting changes.

Native emission now preflights these declarations and reports the unsupported type,
field or callable contract. This is **discovery and admission groundwork**, not native
union emission. A final explicit gate prevents treating the carrier as an ordinary struct
and losing the union/case contract required by future native symbol imports. No metadata
format, public metadata API or runtime change is included in this slice.

`NeoClrMetadataProbe --union-declaration-driver <rvnc.dll> <core.dll> <fresh-dir>`
executes generic and nongeneric source union construction/pattern extraction on .NET (42),
then checks native NEOMETA001 rejection without publication. Both minimal native cases
now reach the synthesized ToString override contract. Unchanged source Option<T> instead
rejects its value-type interface relationship (Propagatable). This test labels native
execution as pending; a rejection check does not satisfy the end-to-end gate.

Next: value-type interface contracts/dispatch for Option, supported synthesized override
and display contracts, and native union/case metadata round trips through introspection
and Raven symbols. Do not omit unused generated members or turn off structural display
implicitly to bypass these gaps. The existing explicit CLI primitive bootstrap remains.
Seven focused declaration/backend tests cover scoped ownership, constructed cases,
canonical accessor enumeration, synthesized body lookup and backend validation; paired
ordinary-driver controls establish the current executable boundary. This is target
integration groundwork, not a demonstrated independent .NET bug fix.

## Value-interface declarations and constrained metadata calls (2026-10-03)

Raven's native adapter now opts into the compiler-owned ValueInterfaceImplementation
category. Ordinary source value types can declare the supported owned/external interface
relationships; concrete calls retain addressed value receivers. The .NET portable adapter
does not opt into this category and ordinary Reflection/Emit behavior remains the default.
No importer objects are used to author relationships, and Runtime Contracts/bootstrap
selection are unchanged. The old negative value-interface fixture is now a negative
boxed-conversion case, since a struct implementing an interface is no longer unsupported.

Separately, the metadata library's IILGenerator now exposes CallConstrained(receiverType,
target) and Emit(Callvirt, receiverType, target). This bounded profile admits owned
nongeneric value receivers and owned nongeneric interface targets. It emits CLI
constrained./callvirt and native borrowed callself, preserving exact addressed storage
without boxing. Definition and builder interface authoring share validation; native
snapshots/introspection retain value relationships and generic owner arguments. The
existing runtime executes the emitted assembly; no runtime or schema change is required.
Raven's general constrained-call lowering, external/constructed constrained targets,
open receiver parameters and boxed interface conversions remain future work.

Validation: 117 metadata C# groups; CLI/native constrained dispatch returns 42 while
checking mutation and independent copies; wrong addresses, unsupported operands,
unboxed virtual receivers and incomplete implementations reject. The ordinary
`--value-interface-driver` paired case compiles and executes source interface/struct
relationships with concrete calls on both targets (42), and rejects boxed conversion
before publication. Eleven focused Raven declaration/backend/local-emission tests pass.
The explicit primitive bootstrap is unchanged; no independently reproduced .NET binder
regression is claimed. API manual/XML coverage is current, while the previously documented
RavenDoc snapshot regeneration blocker remains.

Unchanged source Option now passes the value-interface declaration boundary and reaches
its synthesized ToString override. Next address generated override/display contracts and
native union/case metadata preservation; this slice does not complete source union emission.

## Generated union body planning (2026-10-03)

Portable callable planning now recognizes generated case constructors whose location is
a CaseDeclarationSyntax and generated payload getters whose location is a ParameterSyntax.
These symbols previously failed declaration admission even though Raven already provides
their bound bodies. The union anchor must match a declaring syntax reference of the actual
owning union. Authored methods keep their own syntax/body; an unrelated union cannot
supply the generated-body anchor. This is internal target planning, not a binder rewrite.

The existing synthesized-body factory and Lowerer remain the semantic owners. C# tests
now require successful shared lowering of carrier constructors, case constructors,
TryGetValue, payload getters and available deconstructors for generic/nongeneric unions.
Seven focused planning/backend checks and two existing .NET union execution regressions
pass. The .NET Reflection/Emit path, Runtime Contracts, primitive bootstrap and metadata
library APIs are unchanged. No native union execution is claimed from plan admission.

Investigation of the next blocker confirms that synthesized ToString must retain a real
Object virtual-slot contract. It cannot be emitted as an ordinary nonvirtual method.
The metadata declaration API now authors that override for CLI output; native runtime slot
validation also needs the explicit retained System.Object dependency and correct target
identity/name. The generated formatting helper additionally uses object/string/character
operations. Next add the bounded override/reference contract and formatting dependencies,
then preserve union/case metadata for native imports. The production union publication
gate stays closed throughout; supported core bodies are not a complete union contract.


## Union override declaration foundation (2026-10-03)

The separate metadata library now exposes TypeBuilder.AddOverride and matching detached
MethodDefinition flags for a value-type ToString override. Definitions remain authoritative;
builders are convenience and bodies use GetILGenerator. CLI flags reuse the Object slot,
even when the method also implements a ToString interface contract. C# execution covers
ordinary/generic values and interface dispatch; 118 metadata groups pass.

This is a declaration foundation, not native override emission. WriteNativeAssembly rejects
these methods before returning bytes. Native encoding still needs the explicit retained
System.Object slot and dependency identity; the runtime slot validator already requires the
actual System module and matching virtual signature. The current writer's hex-encoded
ordinary method name cannot identify that slot. Native reader materialization must preserve
the same override meaning when that encoding is added. No Runtime Contract or compiler
capability is enabled, and source Option still stops at synthesized ToString admission.
Formatting operations and native union/case contracts remain subsequent gates.
See [evidence](experiments/extended-cli-metadata/value-override-authoring-2026-10-03.json).


## Native value Object override binding (2026-10-03)

The metadata writer now emits the bounded value ToString override after validating one
explicit BindNativeLibrary mapping to the retained System module. It verifies the CLI
bootstrap slot and the actual native public virtual String-returning instance contract.
That registration contributes an assembly/module dependency even without an IL call.
Missing, ambiguous and incompatible slots fail before encoding returns bytes. This is
an explicit bootstrap bridge only; application/native library references do not fall back
to a CLI projection. The System source bundle and checked-in CoreProbe revision/hashes
are recorded in [execution evidence](experiments/extended-cli-metadata/native-value-overrides-2026-10-03.json).

Native method rows use the runtime slot name and existing is_virtual/is_override fields.
The reader retains Virtual/ReuseSlot and rejects invalid flags, names or missing System
scope. Imported native value calls preserve that name without using runtime reflection.
A separate metadata-generated consumer runs against the generated library (stdout
`native override`, exit 42). An independent neoIL harness boxes nongeneric/generic values
from that library and dispatches through the real System.Object.ToString (exit 42).
The harness supplies boxing instructions, not replacement declarations or method bodies.
No runtime source or format version changes were needed; this follows supported CLR
slot semantics with explicit native linking. Native instructions/bodies remain runtime
validated, not interpreted by the metadata reader.

Raven source-union preflight is still closed at ToString admission. Next expose the bounded
override contract through compiler symbols/capabilities and the target adapter, then close
generated formatting operations and union/case metadata preservation. Runtime Contract
configuration and the .NET Reflection/Emit backend remain unchanged. This is not the
full source Option or broad application gate, and does not establish compiler boxing.


## Raven override emission and next union body gate (2026-10-03)

Raven `29268815d` consumes metadata `b94bdf79` through a bounded ObjectToString
callable capability. The contract comes from bound source symbols and their resolved
System.Object slot; reference-nullability annotations are erased only for physical slot
classification. General virtual/class methods remain gated. The target adapter chooses
AddOverride, while body instructions use the independent Raven/metadata generator boundaries.
Direct addressed calls work on local ordinary and constructed generic values.

`rvnc neoclr --core-reference NeoCLR.CoreProbe.dll --runtime-seed System.neox ...`
now registers an explicit retained seed binding. This imports no extra symbols and does
not fall back for native application references. It requires an explicit core, module
System, bounded input images, a distinct output and no legacy --system-symbols selection.
Source ownership manifests are checked against seed type inventory (including generic
arity normalization); duplicate declarations reject before publication. Supply the same
seed to runtime --system. Intrinsic opt-in and semantic Runtime Contract selection remain
separate. The full seed is not valid alongside source-owned iteration contracts until
those copies are removed; this slice does not claim filtered-seed completion.

The driver check compiles/runs ordinary and generic struct overrides (stdout
`native override`, exit 42), verifies native override flags and rejects missing seed/core
and duplicate source ownership without output. The unchanged source is attempted on .NET;
Raven's existing exact return-nullability rule rejects it because host Object.ToString
returns string? while the native bootstrap returns string. This is explicitly recorded
as RAV0307, not a successful dual-target fixture. No binder behavior was changed; existing
.NET union execution controls still pass. Twelve focused Raven tests pass.

Union preflight now admits generated override declarations and validates shared lowered
bodies. Ordinary/generic union controls and unchanged source Option now reach
`union body ToString: lowered expression BoundConversionExpression`, with no output.
The next slice is generated display conversion/formatting support, then union/case metadata
preservation. Compiler boxing and separately imported override reference authoring remain
unproven. See [driver and source evidence](experiments/extended-cli-metadata/raven-value-overrides-2026-10-03.json).

Typed boxing continuation (2026-10-03): the metadata IL generator now exposes Box and
raw Emit(Box, signature), producing standard CLI and existing native boxing instructions.
Raven's target-owned capability lowers value/generic-to-object conversions without
changing binding; .NET retains its existing backend. The normal native command executes a generic box
smoke case with the explicit core/retained-System seed. C# API tests separately observe
value dispatch, primitive display and reference identity; the smoke case alone does not
prove these semantics. Union preflight now stops at generated value-payload addressing;
Object virtual calls/formatting and union metadata preservation remain open. Runtime
Contract settings and format versions are unchanged. No implicit reference fallback.

An independent .NET arrow-method bug exposed by this regression work is fixed by
consuming the existing bound return block, retaining implicit generic boxing. Ten tests
pass both on the integration line (Raven 697a093d7) and the main-based fixes branch
(c96305e50); main is unchanged. No new language/Runtime Contract rule is introduced.

See [typed-boxing evidence](experiments/extended-cli-metadata/typed-boxing-2026-10-03.json)
for commands, source/artifact hashes and cross-repository validation (Raven 558462967).

Owned field-address continuation (2026-10-03): metadata IILGenerator now supports
LoadFieldAddress/Emit(Ldflda) for owned mutable definitions and constructed field references.
Raven admits a separate FieldAddress capability carrying only IFieldSymbol; its adapter
resolves owned field handles. Getter/method receivers nested in value fields use the
actual storage; no spill-copy workaround is introduced. Imported/readonly field addresses
remain explicit limits. Ordinary .NET stays on its existing backend. API and ordinary
compiler-command cases execute nested generic mutation through object aliases (42) on both
targets; 120 C# metadata groups and ten focused Raven tests pass. No runtime/format or
Runtime Contract configuration change. Unchanged Option and the source-union controls
now reject at `<RavenFormatUnionValue>` BoundBinaryExpression (null comparison), with no
native output. Null/reference operations, remaining formatting and union/case metadata
preservation are next; full union execution is not claimed.

[Field-address evidence](experiments/extended-cli-metadata/owned-field-addresses-2026-10-03.json)
records commands, artifact/source hashes and Raven c951e33c0.

Reference-test continuation (2026-10-03): the metadata IL generator now supports IsNull,
IsInstance and String checked casts. CLI uses standard ldnull/ceq, isinst and castclass;
native uses existing ref.isnull/isinst/castclass, with no runtime or format change. Raven
has explicit null-test/discard-type-test capabilities; non-user-defined null comparisons
reuse bound operator facts. Overloaded equality is not replaced. .NET retains its general
backend and Runtime Contract settings are unchanged. API and normal driver executions
return 42 on both targets; 121 metadata groups and focused capability tests pass. Source
union preflight now stops at Object.ToString dispatch inside the formatting helper, with
no native output. Wider patterns, core virtual dispatch, remaining formatting and native
union/case metadata remain open.

[Reference-test evidence](experiments/extended-cli-metadata/reference-tests-2026-10-03.json)
records commands/hashes and Raven 5bb8f4dfc.

Core display dispatch continuation (2026-10-03): imported public instance
System.Object.ToString() -> String from the exact explicit CLI core snapshot now
supports CallVirtual. Native output requires the validated System slot binding;
missing, ambiguous, nongeneric/signature-mismatched or nonvirtual slots reject.
This reuses CLI callvirt and native virtual dispatch without a format/runtime change.
Raven opts into a bounded semantic Object display capability; other virtual class
calls remain unsupported. Runtime Contract and importer/emitter boundaries are unchanged.
API-authored boxed value overrides execute through Object; ordinary Raven commands
print `42` and `text` on both targets. The union preflight now reaches a synthesized
get_Value null literal. Full native union/case metadata and execution remain pending.

[Core display evidence](experiments/extended-cli-metadata/object-display-2026-10-03.json)
records Raven ec23ae9bc, artifact/source hashes, commands, and the remaining gate.

Typed null continuation (2026-10-03): Raven ad3d8a71f emits contextual reference nulls
through the existing metadata LoadDefault operation. Return/local/argument driver cases
execute on both targets; 11 focused tests pass. Plain/generic unions reach the native
union/case metadata gate. Unchanged Option next rejects its TryGetOutput case pattern.
No metadata/runtime change or union execution is claimed. See
[null evidence](experiments/extended-cli-metadata/typed-null-2026-10-03.json).

Option body continuation (2026-10-03): Raven a99cd3c3e reuses existing case-pattern
lowering in conditional branches and Boolean values, and emits RuntimeUnitContract's
inhabited value as a nominal default. Native configured unit out/value arguments
execute (42); .NET configured ValueTuple storage also executes. Existing CLI/native
instructions and runtime mappings are unchanged. Unchanged Option and its dependencies
now pass source-body preflight and reach the union/case metadata publication guard.
Next implement general custom-attribute authoring/reading and introspection, retaining
Raven's .NET UnionAttribute, case-name/ordinal and generic companion contracts through
the existing runtime custom_attributes representation. No union-only wire format or
unmarked-struct fallback is planned. Encoding and native union execution remain unproved.
[Case/unit evidence](experiments/extended-cli-metadata/union-case-unit-2026-10-03.json).


Union execution bootstrap continuation (2026-10-03): the host metadata adapter now maps
explicitly bound core String static owners and core Char type-test operands to canonical
native primitive encodings. CLI call/type tokens remain ordinary CLI encodings. This is a
representation difference of the retained seed, not a language-level String/Char change.
Exact core identity, System module and method contract validation remain required; nominal
String instance import remains unsupported. The temporary CLI bootstrap owner is the host
binding layer; native semantic primitive import replaces it as class-library ownership
expands. Native PE/#Neo remains transitional. See the executable
[union evidence](experiments/extended-cli-metadata/union-local-execution-2026-10-03.json).


## Native union imports (development, 2026-10-03)

The normal driver now imports separately emitted plain/generic union libraries through
native metadata. The facade resolves attributes and parameter names; Raven reconstructs
carrier, logical case and physical companion relationships. Case ordinals and ownership
must be consistent, and cases require one instance constructor. Corrupt relationships
diagnose before publishing output. Constructor parameter names are retained because
Raven's case payload-property association depends on them.

`NeoClrPrimitiveBootstrap.ReadAssembly(ReadOnlySpan<byte>)` captures an explicit CLI core
snapshot and exposes its matching `PortableExecutableReference Reference`. Hosts must
include that exact reference and select its assembly as the target core. The new
`NeoClrMetadataReference.ReadAssembly(image, bootstrap)` overload admits only that exact
core identity during native metadata resolution; application and rebuilt-library
references remain native. Conflicting bootstrap snapshots and missing dependencies reject.
The existing overload remains available for catalogs without that explicit bridge.

Emission uses compiler symbol facts and host identity/digest contracts to author external
value and physical nested owners. It does not reopen native importer objects. The current
Reflection/Emit backend remains unchanged. A small common union-companion symbol contract
replaces concrete PE checks in binding; language decisions still belong to Raven.

Validation: local and separate-library plain/generic cases compile through ordinary driver
commands and run on both runtimes (42). Local cases also print the expected Some/None
strings. Separate consumers use only emitted references, preserve an independent carrier
copy after replacing the original, and match the payload. Malformed duplicate ordinals
reject with no output. 21 focused .NET regressions and 124 metadata contract groups pass.

Limitations: native embedded marker types retain the documented bounded attribute profile;
CLI IUnion projection, imported ToString overrides and unchanged Option/Result completion
are not claimed. The source Option ownership probe now reaches an unregistered System.Void
residual type argument. This requires a coherent unit-value/bootstrap contract, not removal
of source declarations or a fallback to CLI library imports. The API documentation guest
snapshot remains stale for the previously recorded bridge issue.


### Generic inhabited unit execution (2026-10-03)

The exact RuntimeUnitContract is now honored by Raven overload argument validation.
Unconfigured CLI void remains rejected. The new regression reproduces RAV1501 before
the fix and passes afterward. Native driver execution constructs Residual<System.Void>
with an out-initialized unit, matches the payload and passes it as an ordinary argument,
returning 42. Runtime and metadata representation were already sufficient; this was a
binding gap. The compiler fix is independently validated on the main-based fixes branch
(fc32e3b9e), with 16 focused tests passing there and on the integration branch (ca7164aa2).
Main is unchanged. See [execution evidence](experiments/extended-cli-metadata/inhabited-unit-execution-2026-10-03.json).
Unchanged Option still requires coherent bootstrap/source ownership before execution.


### Unchanged source Option/Result execution (2026-10-03)

The bounded [union bootstrap](experiments/extended-cli-metadata/bootstrap/README.md)
now supplies executable primitive services without duplicate collection/union definitions.
The ownership manifest assigns iteration, Propagatable, Option and Result to the source
library. A generated storage core supplies explicit primitive symbols only; native library
references use the metadata importer. The seed's Object display adapter uses existing
native type-handle queries rather than constructing the guest introspection facade.

Compilation exposed two adapter gaps: authored value types could not retain interface
relationships, and imported methods could not accept the configured inhabited unit as a
value parameter. Metadata now accepts top-level value implementation edges while keeping
boxing requirements; Raven authors direct calls to concrete nonoverride value methods,
including CLI virtual interface implementations, with managed receivers. Unit parameter
positions map to the selected nominal unit representation; return positions preserve the
existing no-result mapping. Importer objects remain outside emission.

The unchanged sources compile into NeoCLR.Collections.dll. A separate consumer using only
that native reference executes with exact output `Option.Some(40)` and `Result.Error(7)`,
exit 42, and checks copies, output initialization and residuals. Missing-library and duplicate
seed ownership tests reject without output. This does not complete the executable .NET
adapter, broader collections or application-order-collections gates. See the reproducible
[evidence](experiments/extended-cli-metadata/source-unions-2026-10-03.json).


### ArrayList source assessment (2026-10-03)

The next bounded bootstrap adds existing CLI callback declarations and the System.Fail
namespace marker to primitive symbols. The executable seed extends the union seed with
terminal failure; no collection implementation is replaced. Unchanged ArrayList and its
internal iterator now compile alongside the source union/iteration library. Source-included
execution passes alias mutation, copy independence, iteration and Find callback checks
(exit 42), and negative capacity terminates with its expected error.

The separate consumer still fails before publication: native declaration materialization
rejects the callback function-signature category used by ArrayList.Find and related methods.
This is the next reader/facade/importer task. The source-included run is a diagnostic control,
not a completed native-reference gate or the dual-target application gate. The existing
separate Option/Result consumer still passes. See the bootstrap README and
`docs/experiments/extended-cli-metadata/arraylist-source-assessment-2026-10-03.json` in neoCLR.
The CLI callback declarations are existing bridge transport; no new Function semantics or
integration of the separate structural Function branches is claimed.


### Separate ArrayList callback import executes (2026-10-03)

Native metadata reading now retains the existing function signature category. The
metadata-only FunctionTypeInfo facade owns generic substitution, canonical signature
views and dependency resolution. Raven maps value-returning shapes into its existing
callable symbols using the explicit primitive bootstrap; emission independently authors
callback operands from those symbols. There is no importer-object reuse, format revision,
Reflection backend change or integration of the separate Function language experiments.
Explicit no-result callback imports reject rather than silently acquiring an inhabited
unit result; the facade preserves both categories.

The ordinary driver compiles unchanged ArrayList, iteration and Option/Result sources to
NeoCLR.Collections.dll, then compiles its consumer with only the emitted reference.
Alias mutation, independent copies, iteration and Find callbacks execute (42). Negative
capacity also executes through a separately compiled consumer and fails as expected.
The assessment driver now requires native import success; the previous failure record
remains historical. Metadata contracts pass 125 groups, all seven existing native consumers
pass, and the separate source-union gate still passes. The .NET class-library adapter,
HashMap/comparers, queries and broad application gate remain open.

### Native source HashMap gate (2026-10-03)

The cumulative source-owned library now includes comparer policies and HashMap, with
unchanged runtime sources and separate native consumer import. Explicit collection
primitive bootstrap and retained seed configuration remain unchanged. No CLI projection
is used for the rebuilt library. Collision/growth, replacement, missing keys, key snapshots,
callback interface dispatch and shared value mutation execute with exit 42 and no stdout.
The existing capacity-failure and dependency ownership guards pass too. No compiler,
metadata or runtime behavior changed; .NET executable library adapters and query/broad
application acceptance remain pending. See
[reproducible workflow](experiments/extended-cli-metadata/bootstrap/README.md#separately-compiled-hashmap-and-comparers)
and [evidence](experiments/extended-cli-metadata/hashmap-import-2026-10-03.json).

### Extension declarations reach native separate-library execution (2026-10-03)

Raven's extension declaration lowering is now consumed by the native adapter: receiver
generics are lifted onto methods in a nongeneric static container, as on the CLR backend.
A bounded embedded ExtensionAttribute on that container survives native reading and the
introspection facade. Native namespaces implement Raven's existing extension discovery
contract. Binding still owns applicability and inference; emission authors from symbols.
The retained seed and explicit collection primitive Runtime Contract are unchanged.

A separate extension library and native consumer execute receiver-generic predicates
and a method-generic selector (42); the cumulative HashMap gate also executes. This is
an isolated regression, not a stub for System.Linq. The unchanged full Operators source
still fails before publication at object-to-generic conversion in OfType<U>. Constrained
extensions, static extension members and extension properties are not newly supported.
The embedded marker uses the same temporary nominal constructor profile as unions,
without a CLI Attribute base; ordinary .NET metadata retains its existing backend.
No format version change is required. See the
[workflow](experiments/extended-cli-metadata/bootstrap/README.md#native-extension-declarationimport-regression).

Validation: Raven `0cca93a00`; 23 focused shared capability and .NET codegen tests pass.
The native extension/HashMap driver gate and unsupported-property publication guard pass.
Source and compiler payload hashes are recorded with the executable evidence.

### Unchanged native query library execution (2026-10-03)

Raven `e3556fabe` lowers built-in reference-to-generic/value casts through an explicit
UnboxAny capability. The metadata IL generator and raw opcode authoring emit ordinary
CLI unbox.any or the existing native instruction. The runtime implementation is unchanged;
no native metadata version or new semantic category is introduced. Generic signatures
remain compiler-owned operands; native import/emission boundaries remain independent.

The cumulative unchanged Operators/SingleError and collection/union sources now compile,
import separately and execute OfType, Filter, Map, ToList and Single. The consumer checks
exact boxed integer extraction and shared reference identity (42, empty stdout); an
incorrect box faults with InvalidCast. 126 metadata C# groups and 15 focused Raven tests
pass. Runtime Contract selection uses the same explicit collection primitive core and
retained seed. Full .NET source-library adapters are still pending.

The unchanged broad sample now rejects Order[].Filter during binding and publishes no
output. Array participation in the configured iteration contract for extension receiver
inference/conversion is next. See [query evidence](experiments/extended-cli-metadata/query-import-2026-10-03.json)
and [broad assessment](experiments/extended-cli-metadata/query-broad-assessment-2026-10-03.json).

### Nominal array backing remains the integration contract (2026-10-03)

Author direction keeps arrays backed by Array<T>, with structural semantics deferred.
Unchanged source Array.rvn compiles into the cumulative library and a separate array
query consumer compiles against it. Runtime execution fails interface implementation
selection; it is not an end-to-end pass. The next integration task is explicit nominal
array descriptor/storage/iterator linking using canonical dependency identities.
The configured shape and source propagation selections are in `array-ownership.json`;
existing successful query acceptance configuration is unchanged. See
[nominal array evidence](experiments/extended-cli-metadata/nominal-array-assessment-2026-10-03.json).

### Explicit native nominal array backing (2026-10-03)

The array ownership manifest now drives the native emitter's SetArrayBacking selection:
only an output-owned declaration in the configured RuntimeIterationContract assembly is
eligible. The adapter uses symbol identity and builder ownership; it does not reopen
importer objects. Ordinary .NET emission and CLI vector signatures are unchanged.

The native assembly manifest optionally carries array_backing (module, revision, TypeDef
index). The linked runtime requires a unique concrete, unconstrained generic root record
with a single private T[] field. Vector interface closure and dispatch use that descriptor;
reading its storage field aliases the original vector, so source-authored indexers and
ArrayIterator<T> share mutation/identity. Nominal object allocation for the descriptor is
rejected; vector allocation remains the storage operation. Replacing/addressing the
backing field is unsupported. No name-based source-array inference or structural type
conversion was added. Without the selection, existing legacy array behavior remains.

This is optional format-5 native execution metadata, not a change to ECMA CLI vector
signatures. Older runtimes/readers reject the new field; deployments must pair compiler
and runtime revisions. Native immutable reads preserve the selection; general native
rewriting remains outside this bounded slice.

Reproduce using bootstrap/verify_source_unions.py --arrays with the documented compiler,
primitive storage core and runtime arguments. The separate consumer mutates through
MutableSequence<int>, observes the change through the original vector, then evaluates
Filter/ToList and returns 42 with empty stdout. Negative capacity, wrong unboxing, missing
library and duplicate seed checks also pass. C# authoring/reader tests and runtime checks
cover malformed identity, incompatible storage and duplicate backing selection; seven
legacy array tests pass. No broad application or .NET source-library completion is claimed.

Validation note: the guest RavenDoc snapshot check remains stale (the previously recorded
snapshot mismatch). This host-only API is covered by api-docs/experimental-metadata.md;
no guest API snapshot or website build is claimed by this slice.

Cross-repository evidence: Raven `85f2b2be6`; runtime based on `ced73e9d` plus this
slice (implementation and artifact hashes recorded in
[nominal-array-execution-2026-10-03.json](experiments/extended-cli-metadata/nominal-array-execution-2026-10-03.json)).
The unchanged broad application now fails before publication on the explicit retained
seed's missing Console.WriteLine(Int32) contract. That service overload is the next
bounded task. No application sources were rewritten.

### Unchanged broad native application gate (2026-10-03)

`bootstrap/verify_source_unions.py --application` separately compiles the cumulative
source library selected by array-ownership.json, then compiles unchanged
application-order-collections using only emitted native references. Exact checked-in
stdout and exit 0 pass, covering union propagation, maps/lists, callbacks, query composition,
vector iteration and mutation through shared object identities. The retained seed now
implements WriteLine(Int32) via existing Int32ToString and WriteLine(String) runtime
services, matching the current source Console implementation. Primitive CLI core and
retained seed remain explicit bootstrap dependencies; library/application imports are native.

The last linking gap was imported SingleError.ToString: native overrides use the existing
inherited-slot name, while authored references previously used ordinary method names.
Introspection now exposes the CLI NewSlot flag. Raven stores the inherited-slot fact as
IsOverride in its native method symbol; emission passes that semantic fact to the metadata
reference API without reopening the importer. The bounded public value ToString override
contract preserves managed receiver behavior. Ordinary .NET Reflection/Emit is unchanged;
no native format change or new runtime instruction was required.

A minimal separately imported union display consumer prints Multiple and 42; the broad
consumer checks all expected lines. Metadata tests reject conflicting and unsupported
reference contracts; the native value-override C# runtime fixture also tests metadata slot
facts and directly authored references. These are target-specific changes, not a shared
binding fix needing main-based extraction. Full paired .NET execution against rebuilt
source libraries remains the next gate, not a completed claim.

Validation: 127 metadata groups and 15 focused Raven external-signature tests pass.
The guest API snapshot check remains the previously recorded stale-snapshot failure;
the host API manual reference is updated. No website build or publication is claimed.

Cross-repository validation: Raven `80f57d2f9`, runtime `97cd6e90`, metadata changes
committed with [broad-native-execution-2026-10-03.json](experiments/extended-cli-metadata/broad-native-execution-2026-10-03.json).
The evidence records compiler payloads, source ownership, artifact hashes and exact
commands/output. The optional host API parameter requires rebuilding binary consumers.

### Executable .NET adapters and unit-storage assessment (2026-10-03)

`bootstrap/verify_dotnet_sources.py --compiler <rvnc.dll> --output <fresh-directory>` uses
ordinary compiler commands, .NET targeting-pack references, --emit-core-types-only and
the same array ownership manifest. Embedded-core mode avoids conflicting Raven.Core union
copies. A separate .NET service assembly provides CheckedStorage.Reserve<T> using CLR array
allocation and RuntimeFailure.Terminate using stderr plus process exit 1. The unchanged
System/Functions.rvn delegates System.Fail to that service. No collection/union declaration
or consumer body is replaced. Dedicated Raven consumers prove storage mutation (42) and
terminal failure (message, exit 1).

Compared with native reserved vectors, CLR arrays eagerly provide default-initialized slots;
the library tracks its used extent. This adapter does not reproduce native uninitialized-slot
faults. Terminate exits the host process and cannot be recovered by guest exception handling.
These bounded execution adapters are not published guest library APIs.

SingleError's nonnullable ToString exposed the shared binder's exact return-nullability rule.
Allowing the same underlying reference type to strengthen a nullable return follows the
[C# override contract](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/classes#1565-override-methods).
The isolated main-based fix is Raven 54fc1e0aa; integration is 1836ca9ef. Thirteen focused
semantic/execution tests pass, including weaker return rejection and value-nullable ABI
regressions. The unchanged native broad application is revalidated. No merge into main
was performed.

The .NET library emits but is not a valid execution gate: separate consumer emission fails
loading Propagatable<Option<T>,T,System.Void>. CLR void cannot be an ordinary generic/storage
value. Native inhabited Void and .NET unit representation need an explicit target mapping,
without modifying library sources or retargeting all primitive types to a fake core.
The current CLI RuntimeUnitContract couples the unit scope to TargetCoreAssemblyName;
relaxing that boundary needs focused design and roundtrip tests. The assessment also records
output publication after failure; compiler crashes are not accepted diagnostics or success.
Custom array-interface adaptation still needs .NET execution evidence after the unit blocker.

### Explicit CLR unit value and separately executed consumers (2026-10-03)

The .NET ownership manifest adds Unit with assembly NeoCLR.DotNetServices, type
System.Runtime.CompilerServices.UnitValue and MapClrVoidToUnit=true. Its Libraries catalog
must equal the native manifest's catalog (the driver asserts this). The adapter value is a
public empty readonly CLR struct. No fake core, input-source rewriting, native projection
fallback or metadata-format extension is involved.

The compiler opt-in binds source System.Void type syntax as unit in declaration and
expression contexts. An explicit assembly-scoped representation supplies emitted storage;
ordinary imported/no-result CLI void remains void. The existing unit projection now covers
interface and MethodImpl relationships as well as generic signatures/locals. Default
.NET and native profiles are unchanged. The compiler rejects invalid representation types
and incompatible core/target configurations. This bounded API extension changes the host
record constructor; binary consumers must rebuild.

The shared changes are isolated on Raven's main-based fix branch (eb5df24b1); integration
is 97d07b901. Both lines pass 22 focused C# tests, including normal unit behavior, out-unit
interface dispatch, metadata scope inspection and invalid selection diagnostics. Ordinary
commands separately compile and execute unchanged union, ArrayList, HashMap and query
consumers (42); adapter storage/failure tests also pass. Native application-order-collections
still matches its complete expected output with exit 0.

The .NET broad consumer compiles and prints the expected prefix through PrintPending's
303, then terminates with signal 10 on this macOS host at array query use. Semantic
ArraysImplementIterable metadata cannot make CLR vectors implement a custom interface.
The next step is explicit .NET array adaptation at conversion emission, retaining source
library definitions and shared element identities, plus rejection when an adapter is
unavailable. The full .NET broad gate is still failing; partial output is not completion.


### .NET emitter simplification (2026-10-03)

Raven `c2a66d82a` selects its established MethodBodyGenerator for all ordinary .NET
method bodies, removing the additional release-only portable adapter. Native emission
still uses compiler symbol operands, NeoCLR builders and GetILGenerator. Runtime Contract
configuration, primitive bootstrap, native reference identities and metadata encoding are
unchanged. The separate .NET unit/source-library compatibility experiment remains opt-in;
no CLR array adapter is added. Native module functions and nominal Array<T> backing retain
their existing runtime behavior; no further CLI projection is introduced.

Validation: 94 selected .NET checks pass before/after, and unchanged native
application-order-collections executes with its separately built source library. Shared
array lowering and the known per-iteration capture bug remain unresolved. See
[recorded evidence](experiments/extended-cli-metadata/dotnet-emitter-simplification-2026-10-03.json)
and Raven's `docs/compiler/architecture/neoclr-refactor-parity.md` for remaining work.


### Portable array-loop boundary (2026-10-03)

Raven `19a3e84c0` requests vector-for expansion from the portable planner rather than
ordinary shared lowering. .NET keeps its established loop emitter. Native vectors still
use nominal Array<T> backing, native import and builder/ILGenerator emission; no new
Runtime Contract option, CLI projection or metadata encoding is introduced. All 86
focused .NET tests pass; unchanged native application-order-collections and a labeled/
nested array-loop program execute successfully. The lexical closure-lifetime bug on main
remains unresolved, explicitly outside passing results.
[Evidence](experiments/extended-cli-metadata/portable-array-boundary-2026-10-03.json).


### Native field/property import cleanup (2026-10-03)

Raven `12b545bc1` consumes existing introspection FieldInfo/PropertyInfo views directly
and resolves their accessors to canonical Raven method symbols. Raven still owns language
accessibility policy. No Runtime Contract, CLI representation, public metadata API or
encoding changes. Definitions remain in other native declaration categories; no claim of
a completely facade-only importer is made. All seven native consumers execute before and
after; generic/external signatures, fields, indexers and private/static accessor contracts
remain covered. Ordinary .NET loading and emission are unchanged.

The caller inventory in Raven `docs/compiler/metadata-backend-boundaries.md` retains the
explicit primitive/runtime seed binding, CLI comparison probes and opt-in partial System
projection. Those are not fallback paths for native application/library references.
[Validation](experiments/extended-cli-metadata/native-member-facade-2026-10-03.json).


### Native callable facade adoption (2026-10-03)

Raven `015e6f66d` consumes introspection MethodInfo views for declared methods,
constructors and namespace/module functions. Type method/constructor lists are merged in
metadata-token order; constructors/accessors reuse canonical module symbols. The previous
definition-taking callable-symbol entry point is removed. This preserves module-function
semantics and requires no Runtime Contract option, CLI projection, public metadata API or
format change. Other type/union definition uses and explicit legacy bindings remain.
All seven native consumers execute with expected exit 42; C# checks cover constructor
identity across enumeration/member lookup, generic scopes, accessor identity and dependency
errors. Ordinary .NET loader and emitter paths are unchanged.
[Evidence](experiments/extended-cli-metadata/native-callable-facade-2026-10-03.json).


### Native type declaration views (2026-10-03)

Raven `003b9a38d` consumes its retained nominal view for names, arity, accessibility,
generic parameter names/positions and interface traversal. The matching host metadata
library adds GenericParameterTypeInfo.Name with explicit NotSupportedException for CLI
snapshots whose parameter names are not materialized. Native supported snapshots retain
declared names and owner-scoped identity. There is no new Runtime Contract, CLI projection
or format change; .NET loading and emission remain unchanged. Type/union transport still
uses definitions elsewhere. All 127 metadata C# contract groups and seven native consumers
pass. The manual host API reference is updated; the guest snapshot check remains stale.
[Evidence](experiments/extended-cli-metadata/native-type-facade-2026-10-03.json).

### Native type and union materialization cleanup

Native type materialization cleanup (2026-10-03): Raven `777499170` constructs ordinary
type and union/case/companion symbols from canonical introspection views and removes
obsolete raw-signature mapping helpers. The separately compiled source-union library
and unchanged application-order-collections pass, as do all seven native consumers.
Runtime Contract/bootstrap configuration, metadata format and ordinary .NET paths are
unchanged. The native gate is evidence for this cleanup, not full dual-target completion.
[Evidence](experiments/extended-cli-metadata/native-type-materialization-2026-10-03.json).

Physical metadata nesting remains distinct from Raven union membership. The importer
uses facade ownership to preserve both; emission still receives symbol facts and host
artifact identities. No new bridge encoding, fallback or public metadata API is added.
The known stale guest API snapshot and full dual-target gate remain open.

### Native catalog validation

Native catalog validation (2026-10-03): Raven `42503ec23` converts conflicts between
native snapshots and the explicit primitive bootstrap into RAVT003, including failures
while constructing the introspection catalog. A C# regression reproduces the previous
uncaught exception and now verifies both reference orders and unchanged output bytes/
position on failed emission. Seven native consumers pass. No Runtime Contract, format,
runtime or ordinary .NET behavior changes; prior broad-application evidence is reused.
[Evidence](experiments/extended-cli-metadata/native-catalog-validation-2026-10-03.json).

The metadata context owns exact identity checks; the native compiler adapter translates
invalid/unsupported catalog failures before semantic setup and publication. Native inputs
never fall back to a CLI projection. This target-specific correction requires no general
binder fix or new host metadata API. Full dual-target completion remains open.

### Native array interface receiver execution

Native end-to-end sample expansion (2026-10-03): Raven `3367f3200` fixes vector
receivers for projected inherited interface accessors, exposed by Array<int>.Count in
unchanged collection-capabilities. Existing reference conversion uses semantic symbols
and target capability checks; nominal Array<T>, metadata and runtime remain unchanged.
The source-library driver gate now includes a Count/mutation regression and three
unchanged samples (Option, propagation, collection capabilities), all verified/executed
with exact output/status alongside the broad order-collections application. Seven native
consumers and 13 focused .NET tests pass. Next bounded native task: no-result callback
import/emission for unchanged library-array-callbacks. Captured functions and missing
numeric/date source coverage remain separate gaps; full-library completion is not claimed.
[Evidence](experiments/extended-cli-metadata/native-array-receiver-2026-10-03.json).

Unlike CLR vectors implementing CLR collection contracts, native vectors dispatch through
the configured source Array<T> backing. Portable planning now emits the existing reference
conversion for an implicit array-to-interface receiver; the native adapter maps the semantic
operand without reopening importer objects. Bootstrap ownership and Runtime Contract
options do not change. No CLI bridge expansion, metadata API addition or format fork is
required. The ordinary .NET emitter remains in use; this portable-planner adjustment is a
deferred general candidate until an independent main-line caller is established.

### No-result callbacks and nested array execution

Native array callbacks (2026-10-03): Raven `35464aabf` preserves imported no-result
callback contracts separately from inhabited source unit and emits receivers through the
configured nominal array backing. The metadata library admits nested vector signatures
using existing CLI SZARRAY/native ArrayRef encodings; the runtime accepts exact nominal
backing casts without copying storage. Unchanged library-array-callbacks and a nonempty
nested-array mutation consumer now pass alongside the broad source-library application.
Validation: 127 C# metadata groups, 27 focused .NET tests, 19 runtime tests and seven native
consumers pass. Next bounded native gap: captured callbacks in library-list-filters.
[Evidence](experiments/extended-cli-metadata/native-array-callbacks-2026-10-03.json).

The importer uses a compiler-owned explicit no-result factory rather than the source
unit policy. Emission uses symbol contracts only. Runtime Contract options and bootstrap
ownership remain unchanged. This is the existing nominal callback transport, not structural
Function branch integration. Nested vectors match CLR jagged-array encoding; rectangular
arrays and covariance remain unsupported. Native array backing casts require the registered
nominal definition and exact element arguments, preserving the same storage. No format
version change, new opcode or CLI fallback is involved. Older readers may reject the newly
admitted nested vectors. Ordinary .NET loading/emission remains the established backend.
The known stale guest API snapshot remains open; host C# manual documentation is updated.

### Receiver-bound callback transport

Receiver-bound callbacks (2026-10-03): Raven `a53b6412a` and the metadata writer now
support owned nongeneric nonvirtual reference-instance callback targets. The runtime's
existing instance binding preserves shared mutable receiver identity, confirmed by CLR
and NeoCLR execution and a source method-group consumer of separately built ArrayList.Find.
All 128 metadata groups, 31 focused .NET tests, seven native consumers and the expanded
broad source-library gate pass. This is a captured-lambda prerequisite, not completion:
next work remains closure-frame lowering for unchanged library-list-filters, including
shared mutable captures and correct lexical lifetimes. No runtime/format-version change.
[Evidence](experiments/extended-cli-metadata/instance-callbacks-2026-10-03.json).

FunctionBinding and IILGenerator.BindFunction consume the object receiver for instance
targets. CLI emission uses that receiver with ldftn/newobj; native emission sets the
existing instance-target bit. Static calls remain unchanged. Targets must belong to the
output; generic, virtual/abstract, constructor and value-instance bindings remain rejected.
Raven consumes symbol operands through its separate emitter contract. Runtime Contract
configuration and bootstrap ownership are unchanged, and no importer objects enter emission.
No structural Function branch is integrated. The host API manual/XML is updated; the
known stale guest API snapshot remains open.

## Native reference capture milestone (2026-10-03)

Raven `623cbc1d8` emits private closure frames for immutable reference locals using
existing metadata instance Function bindings. Compared with CLR closures, this bounded
profile supports shared object mutation and escaped callbacks, but not shared reassigned
variables, value captures, parameters or receiver captures. These reject explicitly;
this is not a replacement for general lexical closure lowering. Ordinary Raven/.NET
emission and Runtime Contract/bootstrap configuration remain unchanged.

The array/application acceptance gate now runs unchanged library-list-filters against
its separately built native library and checks every output line. The additional
captured-reference-consumer checks escaped lifetimes, distinct factory invocations and
two callbacks sharing one object's mutations. A mutable reference binding rejects before
publication. Seven native consumers and 31 focused .NET function tests also pass.
Metadata/runtime are unchanged; the preceding 128-group metadata evidence is reused.
No public metadata API or website API snapshot changes are required for this compiler
slice. Structural Function experiments remain separate; the known stale guest API
snapshot is not regenerated by this change. Full-library completion is not claimed.

## Native primitive capture prerequisite (2026-10-04)

Raven `1b15715bf` admits immutable Int32, Int64, Boolean and Byte locals alongside
reference captures. Fresh frames copy these immutable values on lambda evaluation;
mutable locals, arbitrary struct values, parameters and receiver captures still reject.
The primitive-capture consumer imports FunctionEqualityComparer and HashMap from the
source-built library, checks an integer divisor policy, escaped callbacks, long/byte/bool
captures and distinct array-loop item captures (123). This is native execution evidence,
not a claim that the separately recorded .NET loop-capture issue is repaired.

The test also exposed missing Byte/Int32-to-Int64 promotion in portable binary emission.
Emission now follows the existing bound operator operand types, including mixed signed
comparisons, without changing binding or ordinary CLR emission. Relative to .NET this
restores the expected supported integer behavior, rather than adding a numeric semantic
divergence. No metadata writer/runtime change or new public API is required. Explicit
Runtime Contract, primitive core and source/seed ownership remain unchanged.

The expanded broad gate, seven native consumers and 37 focused C#/.NET tests pass.
Both mutable reference and mutable scalar captures reject without publishing output.
The full unchanged comparer sample remains blocked by missing StringComparer service
bindings and primitive CompareTo contracts; this focused case is not a replacement for
that sample. The promotion fix is recorded in Raven as a deferred general candidate.

## Source-built comparer and explicit primitive bootstrap (2026-10-04)

`comparer-ownership.json` extends the source library with unchanged
`System/StringComparer.rvn`; applications import the emitted artifact, not that source.
The new `--reference-comparer-storage-core` exporter mode supplies declaration-only
Int32.CompareTo, String.CompareOrdinalIgnoreCase and RuntimeServices.StringHashOrdinalIgnoreCase
on the primitive storage core. The executable `comparer-seed.neoil` supplies overflow-free
integer comparison and the case-folded hash adapter. Shared union seed adapters now
expose String equality/comparison operators and Object hashing/reference identity through
existing runtime services. No placeholder CLI body is executed. Full primitive source
compilation remains open, so these members remain bootstrap-owned and explicitly scoped.

Generate the core with the existing Probe build against the matching Raven compiler:

```sh
dotnet docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --reference-comparer-storage-core /tmp/ComparerCore.dll
python3 docs/experiments/extended-cli-metadata/bootstrap/verify_source_unions.py \
  --comparers --compiler /path/to/rvnc.dll --runtime target/debug/neoclr \
  --core /tmp/ComparerCore.dll --output /tmp/fresh-comparer-gate
```

Raven `7b3928239` emits the bound static operator call and explicitly admitted Object
hash slot. Metadata import maps only selected explicit core String/Int32 members to
intrinsic receiver storage; it does not import String as a nominal layout. String uses
its reference value and Int32 a managed address. Missing/nonvirtual Object hash slots,
wrong results, virtual primitive members and incorrect receiver modes reject. Native
and CLI encodings remain unchanged. The emitter still uses compiler symbols and host
artifact contracts; native library references have no fallback to CLI projection.

The focused string-comparer-consumer verifies UTF-8 scalar order (intentionally different
from .NET UTF-16 ordinal order), Unicode simple-folded equality/hashes, matching Object
hashes, map replacement and extreme signed comparisons. It passes along with the broad
application and seven native consumers. 128 metadata groups, dedicated native/static
String binding checks and 34 focused C#/.NET tests pass. Guest API snapshot validation
still reports the known stale snapshot; the host API manual reference is updated.

The unchanged full library-comparers sample now reaches an unsupported integer-range
BoundForStatement. It was not edited or replaced as an acceptance claim. Range lowering
is the next bounded task; general primitive source ownership and the larger guest
introspection library are separate subsequent work.

## Unchanged comparer sample execution (2026-10-04)

Raven `1fb1bbd45` removes the signed integer-range blocker. The `--comparers` gate now
compiles and executes unchanged library-comparers against the separate source-built
library, requiring exactly `Comparer contract passed` and exit 0. The original sample
was not rewritten. The focused range-consumer returns 42 after checking evaluation
order/once-only bounds, inclusive/exclusive ascending and descending loops, zero-step
and direction-mismatch empty loops, nested labeled continue, break, loop captures and
Int64 ranges. Ordinary .NET range emission remains the established independent path.

A subsequent comparer parameter-receiver gap required argument-address emission.
The metadata IILGenerator now offers LoadArgumentAddress(index) and raw Ldarga; CLI
and native writers share exact-type/slot validation. Static and instance parameters,
generic method scopes and mutation execute on both runtimes. Receiver slots, already
byref parameters, invalid indices (including unreachable code) and mismatched stores
reject. Runtime ldarga already existed, so no VM or format change was necessary.
The final Object.ReferenceEquals call required support for CLI ELEMENT_TYPE_OBJECT;
that signature now retains the output's explicitly supplied core identity, including
Object array elements. No implicit dependency resolution or nominal layout is inferred.

The comparer core, retained seed and ownership manifest are unchanged from the preceding
slice. CLI remains the primitive bootstrap only; source library/application references
remain native. 129 metadata groups, 55 focused .NET tests, seven native consumers,
native argument-address/binding checks and the expanded application gate pass. API
manual docs include the new host member; the separate guest API snapshot remains stale.
Signed range increment retains existing add semantics, with no new overflow policy;
unsigned/fractional ranges and broader captures remain outside this native profile.

## Native integer sample bootstrap (2026-10-04)

With compiler `1fb1bbd45`, unchanged `library-integers.rvn` now compiles and executes
through the ordinary native driver. The comparer-storage primitive core explicitly
adds `Int32.Equals(Int32)` and a nonvirtual `Int32.ToString()` declaration. The retained
seed supplies readonly byref receivers: equality uses `ceq`, formatting calls the existing
Int32ToString runtime service. The sample checks local and parameter receivers, equality,
comparison and formatting at both signed extrema. The expanded `--comparers` acceptance
gate requires exact stdout and exit 0, alongside the separate native library/application.

This is explicit primitive bootstrap ownership, not compilation of `System/Int32.rvn`.
As with existing bootstrap primitive methods, ToString is a concrete direct member;
it does not establish .NET's virtual Object-slot override or boxed dispatch semantics.
Declaration-only CLI placeholder bodies never execute. The eventual native primitive
source contract must preserve those distinctions explicitly. Application/library references
remain native, and metadata/runtime encoding and both compiler backends are unchanged.
No new .NET regression run is needed for this bootstrap-only slice; the preceding
55 focused tests remain the compiler baseline. The full dual-target library gate stays open.

## Native value-result receivers (2026-10-04)

Raven `9d06edf80`'s portable body planner now gives supported value-returning property/indexer getters
and ordinary calls a temporary local address for instance calls. It evaluates the
receiver once, before arguments, and never writes the copy back. Existing local,
parameter and field receivers retain their storage addresses; parenthesized receivers
preserve that distinction. Existing managed-reference/local-address capabilities govern
admission. Byref results and other unsupported expressions are not guessed into copies.
No importer objects or target-specific builders enter the shared plan. The established
.NET body emitter is unchanged; C# controls verify getter-copy versus field mutation.

NeoCLR's explicit primitive bootstrap now includes Int64.CompareTo with exact-width
readonly byref receiver validation. CLI uses the ordinary Int64 member reference and
native output uses the existing primitive owner form; no format or VM changes. This is
not source-built Int64 and does not add arbitrary primitive virtual dispatch. Hosts must
regenerate the comparer core and matching retained System seed together. Native library
and application references still use direct metadata import.

The new value-result-consumer executes against the separately compiled native collection
library: ArrayList<long> copy/indexer behavior, getter/call evaluation order, signed
extreme comparisons, and mutable-struct copy versus stored-field mutation all pass.
Expanded application acceptance, seven native consumers, 129 metadata groups, dedicated
native binding execution/rejection checks, three new C#/.NET checks and 26 existing
range/function checks pass. The unchanged full library-generic-collections sample now
rejects only because Date is absent, before output publication. Date's source depends on
larger globalization contracts; no stub or modified sample substitutes for it.

This portable planner extension is a deferred general candidate for independent
main-based validation when another backend consumes it. The current .NET emitter
already implements temporary receiver behavior; no .NET repair is claimed.

## Source-built Duration foundation on both targets (2026-10-04)

The native acceptance tool now provides `--calendar-foundation`, which extends the
comparer ownership manifest with unchanged `ComparableTo<T>`, `EquatableTo<T>` and
`Duration` sources. These declarations belong to the source-built library; the retained
seed and primitive core are unchanged. A native consumer imports only the emitted
library and exercises ArrayList<Duration> storage, default values, copying, iteration,
equality and signed-extreme comparison. The existing broad native application still runs.

The same value-contract consumer also compiles and executes on ordinary .NET against
an independently emitted library containing those three unchanged sources. The .NET
control uses the net10 targeting pack and installed Microsoft.NETCore.App with embedded
compiler shims, no reference-only runtime service stubs. Both consumers return 42 with
empty stdout. This validates a bounded common source subset, not full collection-library
parity. No compiler, metadata, runtime or guest API implementation changed in this slice.

A compile inventory of unchanged Date/calendar/globalization dependencies reaches
binding errors for missing string indexing, RuntimeServices.SystemCultureName and
RuntimeServices.UnixTimeToLocal. The latter produces cascading invalid-index diagnostics.
No output is published. These are the first observed blockers, not an exhaustive list of
emission/runtime gaps. Date is not replaced with a stub or a source-edited approximation.
Next work must give those primitive/service contracts explicit owners and executable
adapters while preserving the shipped grapheme-based Char/indexing contract; existing .NET behavior is the control.

## Native calendar service and grapheme-length bootstrap (2026-10-04)

The comparer-storage declaration core and retained seed now expose the existing
RuntimeServices.SystemCultureName() and UnixTimeToLocal(long) contracts. The latter
returns the runtime's legacy Int32 value array; the seed adapter copies its eight fields
into a fresh nominal array reference, following the existing translated-library adapter.
That explicit conversion costs one allocation and eight element copies per call; no
performance improvement is claimed. The primitive core contains metadata-only declarations,
while execution calls the real runtime services. Rebuild the core and seed together.

The base seed also implements String.get_Length through StringGraphemeCount. Native
String.Length counts extended grapheme clusters, unlike .NET's UTF-16 code-unit length.
The preceding Duration integration note's reference to preserving scalar indexing was
incorrect: the shipped Char/indexing contract is grapheme-based. Scalar traversal is a
separate API. This slice does not yet add String's indexer or native Char signature support.

The calendar-foundation consumer checks combining/ZWJ grapheme length, host culture
service invocation without assuming a locale, eight date/time fields, positive and
negative fractional Unix ticks, fresh independent array storage and the unchanged
out-of-range fault. Expanded native application acceptance and paired .NET/NeoCLR Duration
consumption pass. An old seed fails member validation before output publication.
The unchanged Date/calendar/globalization inventory now has only two string-indexing
binding errors; the missing services and cascading array-index errors are resolved.
Later emission/runtime gaps remain unassessed while binding fails. No compiler, metadata
or runtime implementation changed, and ordinary .NET emission is untouched.

## Direct native grapheme character imports (2026-10-04)

Raven `2bd39805c`'s native emitter now admits the configured core Char in imported native callable
signatures and resolves it through the explicit host artifact binding. This keeps
character signatures in the symbol-to-emitter path; no importer object is reused.
Metadata decodes/encodes canonical CLI CHAR while native output uses the existing
grapheme Char representation, including the intrinsic method owner. The primitive
core/seed expose String's indexer and Char.ToString; regenerate the comparer core.

A separately compiled CharacterContracts library returns and accepts char. Its separate
consumer indexes combining and emoji ZWJ graphemes and preserves their full text through
native import, calls and ToString. The broad native gate, paired Duration controls,
seven native consumers and 130 C# metadata groups pass; a dedicated C# native execution
check covers reimport, projection and invalid character aliases/receiver contracts.
CLI C# controls preserve UTF-16 code units, including surrogates. The ordinary .NET
emitter and runtime are unchanged; CLI declaration projections cannot carry native
multi-scalar grapheme values as executable CLR char values.

The unchanged Date dependency inventory now passes binding and reaches unsupported
BoundPatternAssignmentExpression emission (discard assignments after propagation).
A separate character-array receiver test explicitly rejects before publication: the
portable emitter still lacks array-element addresses. The successful character consumer
does not claim that capability. Both gaps remain visible; next is the Date discard path.

## Discard and propagation emission (2026-10-04)


Raven `a99152da0`'s portable planner now evaluates supported discard assignment operands and drops their
values, without popping a no-result call or emitting a discarded unit literal. General
lowering normalizes `_ = operand?` into the existing once-only operand/check/residual
return sequence; the unused success value is not loaded. This avoids teaching the target
emitter propagation semantics. Other pattern assignments remain unsupported in this path.

The focused native consumer imports the separately built Result library and checks that
success continues, failure returns early and side effects occur exactly once. The expanded
native application/character/calendar gate and paired Duration consumers pass. Eighteen
focused .NET/planner/propagation tests pass. The independent lowerer change and its tests
also pass six checks and are committed as `8e0f88eda` on the main-based
`codex/compiler-fixes-from-neoclr` branch; no experimental backend is needed for that fix.
The portable discard planner remains on the target integration line.

Unchanged Date/calendar sources move past discarded propagation but still reject an
unlowered propagation expression. A minimal `let value = Read()? + 1` reproduces that
remaining gap before output publication. Nested expression propagation is the next bounded
slice; Date execution and full-library completion are not yet established. No metadata,
runtime, bootstrap or public API changes accompany this slice.

The author-authorized backport is committed on Raven main as `22539952c`.
All six focused propagation checks pass there; only the general lowering fix, tests
and documentation were backported. No push or experimental emitter merge was performed.

## Eager binary propagation initializers (2026-10-04)

Raven `cb0f48bd7` lowers propagation nested in eager binary local initializers
into statement-level checks. Earlier operand values are saved before later operand
evaluation, including when a later operand mutates earlier storage. Success and early
failure execute through native Result imports from the separate source-built library.
The native consumer validates nested arithmetic, skipped operands/statements after
failure and field-value snapshots. The expanded native application gate and paired
.NET/NeoCLR Duration consumers pass.

Twelve focused integration tests pass. The general change is also committed on Raven
main as `7db0f3dfe`, with eleven focused tests passing there. No native emitter,
metadata, bootstrap, Runtime Contract or runtime implementation changes are required.
Short-circuit operators and arbitrary argument/other expression propagation are outside
this bounded initializer normalization.

Unchanged Date/calendar sources now reach metadata verification:
`Date.ToString: local loaded before store on some path`. No output is published.
Investigating that definite-assignment/control-flow failure is next; full Date execution,
array-element receiver addresses and the full dual-target library gate remain open.
See [execution and inventory evidence](experiments/extended-cli-metadata/nested-propagation-2026-10-04.json).

## Source-built Date/calendar acceptance (2026-10-04)

Raven `5c60425db` preserves the existing terminal System.Fail semantic fact in
portable emission: execute the original call and message, then guard an impossible
return with the existing compiler-failure instruction. CLI void signatures do not
encode non-returning behavior. This closes the failure branch of let-else without
default-initializing pattern variables or weakening metadata verification.
The existing identity policy is unchanged; unrelated Fail methods remain ordinary calls.
This target-policy fix stays on the integration line; no additional main backport is
needed. Sixty focused .NET identity/control-flow and shared planner checks pass.

The new `--calendar` acceptance option selects `calendar-ownership.json`: 43 unchanged
sources covering collections, unions, query, Date, Time, LocalDateTime, calendars and
globalization. The explicit primitive core and retained seed are unchanged; application
and rebuilt-library references still use native semantic import. The independent
consumer verifies leap-day creation, invariant formatting and AddDays. Unchanged
library-generic-collections now runs with exact output, including ArrayList<Date>.
Expanded application acceptance, dynamic-message failure checks and paired Duration
also pass. The complete calendar subset has not been executed on .NET.

Additional unchanged sample inventory: library-date-formatting compiles and runs with
its existing Hebrew/Gregorian expected output. library-globalization compiles but the
ordinary CLI run reaches InstructionLimitExceeded. The earlier bridge harness uses an
explicit larger runner budget for this sample. Next bounded work is making this budget
explicit for the native multi-assembly execution path and verifying the broader sample;
no optimization or performance improvement is claimed. Array-element receiver addresses
and full dual-target library coverage remain open.

No metadata/public API, runtime implementation or wire-format change is introduced.
See [evidence](experiments/extended-cli-metadata/calendar-source-2026-10-04.json).

## Explicit native globalization budget (2026-10-04)

The larger unchanged library-globalization sample completes through ordinary native
compiler/run commands with --instructions 1000000. The --calendar acceptance driver
checks its deterministic formatting lines and success marker, allowing host locale
lines to vary. It also executes the unchanged Hebrew formatting example, broad
application, generic collections and paired Duration controls. Six CLI tests pass,
including default-budget exhaustion, successful override and invalid-option rejection.

This exposes the existing Limits.instructions host setting, with the default still
100,000. Native metadata, bootstrap identities, compiler behavior and instruction
semantics are unchanged. The flag is run-only and applies across the supplied modules;
other resource limits remain unchanged. This is not a performance improvement.
Raven implementation revision remains 5c60425db (documentation dd179ac27).

The next native source inventory rejects clock/time samples because Clock/SystemClock,
TimeZone, Instant and related source declarations are not yet in the owned subset.
Start with coherent Clock/SystemClock dependencies, then expand instant/offset/time-zone
contracts; do not add consumer stubs. Full .NET calendar-library parity and array-element
receiver addresses remain open.
See [evidence](experiments/extended-cli-metadata/globalization-budget-2026-10-04.json).

## Source-owned Instant and clock integration (2026-10-04)


The native library ownership manifest now includes OverflowError, Instant, Clock and
SystemClock (47 sources cumulatively). The comparer primitive declaration core adds
only RuntimeServices.UnixTimeTicks; its matching retained seed forwards to the existing
host service. Regenerate the comparer-storage core and seed together. Old cores reject
the missing declaration; old seeds reject the missing method contract before output.

Instant.ToLocalDateTime now calls the source-owned internal
LocalDateTime.FromUnixTimeTicks directly. The old bridge-only RuntimeServices.LocalDateTime
alias already translated to that same factory. This removes a bootstrap declaration
that would otherwise reference a rebuilt-library type; it does not change the public
API or local-time semantics. Primitive bootstrap and imported library ownership remain
separate. No Raven compiler, metadata format or runtime instruction change is needed.

The unchanged library-clock sample runs through native Clock interface dispatch and
prints six valid local date/time components. A separate native consumer implements the
imported Clock contract, checks Instant value copies, signed extrema and Add overflow,
and invokes SystemClock through the interface. Full .NET clock/calendar execution is
not established; the paired Duration control remains the .NET source-library gate.

Legacy translated snapshot regeneration was attempted but --reference-library-core
fails in SourceUnionReferences.Project with RAV0103 ('None' is not in scope). No legacy
generated output or hashes were rewritten; its Instant source digest is now stale.
Native artifacts are rebuilt directly from source and do not consume that snapshot.


The complete --clock gate passes: calendar/globalization, broad native application,
generic collections and paired Duration remain green. Both existing runtime Instant/Clock
tests pass. Raven implementation remains 5c60425db; matching integration docs are
236b0f49c. No public API shape changed; on-site APIs and guest documentation signatures
are unchanged. The site time overview now distinguishes this native development gate.


## Source-built fixed offsets (2026-10-04)

The --offsets acceptance mode selects offset-ownership.json and adds unchanged
TimeOffset to the source-owned native library (48 sources). Its separate consumer
checks whole-second offsets through +/-18 hours, negative fractional Unix ticks,
value-preserving round trips, signed Int64 extrema and the exact civil boundaries
0001-01-01 through 9999-12-31. Values one tick beyond the shifted boundary reject
through the declared Result error. No host-local zone is consulted for these checks.

Raven bc30fb0f2 fixes portable static-property admission for imported value-type
owners, exposed by TimeOffset.Zero. It uses the existing external-value capability;
reference owners retain their separate capability. Seventeen C# external-signature
tests pass, including independently emitted property owners and ordinary .NET execution.
The shared portable layer is absent on main, so this fix cannot be cherry-picked
independently of that abstraction; no general binder/.NET behavior fix is held back.

The complete native offset/clock/calendar/globalization/application gate and paired
Duration controls pass. Reuse the Clock comparer core and seed; no bootstrap, metadata,
runtime, source implementation or public API changes are required. Full .NET
calendar/offset library parity remains unproved. Existing API signatures remain current;
no new RavenDoc selection or snapshot regeneration is needed for this slice.

The next source inventory adds the five time-zone declarations only for diagnosis.
It rejects before output because RuntimeServices lacks TimeZoneDatabaseVersion,
TimeZoneExists, SystemTimeZoneName, TimeZoneOffset and TimeZoneMapLocal; further
index/case errors follow the unresolved signatures and are not yet independent blockers.
Next wire those explicit services, including nominal Int64 array mapping, then resume
unchanged time-zone consumers. The legacy translated snapshot's union-reference
regeneration failure and array-element receiver addresses remain open.

### Combined source array contracts (2026-10-04)

Raven must not cache a missing source-owned array shape during declaration setup.
Deferring that lookup and provisional array-interface caches allows unchanged
Utf8.Encode to return imported byte[] as source-owned Sequence<byte> in the same
57-source native library. Arrays retain nominal Array<T> backing; no structural
array semantics, service stubs or metadata-format changes are introduced. The
primitive CLI core remains the explicit temporary service signature source; rebuilt
libraries use native metadata import. See the combined-library-array evidence and
System compilation strategy for commands, limits and remaining initializer binding.

### Scheduling callback ABI (2026-10-04)

The generated primitive core exposes ScheduleTask(Action) and DrainEntryTasks().
Action transports a no-result function signature, `fn<noresult Void>`; native unit
callbacks with `fn<Void>` remain a separate accepted ScheduleTask overload. The
runtime retains the bound receiver and verifies the exact function signature.
No Raven emitter special case or Object-boxing bridge is introduced. The metadata
library's existing function signature encoding preserves this distinction. Native
helper library/consumer execution verifies mutation after explicit drain; the full
source Tasks/Workers gate still requires enum and queue-ownership integration.

#### Native source enum gate (2026-10-04)

`bootstrap/verify_enums.py` (relative to the extended-cli-metadata experiment) accepts
`--compiler --runtime --core --seed --base-library --ownership --output`. Use a matching
57-source bundle and its ownership manifest. It compiles unchanged TaskState plus
`enum-contracts.rvn`, imports only that library into `enum-consumer.rvn`, and verifies
native execution; the same sources compile and execute through ordinary .NET commands.
Both exit 42 with no stdout. Evidence includes source/artifact hashes and revisions.
The native target explicitly admits top-level Int32 enums. Symbol facts and artifact
identities author references; emission does not access importer objects. Ordinary .NET
Reflection/Emit behavior remains unchanged. Flags/other widths/nested enums are explicit
limits, and full source Tasks/Workers still requires queue ownership integration.


### Native source-owned Tasks/Concurrency (2026-10-04)

The [task gate](experiments/extended-cli-metadata/bootstrap/README.md#source-built-tasks-and-concurrency-2026-10-04)
uses a separately emitted six-source library and native semantic import. Typed generic
queue services preserve source-owned identities; legacy nominal queue services remain
supported. Runtime task atomic regions recognize matching source type/method origins,
and draining resolves the registered queue's exact methods. Constructed/interface
callbacks use the metadata IL generator. Native Function types are signature-based;
Func/Action are only Raven/CLI transport. Unit-return adapters explicitly invoke a
no-result callback then produce unit, retaining receiver identity with an allocation cost.
The matching Raven branch is codex/metadata-consumer at 598b16be7 (pre-slice parent 8407c122f).
No full async frontend or full-System claim follows from this bounded native gate.


### Native Self contract pipeline (2026-10-04)

Raven's bootstrap ownership manifest can now select its existing RuntimeSelfTypeContract
through `self: { assemblyName, typeName }`. The checked comparer/storage core includes
System.Runtime.CompilerServices.Self. Exact identity is validated; missing configuration
for a native Self signature rejects explicitly. Introspection supplies Self facts to
symbols; emission writes native Self from the exact configured semantic marker, with
no importer reuse. The marker is temporary semantic transport, not a nominal native type.

The actual System.Clonable source, a separately compiled implementation and an artifact-only
consumer now execute with exit 42 and independent clone mutation. Metadata conformance
substitutes the implementing owner into local/external Self contracts. Native SelfType
encoding is unchanged. This bounded gate uses concrete calls; erased-interface and
constrained generic Self dispatch, static Number members and primitive-source ownership
remain open. The .NET Reflection/Emit backend is unchanged. See
[reproduction](experiments/extended-cli-metadata/bootstrap/README.md#native-self-contracts-2026-10-04)
and [evidence](experiments/extended-cli-metadata/self-native-2026-10-04.json).

Compiler slice: Raven `0014a1241`; matching metadata/bootstrap changes and evidence are
committed on neoCLR `codex/extended-cli-metadata`.


### Checked numeric parser family (2026-10-04)

All eleven existing parser RuntimeServices signatures are selected in the native catalog
as String -> Value; exact binding tests cover 37 total declarations. Native runtime grammar,
payload/status conventions and metadata formats are unchanged. The separately compiled
actual Boolean/BooleanParseError source library executes true/false/error cases alongside
byte/Int32/Int64 boundaries and the other parsers' invalid-format results. Wider primitive
success payloads, static Number/Self emission and complete source-owned primitives remain
open. The source nominal Boolean API and canonical bootstrap bool stay distinct.

Raven's shared primitive preference now honors MetadataImportOptions.CoreAssemblyName.
The bug was independently reproduced on .NET main; fix f749c1a75 and 20 focused tests
are integrated there without the experimental target. [Evidence and reproduction](experiments/extended-cli-metadata/bootstrap/README.md#source-boolean-parsing-and-numeric-service-family-2026-10-04)
identify the matching bundle. Ordinary .NET defaults remain System.Runtime.


### Source floating primitive ownership (2026-10-04)

Unchanged Single.rvn, Double.rvn and NumberParseError.rvn now compile into a native
library and execute through a separate artifact-only consumer. The explicit ownership
manifest selects nativePrimitives by canonical name and owning library. Raven retains
the declared CLI bootstrap while compiling the provider, checks its sole private
mutable m_value field, and emits scalar receiver operations rather than record storage.
Consumers select native declarations consistently for keyword, namespace and metadata
name lookup. Missing providers fail before publication. Emission authors references
from symbols and host dependency identities/digests, independent of reader objects.
The metadata writer preserves canonical scalar dependency names for authored references.

This matches the CLR distinction between primitive signatures/storage and ordinary
value types; allowing an explicitly selected external native provider is a NeoCLR host
configuration, not a .NET replacement rule. No runtime instruction or schema changed.
Generic Number-constrained calls, integer source providers and complete numeric-family
acceptance remain open. Run bootstrap/verify_native_floating.py with explicit compiler,
runtime, core, seed, base-library, number-contracts and ownership artifacts. It verifies
parsing payloads/errors, NaN ordering, scalar methods, identities and arrays (exit 42,
empty stdout), and failed missing-provider publication. See native-floating-2026-10-04.json.
38 focused .NET metadata-import/Self/operator/interface controls and 138 C# metadata
contract groups pass. The guest API snapshot remains the recorded stale artifact;
this change updates the development C# API manual and introduces no guest API.


### All numeric source implementations (2026-10-04)

The cumulative source-library subset now builds with unchanged Number, all ten numeric
structs, NumberParseError and IntegerDivisionError under one native owner. A separate
consumer imports that artifact without sources and executes parsing boundaries/errors,
identities, ordering, integer formatting, checked division and floating arrays/NaN cases
(exit 99, empty stdout). This goes beyond compiling declarations; runtime linking and
verification succeed without seed-owned Int32/Int64. Generic Number-constrained calls
remain open, so this is not the completion of the Number story.

Replacing seed primitive owners requires rebuilding their existing consumers. Loading
the old collections artifact alongside a new numeric provider correctly rejects its
undeclared dependency. The fixture rebuilds the actual cumulative source subset rather
than relaxing direct-reference rules. Provider emission maps exact bootstrap member
contracts to output-owned source methods. Object remains the bootstrap anchor.

Primitive members now retain canonical runtime names (System.Int32.ToString, etc.).
Native readers also accept the earlier encoded member names and preserve accessor
associations. Numeric interface matching can use the metadata declaration name when
an interface executable name is encoded; signature and accessibility checks remain.
This preserves the CLR-like distinction between declaration identity and executable
representation without introducing a new runtime instruction or metadata schema.
The checked service catalog adds existing Int32ToString/Int64ToString bindings, and
numeric-seed.neoil retains no duplicate numeric declarations.

See numeric-source-family-2026-10-04.json and bootstrap/verify_native_numbers.py for
commands, hashes and source ownership. The metadata C# suite covers canonical and
legacy names/accessors; focused runtime interface tests cover semantic name matching
and inaccessible implementation rejection. The stale guest API snapshot is unchanged;
no guest public signature changed.


### Static constrained-call prerequisite (2026-10-04)

The reduced Raven repro `Sum<T>(left: T, right: T) -> T where T: Number => left + right`
binds successfully against the rebuilt numeric library, then rejects with NEOMETA001
at callable-declaration admission. Inspection found two independent gaps: the portable
callable filter rejects method constraints, and the metadata IL generator exposed only
concrete instance constrained calls. Do not relax the filter until bounds are preserved.

The latter gap is now closed. The existing CallConstrained operation also accepts owned
static interface contracts on owned nongeneric class/value implementations. Raw Emit
uses Call for static and Callvirt for instance methods. Both use the same validation
path. Self substitutes the implementing type in stack arguments/results. CLI uses its
standard constrained./call pair; native uses existing nonborrowed callself. No runtime
or schema change was necessary. The C# suite executes .NET dispatch; ordinary and
primitive-Self native artifacts both verify/run with exit 42 and empty stdout.

The next slice is method generic bounds across definitions/builders, CLI GenericParam
and GenericParamConstraint, native function constraints, reader snapshots and introspection.
Then admit those semantic bounds through the explicit NeoCLR capability and lower
static interface calls with a method-parameter implementing type. Imported bounds and
reference-only generic consumers must be tested before claiming generic Number support.
The importer/emitter boundary and ordinary .NET backend remain unchanged. Baselines:
138 metadata groups and 20 focused Raven Self/static-interface tests pass. See
static-constrained-2026-10-04.json for artifact hashes and executable evidence.


### Method-bound preservation checkpoint (2026-10-04)

Owned nongeneric method interface bounds now survive builder/definition authoring,
standard CLI GenericParamConstraint rows, native function TypeBound records, snapshots
and introspection. Raven obtains canonical constraint symbols through the facade,
including inherited interfaces, and its existing binder rejects incompatible arguments.
The focused probe also confirms constrained-call emission fails before publishing bytes.
No importer objects are reused by the emitter and no Reflection/Emit behavior changes.

C# validation covers CLI execution and native/legacy projection round trips. The
API-generated native bounded method verifies and returns 42; seven existing native
consumers still execute. See method-interface-bounds-2026-10-04.json. The next bounded
work is external interface bound identity, then open constrained-call operands and
semantic authoring in Raven's emitter. Keep its constrained callable rejection until
these contracts are complete; this checkpoint does not claim generic Number execution.


### External method bounds checkpoint (2026-10-04)

Method constraints now retain external nongeneric interface identities through standard
CLI TypeRefs, existing native TypeBound records and reader snapshots. Introspection
resolves only through its explicit catalog and rejects missing/wrong dependencies and
noninterface definitions. The experimental constraint record now carries TypeReference;
local definition authoring remains supported. No runtime schema change was necessary.

Both targets execute the API-authored separate-contract bounded-method fixture with
exit 42. Raven's direct importer preserves canonical external bounds and its existing
binder rejects incompatible arguments. Seven native consumers remain passing. See
external-method-bounds-2026-10-04.json. The legacy reference-only projection explicitly
rejects external bounds; it is not a fallback. Primitive bootstrap and runtime seed
configuration are unchanged. Open constrained calls, imported bounded-method authoring
and Raven's constrained emission admission remain the next implementation work.


### Open constrained method calls (2026-10-04)

The metadata IL generator can call owned static interface declarations through bounded
method parameters. Typed and raw overloads validate the method scope and direct/inherited
local interface bound. Self is substituted with the method parameter in stack signatures.
Standard CLI constrained./call and native callself already encode the operation; no
runtime changes were needed. API-generated ordinary and native Self/Double fixtures
verify and execute with exit 42. C# metadata checks pass 140/140 groups.

See open-constrained-methods-2026-10-04.json. The next work is external interface target
operands, followed by Raven semantic constraint authoring and open-call lowering behind
an explicit capability. Its CallableSignature admission still rejects method constraints;
do not relax that guard until the emitter can retain bounds and dispatch correctly.
The .NET backend, explicit primitive bootstrap and runtime seed selection are unchanged.
The C# API manual is updated; the existing stale guest API snapshot remains unresolved.


### Generic Number end-to-end gate completed (2026-10-04)

The Number feature now passes through ordinary compiler commands with direct native
references. The gate rebuilds the cumulative class-library subset plus all ten unchanged
numeric sources as Numbers.dll, compiles native-number-algorithms.rvn as a separate
library, and compiles native-number-generic-consumer.rvn with only emitted references.
NeoCLR verifies and executes all ten instantiations, checking +, -, *, /, Zero, One,
inherited ComparableTo<Self>.CompareTo and constrained generic forwarding. Exit 42 and
empty stdout are required. The earlier parsing/boundary/array consumer still returns 99.
A string type argument fails binding with RAV0320 and publishes no output.

Method bounds are emitted from semantic symbols through an explicit native capability;
the importer remains independent. Shared linear calls carry compiler-owned method/type
operands. The native adapter uses metadata IL-generator calls, with managed addresses for
constructed instance dispatch. External numeric ownership and interface conversions are
explicit semantic facts. The default .NET Reflection/Emit implementation is unchanged.
CLI uses its existing constraint tables and constrained instructions; native uses existing
TypeBound/callself behavior. No runtime schema fork, new arithmetic semantics or performance
claim was needed. No independently useful binder fix was introduced for main backport.

Run bootstrap/verify_native_numbers.py with the same explicit primitive core, numeric
runtime seed and cumulative ownership manifest as the preceding source-family gate.
It records source/artifact hashes, revisions, commands and failures. See
number-generic-end-to-end-2026-10-04.json for this run's evidence. This completes the
Number numeric-family story; it does not claim full class-library compilation or broader
generic constraint categories. Generic owner-parameter forwarding, special method bounds,
structural Function experiments and replacing .NET Reflection/Emit remain separate work.


### Source-order-independent numeric library binding (2026-10-04)

Raven 459856a71 fixes provisional conversion caching during declaration binding. Native
source intent, primitive ownership, Runtime Contract settings, temporary CLI bootstrap,
metadata encoding and importer/emitter separation are unchanged. The 70-source numeric
library now builds with an empty file first and in reverse order; separate numeric,
generic Number and unchanged broad application consumers execute. The 76-source Tasks
combination also compiles and its artifact-only consumer executes. The shared .NET
regression is independently fixed and validated on main at f0c3b75a0, with 139 tests
passing on each line. [Evidence](experiments/extended-cli-metadata/source-order-conversions-2026-10-04.json).
Next capability batch remains source-owned String/Char and common text-service binding;
full System and complete rebuilt-library dual-target execution remain open.


### Checked text-service family (2026-10-04)

The explicit native service catalog now wires 20 existing text operations from the
same signature inventory used by legacy translation. Char and UInt32 scalars/vectors
are admitted by the catalog; vector results are copied into native managed arrays.
String equality uses native value comparison; existing seed host functions are reused.
Native intent is Unicode graphemes and UTF-8 strings, not CLR char/String internals.
The CLI primitive core only supplies signature transport; executable native seed
bindings call the existing runtime services. No compiler, native format or runtime
implementation changes are needed by this prerequisite. No .NET text parity is claimed.

Separate native library/consumer execution includes the unchanged UnicodeScalar source,
character-array mutation and UTF-8 success/error payloads. The expanded-seed numeric
gate also passes. An incomplete old core rejects with RAV0117 and publishes no output.
Canonical source-owned String/Char storage remains open; actual String reaches its
explicit interface Count-property rejection. The eventual replacement is native source
primitive ownership and accessor emission, not additional CLI application projections.
[Evidence](experiments/extended-cli-metadata/text-services-native-2026-10-04.json).

## Local inheritance driver checkpoint (2026-10-04)

Raven `f38dbfb75` selects local nongeneric inheritance through an explicit native
capability. Semantic base identity and constructor calls stay compiler-owned; native
builders emit existing CLI Extends/native base relations and correct inherited storage
indices. No importer handles cross into emission, and no application CLI projection is
introduced. Primitive core, retained seed and Numbers ownership remain explicit inputs.
The equivalent source executes on ordinary .NET. The general constructor-binding fix
is separately validated on main-based `2416a1646`; main is not merged by this checkpoint.
[Gate, limitations and hashes](experiments/extended-cli-metadata/class-hierarchy-foundation-2026-10-04.md).


## Local assignment propagation checkpoint (2026-10-04)

Raven `eb83baa4d` normalizes direct/eager-binary propagation in local assignments
through shared lowering. The reduced native consumer verifies and returns 42 using
the existing explicit seed, source-built libraries and ownership catalog. No bridge
encoding, target configuration or metadata changes. The independent fix is integrated
into main at `9faabb1a2` with 24 focused .NET tests; its temporary branch is removed.
See [evidence and remaining JSON gap](experiments/extended-cli-metadata/local-assignment-propagation-2026-10-04.md).


## Conditional propagation and JSON library checkpoint (2026-10-04)

Raven `3a99915c8` normalizes conditional propagation through shared lowering, with
independent main integration `e1df355a2`. No bridge encoding, metadata API or Runtime
Contract changes. The five unchanged JSON sources compile, verify and serve a separate
native DOM consumer returning 42. The public serializer still needs introspection
and reflection services in its dependency catalog. [Gate and limitations](experiments/extended-cli-metadata/conditional-propagation-json-2026-10-04.md).


## Native internal JSON codec gate (2026-10-04)

The unchanged document reader/writer now executes Unicode round trips, shared mutation
and invalid-input/cycle rejection through a test-only entry point compiled beside the
library. Separate artifact-only consumers and explicit JSON ownership manifests pass.
No bridge mapping, compiler or Runtime Contract changes are introduced. The public
serializer remains pending native introspection/handle dependencies. See the
[gate and next dependency sequence](experiments/extended-cli-metadata/source-json-codec-2026-10-04.md).


### Native source-owned internal calls (2026-10-05)

Raven consumes the configured core's explicit MethodImpl(InternalCall) marker for
internal nongeneric functions in neoCLR.Runtime. Runtime implementations are selected
by exact native signatures; ordinary extern/PInvoke and .NET emission are unchanged.
The marker's CLI bootstrap constructor never executes. Runtime handle-service source
now lives in runtime/raven/native and is compiled beside its source-owned callers.
Public wrappers can be separately imported. Public/generic/type-owned service
contracts and production descriptor factory/snapshot ownership are still pending.
See [the integration and validation record](experiments/extended-cli-metadata/source-internal-calls-2026-10-05.md).

### Native parameter arrays (2026-10-05)

Final by-value array parameters now preserve ParamArrayAttribute across native import
and emission. Runtime Contract core/seed selection must provide the canonical marker;
CLI is the primitive bootstrap only, not a fallback for application references.
[Implementation and executable evidence](experiments/extended-cli-metadata/parameter-arrays-2026-10-05.md).

### Native mapper control-flow gates (2026-10-05)

Raven's portable adapter now emits the existing reference null test/unbox operations for
null-coalescing guards and boxed primitive patterns. A coalescing return fallback is
bounded to a local initializer with an empty evaluation stack. Terminal return wrappers
in match lowering are normalized without applying unreachable conversions. Runtime
Contract configuration is unchanged; the ordinary .NET backend is untouched.
`bootstrap/verify_mapping_guards.py` in the metadata experiments compiles a library and
artifact-only consumer and verifies/runs to 42, including both Result match branches.
Eight focused .NET pattern checks also pass. This is a prerequisite, not the JSON gate.

### Inherited descriptor interface methods (2026-10-05)

The metadata writer searches local base classes for exact public interface methods.
This admits RuntimeMemberInfo's inherited methods on concrete source descriptor providers.
CLI virtual flags cover only selected implementations. Native hierarchy dispatch already
supports this relationship; no new runtime instruction or Runtime Contract mapping is added.
152 C# contract groups pass; `--inherited-interface-image <path>` writes the focused native
consumer, which verifies and executes to 42. CLI execution also returns 42. Source descriptor
materialization and the production JSON mapping gate remain open.

### Final reference classes and Boolean ownership (2026-10-05)

Metadata now preserves ordinary CLI Sealed flags on authored reference classes and
rejects sealed base inheritance. Raven must retain finality when emitting source models:
JSON validation distinguishes these from open and abstract classes. Explicit Boolean
primitive ownership uses the existing scalar representation; it neither adds a new
element code nor changes the .NET backend. C# contract tests cover builder/manual
parity, CLR finality, native facade round trips and invalid bases.

### Source-built JSON mapping execution (2026-10-05)

The [artifact-only JSON gate](experiments/extended-cli-metadata/source-json-mapping-2026-10-05.md)
now passes using production sources and source-owned descriptors. Runtime snapshots are
materialized against scoped service result signatures, including owned arrays and the
internal ParameterSnapshot adapter; property setters execute through exact InternalCall
contracts. Raven preserves finality, explicit auto-accessors and bound conversion calls.
Boolean and typeof ownership are explicitly selected in the generated manifest. The
primitive core and retained seed remain bootstrap dependencies, with no CLI fallback
for application libraries. Compiler commit 8f015386e and metadata df6d8e51 are required.
The linked record includes commands, hashes, .NET controls and the explicit execution
budget; it supersedes the earlier statements that production mapping remains open.

### Source primitive member binding (2026-10-05)

The explicit current-output primitive set now selects source member declarations while
retaining bootstrap scalar signatures. This fixes String.SliceUtf8 lookup without
adding source members to the CLI bootstrap or changing ordinary .NET lookup.
Raven de5645e96 passes 25 focused tests. The cumulative 109-file source library and two
artifact-only native JSON executions pass; [evidence and limits](experiments/extended-cli-metadata/source-primitive-members-2026-10-05.md).

### Cumulative library type budget (2026-10-05)

The 115-file cumulative System subset now emits after raising the host authoring/native
reader bound from 256 to 4,095 declared types, matching the existing CLI reader's 4,096
rows including the module row. No Runtime Contract, guest instruction or format version
changes are needed. Other resource budgets remain enforced; use the existing library
container profile for large outputs. Boundary tests and native execution evidence are
recorded in the [cumulative-library type-budget integration note](experiments/extended-cli-metadata/cumulative-library-type-budget-2026-10-05.md).

### Source storage execution (2026-10-05)

The remaining eleven production storage sources now compile against the cumulative
115-input library using explicit native service adapters. The unchanged storage-poc
sample executes from emitted references: creation, UTF-8 reads, seek/re-read,
listing, child lookup and error cases. An additional FileText consumer checks bounded
writes/reads and preservation of file contents on rejected writes. This closes the
seven missing storage service declarations from the post-JSON audit.

Raven's portable emitter preserves empty-stack context through value-block wrappers;
the metadata writer admits external value types in module-function references using
existing encoding. Neither change alters Runtime Contract configuration, ordinary
.NET semantics or primitive/source ownership. The next high-unlock family remains
DNS/socket service contracts and their imported Error identities, followed by full
source/bootstrap ownership. No complete-System or dual-target gate is claimed.
See `docs/experiments/extended-cli-metadata/source-storage-2026-10-05.md` for exact
artifacts, commands, source hashes and validation.

### Source DNS/socket execution (2026-10-05)

Six production DNS/IPAddress/socket sources now compile into a separate native library
with explicit service adapters. The unchanged network-cancellation sample executes
against that artifact and the cumulative source library: localhost DNS, pre/pending
cancellation, loopback accept/connect, transfer buffers and resource reuse all pass.
Raven admits converted temporary value receivers and immutable by-value parameter
captures through its existing adapters; 59 focused .NET checks pass. There is no new
Runtime Contract switch, instruction encoding or nominal delegate representation.

The apparent imported Error-to-Error diagnostics disappear once service declarations
are present; no union identity workaround was added. The full networking/web source
group now reaches an unlowered BoundPropagateExpression. Prioritize Result propagation
in HTTP next, then reassess full-source/bootstrap ownership. This is native execution
evidence, not .NET class-library or full-System completion. See
`docs/experiments/extended-cli-metadata/source-network-2026-10-05.md`.

### HTTP condition propagation (2026-10-05)

Shared Raven lowering now handles nested propagation in HTTP's short-circuit conditions.
A native artifact-only consumer executes skip/success/error paths for AND/OR, including
one-time side effects. Integration validation passes 21 propagation tests and 65 focused
shared-emitter/runtime-contract tests. No runtime, metadata format or ownership change
is needed. Full HTTP source compilation advances to callback emission admission; that
is the next bounded task before HTTP execution. Full-System bootstrapping remains open.
Evidence: `docs/experiments/extended-cli-metadata/condition-propagation-2026-10-05.md`.

### Generic HTTP callbacks (2026-10-05)

The native adapter now retains method arguments when binding owned generic callbacks,
including HttpClientJsonExtensions.Convert<T>. Metadata authoring uses existing
GenericMethodInstance/ConstructedMethodReference identities: native function.bind keeps
generic_arguments; CLI ldftn uses MethodSpec. No new format version or Runtime Contract
switch is required. Imported callback targets remain unsupported. Ordinary .NET retains
its established emitter. Complete HTTP/network sources emit with the source-network
bootstrap and adapters; artifact-only header and base-address consumers execute.
The C# metadata suite passes 154 groups, including generic callback execution/readback.
This is not full-System bootstrap completion.

### Captured receivers and lexical access (2026-10-05)

Raven's native closure frames now capture reference-type self and retain lexical nesting
through existing NestedClass metadata. Implicit fields and explicit self calls load that
capture, preserving object identity. Runtime private-access checks walk resolved enclosing
definitions with a depth bound; unrelated peers do not gain access. No member visibility
is widened, no new Runtime Contract or metadata version is needed. This follows the
.NET nested-type access model ([Microsoft documentation](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/classes-and-structs/nested-types)).
Value-type self, mutable-local capture and generic closure owners remain unsupported.

The native status Server.rvn consumer executes ten loopback requests: eight valid
status responses and rejection of two invalid responses. A minimal self-capture consumer
checks deferred execution, private field mutation and a private method call on the same
object. Runtime nesting/accessibility tests also reject unrelated callers. Main's .NET
closure implementation is unchanged; portable-only compiler changes need no main backport.

### HTTP property patterns and enum Object calls (2026-10-05)

Reference property patterns now evaluate each getter once, short-circuit in source
order, reject null and bind payloads. Integer/Boolean/enum constants are supported.
Value receivers of inherited Object calls box explicitly; the runtime recognizes
Int32-backed enum metadata for formatting/equality/hash behavior. No representation
extension or new Runtime Contract is introduced. The native property-pattern consumer
and HTTP status consumer execute; 59 existing shared-body .NET tests and one focused
property-pattern control pass. The shared propagation fix is also on local Raven main
as e33591945, independently validated with 31 tests; the temporary branch was deleted.

Focused enum display/equality/hash tests pass. An existing Object-array-display test
fails with "type parameter index outside arguments" both before and after this change;
record it as separate nominal Array<T>/Object dispatch debt, not an enum regression.

### String-array entry points (2026-10-05)

The ordinary native compiler already authored Main(string[]) from symbols; metadata
validation now admits that CLI-shaped signature. Native startup supplies a managed
String array of user arguments using existing bounded array storage. Environment argv
retains the input path; Main excludes it. No wrapper, new internal call, metadata version
or Runtime Contract configuration was added. C# tests and a produced native image pass;
the unchanged eleven-case stream-upload source now compiles and executes with URL/mode
arguments. Native name-based entry ambiguity rejects explicitly. Ordinary .NET startup
remains unchanged.

### Source Object ownership boundary (2026-10-05)

The [root-ownership investigation](experiments/extended-cli-metadata/object-root-ownership-2026-10-05.md)
identifies a required explicit selection across bootstrap configuration, semantic root
identity, writer boxing/slot authoring and runtime admission. Merely recognizing any
loaded System.Object grants application lookalikes intrinsic behavior. That attempted
relaxation was reverted; retained seed ownership and compiler configuration remain
unchanged. Runtime controls now cover binary seed/application loading and invalid
ownership. Source Object binding/emission remains an open prerequisite.
