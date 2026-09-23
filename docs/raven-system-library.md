# Authoring the foundational library in Raven

## API-preserving source port — 2026-09-19

The author directs source migration first and proposal API alignment afterward.
The acceptance gate is a functioning neoCLR: Raven programs must compile, import,
verify and run against the regenerated library. Original proposal texts are indexed
[separately](proposals/README.md). The authoring project is now
[System.Runtime.rvnproj](../runtime/raven/System.Runtime.rvnproj), with the explicit
reference/runtime mapping described in [the assembly plan](system-runtime-assembly.md).

This slice adds Raven sources for the fundamental and collection interfaces,
SystemClock, LocalDateTime, IntPtr/UIntPtr comparisons, the complete Int32 member
surface, Console and Environment. File.ReadAllText joins WriteAllText in Raven.
The follow-up ports String and opaque Error, then five empty error types and Void,
followed by seven typed error carriers, and the Propagatable declaration, then Option/Result and their cases, then the inherited descriptor family and NativeMemory, then System.Fault, Func declarations and the normal BindingFlags enum and Object/UnionAttribute markers, followed by managed Array members and iteration, bringing the source port to 73 slices; subsequent AssemblyInfo and ModuleInfo providers bring the current total to 75.
The legacy Neo profile retains its receiver/array conventions where it differs;
the Raven profile selects generated contracts and implementation bodies.

