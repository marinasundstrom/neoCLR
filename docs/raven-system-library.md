# Authoring the foundational library in Raven

`runtime/raven/System.rvnproj` is the shared authoring project for ordinary
foundational runtime APIs. Its first source is `src/Math.rvn`; additional namespaces
and types should join this project as their importing requirements are validated.
Do not create an assembly per utility namespace. This is an incremental source
migration, not yet a self-hosting build of the complete core reference assembly.

The bootstrap currently has three distinct artifacts:

- `NeoCLR.CoreProbe.dll`: compiler-facing reference metadata for the supported System
  surface. Its placeholder bodies must never execute.
- `NeoCLR.System.dll`: compiled Raven implementation input, currently the five scalar
  Math functions. It is imported into neoIL, not loaded dynamically by the runtime.
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
This transitional mapping is explicit and limited to the existing Math catalog.
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

This importer accepts public namespace functions matching existing reference
signatures, including the bounded generic body gate below. It rejects stateful
containers, unexported helpers, new application-type identities, constrained or nested
generic signatures, byref/out exports and no-result exports. Support for these is future work, not implied by compiling the pilot.
Generated Result adapters are scoped to the library implementation to avoid clashes
with consumer adapters. Reference declarations remain separately maintained and
checked against exports; complete generation of reference metadata from Raven source
is not implemented yet.

Before migrating collections, extend the bounded generic gate to constructed generic
signatures and required constraints. Before migrating foundational type definitions, establish
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
