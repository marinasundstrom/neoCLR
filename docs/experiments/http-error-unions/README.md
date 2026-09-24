# Standard-syntax HTTP error union investigation

This reduced application probe authors cases, a computed property and an override
directly in Raven `union` declarations. It is not the public HttpError API. Standard
syntax is the [class-library default](../../raven-conventions.md); manual carriers
require a specific reason and a condition for revisiting that exception.

## Reproduce

Build `../raven-target/Probe.csproj` with the matching Raven checkout, and use a
matching development neoCLR bundle containing that bridge, core metadata and Raven
SDK. The shape reporter uses Mono.Cecil from that local bridge build. Build the
`measure_async` example for forced collections and final cleanup measurements.

```sh
python3 docs/experiments/http-error-unions/verify.py \
  --bundle /path/to/matching/neoclr-bundle \
  --runner target/release/examples/measure_async
```

The script compiles in a temporary directory, inspects the CLI shape, imports and
verifies the program, and executes it under collection pressure. It checks:

- Default unions report no case; unsuccessful patterns do not invent a payload.
- Nested standard unions, empty-case-only unions, computed properties, authored
  ToString and generated display through Object.
- Copies and boxed values retain their payload after the original is replaced.
- Live values survive collections; all tracked objects are reclaimed at completion.
- Constructor receiver initialization is accepted; the same initobj in an ordinary
  method is rejected by the importer.
- Ordinary out methods still must assign their output. A union extraction method
  that returns true without assignment also faults.
- Explicit layouts lacking the union marker, containing payload data, or overlapping
  the tag remain rejected.
- The same program consumes a separately compiled union library, with matching
  output and collection/cleanup checks.
- Mixing the old erased SocketError into this carrier remains rejected during
  runtime verification because its System.Value field has no managed default.

## Findings and implemented bridge support — 2026-09-24

The compiler emits a sequential value type with a byte tag and typed fields for all
cases containing managed references. Nested case types contain their payloads.
Constructors initialize the receiver with `initobj`; `TryGetValue` writes its output
only on the matching branch. The compiler also emits Value, HasValue and an IUnion
interface in this application. The physical layout remains compiler-owned; the
report and tests describe this bounded fixture, not a permanent neoCLR ABI.

The bridge now admits known error byrefs, value-constructor initialization of its
own receiver, and conditional output for a recognized union's public Boolean
TryGetValue method with one out parameter of a nested value case. Recognition
requires the core UnionAttribute. Both method declarations and call-site assignment
tracking use the runtime's existing `out(true)` contract. Ordinary outputs retain
`out`; no new VM instruction or relaxed managed default is introduced.

Empty-case-only unions use explicit CLI layout in Raven. The importer now admits
that bounded shape when a marked sealed value carrier contains one private byte tag
at offset zero and one private field per empty nested value case at a positive
offset. There is no payload data whose overlapping storage must be preserved, so
these become ordinary managed field slots. This is not permission to import general
explicit-layout structs or nonempty overlaid cases.

Application/dependency imports are covered above. The bounded bootstrap experiment
below now imports empty-case union implementations.
Production reference catalogs and public API migration still need integration.
The previous probe stopped at SocketError byref and then receiver initialization.
`LegacyErrors.rvn` retains that mixed-form investigation; it now imports but the
runtime verifier correctly rejects its non-defaultable erased payload.

## Comparison and remaining decisions

Raven's language spec currently describes Value as the only instance storage; the
artifact here has tagged typed fields instead. Reconcile the spec independently.
Do not infer .NET binary compatibility from a metadata member convention. Existing
comparisons are in [union conventions](../../union-convention.md) and
[unions and enums](../../unions-and-enums.md). CLI zero-initialization is useful here,
but differs from the legacy neoCLR carriers whose erased storage has no default.
We retain that distinction rather than fabricate a valid legacy error case.

Compiler-owned case machinery reduces handwritten library code; it requires explicit
bridge support and validation. This slice does not cover generic or payload-bearing explicit-layout
unions, equality synthesis, all IUnion interface calls or public runtime-library migration.
Next, resolve the mixed-carrier boundary and production reference integration before shipping
HttpError/BaseUri. Existing carriers are migration candidates, not an instruction
to rewrite all working unions in one change.

## Validation for this slice

The focused verifier passes: 103 tracked allocations, two collections, peak 64 and
zero live objects at completion in both the single-assembly and separate-library
arrangements. All six mutated-contract rejection checks and
the mixed-legacy default rejection pass. The existing constructor-argument regression
also passes. The prior broader records verifier attempt with the installed
bundle fails before import on record-to-Equatable conversions and ambiguous
Equals overloads; it is not counted as passing. That compiler/SDK validation gap
requires separate investigation. No website or public API signature changes occur
in this slice, so the existing API snapshot is unchanged.


## Empty-case union bootstrap

```sh
python3 docs/experiments/http-error-unions/verify_bootstrap.py \
  --bundle /path/to/matching/neoclr-bundle \
  --bridge docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --runner target/release/examples/measure_async
```

This fixture compiles a standard `Probe.Limit` union with two empty cases and authored
members. A test-only command creates a separate core reference containing its carrier,
cases and IUnion metadata. Reference method bodies deliberately throw. The normal
`--library-implementation` path matches the family and imports the source bodies as
named runtime library types; an explicit neoIL harness exercises the resulting fragment.
It does not execute the reference stubs or substitute application type identities.

The bootstrap path compares field layouts, case identities, signatures, parameter
names/output modes, properties and interface mappings. It retains conditional outputs
in both declarations and calls. Generated static helper names use normal metadata-name
encoding. This is a bounded empty-case family path, not a generic source-to-reference
packaging system; payload-bearing and generic unions still need their own validation.

Native value constructors hold an unpublished receiver capability. To retain its
existing checks, the importer lowers the compiler's receiver `initobj` to checked
default writes to each field; an empty case needs no field writes. The runtime and
its general default-value policy are unchanged. This adapts CLI constructor behavior
to neoCLR's construction model without hand-authoring union constructors in Raven.

Validation covers both cases, unsuccessful extraction, a computed property, empty
defaults, boxed-copy display after reassignment, and Value extraction. It reports
three allocations and zero live objects after final collection. Five independently
altered reference contracts—case identity, return type, output mode, nonempty case,
and missing marker—are rejected before producing an implementation artifact. The
existing instance-library regression, including its private `var` storage and five
contract rejections, also passes.

The fixture compiles against the existing core, so Raven supplies IUnion in the
source assembly. The bootstrap currently requires that source protocol and a matching
reference declaration. Making the production core own the protocol once, and admitting
that core-owned reference during compilation, are explicit follow-up work.

No public core-library type has migrated yet. Production reference/catalog integration,
consumer binding and API documentation must accompany the first real migration.
Private storage `var`/`val` should be used normally; checking the emitted field layout
is not a requirement to write explicit `field` syntax. The fixture uses qualified
empty-case patterns so a bare identifier is not interpreted as a new binding.
