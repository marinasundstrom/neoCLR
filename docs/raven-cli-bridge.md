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