The [final source-port gate](raven-library-port-validation.md#final-source-port-gate--2026-09-19)
records completed source migration and functioning programs. Subsequent API alignment
establishes sealed Info interfaces and direct TypeInfo acquisition through typeof
and Object.GetType. RuntimeContext.Current supplies the configured handle resolver;
ExecutingAssembly now provides assembly/module discovery, direct references and
module-scoped tokens; new discovery collections return Sequence<T>. See the
[current contract and compatibility notes](introspection-design.md).

Importer admission checks invariant generic arity, parameter positions, base
interfaces, exact method/property signatures and external identities. Variant or
constrained parameters, default/static interface bodies and unrelated shape changes
remain rejected. LocalDateTime's nested Date/Time fields have checked order and
identity. Library method parameter names survive import for introspection.
Its FromUnixTimeTicks factory retains internal visibility. Clock behavior
continues to use the existing host tick/local-calendar services.

Bootstrap-only RuntimeServices bindings allow checked inspection/extraction of
native String, Byte, Int32 and Void payloads. Erased locals require definite
assignment; they are not initialized with fabricated default payloads. Native status
interpretation and typed Result/Option construction are Raven code. Native integer
comparisons widen signed/unsigned storage with the existing conv.i8/conv.u8
instructions before ordinary comparisons. Int32.ToString retains its scalar neoIL
receiver and Console's no-result CIL exports retain the existing neoIL Void boundary.
Environment produces a fresh managed argument array in the Raven profile.
No new public API, namespace migration or Runtime Contract setting is introduced.

The compiler investigation found a general generic-interface base-scope bug.
Raven main commit `2e3856a6b` fixes it independently using ordinary .NET references;
`0b6f11bb5` applies that fix to the neoCLR feature branch. Three cases failed before
the fix; 87 focused main tests and 99 feature-branch checks pass, including the
context-owned typeof contract. This establishes .NET 11 metadata behavior, not
execution on .NET Framework or NanoFramework. The subsequent branch audit reproduced
a delegate bridge InvalidProgramException on ordinary .NET and integrated the
independent fix as `5a37cd56c` on Raven main (19 focused delegate/unit tests pass).
The experimental branch already contains equivalent behavior in `8e0f6cb7d`.

This migration reuses the .NET comparisons in [common interfaces](common-interfaces.md),
[collection contracts](collection-contracts.md), [integer types](integer-types.md),
[date/time design](date-time-design.md), [console I/O](console-io.md),
[environment](environment.md) and [file input](file-input.md). The principal cost is
additional generated adapters and a larger reachable call graph; no performance
improvement or new native ABI is claimed.

Every selected Raven-profile method/function body now comes from generated Raven
snapshots or explicit runtime services. The former TypeOf<T>.Of helper has been
removed at the author's request; use typeof(T) and rebuild existing callers.
Native service declarations remain runtime-owned. Generated neoIL remains a build
artifact rather than a competing implementation. These boundaries are not silently
claimed to have become Raven source.

String authoring uses a final CLI class with exactly one private `m_value: string`
bootstrap field. The importer checks that shape and erases the field into existing
intrinsic storage; it does not allocate an ordinary class. Equals/ContainsOrdinal/
StartsWithOrdinal/EndsWithOrdinal retain readonly byref neoIL receivers; byte count,
empty and slicing retain value receivers. Primitive equality avoids recursively
calling String.Equals. Compiler-facing operators remain intrinsic, not additional
runtime methods. Raven named arguments retain the existing reference contract (`value0`/`value1`);
generated runtime metadata retains the prior descriptive names (`left`, `other`,
`byteStart`, and so on). Their pre-existing disagreement is left for API alignment.

Error is a fieldless, opaque value created only by the native message factory.
The importer removes the reference assembly's placeholder one-byte layout and
checks exact source storage/signatures. It omits only checked implicit constructors,
rejects direct opaque construction/default Error bodies and String field writes,
and projects Error's CLI receiver loads onto the existing by-value neoIL ABI.
No Raven compiler code or Runtime Contract option changes are required. These
admission policies stay in neoCLR's importer and bootstrap reference assembly.

This preserves the established behavior in [String](raven-string-api.md),
[string default storage](string-default-storage.md), and [errors](errors.md),
including their .NET comparisons. CLI class/value metadata serves Raven compilation;
neoCLR retains its own immutable string/message storage and native UTF-8 operations.
The benefit is source ownership with checked boundaries; the cost is generated
Boolean/union adapters and a larger call graph, without a performance claim.

The payload-free error types (InvalidRangeError, InvalidDateError, InvalidTimeError,
OverflowError and EnvironmentError) and Void are also Raven-authored.
Their runtime defaults/constructors remain distinct from opaque Error: an empty
error is valid. Message failures use ordinary strings; the legacy Error wrapper is
retired. Bootstrap metadata
normalization removes only artificial empty-struct byte sizes, then checks the
source shape and public methods. No new error cases or proposal APIs are introduced.

Void authoring exposed a general Raven configuration-lookup defect: a source type
could shadow the unit contract's explicitly named metadata assembly. Main commit
`c17cb8397` resolves that assembly exactly (19 independent .NET unit/target-core checks
pass, including a regression that failed before the fix); feature commit `b6f12353f`
applies it to neoCLR. The ordinary .NET regression uses ValueTuple and checks emitted
identity and execution. It does not establish .NET Framework/NanoFramework execution.

Seven typed error carriers now own their cases, predicates, extraction and formatting
in Raven: FileReadError, FileWriteError, ConsoleReadError, Utf8SliceError,
Int32ParseError, IntegerDivisionError and Linq.SingleError. Each retains exactly one
erased Stored field and the existing empty nested cases. Bootstrap-only ValueStorage
pack/test/unpack intrinsics admit those checked cases; arbitrary payloads are rejected.
The importer checks constructor CIL before lowering the single field assignment to
neoCLR's existing by-value constructor ABI. It rejects constructor side effects and
fabricated defaults, preserving wrong-case faults and current message strings.
This reuses the established [error contracts and .NET comparison](errors.md); no
exception model, Runtime Contract option or public API is changed. The reference
assembly and importer carry this target-specific representation boundary.

Propagatable is a Raven interface declaration with the exact three generic positions
and two out parameters. Checked admission preserves its existing readonly receiver
and conditional `out(true)` ABI; ordinary interfaces still reject byref exports.
The [propagation contract](propagation-contract.md) explains the .NET ordinary-out
comparison and why failed extraction must not initialize a destination. This slice
changes declaration ownership only; carrier bodies follow separately.

Option/Result now own their case storage, constructors, factories, predicates,
extraction and propagation in Raven. Their checked authoring family includes the
nongeneric case containers and generic carriers; runtime case fields retain the
existing public Value identity although Raven uses a private field and getter.
Compiler-only recognition members in the consumer reference are not runtime exports.
Generic case/erased-payload constructors are checked before projecting their single
assignment onto the existing value constructor ABI.

A bootstrap-only LeaveUnassigned(out T) marks a failed extraction in source. The
importer accepts it only for the current conditional-output parameter followed
immediately by return false, and discards the address without writing storage.
Literal Boolean returns remain literals, preserving the runtime verifier's true-only
assignment proof. Readonly receiver adapters copy values without requiring writable
references. Invalid generic carrier/case defaults remain unreadable. These rules
preserve the [existing propagation/.NET comparison](propagation-contract.md), rather
than introducing a new exception or union API.

The general direct out-forwarding compiler correction is on Raven main (`5f6e17347`)
and the feature branch (`2d2a1d586`); both pass 41 focused parameter checks. No new
Runtime Contract setting is introduced. The independently reproduced unqualified generic constructor lookup correction is
on Raven main (`d7292b935`) and the feature branch (`d833ef2f3`). Accessible same-name
namespace declarations are retained across generic arities; invalid arities report
RAV0305. The ordinary .NET regression returned 0 before and 42 afterward. All 78
focused main checks and 18 feature-branch checks pass. The source keeps explicit
qualification for readability; no additional Runtime Contract option is required.

MemberInfo, FieldInfo, MethodInfo and PropertyInfo now share a Raven-authored
source slice. Import checks their exact ordered snapshot fields and inheritance;
private source Stored fields retain the existing runtime field names and indices.
The protected source base constructor becomes an internal runtime constructor,
admitted only for the checked derived constructor chain. No general protected-call
permission is granted to applications.

Runtime factories still own snapshot creation. A bootstrap-only ParameterSnapshot
read view maps precisely to the existing ParameterInfo vector; only Length/Get are
admitted. Raven owns the managed-array allocation and copying in GetParameters and
GetIndexParameters. Each call returns independent array storage, as on .NET reflection
APIs, while contained descriptors retain identity. Property accessors preserve public
filtering and the explicit nonpublic overload. This reuses the
[reflection snapshot contract](raven-reflection-api.md); it does not add runtime invocation or
change Runtime Contract configuration. The legacy Neo profile retains its value-based
descriptors; the Raven profile selects these class implementations explicitly.

The source-body ownership gate is `verify_source_ownership.py`. It compares every
selected method/function declaration with generated fragments or the explicit
native-service catalog. Historical Neo implementations with different receiver and
array conventions remain separate. Generated enum/marker/iteration ABI lowering
is compiler-owned; it is not a second managed-library implementation. Complete
execution validation is required in addition to this ownership check.

Validation commands and recorded results are in the
[port validation record](raven-library-port-validation.md).

## Planned System.Runtime assembly — 2026-09-17

The author requests a minimal managed `System.Runtime` project/assembly, starting
from the foundational types in the current System project. The
[assembly plan](system-runtime-assembly.md) separates assembly ownership from
namespaces and coordinates reference identity, runtime artifacts and compiler
contracts. It does not duplicate the library or make optional Reflection/Emit
mandatory. Inventory the working POC's dependencies before selecting the exact set.

## Constructor argument conversions — 2026-09-17

The neoCLR importer now uses its per-argument coercion adapters for application
and Raven-library constructors, as it already did for method calls. Previously,
converting several CLI Int32 Boolean operands in place repeatedly converted the
top stack value. Mixed constructor arguments could fail runtime verification.
The adapter reloads and converts each argument in declaration order after source
expressions have been evaluated, preserving values and single evaluation. Reference
constructors and application value-constructor factories use the same path.

This restores the existing CLI-stack/neoCLR Boolean boundary; it adds no language
feature or Runtime Contract configuration and makes no change to the Raven compiler.
`verify_constructor_arguments.py` executes mixed Boolean/integer class and value
constructors and checks observable argument-evaluation order. The introspection
field adapter exposed the bug, but the fix is not descriptor-specific.

## Checked interface declarations — 2026-09-17

The runtime-library importer now accepts an explicitly selected public nongeneric
interface only when its abstract instance methods and properties match the supplied
reference contract, including parameter names and referenced type identities. It
rejects class/interface substitutions, inherited/generic interfaces, storage, events,
default/static implementations and added or changed members. Unsupported forms remain
out of scope rather than being copied through unchecked.

`runtime/raven/src/System/Clock.rvn` is the first integrated contract: the Raven
profile uses its generated declaration while the original neoIL profile retains
its existing declaration. SystemClock still implements the wall-clock service;
Clock.Now still returns Instant. The public API, namespace and runtime behavior
are unchanged. This provides the checked library-declaration path needed before
the new introspection interfaces can replace their production class identities.
It does not perform that identity migration or add RuntimeContext.

This is a source-ownership change, reusing the existing
[clock/.NET comparison](date-time-design.md), not a new clock policy. Emitted neoIL
retains the interface method and property; no executable reference-stub body is
introduced. No Raven compiler or Runtime Contract configuration change is needed.

Validation: `verify_interface_library.py` checks positive/negative authoring and
executes FixedClock and SystemClock consumers, including host-clock comparison.
The introspection prototype and existing consumers remain regression checks.

## Migration priority — 2026-09-17

The [refined introspection proposal](introspection-design.md) now takes precedence:
one `*Info` interface model, no public Type/TypeInfo pair, runtime-backed v1 through
RuntimeContext, and Reflection/Emit as separate capabilities. First prove the minimal
Raven interface/provider path, then migrate runtime identity and member signatures
together. Offline MetadataContext and typed introspection are deferred. The earlier
class-port sequence below records completed work and the superseded plan.

The [interface/provider probe](experiments/raven-target/introspection-v1/README.md)
exercises TypeInfo and MemberInfo with existing runtime metadata. It is isolated
from the shipped library and preserves BindingFlags. Its minimal RuntimeContext now
resolves compiler-generated typeof handles to the shared TypeInfo interface.
See the [POC configuration and validation](experiments/raven-target/introspection-v1/README.md).

### Previous migration sequence — 2026-09-16

TypeInfo's existing queries and internal factory are now Raven-authored alongside
ParameterInfo. Runtime-backed descriptive services remain part of Introspection;
runtime independence is not required. MemberInfo and its derived descriptor bodies
are the remaining hierarchy port. Reflection and Emit can attach capabilities and
implement constructs in this same model without adding invocation to its shapes.

First slice completed: descriptor identities and BindingFlags now use
System.Introspection across reference metadata, the runtime and consumers.
Type's Raven source uses that namespace; member hierarchy bodies remain neoIL and Info
is still an instance property. See the [migration](raven-reflection-api.md).

The next slice ports ParameterInfo's six readers to
`src/System/Introspection/ParameterInfo.rvn` with checked snapshot layout and private
construction. The remaining member hierarchy bodies are still neoIL.

The author directs the existing runtime class library to move from handwritten
neoIL to Raven while adopting the namespaces and structure of the API proposals.
Generated neoIL remains an execution/bootstrap artifact of Raven sources.

The author's subsequent clarification makes API shape the next release's demo/POC
objective across all proposal families. Complete implementations are not required;
Raven declarations, matching metadata and clear capability boundaries can precede
backends. See the [release objective](library-preview.md#next-release-objective--2026-09-16).
The sequence below recommends implementation slices, not a gate on exposing the
other families' proposed structure.

The recommended next slice is the existing introspection family: keep System.Type
as the core descriptor, move TypeInfo and the existing member/parameter descriptors
to System.Introspection, and port their library bodies to Raven. Implement the
proposed extension Info boundary with an explicit callable representation for other
frontends. Check the importer and reference metadata support before assuming that
extension-property syntax is available. Preserve existing query semantics while
migrating references, generated artifacts, consumers and editor samples together.
See the [proposal](reflection-model-review.md).

Follow with the existing Date, Time, LocalDateTime, Instant, Duration, Clock and
SystemClock family under the proposed System.Time namespace. Resolve the existing
System.Time type versus proposed namespace collision explicitly; the eventual
time-of-day type would be System.Time.Time. Finish remaining neoIL library bodies
alongside that migration. Globalization, stream capabilities, runtime async and
dynamic reflection introduce new functionality and need their own implementation
slices; they are not prerequisites for porting existing APIs.

For each slice, validate Raven compilation, reference/export identity matching,
generated-artifact freshness and existing behavioral tests, with focused consumer
and direct-IL coverage. Compiler-affecting changes require documentation and
changelogs in both repositories under the Raven integration workflow.

`runtime/raven/System.Runtime.rvnproj` is the shared authoring project for ordinary
foundational runtime APIs. Its sources include `src/System/Math/Functions.rvn`, `src/System/Linq/Operators.rvn`, `src/System/Int32.rvn`, `src/System/Char.rvn`, `src/System/Collections/ArrayList.rvn`, `src/System/Collections/HashMap.rvn`, `src/System/Time.rvn` and `src/System/Date.rvn`; additional namespaces
and types should join this project as their importing requirements are validated.
Each declared namespace has its own folder below `src`: for example, `System/Date.rvn`,
`System/Collections/ArrayList.rvn` and `System/Math/Functions.rvn`. Namespace functions
use `Functions.rvn`; types use their type names. Do not create an assembly per utility namespace. This is an incremental source
migration, not yet a self-hosting build of the complete core reference assembly.

`src/System/Type.rvn` is also built as a selected class slice. See the
[Type/reflection boundary](raven-reflection-api.md#type-source-and-metadata-boundary-2026-09-15).
The importer accepts checked static factories and private constructors on matched
classes, and matches self types inside array signatures. Reference signatures,
visibility and the runtime-created Type layout remain validated. Reflection native
services are available only while importing a library implementation; reference
stub bodies are never executed. TypeInfo and ParameterInfo are now Raven-authored; member hierarchy descriptors remain in neoIL.

Raven-authored public APIs use descriptive parameter names. Date exposes `year`,
`month`, `day`, `dayNumber` and `other`; Time exposes `hour`, `minute`, `second`,
`fractionTicks`, `ticks` and `other`; Type exposes `handle`, `index` and `other`.
These names also appear in reference metadata, enabling named arguments and useful
editor parameter hints. Calls using previous `arg0`/`arg1` labels must be updated;
positional calls keep their behavior. Generated internal adapters and generic delegate
argument positions may retain numbered names because they have no domain-specific
meaning. This changes the experimental library contract, not Raven compiler behavior.

The bootstrap currently has three distinct artifacts:

- `NeoCLR.CoreProbe.dll`: compiler-facing reference metadata for the supported System
  surface. A bootstrap variant includes CheckedStorage for authoring; consumers use
  the normal reference surface. Placeholder bodies must never execute.
- `System.Runtime.dll`: compiled Raven implementation input, currently twenty
  Math functions, nine query overloads and their private deferred iterator classes,
  Int32.Divide, Path and file-write functions, plus separately compiled collection,
  calendar and primitive structs. Memberless Value and RuntimeTypeHandle declarations
  are also Raven-authored.
  These implementation inputs are imported into neoIL, not loaded dynamically by the runtime.
- `runtime/System.neoil` and its includes: the executable foundational library,
  combining generated Raven bodies with remaining handwritten bodies and intrinsics.

This resembles the managed implementation/reference separation used in .NET,
without copying its historical assembly partition or requiring one public utility
class per namespace. Assembly partitioning, public namespaces and implementation
language are independent choices. The eventual core assembly identity and additional
platform-specific libraries remain decisions to make as the migration progresses.

## First source and namespace contract

`System.Math` is a namespace in the Raven reference surface. The source declares
public functions `Abs`, `Min`, `Max`, `Sign` and `Clamp` for Int32. Abs and Clamp
preserve their existing typed Result outcomes, including minimum-Int32 overflow and
invalid bounds. Double operations are also Raven-authored wrappers over the same native services.

Raven's usual CLI contract represents namespace functions as static methods on a
container marked with the target's TopLevelAttribute. The importer recognizes that
marker and verifies exported signatures against the supplied core, including nominal
assembly identities. It does not infer namespace membership from container spelling.
No new metadata format or instruction is introduced.

The generated bootstrap fragments retain the existing internal `System.Math` method
owner so direct IL, library dependencies and the archived Neo frontend keep working.
Raven sees the namespace contract; that internal owner is not the public Raven API.
This transitional namespace mapping is explicit and limited to the existing Math catalog.
It is not a new requirement for languages to declare static utility classes.

## Build and verify

Use an experimental Raven compiler containing the general qualified-namespace lookup
fix described below. Keep all neoCLR-specific Raven policies on the experimental
branch. Build its compiler normally, then build the importer against that checkout:

```sh
dotnet build docs/experiments/raven-target/Probe.csproj -c Release \
  -p:RavenRoot=/absolute/path/to/Raven -p:BuildProjectReferences=false

python3 docs/experiments/raven-target/build_runtime_library.py \
  --compiler /absolute/path/to/rvnc.dll \
  --bridge docs/experiments/raven-target/bin/Release/net11.0/Probe.dll

python3 docs/experiments/raven-target/build_runtime_library.py \
  --compiler /absolute/path/to/rvnc.dll \
  --bridge docs/experiments/raven-target/bin/Release/net11.0/Probe.dll --check

cargo build --locked
python3 docs/experiments/raven-target/verify_math_library.py \
  --compiler /absolute/path/to/rvnc.dll \
  --bridge docs/experiments/raven-target/bin/Release/net11.0/Probe.dll \
  --runtime target/debug/neoclr
```

The builder creates fresh bootstrap metadata, uses the ordinary Raven project
compiler, imports the implementation, and writes checked-in neoIL fragments and a
hash manifest. `--check` regenerates in a temporary directory and compares bodies.
`--check-snapshot` checks source/artifact hashes without Raven or .NET; source-release
validation runs that check. Ordinary Rust builds use the checked-in fragments and
do not require Raven. Regenerate and review the manifest alongside source edits.

The consumer test covers qualified and wildcard-imported calls, a consumer-defined
type in the same namespace, boundary values, Result payloads/errors, a remaining
Double overload and a flattened executable library without source includes. Invalid
export names, parameter names/signatures, generic arity and unmarked containers must be rejected
without executable output. Existing Rust Math tests cover the direct runtime and
archived Neo callers.

Qualified namespace calls with consumer declarations exposed a general Raven lookup
bug: source-only namespace lookup omitted referenced namespace functions/constants.
Its regression uses ordinary .NET reference metadata. The fix is `3ec32c96e` on
Raven main and `008cb3245` on the experimental branch; the previously installed
`.14` SDK predates it. Keep this compiler correction
on Raven main independently of the experimental target; the System migration must
not compensate with target-specific binding rules.

The source imports `System.Result.*` and constructs `Ok`/`Error` directly. It uses
specific error-type imports (originally needed to avoid the now-retired `System.Error`
name), and fully qualifies `System.Result<...>` in return annotations. The observed
unqualified-return diagnostic issue was fixed independently; see the
[resolution follow-up](raven-target-evaluation.md#resolution-follow-up--2026-09-14).

## Initial namespace gate

The initial namespace/static gate accepts public namespace functions and static API methods matching existing
reference signatures, including the bounded generic body gate below. It rejects stateful
containers, unexported helpers, new application-type identities, constrained
generic signatures and unsupported constructed shapes, byref/out exports and no-result exports. Support for these is future work, not implied by compiling the pilot.
Generated Result adapters are scoped to the library implementation to avoid clashes
with consumer adapters. Reference declarations remain separately maintained and
checked against exports; complete generation of reference metadata from Raven source
is not implemented yet. The matched class, private helper and collection gates
below describe subsequent additions.

The collection terminal migration now admits constructed Iterable/Iterator/ArrayList,
Func and Option/Result method signatures. Before migrating foundational type definitions, establish
how implementation and reference assemblies share their identities without circular
bootstrap dependencies. Keep native services and neoIL conformance programs where
those representations serve the platform.

## Generic implementation gate — 2026-09-14

The importer now preserves unconstrained generic namespace method definitions and
calls within the same implementation fragment. Parameters and returns may use a
method type parameter directly. Generic arity and parameter positions are checked
against the separate reference contract; parameter names and existing nominal type
identity checks remain enforced. Generic parameter *names* need not match. Output
uses positional names (`T0`, etc.) and the runtime’s existing generic function syntax.

This follows ordinary CLI generic method definitions/specifications and reuses the
runtime machinery described in the [generic constraint comparison](generic-constraints.md#comparison-with-clr-constrained-calls).
There is no new metadata format or opcode. Preserving an open body avoids introducing
a separate importer-specific monomorphization scheme. The cost is explicit validation
and substitution at calls; full CLI generic support is not established by this gate.

The test-only `Probe.Generic` namespace supplies a separate metadata contract. Raven
compiles `Choose<T>` plus Int32/String wrappers; the imported open body executes both
branches for each representation. The test deliberately uses a different type parameter
name in the reference declaration. Mismatched parameter names and arity are rejected
without executable output. A direct neoIL driver isolates implementation import and
execution; this is not yet general consumer binding to arbitrary generic core APIs.

```sh
python3 docs/experiments/raven-target/verify_generic_library.py \
  --compiler /absolute/path/to/rvnc.dll \
  --bridge docs/experiments/raven-target/bin/Release/net11.0/Probe.dll \
  --runtime target/release/neoclr
```

No public System API is added by this probe. Generic classes, constrained bodies,
composite signatures such as `Iterable<T>`, and broader cross-fragment dependencies remain subsequent gates. Generic locals must be assigned
before use; this slice does not project implicit generic defaults.

The probe exposed a general Raven RuntimeUnitContract emission defect in generic
method specifications. The independent fix is `5f93eef6a` on Raven main and
`64a5497ec` in the experiment, with 12 focused .NET tests passing.
The subsequent generic unit-return defect is
fixed on Raven main as `327335699` and in the experiment as `ef352917e`: a call
whose original return type is a type parameter already returns a value when that
parameter is unit. It must not synthesize a second value, and must pop the real
value when discarded. Twenty-two focused .NET checks pass across default, explicit-core
and ValueTuple-contract emission, including methods on generic types and no-result
wrappers. Raven’s compiler documentation records the same distinction.

The generic library probe also instantiates `Choose<()>(...)` using neoCLR’s Void
contract, stores one result and discards another. Existing importer handling preserves
nominal Void correctly; no new importer or runtime instruction was needed. The runtime
verifier and execution validate the stack behavior. This extends the earlier generic
storage and Result<Void, E> coverage to generic unit-returning invocations.

## Collection terminal migration — 2026-09-14

`src/System/Linq/Operators.rvn` implements `System.Linq.Operators.ToList<T>`, `First<T>`, `Last<T>`
and `Single<T>`, including the three predicate overloads. `Operators` replaces the
previous Enumerable owner; extension-call syntax and `import System.Linq.*` are
unchanged. Existing consumer assemblies must be rebuilt against the new reference
metadata. Public declarations remain separately maintained and verified.

Constructed generic signatures are compared structurally against the core, preserving
parameter positions and nominal identities. Open parameter mapping is scoped to the
active implementation method; ordinary application admission remains bounded. Adapter
functions declare their free method parameters, including dependencies on other
adapters. This follows CLI generic signatures and uses existing neoCLR generic
instructions rather than adding opcodes or specializing a body for each consumer.

Both namespaces build in one shared project. The builder validates every fragment
before replacing snapshots, and the collection-profile builder expands the generated
includes for standalone distribution. The reference metadata retains extension-method
attributes while the authored bodies are ordinary static methods.

Validation covers 30 query integration cases, 121 signature-contract checks and five runtime terminal tests
(including callback/disposal faults and allocation comparisons), generic Iterable
iteration with an independently declared contract, three rejected generic contracts,
69 Math outcomes/six rejected Math contracts, and Void result propagation. Snapshot
regeneration is reproducible with the recorded compiler. This is source validation;
it does not refresh the installed SDK or publish a release.

The remaining migration boundary is **type implementation identity**, not more static
terminal algorithms: ArrayList/Map state, constructors, interface implementations,
and deferred Where/Select iterator classes require importing instance bodies against
shared reference identities and their fields. Those bodies remain executable neoIL.
Native services and scalar intrinsics still belong to the runtime; copying their
wrappers into Raven would not migrate their implementation. Constrained generic
exports and whole-core bootstrap/reference generation remain separate gates.

## Instance implementation identity gate — 2026-09-15

`--library-implementation` also accepts a single public, nongeneric reference class
with an independently declared core contract. It checks class category and sealing,
constructors, instance methods (including self-typed arguments/returns), dispatch
flags, parameter names and types, and property/accessor shape. Public members must
match exactly. Private mutable fields belong to the implementation and need not
appear in the reference declaration. Their runtime visibility remains private.

Only the explicitly selected implementation definition receives the public runtime
owner name. Ordinary application types retain assembly-qualified identities, and
additional implementation types remain rejected. Imported constructors, methods and
properties live inside that runtime type; source maps report the same identity and
use the `instance-library-fragment-v1` profile. Reference declaration bodies are
never imported as implementations.

This extends the existing [reference/implementation comparison](raven-target-evaluation.md#comparison-and-tradeoffs).
The bootstrap still has two distinct CLI assembly identities. An explicit checked
pair is a provisional bridge to the runtime identity, not general CLI assembly
unification. It avoids per-member forwarding wrappers and preserves class-reference
behavior. The cost is maintaining and validating the reference surface separately;
eventual core assembly generation still needs a design. No metadata format, opcode,
Raven semantic rule or Runtime Contract setting changes here.

The independent `Probe.Counter` fixture has unusable reference stubs and no reference
fields. Its Raven implementation executes construction, property access, mutation,
self-return and a self-typed parameter. A direct neoIL consumer observes shared
reference updates (`42`, then `99`). Five mismatched contracts are rejected before
runtime IL is written. The existing 13 cross-library checks still cover distinct
assemblies with identically named types, access control, constructors, inheritance,
interfaces and delegates. Generic static-library checks and clean Math/query snapshot
regeneration also pass.

```sh
python3 docs/experiments/raven-target/verify_instance_library.py \
  --compiler /absolute/path/to/rvnc.dll \
  --bridge docs/experiments/raven-target/bin/Release/net11.0/Probe.dll \
  --runtime target/release/neoclr
```

This is groundwork, not another System API port or general consumer admission for
arbitrary new core classes. The gate currently excludes generic classes/methods,
interfaces, inheritance beyond Object, value classes, static members, nested types,
events and additional implementation dependencies. The next migration gate is open
generic instance state plus interface dispatch, followed by the deferred query
classes and ArrayList/Map. Native-backed scalar and reflection layouts must retain
their trusted construction contracts when migrated.


## Scalar algorithm migration — 2026-09-15

The shared Raven project now supplies `Int32.Divide` and seven Char predicates:
IsSurrogate, IsHighSurrogate, IsLowSurrogate, IsAscii, IsAsciiDigit, IsLetterOrDigit
and IsWhiteSpace. Divide preserves Result errors for zero divisors and signed overflow.
Char preserves UTF-16 code-unit behavior, including surrogate classification, ASCII
bounds and existing Unicode classification. Parsing and Unicode category lookup remain
native services. This is an implementation-language migration, not a new numeric or
text contract; existing [character research](character-classification.md) remains the
baseline.

The two sources use marked namespace containers as implementation fragments. Their
methods match the existing static members on the core Int32/Char reference types;
public metadata still exposes `Int32.Divide` and `Char.IsAscii`, etc. They do not
redeclare primitive types or turn their public APIs into namespace functions. Static
fragments can now target a public nongeneric class/value owner with matching static
methods, rather than requiring the reference owner itself to be a static class.
This keeps primitive layout and instance operations in the existing runtime while
moving the algorithms. Source and reference signatures remain checked independently.

The port exposed Boolean stack joins in short-circuit expressions: CLI Boolean loads
and ordinary call results need the Int32 evaluation-stack representation already used
for comparison results. The importer now normalizes arguments, locals, fields and
ordinary call results, converting back at typed boundaries. Conditional-out calls
remain directly connected to their branch so the runtime verifier retains assignment
proof for extracted union payloads. No Raven compiler change or opcode was required.
The explicit conversion helpers can add interpreter instructions; this slice does
not claim a performance improvement.

`verify_scalar_library.py` compiles a separate Raven consumer and checks 203 outcomes:
division successes/failures, character boundaries and Unicode examples, and Boolean
short-circuit expressions using fields, locals and calls. Run it with the same
`--compiler`, `--bridge` and `--runtime` arguments as the other authoring checks.
Checked-in fragments are included by the base runtime as well as the Raven profile;
API inventory and coverage list their generated origins. Generic instance collection
state and deferred iterators remain the next authoring boundary.

Validation also passes the 30 current-profile query cases, 19 focused runtime tests
(character classification, integer behavior, arithmetic outcomes and query terminals),
Math/generic/instance authoring probes, clean snapshot regeneration and API audit.
These are source checks; installed tools and release artifacts are not refreshed.

## Generic instance implementation gate — 2026-09-15

The checked class-import path now admits unconstrained, invariant generic class
parameters and matching supported interface declarations. Public constructor,
method and property signatures are checked structurally against the selected core
reference type, including generic arity, parameter positions and self-typed returns.
Constructed field and method references are checked after substituting their owner
arguments. Ordinary application generic-class admission remains unchanged.

One open runtime definition is emitted with positional type parameters; constructed
references use the same owner with their actual arguments. Constructors, private
fields, generic locals, instance calls and returns retain those parameters. Interface
assignment and dispatch use the instantiated declaration. This reuses CLI generic
signatures and neoCLR's existing generic class instructions, rather than specializing
one implementation per payload. The [generic-signature comparison](generic-constraints.md#comparison-with-clr-constrained-calls)
remains the research baseline. Constraint-bearing classes, additional implementation
types and broader inheritance are still outside this gate.

The independent `Probe.Cell<T>` declaration has no executable reference bodies or
implementation fields. Its Raven implementation initializes and replaces private
storage, reads a generic local, returns itself, copies from another instance, creates
a new instance in Copy, and projects itself as Iterable<T>. GetIterator uses the
existing ArrayList to expose one stored element. The direct neoIL consumer checks
Int32, String and Void payloads, mutation through an alias, independent copied storage,
interface dispatch and iterator exhaustion/disposal. Generic arity and interface
mismatches are rejected before runtime IL is written.

```sh
python3 docs/experiments/raven-target/verify_generic_instance_library.py \
  --compiler /absolute/path/to/rvnc.dll \
  --bridge docs/experiments/raven-target/bin/Release/net11.0/Probe.dll \
  --runtime target/release/neoclr
```

This exposed a general Raven binding bug in `Cell<T>(value)` inside Cell<T>. Explicit
own type parameters were mistaken for omitted arguments. The independent compiler
fix is `796cb3e34` on Raven main, cherry-picked as `ddaf1fa94` in the experimental
branch. Four ordinary .NET execution cases cover qualified/unqualified calls with
and without explicit metadata-core configuration; all 17 focused generic tests pass.
Raven's spec and compiler documentation record the correction. No new Runtime
Contract setting or neoCLR policy was added to the compiler.

This gate is a prerequisite, not another shipped System API port. The deferred
Where/Select and collection bodies still need a faithful authoring surface for their
checked generic array storage and fault operations, plus private implementation
dependencies. Their existing neoIL implementations remain authoritative meanwhile.

The general fix passed Raven's integration gate before merging to main: 311 compiler,
73 core and 249 language-server checks (three existing skips). The temporary general
feature branch was removed after fast-forward integration. Existing nongeneric and
static-generic library probes, 203 scalar checks and 13 cross-library checks pass.
Regeneration changes only recorded compiler hashes; shipped library bodies remain
identical. No SDK installation or publication is part of this slice.

### Dynamic terminal fault API — 2026-09-15

[System.Fault(message)](system-fault.md) is available as a namespace function through
Raven's existing TopLevelAttribute metadata contract. The importer maps its validated
CLI void/String signature to the public runtime function. No Raven compiler change or
new Runtime Contract setting is required. `verify_fault.py` checks computed-message
reporting and guest termination. Non-returning-call flow analysis is not introduced;
checked generic storage and private implementation dependency support remain necessary
for deferred query migration.


### Checked storage and private implementation dependencies — 2026-09-15

The library importer now admits reachable nonpublic, nongeneric or unconstrained
generic helper classes from the implementation module. They receive internal runtime
identities scoped to the selected library owner and encoded from their metadata name.
They must have private mutable fields, public instance bodies, an Object base and no
nested types, events, generic methods or custom layouts. Exported signatures still
match the separate consumer reference contract; helpers do not become public API.
Public unmatched generic types and exposed helper state are rejected. Ordinary guest
generic-type admission is unchanged.

`--reference-library-core` produces a bootstrap-only reference surface containing
[checked array reservation](reserved-array-capacity.md). The consumer `--reference-core`
surface excludes that intrinsic. Generic vector signatures, field substitution and
method-parameter substitution preserve the caller's element identity; array reads
and writes still undergo runtime type and initialization checks. No new opcode or
Raven target configuration is required. Private type admission is a bounded migration
mechanism, not general CLR library loading or support for arbitrary helper shapes.

The generic instance probe can exercise direct checked storage or a private Storage<T>
helper with `--checked-storage` or `--private-storage`. Both variants execute Int32,
String and Void payloads, copies, mutation and Iterable dispatch; they check unwritten
reads, zero/negative capacity, invalid exported contracts and absence of the intrinsic
from the consumer core. The private variant also rejects public helper identities and
public storage fields. Existing generic and nongeneric instance probes still pass.

Two general Raven defects were validated independently with .NET 11 execution and
integrated separately: `730adc7b0` handles imported generic calls with source parameters
under explicit metadata-core settings; `8dfb64a2b` preserves generic array element types
for loads, stores, literals and iteration. Their experimental cherry-picks are
`f507ce44a` and `cfbdefa7b`. The focused suites passed 12 and 14 tests respectively.
Each main integration passed 311 compiler, 73 core and 249 language-server checks
(three existing skips). The new execution tests use default and explicit System.Runtime
metadata options; no .NET Framework or NanoFramework execution is claimed.


### Deferred query migration completed — 2026-09-15

Where/Select sequences and iterators now join the seven terminal overloads in
`src/System/Linq/Operators.rvn`. The build uses the bootstrap reference variant and retains reachable
internal classes alongside generated adapter functions. `runtime/raven/Linq.neoil`
only includes generated bodies; it no longer contains handwritten query algorithms.
Public methods, Option/Result outcomes, callback timing, iterator caching, Dispose
order and repeated-iteration semantics are unchanged. Current uses an ordinary guard
calling System.Fault before reading its cache; no compiler non-returning-call analysis
or artificial fallback value was needed.

The port exposed a runtime class-construction gap for delegate fields. That was fixed
separately: delegate fields can be assigned during construction, early reads fault and
the constructor must initialize them before returning. Null/default delegates remain
unsupported under the existing preview contract.

Validation: 30 Raven query cases, 25 delegate tests, five query terminal/cleanup tests,
three reserved-array tests, 203 scalar outcomes, generic/nongeneric instance probes,
the static-generic probe and 13 cross-library cases pass. Regeneration matches the
checked-in snapshot; scalar implementation bodies are unchanged. SDK installation and
release packaging were not performed as part of this migration.

ArrayList and its private iterator subsequently passed this gate; see the migration below.


### ArrayList migration — 2026-09-15

`src/System/Collections/ArrayList.rvn` now implements both constructors, Count/Capacity, the indexer,
Add, Copy, GetIterator and all seven predicate-search operations. Its internal
ArrayListIterator implements Iterator and inherited Disposable. The Raven profile
includes generated class bodies instead of rewriting the historical ArrayList IL;
the separate handwritten search fragment has been removed. Array-backed enumeration
outside ArrayList still uses its existing IL helper.

The ordinary class owns its checked array and count directly. The old intermediate
ArrayListState object served the original value-wrapper model and is no longer needed
for class aliasing. Assignments still share the list; Copy creates independent storage
with shallow element copies. Growth doubles capacity (zero grows to four). An explicit
integer bound faults before capacity multiplication can wrap; its diagnostic is now
`ArrayList capacity overflow`. Capacity slots remain unreadable until initialized.

Iterators and each search capture the buffer and count at entry. Appended items are
excluded, later writes within that buffer remain observable, and a reallocation does
not retarget an ongoing scan. Find/FindAll retain the value seen by the callback even
when the callback replaces its slot. Dispose is idempotent. These are the existing
preview contracts, not a new .NET List version-check policy; changing mutation during
iteration needs a separate API decision. Option results and public parameter names
are preserved (`@match` escapes Raven's keyword while emitting the name `match`).

The shared project has a temporary `NeoCLRLibrarySlice=ArrayList` authoring mode.
`build_runtime_library.py` compiles that class separately against the bootstrap
reference assembly, while other sources consume its reference declaration. This
avoids source/reference identity collisions without allowing arbitrary same-name
application types into library imports. Both compilations use the same project,
Runtime Contract and compiler. Outputs are published together only after every slice
passes admission. Default project compilation still builds the namespace/static
slices; the script builds the complete set. This is not yet a single self-hosted core
assembly or automatic reference-metadata generation.

Validation: 18 runtime collection/query tests, 30 Raven query cases and 63 saved-project
checks pass,
plus `verify_arraylist_library.py` for all seven scans under callback growth and
in-place mutation. The reflection sample's definition-index snapshot changes because
private implementation metadata changed; definition indices are local to a loaded
library, not stable cross-build identifiers. No Raven compiler change was needed.

```sh
python3 docs/experiments/raven-target/verify_arraylist_library.py /path/to/demo/Demo.rvnproj \
  --bridge /path/to/Probe.dll --runtime /path/to/neoclr --system /path/to/System.neoil
```

HashMap subsequently passed the same parity review; see below. Value-shaped APIs such as Date/Time
also need a matched value-type authoring gate; the current class gate must not silently
turn those into reference types.


### HashMap and private instance helpers — 2026-09-15

`src/System/Collections/HashMap.rvn` implements the existing MutableMap contract: construction with
explicit equality/hash callbacks, Count, key snapshots, Find, ContainsKey, TryAdd and
Set. `Map.neoil` now retains only interface declarations and includes generated class
bodies. Runtime Contract configuration and Raven compiler behavior are unchanged.
The shared build compiles this class with `NeoCLRLibrarySlice=HashMap` against the
reference surface, then imports it alongside the other independently checked slices.

Matched public classes may now contain private, nonvirtual, nongeneric instance
helpers with bodies. Public constructors/methods/properties still match the reference
contract exactly. Protected/internal helpers, private constructors, static helpers,
byref/out signatures and generic helper methods remain outside this gate. All admitted
bodies, including unused private helpers, are checked. Emission retains `private`
visibility in IL; the existing same-owner call rule prevents cross-type access. This
uses ordinary CLI private methods rather than adding target-specific Raven symbols
or relaxing guest admission.

Hashing still clears the sign bit, uses chained buckets and doubles bucket capacity
when full. Rehashing uses stored hashes, without calling user callbacks again. Set
preserves an existing key instance. Keys returns independent shallow-copy storage.
Reentrant mutation/lookup from a callback faults; Count/Keys remain readable as before.
Callback faults remain terminal and do not imply rollback or finally-like cleanup.
These are the existing experimental map contracts, not a claim of complete .NET
Dictionary compatibility. The class stores callbacks directly, removing the former
MapCallbacks allocation. Expanded buckets are initialized explicitly before use;
capacity multiplication has an explicit `HashMap capacity overflow` guard.

The generic instance probe's `--private-methods` mode exercises private helpers with
Int32/String/Void, rejects an extra public helper and verifies that external direct IL
cannot call a private helper. Existing runtime tests cover collision chains, growth,
duplicate/missing outcomes, key identity, snapshots, GC retention and reentrancy.
Regeneration and the Raven project suite check the integrated library. Neither Raven
repository is changed by this importer/library slice.


HashMap validation: 18 runtime collection/query tests and all 63 saved-project checks
pass. Generic private-method visibility, private-storage admission and nongeneric
instance-contract probes pass, including malformed-contract/access rejections. Clean
bootstrap regeneration and API inventory/coverage checks pass. No SDK installation
or release packaging was performed by these source migrations.


### First matched value-type gate — 2026-09-15

The library importer now admits a deliberately bounded value representation in
addition to classes: public, nongeneric sequential structs with private mutable
Int32/Int64/Boolean fields, no interfaces and the same public instance API as their
reference contract. Both sides must agree on the value/reference category, field
names, order and types, packing and declared size. This checks the logical layout
contract; it is not a new native-memory layout or interop guarantee. Explicit layout,
reference-containing fields, generic values, interfaces, static factories and readonly
fields/receiver contracts are not admitted by this first gate.

An independent `Probe.Counter` reference fixture supplies actual private field
metadata; its placeholder method bodies remain unusable. Raven source is compiled
separately and checked against it. Value constructors retain instance ownership in
the emitted IL, including private-field access and debugger method identities.
Construction from another value method uses the existing `newobj instance` path.
`ldobj` can copy an admitted library value from its exact managed-reference type;
reads through uninitialized local addresses are rejected. Ordinary guest admission
is unchanged. No new runtime opcode or Raven compiler change was necessary.

This follows the CLI model: value instance methods receive managed-reference
receivers, and `ldobj` copies a value onto the evaluation stack. See
[ECMA-335, Partition I §8.9.7 and Partition II §10.5.1](https://ecma-international.org/wp-content/uploads/ECMA-335_6th_edition_june_2012.pdf)
and [the Microsoft ldobj documentation](https://learn.microsoft.com/lb-lu/dotnet/api/system.reflection.emit.opcodes.ldobj?view=net-7.0).
Keeping constructors as methods preserves ownership checks; making their fields public
or assigning arbitrary namespace functions owner privileges would weaken those checks.
The cost of the bounded gate is that it cannot yet admit the complete calendar types.

`verify_value_library.py` exercises constructor invocation from IL and from Raven,
self copies, copying into independent locals, mutation through managed receivers,
private-field protection and layout/category/API mismatch rejection. Its `--wide`
mode validates 64-bit values beyond Int32 range without relying on a missing console
overload. Both modes pass, as do the existing class and generic-private-helper probes.
Regenerating all current runtime fragments produces identical output.

```sh
python3 docs/experiments/raven-target/verify_value_library.py \
  --compiler /path/to/rvnc.dll --bridge /path/to/Probe.dll --runtime /path/to/neoclr
# Repeat with --wide for the 64-bit storage probe.
```

Date and Time have **not** been ported by this gate. Next, establish matching reference
layout metadata for their stored day number/ticks, retain their Equatable/Comparable
contracts and readonly receivers, and admit checked static Result factories with
private construction. Their current executable IL and public behavior remain in use.


### Time value implementation — 2026-09-15

The Raven profile now uses `runtime/raven/src/System/Time.rvn`, compiled into checked-in
Time fragments. Existing Create/FromTicks Result factories, parameter names, tick
bounds, component properties, equality and comparison remain unchanged. The original
Neo profile still uses its existing IL implementation. Date has not been ported.
API redesign is deferred until after the port; the design proposals are not migration
requirements.

The matched value gate now accepts exact interface contracts and static methods on
value owners. Static factories retain method ownership, and private constructors
retain access protection. Reference calendar metadata now declares the actual private
Int32/Int64 fields, using CLI primitive encodings rather than nominal wrapper tokens.
This preserves the existing logical layout instead of relying on empty reference
structs. It adds no native-layout guarantee.

For public value methods, the importer projects the checked reference contract's
IsReadOnlyAttribute into the existing readonly receiver contract. The runtime verifier
rejects writes through that receiver; source bodies are not trusted to enforce it.
Interface type arguments can refer to the matched implementation owner. This extends
the preceding bounded gate without admitting arbitrary guest value interfaces,
generic value layouts, reference fields or static constructors. No Runtime Contract
configuration, Raven compiler change or runtime opcode was needed.

Shared calendar cases exercise valid and invalid factories, fraction/day boundaries
and comparison in both profiles. Raven-specific negative tests reject receiver writes
and external calls to the private constructor. The saved-project suite also exercises
calendar and clock consumers. Int32 and Int64 admission probes continue to cover
layout/category mismatches and independent value copies.

Validation for this slice: all 63 saved-project checks and all 10 calendar tests
across the two profiles pass. Both scalar value-admission modes and the generic
private-method probe pass. Clean snapshot regeneration, API inventory and coverage
checks pass. No SDK installation or release packaging was performed.


### Date value implementation — 2026-09-15

The Raven profile now uses `src/System/Date.rvn` for Date, preserving the existing Gregorian
algorithm, years 1–9999, day numbers 0–3652058, Result factories, properties and
Equatable/Comparable contracts. Private static calendar helpers and the private
constructor retain their ownership and visibility. Century-cycle clamps are ordinary
conditionals in Raven; their behavior is unchanged. Existing matched value import
support was sufficient: no importer or compiler changes were needed for this port.
The historical Neo profile remains unchanged. Shared pinned .NET calendar cases cover
month boundaries, leap/century cycles and maximum dates, alongside invalid inputs.
API redesign remains deferred.

Date validation: all 10 calendar tests and 63 saved-project checks pass, as do clean
bootstrap regeneration and API inventory/coverage checks.


### Typed host-service authoring and complete Math port — 2026-09-15

All 15 Double Math wrappers now live in `src/System/Math/Functions.rvn` alongside the five Int32
functions. Numerical algorithms stay in the existing native services, preserving
rounding, NaN, infinity and signed-zero behavior. No public API changed.

The bootstrap reference includes `System.Runtime.CompilerServices.RuntimeServices`,
with a fixed catalog of typed host-service signatures. The library importer checks
core identity, static/public nongeneric shape and exact argument/result types before
emitting the existing service call. Normal consumer metadata omits this type; guest
import rejects these calls even when given bootstrap metadata. This is comparable to
the managed/native intrinsic boundary already used by the runtime, not a general
native-call mechanism or a new Runtime Contract configuration. No Raven compiler or
runtime opcode change is required.

The Math consumer/contract probe now also verifies guest rejection of bootstrap
services. Existing numeric tests cover finite domains, NaN, infinity, signed zero,
integer overflow and reversed clamp bounds.

Math completion validation: five runtime tests, 69 consumer results, six rejected
contracts and bootstrap-service guest rejection pass; clean regeneration matches.


### Path wrappers — 2026-09-15

`src/System/Storage/Path/Functions.rvn` implements Combine and GetFileName, retaining their existing public
parameter names and lexical semantics. The checked bootstrap catalog now includes
PathCombine and PathGetFileName; the host still performs platform-specific path
handling. Generated methods retain the existing System.Storage.Path owner for both Raven
and direct IL consumers. There is no normalization, filesystem access or new API
contract hidden in this source migration. Existing path tests exercise empty paths,
rooted paths, Unicode, embedded NULs, separators and runtime-service declarations.

Path validation: the runtime path regression and clean bootstrap regeneration pass.
The final saved-project batch also covers Raven Path consumers.


### File-write outcomes and generic Void matching — 2026-09-15

`src/System/Storage/File.rvn` now constructs the existing WriteAllText outcomes:
Ok(Void), seven typed FileWriteError cases, and a terminal fault for an unknown host
status. Actual UTF-8 encoding, limit checks, filesystem writes and truncation remain
in the native service. ReadAllText retains its existing IL implementation because its
erased native payload needs a different authoring boundary. This is a partial File
port, not a new file API. The fallback return after Fault exists because Raven does
not yet infer that the function never returns; the runtime faults before reaching it.

Library signature matching exposed a metadata representation difference: an external
System.Void generic argument can be read by Cecil as primitive Void, while the
reference contract contains a nominal value token. The importer accepts these as the
same generic argument only when both resolve to the same empty target-core value
identity. This does not equate ordinary no-result and nominal value returns or admit
a foreign/nonempty Void. The source compilation and status tests exercise the real
emission path; `--library-signature-checks` separately checks the matching boundary.
This is neoCLR importer policy, not a Raven compiler change or a change to .NET output.

The current function namespaces are transitional authoring structures. As directed,
Char's functions should eventually become members of a ported System.Char struct;
namespace folders track the declarations in use today and do not settle that design.

File-write validation: all 63 saved-project checks pass, including file operations
and Void propagation. The status regression covers success, every typed failure and
an unknown-status fault without relying on host permissions. Signature checks and
clean regeneration pass.


### Primitive struct family — 2026-09-15

Boolean, SByte, Byte, Int16, UInt16, UInt32, Int64, UInt64, Single and Double are now
Raven structs under `src/System`. Existing CompareTo contracts are preserved, including
unsigned ordering, NaN sorting before non-NaN values and equality of signed zeros.

A checked primitive authoring rule represents intrinsic storage using one private
`m_value` field of the exact primitive kind. This follows the CoreLib approach visible
in [.NET Int32](https://github.com/dotnet/runtime/blob/main/src/libraries/System.Private.CoreLib/src/System/Int32.cs):
the field describes the primitive itself rather than a nested object. Bootstrap-only
metadata supplies that shape. For matched owners, field reads become existing ldobj
loads and the runtime declaration has no extra field. Writes are rejected. The
compiler's exact empty, base-calling default constructor is checked and omitted;
constructors with behavior or other signatures are rejected. The field initialization
warning is suppressed locally because the runtime supplies this storage. Ordinary
guest structs do not receive this projection. This adds no opcode or boxing behavior.

The cost is an explicit, bounded primitive authoring rule in the importer, alongside
separately maintained bootstrap metadata. It is not a general inline-type facility.
Native IntPtr/UIntPtr comparisons and casts are currently unavailable through the
neoCLR reference surface, so their existing IL remains. A small nint comparison
compiles with the same Raven build targeting .NET 11; no general compiler fix is
claimed or made. Int32 and Char remain separate migration work at this point.

`verify_primitive_library.py` checks storage/constructor admission and readonly
rejection. Runtime common-interface tests exercise signed, unsigned, floating and
interface-dispatched comparisons. Build snapshots remain deterministic.

Primitive-family validation: 25 runtime/interface/generic-bound tests, primitive
admission rejection cases and clean regeneration pass. Debug identity maps omit the
backing field because it is not a runtime field.


### Char ownership and complete predicate port — 2026-09-15

`src/System/Char.rvn` now declares the Char struct, its CompareTo member and all
sixteen static predicates. The earlier function-namespace source has been removed.
All existing public signatures remain matched against the reference type, with the
same checked intrinsic-storage rule as the numeric family. The existing CharCategory
native service supplies Unicode categories through the bootstrap catalog; category
ranges and simple code-unit predicates are Raven code.

The current UTF-16 code-unit model is preserved, including isolated surrogate values.
The Unicode/text-model proposal does not change behavior in this port. The requested
move of Char functions onto the struct is now implemented, rather than just planned.

Char validation: four character regressions, 203 independently compiled scalar/Boolean
outcomes and clean snapshot regeneration pass.


### Memberless intrinsic declarations — 2026-09-15

Value and RuntimeTypeHandle now have Raven source declarations under `src/System`.
Declaration-only value imports require matching empty reference shape: no fields,
interfaces, properties or exported methods. Only the verified, empty compiler default
constructor can be omitted. Added storage, methods, constructor behavior and a change
to reference-type representation are rejected. The importer explicitly registers the
matched type even when there are no method roots.

The bootstrap removes its placeholder one-byte layout annotation for these two
intrinsic declarations. This does not specify a native ABI or change their runtime
representations: Value remains an erased payload, and RuntimeTypeHandle remains an
opaque runtime-owned descriptor. Their generated declarations match the prior IL.

Attempting Void itself exposed the current separate-assembly bootstrap boundary:
Raven rejects a unit contract resolved to the implementation assembly rather than the
configured target core (RAVT003). Its existing IL declaration therefore remains. This
needs core-library authoring support, not a replacement Unit type or a weaker unit
contract. No compiler change was made in this slice.

Remaining primitive work: Int32 still combines Raven Divide with its existing IL/native
parsing, formatting, equality and comparison members; IntPtr/UIntPtr need the missing
native-integer operator/conversion surface; Void needs the core-authoring boundary
above. These are explicit follow-ups rather than claimed completed ports.

## Initial time values and clock

`System/Instant.rvn` and `System/Duration.rvn` join the shared authoring project.
Their value layouts, factory names and parameters are checked against the experimental
reference assembly just like Date and Time. Clock.Now and SystemClock use ordinary
CLI interface/class metadata. No Raven compiler changes are required. The bootstrap
LocalDateTime service maps supplied Unix ticks to the existing runtime construction
boundary; it does not select arbitrary host code. See [the contract and validation](instant-clock.md).


System.Fault is now a Raven namespace function over a checked bootstrap-only
RuntimeFailure.Terminate binding. It preserves the dynamic String diagnostic and
existing terminal guest failure service; it does not abort the embedding host.
The no-result consumer signature and compiler control-flow treatment are unchanged.
Three source admission cases, seven fault/query tests and a saved Raven Unicode
failure program pass. Native failure semantics remain owned by the runtime.


All five invariant Func delegate arities are declared in Raven. Admission checks
ordinary CLI runtime constructor/Invoke metadata, exact generic positions and the
complete arity family before emitting the existing delegate declarations. No CIL
stub body is executed; invocation, captures and lifetime remain runtime-owned, as
with CLR delegates. Consumer metadata and unit-result conventions are unchanged.
Six admission checks, 28 delegate tests and the saved Raven delegate sample pass.


BindingFlags is a normal Raven `[System.Flags] enum` with its six existing Int32
literals. Admission requires the exact enum base, underlying field, Flags marker
and literal values. CLI enums contain no authored operation bodies: the importer
lowers the enum declaration to the existing nominal runtime ABI, using the same
intrinsic bit operations as the archived Neo enum emitter. This preserves unknown
bits, equality and reflection filtering without representing the source as a struct.
Consumer enum metadata remains unchanged. The general CLI backing-field flag fix
is independently integrated on Raven main (`266b457f5`); the target declaration
projection remains in neoCLR. No Runtime Contract configuration changes.


Object and UnionAttribute are now empty Raven class declarations, with a checked
parameterless base-calling constructor. Object projects to the existing fieldless
runtime root; the source constructor is not a new runtime allocation API. Its
compiler-facing Equals/GetHashCode/ToString reference members remain recognition
metadata. UnionAttribute uses ordinary CLI Attribute inheritance in source; the
checked target projection retains the existing fieldless marker and no-op Void
constructor. Attribute recognition does not execute that source class or introduce
CLR attribute instantiation. The benefit is explicit source ownership without new
managed behavior; the cost is a bounded declaration projection that cannot be used
for arbitrary classes. Storage, extra methods, constructor effects, different bases
and sealing changes are rejected. No Runtime Contract configuration changes.


Array<T> now owns Empty, Length/Count, element access, ForEach and its private
iterator in Raven. A single private `m_value: T[]` describes intrinsic vector
storage, like the primitive/String authoring rules; writes to that backing field,
ordinary construction and extra storage are rejected. Length is exposed only in
bootstrap shape metadata and lowers to the vector intrinsic. The runtime owns
allocation, bounds checks and delegate invocation. Raven owns the callback loop,
iterator construction, current-position checks and terminal disposal. Later writes
to the original array remain visible through its iterator.

The checked GetIterator body is emitted under the existing internal
ArrayEnumerable.GetIterator<T>(arrayref<T>) ABI: receiver argument zero becomes the
single static parameter without altering the body. This preserves the public array
method surface and existing runtime interface dispatch. The generated array lists
the CLI base-interface closure explicitly; reflected capabilities are unchanged.
Static property metadata now retains its static receiver, including Empty.
The historical Neo array/vector profile remains separate. This reuses the
[generic managed-array contract](generic-managed-arrays.md); no API redesign,
Runtime Contract option or Raven compiler change is introduced.

## Unified TypeInfo acquisition — 2026-09-19

The 73 source slices now include RuntimeContext in place of System.Type. TypeInfo
owns identity, shape and query members; descriptor structural signatures and native
snapshot adapters use TypeInfo throughout. The Type/Info hop is removed from Raven
samples. Object.GetType is Raven code over a checked ObjectTypeHandle native service;
reference upcasts preserve the concrete allocation type for its resolver.

Consumer projects select the existing Raven Runtime Contract settings:
RavenTypeOfAssemblyName=NeoCLR.CoreProbe,
RavenTypeOfInfoType=System.Introspection.TypeInfo and
RavenTypeOfContextType=System.Runtime.RuntimeContext. Source-authoring slices clear
these settings because they shadow the reference types and contain no typeof.
No Raven compiler change is required. Importer fixes retain the provider owner on
internal factory calls and bind source/reference interface pairs before comparing
self-referential signatures. The internal CLI-only Type metadata shell and retained
historical Neo profile are explained in the design document.

Validation: all 73 slices reproduce from source; ownership covers 790 declarations,
including 55 explicit native services. The 33 admission cases, 62 focused Rust
checks and all 74 saved-project checks pass (43 before the direct-native-result
correction, then 31 resumed). The updated acquisition sample also executes an
application class through Object. Language-server checks cover the six interface
kinds, hidden Type/providers, direct typeof queries and RuntimeContext completion.


## Minimal assembly discovery and Info tokens — 2026-09-19

AssemblyInfo and ModuleInfo add two Raven-authored interface/provider slices. The
existing Runtime Contract typeof resolver configuration is unchanged; the target
reference contract adds RuntimeContext.ExecutingAssembly, Sequence-based assembly
queries and MetadataToken/Module properties. Source acquisition and public instance
GetType still return TypeInfo directly. Rebuild matching reference, library and
consumer artifacts; stale snapshots cannot be mixed with the new provider layouts.
Native code supplies checked metadata values, while Raven providers expose the API.
See [the implemented contract and limitations](introspection-design.md#minimal-discovery-and-token-apis-implemented--2026-09-19).

A saved Raven assembly-info sample compiles, imports, verifies and runs, traversing
Demo → System.Runtime and Demo → module → Widget. Language-server checks expose all
eight Info contracts as interfaces, the token/module properties, ExecutingAssembly
and read-only collection capabilities. No compiler semantic/emission changes or
neoCLR policies are integrated into Raven main in this slice.

Validation for this slice: 75 slices reproduce; ownership covers 841 declarations
including 66 runtime services. Sixty-one focused runtime tests, 27 implementation
admission cases and 24 selected saved-project/edit/rejection checks pass. The editor
checks use a freshly generated reference core and the matching language server.


## Sequence results throughout Introspection — 2026-09-19

Following the author's direction, TypeInfo and member/parameter query results now
use Sequence<T> consistently with assembly/module discovery. The reference catalog,
Raven implementations and generated fragments move together. Native services retain
their private array ABI, with the existing checked conversion to the public collection
interface; no new native endpoint or compiler behavior is introduced. Matching samples
use Count, indexing and iteration. Array annotations and writes through the public
indexer are rejected. Snapshot ownership and filtering are unchanged.

Validation: 18 Raven runtime checks, 19 implementation admission cases, 130 signature
checks, nine saved samples plus shared edit/rejection checks, and completion/query/
array-invariance editor checks pass. Snapshot ownership remains 841 declarations and
66 runtime services across 75 source slices. The author closes this Introspection
story here; String API and runtime behavior are a later story.

### Grapheme text update — 2026-09-19

The current development profile contains 77 slices. Char's earlier scalar predicate
port is superseded: Char now owns a grapheme cluster, while UnicodeScalar owns the
numeric classification predicates. String adds Length, Iterable<char> and explicit
GetScalars. The generated String bootstrap fragments omit collection methods for
the archived Neo profile; the Raven profile uses the complete authoring source.
The ownership audit reports 862 declarations and 74 explicit native services.
See [the text contract](design/text-abstraction.md) and
[the runnable example](experiments/raven-target/samples/library-grapheme-strings.rvn).


### Provider-bound Storage descriptors (2026-09-23)

System.Storage.File is now a Raven-authored class with retained provider/Path,
Name and directional stream opening; its static ReadAllText/WriteAllText signatures
remain compatible with native string callers. The baseline library includes its
byte-provider/stream contract dependencies; the full Raven profile adds Directory
and StorageLookup. StorageLookup inherits the byte provider interface and adds
GetFile(Path), leaving byte-only providers valid. The strict importer admits the
exact constructors/members, inherited provider conversion and descriptor arrays.
The SDK disk/memory sample imports these types. No Raven compiler or Runtime Contract
configuration changes are required; use matching regenerated development artifacts.
