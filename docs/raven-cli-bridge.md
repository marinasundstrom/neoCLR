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
