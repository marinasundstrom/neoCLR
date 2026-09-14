# Authoring the foundational library in Raven

`runtime/raven/System.rvnproj` is the shared authoring project for ordinary
foundational runtime APIs. Its sources include `src/Math.rvn`, `src/Linq.rvn`, `src/Int32.rvn` and `src/Char.rvn`; additional namespaces
and types should join this project as their importing requirements are validated.
Do not create an assembly per utility namespace. This is an incremental source
migration, not yet a self-hosting build of the complete core reference assembly.

The bootstrap currently has three distinct artifacts:

- `NeoCLR.CoreProbe.dll`: compiler-facing reference metadata for the supported System
  surface. Its placeholder bodies must never execute.
- `NeoCLR.System.dll`: compiled Raven implementation input, currently five scalar
  Math functions, seven collection/query terminal overloads, Int32.Divide and seven character predicates. It is imported into neoIL, not loaded dynamically by the runtime.
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
invalid bounds. Double operations still use the existing native-service bodies.

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
specific error-type imports to avoid a collision with the separate `System.Error`
type, and fully qualifies `System.Result<...>` in return annotations. The observed
unqualified-return diagnostic issue was fixed independently; see the
[resolution follow-up](raven-target-evaluation.md#resolution-follow-up--2026-09-14).

## Current limits and next gate

This importer accepts public namespace functions and static API methods matching existing
reference signatures, including the bounded generic body gate below. It rejects stateful
containers, unexported helpers, new application-type identities, constrained
generic signatures and unsupported constructed shapes, byref/out exports and no-result exports. Support for these is future work, not implied by compiling the pilot.
Generated Result adapters are scoped to the library implementation to avoid clashes
with consumer adapters. Reference declarations remain separately maintained and
checked against exports; complete generation of reference metadata from Raven source
is not implemented yet.

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

`src/Linq.rvn` implements `System.Linq.Operators.ToList<T>`, `First<T>`, `Last<T>`
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
