# Native core bootstrap frontier (2026-10-08)

The release requires a compiler target that consumes neoCLR metadata without a
.NET semantic bridge. Native output and a standalone executable are not sufficient
proof. This reduced experiment addresses native core production and compiler initialization,
then executes a small consumer; it is not a complete runtime library or release demo.

## Writer correction

Previously `AssemblyBuilder(identity, identity)` could produce native payload data
but PE/#Neo wrapping failed with “external assembly identity collides with output
identity”: the reference projection unconditionally imported its core as external.
For native reference emission with an explicit authored Object root, core references
now use local TypeDef handles. Missing local declarations reject; executable CLI
emission and foreign imports colliding with the output identity still reject.
The reference-only CLI projection remains a container/tooling view, not a compiler
semantic input in this experiment. No format version or artifact naming change.

This follows the existing CLI metadata architecture: TypeDefOrRef permits local
TypeDef references for bases and signatures, and MemberRef parents may be TypeDefs.
See [ECMA-335 sixth edition](https://ecma-international.org/wp-content/uploads/ECMA-335_6th_edition_june_2012.pdf),
Partition II §§22.25 and 22.37 (consulted 2026-10-08). It avoids manufacturing an
external self-dependency; it does not change IL semantics or claim a performance gain.
An alternative is retaining an external bootstrap core, which remains supported but
cannot satisfy the author's no-bridge target gate on its own.

## Reproducer and remaining work

`Program.cs` authors a small core directly through the native metadata API, imports
it and a separate native library through `NeoClrReferenceCatalog.ReadNative`, and
compiles an integer consumer using the catalog’s references and emission dependencies.
No C# reference assembly is generated or passed to Raven. The .NET process hosts the
compiler and metadata tools; compiler hosting is separate from target bootstrapping.
The current core uses the explicit `NativeCore` identity and remains deliberately
incomplete. It is not a full primitive/core contract.

```sh
dotnet run --project docs/experiments/native-core-bootstrap/Probe.csproj \
  -p:RavenRoot=/absolute/path/to/Raven \
  -p:NeoClrMetadataProject=/absolute/path/to/neoclr/tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj \
  -p:WarningLevel=0 -- /tmp/fresh-native-core-probe
```

### Initial failure and native-only follow-through

At Raven `bc3c500e6`, the original three-primitive probe wrote its core artifact but
reported **RAVT004** during .NET metadata-session initialization. The
[initial evidence](validation.json) remains historical. It exposed a compiler
composition requirement rather than a failure to decode native metadata.

Raven commit `10dce0c3b` adds the explicit compiler API mode
`MetadataImportOptions.WithNativeMetadata()`, which now
bypasses that session. The current fixture uses its own `NativeCore` identity,
explicit unit/Object contracts and native semantic references only. It includes
Byte, Int32, Int64, Double, Boolean, Void and String plus minimal Object slots and attribute support.
These are test fixtures; constant hash/display/Equals bodies are not production Object
behavior. `consumer.rvn` exercises a static call into a separate native library,
integer arithmetic and return. No fixture
Object method is selected by the AOT closure.

The probe checks symbols, emission, missing-core rejection, exact core-version
mismatch and default CLI emission refusal. A separate executable verifier assembles
an empty native System seed to avoid a competing default Object, then runs the consumer
in the interpreter and ARM64 native code. Both produce integer 42; the CLI interpreter
returns process exit 42. The C harness checks the native result/status and exits zero.
The executable links only macOS libSystem.

```sh
python3 docs/experiments/native-core-bootstrap/verify.py \
  --raven /absolute/path/to/Raven \
  --runtime target/release/neoclr \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --output /tmp/fresh-native-core-execution
```

[Execution evidence](execution-validation.json) records commands and artifact hashes.
The [catalog execution evidence](catalog-execution-validation.json) records the
follow-through using `ReadNative` and a separate library. The earlier execution report
retains the direct-reference arithmetic-only baseline. Focused Raven catalog checks
cover immutable snapshots, replacement, optional XML, dependency emission, missing
dependencies, duplicate identities/paths and CLI/malformed input rejection; the old
CLI-bootstrap catalog remains supported and its regression probe passes.

The driver now exposes this path with `--native-core-reference Core.dll`, repeated
`--reference Library.dll`, and the usual output/source arguments. Pass optional
`--driver /absolute/path/to/rvnc.dll` to verify.py to compile through that command and
run the result in both modes. [Driver evidence](driver-execution-validation.json)
records parity and rejection of mixed bridge flags without output publication.

The bounded project path is now implemented (2026-10-09): select
`RavenNeoClrNativeCoreReference` instead of RavenNeoClrCoreReference, with
RavenTargetPlatform=NeoCLR and RavenMetadataFormat=NeoCLR. The shared project
provider uses native semantic references and watches the core input. The native
core supplies Object and unit ownership; optional RavenNeoClrRuntimeSeed is only
an execution input. Mixed bridge settings reject. Complete source-runtime
bootstrapping still uses the CLI core path; next test native core declaration
completeness before changing the full bundle. Installed VS Code is not qualified
by the project provider checks. RavenDoc now has an explicit native input provider reusing the same catalog. Build it
with the metadata project property and use `--native-core-reference` plus explicit
`--reference` dependencies, or `apiInputs`/`nativeCoreReference` in site configuration.
The [documentation qualification](ravendoc-validation.json) uses two native libraries
with actual declaring identities, existing member rendering, sidecars, navigation/search
and input rejection. This closes the loader issue, not the complete production bundle
migration. The website keeps its full existing API reference until native inputs pass
its public-API coverage checks.

## Validation

[Recorded evidence](validation.json) distinguishes the successful writer checks from
the original compiler frontier; the new execution report records its bounded resolution. The dedicated `--native-core` test checks local bases,
reference marker ownership, no external/self references, native primitive roundtrip,
determinism and missing-declaration/root rejection. Existing Object-root controls
exercise external-core output. API snapshot checking is independent of target execution.

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- --native-core
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- --object-roots
```

Full source-library bootstrap, editor and Windows qualification remain pending. The module foundation does not satisfy these bootstrap gates.

## Native-only project qualification (2026-10-09)

Pass `--driver /path/to/rvnc.dll --project` to the executable verifier above.
It writes a project with relative core/library/seed paths, compiles it, runs it
through the project driver and independently compares interpreter/native result 42.
It checks that a mixed native/CLI core selection leaves the published output intact.
The [project evidence](project-execution-validation.json) records exact inputs.

Raven's `NeoClrMetadataProbe --native-core-project CORE LIBRARY OUTPUT` additionally
checks native-only semantic references, module binding, emission, watched core,
runtime paths and nine invalid configurations. Existing bridge-backed project
controls also pass. This is a compiler-target prerequisite, not full core/library,
editor packaging or release qualification. No new IL semantics or performance
claim is introduced. The native catalog research and alternatives above still apply.

## Value-type foundation and production union frontier (2026-10-09)

Native semantic loading now recognizes the selected core's System.ValueType as the
special value-type base. The previous nominal symbol existed but had SpecialType.None,
causing the portable type planner to reject ordinary structs and union carriers.
A same-named non-core type remains ordinary; no host fallback is introduced.

Pass `--driver /path/to/rvnc.dll --project --value-types` to verify.py. The
[value consumer](value-consumer.rvn) mutates a struct, copies it, mutates the copy
and checks the original stays 40 before returning 42. Project run, independent
interpreter and ARM64 native execution agree, with libSystem-only native linkage.
The fixture now includes Byte storage for union tags. This still has deliberately
minimal Object/String behavior, not production System.Runtime implementations.

The verifier also compiles unchanged production Propagatable, Option, Result and
UnionAttribute sources. Initial attempts rejected the value-type contract, then the
missing Byte tag, then crashed while synthesizing String.Concat. They now report
RAV1501 and publish no union library. The shared compiler fix also passes 14 focused
ordinary .NET interpolation/union checks, including missing-member rejection and
normal formatting. [Value/frontier evidence](value-foundation-validation.json)
contains source hashes, commands and diagnostic output.

This follows CLR's nominal ValueType classification and explicit runtime-member
requirements, reusing the native core/CLI comparisons above. It changes compiler
classification and failure reporting, not IL execution semantics or performance.
Next provide real native String/core member contracts and return to these production
sources; do not substitute constant formatting to turn the fixture green.
The general missing-Concat repair is a main-line candidate: its helper/tests match
main, but its emitter-boundary port still needs independent validation on main.

## Native text services and module projection (2026-10-09)

`verify.py --driver /path/to/rvnc.dll --project --text-services` selects an
optional fixture core with native grapheme Char and real runtime-service wrappers
for String.Concat, String.CompareOrdinal and instance GetByteCount. The last is a
fixture test member, not a new public String API. The default fixture still checks
the missing-Concat diagnostic. No CLI semantic reference is introduced.

The [text consumer](text-consumer.rvn) checks Unicode concatenation, empty operands,
embedded NUL, ordinal inequality and instance UTF-8 byte count. Project run,
independent interpreter and ARM64 native code return 42; the native executable
links only libSystem. [Evidence](text-services-validation.json) records source/tool
hashes and commands. Native execution uses existing UTF-8 kernels and a bounded
caller-owned arena; it does not replace the production String implementation.

This exposed an AOT projection regression: stripping a primitive method's type
owner created an assembly function with an empty module name. Preserve its canonical
System module for static, borrowed primitive and String instance projections.
Original-scope verification and declaration-module validation remain mandatory;
missing-owner diagnostics now identify the module and assembly. Compared with CLR,
this is an internal AOT projection concern, not an IL or public API divergence.
The existing UTF-8 service contracts remain unchanged; no performance claim.

Compiling the four unchanged production union/attribute sources now reaches the
portable emitter's imported class-base restriction at UnionAttribute : Attribute,
with NEOMETA001 and no output. Imported inheritance is the next full-source blocker.
The fixture still lacks real Object display/equality and String.Replace; synthesized
quoted display without Replace does not escape quotes/backslashes. This evidence
therefore does not qualify full union formatting, production core bootstrap or release
readiness. Website capability claims stay unchanged.

## Production union source subset (2026-10-09)

Add `--unions` to the text-services command. The verifier compiles unchanged
Propagatable, Option, Result, Attribute and UnionAttribute sources into a separate
native library, references it from a native-only project, then executes the
[union consumer](union-consumer.rvn) through project run, independent interpreter
and standalone ARM64 code. All return 42; native linkage remains libSystem only.
[Evidence](union-source-validation.json) records the five source hashes and commands.

The consumer covers Some/None, Ok/Error, let-else success/failure, if-let matching,
nonmatching and else branches, Unicode/NUL error payloads and the production
TryGetOutput/TryGetResidual bodies, including resetting a mismatched output.
Inspection of the selected closure confirms no fixture Object method is called.
No new IL semantics, metadata encoding, compiler change or performance claim.

This refines the preceding frontier assessment: the four-file probe omitted the
production Attribute base. Including that source keeps the existing local inheritance
contract and succeeds; imported class inheritance remains a separately checked
NEOMETA001 rejection, not a blocker for this five-file source subset. The verifier
records it as `importedBaseRejection` in union mode. It does not omit UnionAttribute,
change production sources or manufacture substitute union carriers to obtain success.

The core remains API-authored and incomplete. This is not full source System.Runtime
bootstrap, union formatting/equality/hash qualification, installed editor acceptance
or extracted-package qualification. Next expand the native primitive/String/Object
source contracts and use their real display behavior before qualifying formatted
unions; keep the imported-inheritance capability gap explicit when splitting libraries.

## Generic String boxing and Object display (2026-10-09)

Add `--string-boxing` to the `--project --text-services --unions` verifier command.
The [boxing consumer](boxing-consumer.rvn) converts a String through a generic
`Box<T>` method to Object, displays its Unicode/NUL text, checks alias identity and
checks that separately concatenated equal-content strings remain distinct. It also
constructs an unrelated production Option<int>. This exposed two native admission
errors: unboxed value constructors were treated as Object dispatch candidates, and
closed `box String` was treated as unsupported value-record boxing.

AOT now excludes unboxed values from Object dispatch candidate discovery in both
specialization and selection. It lowers verified `box String` to the existing
String-to-Object view conversion, retaining the CIL instruction's position and
recording `stringBoxSites`. This uses the existing reference arena; it introduces
no new runtime service or stable ABI. General value boxing, Char boxes, unboxing,
array display and default Object metadata display remain outside this slice.

[Execution evidence](string-boxing-validation.json) records project/interpreter/native
result 42 and libSystem-only linkage. The fixture's new Object.ReferenceEquals
wrapper calls the real existing runtime identity service. Its placeholder Object
slot bodies remain deliberately nonproduction; native String display uses the
existing intrinsic text path, and the interpreter uses its matching String behavior.
The focused AOT `object_display_accepts_string_boxing_with_unboxed_value_construction`
regression additionally compares UTF-8/NUL user-fault rendering and stack traces.
Existing Object override/null/fault and invalid override/base/boxing rejection tests pass.

This follows the CLI distinction between reference conversion and copied value
boxing, reusing the [.NET/Object comparison](../../object-model-review.md) and
[boxing research](../../boxed-interface-values.md). The interpreter already handles
this String path, so it needs no change. Native and interpreter allocation policies
still differ; result/identity parity does not imply identical allocation counts or
resource-exhaustion thresholds. No performance claim is made.

A further probe of `Option<string>.Some("hé🙂").ToString()` succeeds in the
interpreter but native admission now reaches `isinst Char` in the generated formatter
(instruction 19), which this profile does not support. That is separate from String
boxing and must not be bypassed by removing the formatter's Char branch. Escaping,
primitive display and production String/Object bootstrap remain open. The website's
bounded work-in-progress description remains accurate; no public API was added.

## Nonmatching Char tests and String union display (2026-10-09)

`verify.py --project --text-services --unions --union-display` now compiles and
executes the [display consumer](display-consumer.rvn) using the unchanged five-source
production union subset. Some with a Unicode/NUL string, None, Ok with a string
and Error with a string produce their expected descriptions. Project run,
independent interpreter and standalone ARM64 code all return 42; native linkage
remains libSystem only. [Evidence](union-display-validation.json) records inputs
and commands. This supersedes the preceding String-payload display rejection.

The generated formatter retains its Char test. The closed native profile still
rejects every Char box producer and has no external boxed-object entry inputs;
therefore an admitted `isinst Char` can only return null. After original verification,
specialization and boxing admission, AOT replaces these sites with a private,
nonallocating helper returning a null reference to the selected Object root.
`nonmatchingCharBoxTests` records the sites and this restriction. The helper is
needed because the backend's bundled verification context has a different Object
identity; manufacturing a host-root result would violate the source contract.
It consumes one helper function within the existing native function budget.

This is bounded closed-world lowering, **not matching Char-box support**. Focused
checks compare null, String, ordinary class and boxed Int32 inputs against the
interpreter; each returns a null reference without a cast fault. A real Char box
matches in the interpreter and rejects without native output. Future Char boxing
must replace this proof/lowering with a real exact-box type test. The formatter's
Char branch is not removed from source or native metadata.

The contract follows the existing [CLI type-test research](../../object-model-review.md):
a nonmatching `isinst` returns null, rather than a default value or exception.
No metadata/ABI format or public API changes, no performance claim, and no interpreter
change. The native profile still excludes matching Char boxes, unboxing and arbitrary
boxed input. Quotes/backslashes in String payloads still need an escaping contract;
primitive payload formatting and production String/Object bootstrap remain open.
The website's bounded work-in-progress claim remains accurate; no website build is
needed for this internal lowering and executable qualification.

## Int32 union display qualification (2026-10-09)

The `--union-display` consumer now also checks Some(0), Some(-1), both Int32 limits,
Result.Ok(42) and Result.Error(-7). These execute the unchanged production union
formatter bodies with the existing boxed Int32 native display implementation;
no replacement union or fixture Object method supplies the expected numbers.
The verifier explicitly selects `--bind-int32-to-string`. A separate invocation
omitting that flag must diagnose the missing formatter and publish no object file.

[Integer display evidence](integer-display-validation.json) records project run,
independent interpreter and standalone ARM64 result 42, plus libSystem-only linkage
and the negative binding check. The preceding String-only report is retained as
historical evidence. This is additional release qualification of an existing native
formatter, not new boxing/IL support or a performance claim. It follows the existing
[boxed Int32 contract](../aot-console/README.md) and CLI comparisons above.

Boolean, Char and wider primitive boxes are not qualified by this sample. Escaping,
matching Char tests and full production String/Object bootstrap remain open. The
website's bounded development description stays accurate; no public API or compiler
contract changes in this slice.

## Constructed receivers and escaping audit (2026-10-09)

`--project --text-services --unions --escaping-audit` deliberately reproduces a
known defect, not a formatting pass. The [audit](escaping-audit.rvn) compares a
quote/backslash payload against both the correctly escaped description and the
current raw-quoted description. Project/interpreter/native execution all confirm
the latter. [Evidence](escaping-validation.json) labels this `knownFailure` and
preserves exact tool/input hashes; its result 42 means the defect was reproduced.
When the implementation is fixed, update this audit into a positive assertion.

The first run exposed a separate portable compiler rejection for a freshly
constructed value receiver: Option<string>.Some(payload).ToString(). Raven
`1190d03d3` on codex/source-object-metadata-resolution now evaluates that result
once into temporary storage before method arguments, using existing value-call
lowering. It preserves mutation on the temporary and existing construction guards.
Ordinary/generic/nested imported constructor receiver checks verify/run with 42;
the audit proves native-only project/interpreter/ARM64 execution. The evidence run
preceded that commit; recorded compiler hashes include the tested correction.
No Runtime Contract configuration or CLI/native encoding changes. Raven main does
not contain this portable-emitter file, so there is no independent main-file port.

The remaining escaping cause is shared: production System.String has no
Replace(string,string), and Raven's CreateQuotedStringValue falls back to raw
quoting when that member is unavailable. This is not an AOT-only discrepancy.
[.NET's Replace contract](https://learn.microsoft.com/dotnet/api/system.string.replace)
(consulted 2026-10-09) supplies replacement text without modifying the original,
rejects an empty search string, and permits null replacement as removal. neoCLR's
non-null UTF-8 service contracts require an explicit design for those argument and
fault policies before claiming equivalent support.

Next implement real replacement support shared by interpreter/native paths, expose
and document the appropriate String member, and convert this audit to positive
escaping checks. Cover quote/backslash ordering, Unicode, NUL, repeated/absent
matches and output limits. A dedicated escaping service is an alternative with a
smaller contract but additional compiler-target coupling; it should not be hidden
in fixture-only formatting. Neither alternative is implemented by this slice.
