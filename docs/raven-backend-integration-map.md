# Raven backend integration map — slice 1

Inspected 2026-09-12. This is a source-backed investigation, not a working neoCLR
compiler target or a binary-format decision. See the [experiment plan](raven-target-experiment.md).

## Pinned inputs and scope

- Raven checkout: `/Users/robert/Projects/Raven`, revision
  `d92b02812740ae052f277c23151e9cc208f7672d`, clean at inspection.
- neoCLR: `3140a2a4bd61713ae3cde41a5c94f72673e5998f` on
  `codex/raven-neoclr-target`, clean before this documentation slice.
- Raven compiler/core projects target net10.0 and net11.0. Its global.json pins Raven
  MSBuild SDK packages to 0.1.12, **not** a .NET host SDK. The locally selected SDK was
  11.0.100-rc.1.26425.128; this records availability, not a successful build.
- Raven.CodeAnalysis declares MetadataLoadContext 9.0.0, Mono.Cecil 0.11.6 and
  Microsoft.CodeAnalysis.CSharp 4.12.0. A Roslyn package reference does not mean Raven
  delegates its normal backend to the C# compiler.

Raven was read only. Its repository compiler-investigation skill and test-impact map
were consulted. No compiler build, execution, dependency restore or Raven test run was
performed: this slice changes only neoCLR documentation. Source-level test coverage
below means test bodies were inspected, not rerun. No existing binaries were used as
proof of the pinned source's behavior.

All Raven source links below are pinned to that revision, making the map independent
of the local checkout path.

## Findings that change the starting assumptions

Raven already contains a **target-core-library identity retargeting hook**. Reuse and
test it before inventing another emission switch. It is not a complete runtime target:
it does not independently remove .NET framework discovery, rewrite all source semantics,
provide neoCLR metadata, or implement a neoCLR loader.

The normal backend uses PersistedAssemblyBuilder, Reflection.Emit types/builders,
System.Reflection.Metadata/ManagedPEBuilder, followed by Mono.Cecil normalization.
A binary CLI-compatible declaration facade is therefore a concrete reuse candidate.
A wholly custom metadata format would require either such a facade or a new metadata
importer and a deeper backend adaptation. This is evidence for the next format probe,
not sufficient evidence to select the final format.

The nearest immediate semantic mismatch for HelloWorld is **Void stack behavior**:
Raven can lower source Unit returns to CLI void, whereas neoCLR's current Void is an
inhabited value. The call/return boundary must explicitly account for that difference;
retargeting System.Void's assembly identity cannot change the stack contract.

## Trace from source to artifact

| Stage | Observed implementation | neoCLR integration implication |
| --- | --- | --- |
| CLI/project input | [Compiler Program](https://github.com/marinasundstrom/raven/blob/d92b02812740ae052f277c23151e9cc208f7672d/src/Raven.Compiler/Program.cs) parses target-framework, references, Raven.Core and emit/publication options. [CompilerWorkspaceFactory](https://github.com/marinasundstrom/raven/blob/d92b02812740ae052f277c23151e9cc208f7672d/src/Raven.Compiler.Core/CompilerWorkspaceFactory.cs) also resolves target frameworks for its API path. | Account for both entry paths; a neoCLR target needs an explicit reference environment, not only a final output filename. |
| Framework discovery | [TargetFrameworkResolver](https://github.com/marinasundstrom/raven/blob/d92b02812740ae052f277c23151e9cc208f7672d/src/Raven.CodeAnalysis/TargetFrameworkResolver.cs) searches installed .NET reference packs, normally Microsoft.NETCore.App.Ref. | Provide target library paths/policy without silently importing a whole .NET framework. Host libraries needed to run the compiler are distinct from target libraries. |
| Imported symbols | [MetadataReference](https://github.com/marinasundstrom/raven/blob/d92b02812740ae052f277c23151e9cc208f7672d/src/Raven.CodeAnalysis/MetadataReference.cs) creates PE references; CreateFromImage writes an image to a file. [Compilation.Setup](https://github.com/marinasundstrom/raven/blob/d92b02812740ae052f277c23151e9cc208f7672d/src/Raven.CodeAnalysis/Compilation.cs#L949) builds MetadataLoadContext and seeds host core/trusted assembly paths. | A neoCLR JSON module is not directly importable. Core identity and symbol resolution must be tested for host leakage even with a target assembly supplied. |
| Bind/lower | Compilation creates BinderFactory and source assembly/module symbols. CodeGenerator.EnsureLoweredBoundNodes requests lowered bound views and handles top-level async lowering. | Parsing/binding are reusable candidates, but lowering already assumes runtime facilities in some features. Do not assume every lowered node is target-neutral. |
| Emit entry | [Compilation.Emit](https://github.com/marinasundstrom/raven/blob/d92b02812740ae052f277c23151e9cc208f7672d/src/Raven.CodeAnalysis/Compilation.Emit.cs) checks diagnostics and invokes CodeGenerator, with a separate macro-plugin path. | A target must reject unsupported guest constructs while keeping any host compiler/plugin execution separate. Macros are outside HelloWorld scope. |
| Types and bodies | [CodeGenerator](https://github.com/marinasundstrom/raven/blob/d92b02812740ae052f277c23151e9cc208f7672d/src/Raven.CodeAnalysis/CodeGen/CodeGenerator.cs#L1406) creates PersistedAssemblyBuilder using EmitCoreAssembly; source/member mappings use TypeBuilder, MethodInfo and ConstructorInfo. | Backend adaptation extends beyond changing opcode constants. A CLI subset may preserve more of this path. |
| Instruction abstraction | [IILBuilder](https://github.com/marinasundstrom/raven/blob/d92b02812740ae052f277c23151e9cc208f7672d/src/Raven.CodeAnalysis/CodeGen/IILBuilder.cs) abstracts labels/locals but accepts System.Type, MethodInfo, FieldInfo and Reflection.Emit.OpCode, plus exception blocks. | This is a useful seam, not a target-neutral IR or complete interchangeable backend interface. |
| Binary serialization | CodeGenerator calls GenerateMetadata, then ManagedPEBuilder with IL, metadata, entry point and debug directory; it also produces portable PDB data. | Existing binary machinery is real. Determine which generated tables, helpers and bodies the neoCLR subset must accept; source simplicity does not prove output simplicity. |
| Final retargeting | [AssemblyReferenceNormalizer](https://github.com/marinasundstrom/raven/blob/d92b02812740ae052f277c23151e9cc208f7672d/src/Raven.CodeAnalysis/CodeGen/AssemblyReferenceNormalizer.cs) uses Cecil to normalize core references and rewrite metadata-method proxies. | Treat this stage as part of the target contract; final MemberRef owners and signatures must bind to neoCLR, not just final AssemblyRef names. |

## Existing retargeting support and its limits

[EmitOptions.TargetCoreLibraryIdentity](https://github.com/marinasundstrom/raven/blob/d92b02812740ae052f277c23151e9cc208f7672d/src/Raven.CodeAnalysis/EmitOptions.cs)
is passed into CodeGenerator. The driver accepts `--target-core-library <path>`, validates
a managed assembly and reads its AssemblyName, then adds/replaces a metadata reference
by identity and supplies the emit option. Normal framework-reference construction still
exists; this is not a no-framework or neoCLR mode.

CodeGenerator.WriteFinalPe selects RetargetCoreLibraryReference when that option is set;
otherwise it uses ordinary System.Runtime normalization. The target branch also has a
metadata-method proxy path for non-generic imported methods. The generic-method case
is excluded from that special path; this does not establish that all generic calls fail,
but requires a separate probe before relying on arbitrary target generic methods.

[AssemblyReferenceNormalizerTests](https://github.com/marinasundstrom/raven/blob/d92b02812740ae052f277c23151e9cc208f7672d/test/Raven.CodeAnalysis.Tests/CodeGen/AssemblyReferenceNormalizerTests.cs)
contains emission tests for retargeted core identity and matching portable PDBs. The
empty-main test targets an mscorlib identity while compiling with normal test references.
That establishes the intended retargeting surface in test code, not neoCLR binding or
execution. The next probe must resolve a real target Console method and inspect final
references, then distinguish that from successful execution on neoCLR.

## Semantic assumptions to address

| Contract | Evidence in Raven / neoCLR | Treatment for the minimal target |
| --- | --- | --- |
| Void versus Unit | [TypeSymbolExtensionsForCodeGen](https://github.com/marinasundstrom/raven/blob/d92b02812740ae052f277c23151e9cc208f7672d/src/Raven.CodeAnalysis/TypeSymbolExtensionsForCodeGen.cs) has explicit Unit-as-void signature/body mapping; CodeGenerator can create a Unit struct shim. neoCLR calls return a real Void value. | Specify a binary import/lowering rule or compatible target emission; test call result discard and Main return. Do not conflate System.Unit, CLI void and neoCLR Void. |
| Class/value representation | Type generation and symbol-to-CLR mapping use IsValueType, Object, byref CLR types and constructor/member builders. | HelloWorld avoids user-type construction, but generated entry containers/helpers may still have type/base references. Inspect actual emitted closure of types before defining loader scope. |
| Free functions | Source functions become methods in the emission machinery; PE entry point is a MethodDef token. neoCLR has free functions as well as owned methods. | Import a bounded generated static method/container or deliberately lower it to a free function; retain a mapping for identity and diagnostics. |
| Framework projections | [standard.json](https://github.com/marinasundstrom/raven/blob/d92b02812740ae052f277c23151e9cc208f7672d/src/Raven.CodeAnalysis/FrameworkProjections/standard.json) projects Int32.Parse to Result using catch-to-result over .NET exceptions, and TryParse to Option. | These are not neoCLR's native Result APIs. Add a target policy to bind direct neoCLR contracts; never enable exception lowering just because source uses Result. |
| Exceptions/cleanup | Statement/Expression generators emit throw, exception blocks and finally. | Reject unsupported guest paths explicitly. Identify generated cleanup too, not only user-written try/catch. Define Result migration separately. |
| Boxing/interface access | Expression generation emits box in several conversion/call paths; neoCLR uses explicit managed interface views. | Do not reinterpret box as a no-op. Defer those constructs or give conversions a precise lowering; test later user-type/interface slice. |
| Core/library shims | Driver discovers Raven.Core; CodeGenerator binds or embeds Unit and other required helpers. | Inventory generated dependencies. Decide which remain compiler helpers and which map to neoCLR types, without loading Raven.Core's .NET implementation into neoCLR accidentally. |
| Generics and nullability | CLR Type-based mapping and existing framework projections encode current .NET assumptions. | Defer broader generic Result/Option and nullable support until identities, effective signatures and initialization are specified. Keeping language syntax is not enough. |

## Minimal neoCLR library path

[System manifest](../runtime/System.neoil) expands into the current System module.
[Console.WriteLine(string)](../runtime/System/Console.neoil) loads its argument, calls
neoCLR.Runtime.WriteLine(string), and returns. The [native declaration](../runtime/neoCLR/Runtime/WriteLine.neoil)
marks that function InternalCall. neoCLR's [loader](../src/lib.rs) currently parses JSON;
it does not load Raven's PE output.

A first source probe should contain only an explicit Main and a literal WriteLine:

```text
import System.Console.*
func Main() {
    WriteLine("Hello from Raven on neoCLR")
}
```

This is a proposed minimal Raven probe, not executed source in this slice. Avoid the
repository's interactive hello-world sample initially: it exercises nullable input,
Parse projections, unions and string interpolation beyond the first contract.

Test the target library in two distinct roles: compiler-readable declarations, and
neoCLR-executed implementation. A declaration facade may be PE/CLI while the runtime
library remains neoIL during an intermediate probe. That proves symbol integration
only, not the final binary-library milestone. Do not redirect the call to host Console
and label it neoCLR execution. If exporting all System initially is too broad, derive
a bounded subset from the real declarations and document how identities remain aligned.

## Slice 2 recommendation: specify and probe before selecting the format

1. Define identities/signatures for System, Object if required by generated containers,
   String, Void/Unit, Console.WriteLine and the native runtime binding. Inventory actual
   generated types/methods using a minimal Raven emission, not a guessed opcode list.
2. Exercise the existing target-core-library hook with a small declaration assembly.
   Check bound symbol owners and final PE references, including host-core and Raven.Core
   leakage. Compare with the normal .NET output as a control. Run retargeting tests first
   using Raven's prescribed build/test workflow and record the actual host SDK.
3. Decide how no-result CLI calls/returns map to inhabited Void, and how an unsupported
   signature/opcode/exception region is rejected. That decision precedes loader coding.
4. Then compare a bounded CLI artifact path with a custom neoCLR artifact plus facade
   or new importer. CLI reuse is the leading candidate from this source inspection;
   reference/Unit/extensions may still require a versioned target contract. No format
   is selected in slice 1.

No universal class/value redesign is required to perform these probes. If a probe exposes
a runtime incompatibility, evaluate it explicitly rather than accumulating compiler-only
workarounds. Preserve the distinction between host compiler execution and target execution.

## Validation and unresolved evidence

Verified source paths, pinned revisions, clean checkout state and the inspected code/test
entry points. No executable target result is claimed. The scope is documentation-only,
so no compiler baseline was run. The next slice must establish a targeted emit baseline
before code changes, following Raven's test-impact map and build instructions.

Still unverified: exact HelloWorld metadata/body inventory, minimal reference-facade
closure required by MetadataLoadContext, retargeted Console binding, default embedded
helper set for that input, and whether PE emission can represent the selected neoCLR
contracts without deeper changes. These are the concrete questions for slice 2.

## Follow-up: tested emission probe (2026-09-12)

Slice 2 now provides the [minimal contract and results](raven-minimal-target.md).
The targeted compiler build and Console retargeting probe passed. The missing-library
negative exposed host fallback; the output also retains a fixture mscorlib dependency.
This supplies the HelloWorld inventory left open above, without establishing a closed
core library, target isolation or runtime execution. Existing retargeting xUnit tests
were not run; the standalone control/fixture probe is the executed baseline.


## Closed address hierarchy constructor checkpoint — 2026-09-25

The author selects a closed IPAddress class hierarchy instead of the previously
planned value union. An isolated neoCLR probe compiles with current Raven but
exposed a target importer gap for protected base constructors. The bridge now admits
a direct `call` from a derived constructor to its immediate base's protected
constructor. Private constructors and unrelated protected calls remain excluded;
this is not a general protected-member admission change. No Raven compiler source,
Runtime Contract setting or emission policy changes. The address probe checks
immutable copied data, root/Object value equality, hash agreement and GC rooting.
Public IPAddress projection, parsing, formatting and DNS/socket integration remain
pending. This is a target integration checkpoint, not a general Raven fix.


## Public address hierarchy and DNS migration — 2026-09-25

neoCLR integrates a closed IPAddress root with sealed IPv4Address/IPv6Address
implementations, typed parse errors and ordinary managed parsing/formatting. DNS
returns Sequence<IPAddress>; Socket adds value overloads alongside strings. The core
projects the permitted hierarchy and the bridge validates its root/leaves/signatures
and rejects external metadata branches. Source/reference/library/callers must be
rebuilt together. No Raven source, Runtime Contract or emission policy changes.

The focused prototype observed a MetadataLoadContext crash for a private helper
returning Result<byte[], string>; filling caller-owned temporary storage avoids it.
This remains a reduced target observation requiring independent general validation,
not a compiler fix. Byte.ToString selected an Object path that faulted in neoCLR;
numeric formatting widens octets to int. Treat that as a separate runtime/library
investigation, not evidence of a general Raven defect. Typed DNS/echo and independent
HTTP-server checks pass; API and generated snapshots are kept current.

### HTTP framing bridge checkpoint — 2026-09-25

HEAD factories/client overloads and private response-framing helpers are projected
through the existing neoCLR target bridge; no Raven compiler or Runtime Contract
configuration change is needed. The class-library import bound rises from 128 to
256 reachable methods to accommodate HTTP overloads and transport machines. Application
imports retain 128 and private transport members remain inaccessible to consumers.
An internal response encoder uses an integer head-only flag because the bridge's
nonpublic argument conversion currently handles only one materialized argument;
no public API carries that workaround. The same frozen compiler SDK builds the
library and consumer/reference snapshots. See the [framing design](http-client-design.md#bounded-response-framing-and-head--2026-09-25)
and focused HTTP fixtures for limitations and evidence.

### HTTP context and configured responses — 2026-09-25

The target bridge now projects HttpContext (including Disposable conversion), Accept,
Complete, configuration-only Respond methods, text convenience and token-bearing
ServeOne. Internal context constructors/sending and server lifecycle helpers remain
inaccessible to consumer imports. The new callback adapter/accept operation are private
library types. Runtime Contract settings and compiler emission policy are unchanged;
private wrappers bind cancellation delegates within the imported library rather than
requiring direct delegate binding to a reference-only core method.

The frozen compiler exposed a separate limitation in a stress fixture: discarding a
propagated `Result<unit, E>` after await leaves a unit stack value on a later resume
merge, including `_ = await operation()?`. The fixture explicitly asserts completion
results instead. The user-facing sample propagates typed Accept failure and returns
Complete's Result directly. Record the discarded-unit lowering case for independent
Raven reduction/validation; no bridge stack checks are weakened and no compiler fix
is claimed here. Future runtime suspension does not make invalid current CIL acceptable.


The cancellation fixture's original combined Main/nested-callback shape also failed
in the frozen compiler with `Missing local builder for 'pending'`. Separating observer,
connect and read callbacks into ordinary functions avoids that emission failure.
This is recorded evidence, not an isolated general root cause or a compiler fix.
The lifecycle stress test uses a counter observer rather than a collection of nested
Task/Result values, which remains outside the current importer's collection profile.

The separated captured-callback variant subsequently reached execution but faulted
reading a null captured pending task. The final cancellation probe keeps operation
state in explicit instance fields and uses method-group callbacks; both cancellation
and shutdown cases pass. The earlier capture failure remains a compiler/bridge
investigation candidate, not evidence of a diagnosed GC bug or a completed fix.

The [captured-callback source and reproduction command](experiments/http-context/repros/README.md)
are retained for that follow-up and are explicitly excluded from passing sample claims.

### Memory stream / JSON DOM I/O — 2026-09-25

The bridge now admits the provisional System.IO.MemoryStream constructor and methods
with conversions to InputStream, OutputStream and SeekableStream. This is ordinary
managed storage/interface dispatch; no compiler semantics or Runtime Contract change.
The JSON adapter remains application code using the existing tree codec, Result and
text wrappers. It is not reflection-based object mapping.

The frozen compiler accepts separate range guards but failed parsing a combined
expression mixing multiple less-than and greater-than comparisons. The implementation
uses separate guards (also matching Console stream validation); no parser fix is
claimed. Empty explicit init was replaced by a constructor initializing the cursor
after the failed parse produced a duplicate-constructor diagnostic. This was not
isolated as an independent constructor defect. Focused target/.NET/GC evidence is in
[JSON streams](experiments/json-streams/README.md); signature checks cover member and
capability admission. Public API docs and bootstrap snapshots accompany the change.

### Public JSON DOM hierarchy — 2026-09-25

The author selected a closed JsonValue hierarchy with kind-specific APIs. The bridge
projects its six leaves and ClosedHierarchyAttribute, rejects external branches,
and admits only the explicit public signatures and leaf-to-root conversions.
Parser/string-codec helpers remain internal; member metadata is checked against the
managed implementation before execution. JsonError uses standard Raven union syntax
and the existing source-projected payload-union path, now cataloguing HttpError and
JsonError. No manual case ABI, Runtime Contract option, compiler semantics or native
runtime instruction was introduced.

The public consumer imports the matching reference and System.neoil without compiling
private JSON copies. It tests typed construction, borrowed text/byte streams, nested
error causes and collection. A corpus compares JSON acceptance and checked Int32
conversion with .NET 10. The measured fixture runner accepts guest arguments after
`--` so corpus documents can be batched under an explicit instruction budget rather
than reloading the runtime once per document. This affects tooling only, not CLI or
runtime default resource policy. See [JSON DOM evidence](experiments/json-dom/README.md).

The pinned compiler still requires the existing explicit discard for propagated
unit results in synchronous code. Reflection/object mapping, general class-family
admission and Raven-main policy changes are not part of this slice.

### Managed entry arguments — 2026-09-25

The collection-profile bridge accepts static nongeneric no-result Main(string[])
through a generated parameterless adapter. It copies the host vector excluding
its executable element; empty startup gives an empty managed array. No Runtime
Contract setting or runtime entry ABI changes. Other parameter shapes and direct
result-returning entries remain rejected. Three argument/GC cases and signature
checks pass. Website build skipped as directed.

See [contract and fixture](experiments/entry-arguments/README.md).

### Object and scalar WriteLine — 2026-09-25

WriteLine(object?) dispatches virtual ToString; null writes an empty line. Direct
Boolean, Char and integral overloads avoid boxing. Signed/unsigned 64-bit decimal
formatters are private runtime services, also used after native-sized conversion;
small integer overloads widen to Int32. Those services are classified as string
operations. No compiler setting or Runtime Contract configuration changes.

Compared with [.NET Console.WriteLine](https://learn.microsoft.com/en-us/dotnet/api/system.console.writeline?view=net-10.0),
object/null handling follows the same basic contract. neoCLR offers exact overloads
for narrow/native integers too, uses invariant decimal output, and Char is grapheme
text rather than a UTF-16 code unit. There is no formatting-provider contract.
The cost avoided is temporary boxed scalar allocation; output still creates text.
Floating-point formatting remains deferred; its current Object fallback does not
promise numeric output. This does not broaden ToString semantics for other types.
