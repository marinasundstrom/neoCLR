> **Current development status:** SocketError is now migrated and integrated into
> consumer/bootstrap references and the generated System library. The former mixed
> erased-carrier rejection is now an execution check. See the migration check below.
> Earlier bootstrap descriptions document the path to that integration.

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
SocketError now uses that integration in the production reference catalog.
The previous probe stopped at SocketError byref and then receiver initialization.
`LegacyErrors.rvn` retains the original probe name; its nested SocketError now
executes successfully with the migrated core type.

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
unions, equality synthesis or general IUnion conversions. SocketError is the first
public runtime-library migration. The next author-directed step is a batch of
applicable existing union migrations before HttpError/BaseUri.

## Validation for this slice

The focused verifier passes: 103 tracked allocations, two collections, peak 64 and
zero live objects at completion in both the single-assembly and separate-library
arrangements. All six mutated-contract rejection checks and
the integrated SocketError execution check pass. The existing constructor-argument regression
also passes. The prior broader records verifier attempt with the installed
bundle fails before import on record-to-Equatable conversions and ambiguous
Equals overloads; it is not counted as passing. That compiler/SDK validation gap
requires separate investigation. The subsequent SocketError integration updates
the public API snapshot and networking website documentation.


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
three allocations and zero live objects after final collection. Nine independently
altered reference contracts—case identity, return type, output mode, nonempty case,
missing marker, missing case metadata, wrong case ordinal, unresolved case name and malformed protocol—are rejected before producing an implementation artifact. The
existing instance-library regression, including its private `var` storage and five
contract rejections, also passes.

The first compilation uses the existing core, so Raven supplies IUnion in the
source assembly. A second compilation uses the projected reference, which owns the
interface. The bridge accepts that exact supplied-core identity, validates its public
abstract Object-returning getter and imports the union without redeclaring the shared
interface. The same native harness exercises both arrangements. A Raven application
also boxes its source union as Object, casts that reference to IUnion and dispatches
through the shared getter. Direct implicit union-to-IUnion assignment is rejected by
the installed Raven SDK (RAV1504); that compiler conversion gap remains open.
The application checks the returned case identity and reclaims both tracked allocations.

This follows ordinary CLI interface ownership/dispatch rather than introducing a union
opcode or requiring each library to synthesize a competing interface identity. The
benefit is one shared contract for multiple library slices; the cost is another
provisional compiler bridge mapping. The selected member shape remains Raven-driven,
not a new platform case-mapping standard. Production core packaging still needs to
supply that interface once and project each migrated union's reference metadata.

SocketError is the first migrated public error. Its reference, runtime callers and
API documentation now use the source-generated contract. The fixture now compiles a separate Raven consumer
that constructs and matches the projected core union; that compilation does not yet
validate execution through the production consumer pipeline.
Private storage `var`/`val` should be used normally; checking the emitted field layout
is not a requirement to write explicit `field` syntax. The fixture uses qualified
empty-case patterns so a bare identifier is not interpreted as a new binding.


## Raven metadata at the bridge boundary

The bootstrap fixture preserves Raven's case metadata names, logical names and
ordinals and checks them against the source family before import. This is a bounded
Raven adapter: no `Raven.Runtime.CompilerServices` dependency appears in the native
implementation fragment. The fixture copies the selected case attribute definition
and usages, not every custom attribute in the source assembly.

`RavenUnionCompanionAttribute` associates a generic union's separate case container
with its carrier. Nongeneric unions in this fixture own their cases directly and do
not need a companion. A separate metadata-only probe compiles a generic producer and
consumer, checks the association and rejects missing/wrong companion targets:

```sh
python3 docs/experiments/http-error-unions/verify_metadata.py \
  --bundle /path/to/matching/neoclr-bundle \
  --bridge docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll
```

This validates compiler metadata consumption, not generic-union execution in neoCLR.
The Raven convention remains compiler-owned. No platform case-map standard is being
introduced; reconsider it later without blocking the class-library/HTTP objective.


## Replacing an existing reference family

The bridge now exposes a bounded source-to-reference projection command:

```sh
dotnet /path/to/Probe.dll --project-union-reference \
  /path/to/Implementation.dll /path/to/NeoCLR.CoreProbe.dll \
  System.Networking.Sockets.SocketError /path/to/output/NeoCLR.CoreProbe.dll
python3 docs/experiments/http-error-unions/verify_reference.py \
  --bundle /path/to/matching/neoclr-bundle \
  --bridge docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll
```

It copies the supported empty-case union shape emitted by Raven, including members,
field layout and selected case metadata. Existing carrier and nested case definitions
are updated in place, preserving references from other core signatures. Replacement
requires the same case identities; a mismatch fails without an output artifact.
Shared support definitions are reused on subsequent projections. As in the previous
fixture, concrete reference method bodies throw and are never the runtime implementation.
This remains a bounded projection, not a general CLI assembly merger or lossless
custom-attribute copier. Generic unions remain outside this command. Nongeneric sequential payload families
are covered by the later HTTP library probe below; overlapping payload layouts remain rejected.

The focused check projects all thirteen SocketError cases from normal Raven syntax,
compiles consumers of construction, matching and Socket.Connect's nested Task/Result
signature, then recompiles and projects again with core-owned support definitions.
It also imports the resulting native union implementation. A validated standard
empty-case family now takes precedence over the legacy error catalog's erased-storage
constructor/default assumptions. The check also recompiles and imports the existing erased SocketError source as a
regression; legacy error carriers retain their existing behavior.

This follows CLI reference type identity while taking the reference shape from emitted
source rather than maintaining a second hand-written union declaration. The cost is a
staged compile/project/recompile bootstrap that must be wired into SDK production.
The public SocketError source, generated System library, call adapters and API
reference snapshot have now migrated together. This check does not claim that a network application has
executed with the replacement. The runtime remains independent of Raven attributes.


## Integrated SocketError execution

```sh
python3 docs/experiments/http-error-unions/verify_socket_union.py \
  --bundle /path/to/matching/neoclr-bundle \
  --runner target/release/examples/measure_async
```

Core generation compiles embedded SocketError Raven source using the bridge's matching
compiler, then projects it into both consumer and bootstrap references. No extra binary
shape snapshot or checkout path is required by a packaged bridge. The generated System
library supplies IUnion once as an ordinary interface; it does not execute Raven
metadata attributes. Per-case Is*/Get* helpers are removed, not a requirement carried
forward from the manual carrier. The default has HasValue false and formats as Empty.

The integrated check covers all 13 display names with unqualified match cases,
matching/nonmatching cases, inactive Value, nesting in
HttpError, copying and boxed values under GC pressure: 127 allocations, three collections,
zero live objects. Socket client, separate neoCLR listener/client and selected HTTP
success/failure checks pass with the matching artifacts. The next author-directed
work is batch migration of applicable existing unions before resuming public HttpError
and BaseUri. Generic families remain a separate validation boundary.

## DNS and URI error migration batch

DnsError and UriError now use normal union declarations alongside SocketError.
Core generation compiles each embedded source against the preceding projected
reference, reusing the existing IUnion and case metadata definitions. The public
case names and producer outcomes are unchanged; handwritten Is*/Get* helpers are
removed. Use patterns and rebuild matching SDK/library/application artifacts.
Defaults are inactive and format as Empty.

The focused verifier now covers every case in these three families, direct and
boxed display, defaults, copies and nesting in data-bearing source unions. A
separate all-value payload union remains rejected: Raven emits overlapping payload
storage, while the bridge only admits explicit layout for empty-case unions. The
mixed managed payload probe is supported. This is a concrete remaining layout
boundary, not evidence that arbitrary payload unions work in the class library.

Existing .NET API comparisons still apply: neoCLR models expected DNS/URI failures
as result values rather than copying exception inheritance. This slice changes
implementation and case access, not the error categories or a claim of .NET ABI
compatibility. Enums remain an option for named constants; this is a migration of
existing union contracts, not a new universal representation rule.

Batch checks: 147 allocations, three collections and zero live objects. DNS/TCP
passes with 1856 allocations and zero live objects; its expected invalid-name case
is checked through a pattern. The all-value payload layout is explicitly rejected.

URI grammar/resolution, Object checks and the recorded .NET comparison also pass.

## Remaining empty-case batch and enum distinction

The remaining StreamError, TextReadError, StorageLookupError, FileReadError,
FileWriteError, ConsoleReadError, Utf8SliceError, Int32ParseError, IntegerDivisionError
and SingleError now use standard union declarations. All named cases, boxing and
inactive defaults pass the integrated verifier (254 allocations, five collections,
zero live objects). EntryKind is instead an enum with File = 1 and Directory = 2;
its comparisons, numeric values and boxing/unboxing round-trip are covered.

A library case type in an `isinst` operand can lack Cecil's value-type signature
flag. The bridge resolves the supplied-core definition before admitting that token;
it does not infer a foreign type's representation from its name. Wrong-family case
patterns remain false. Handwritten Is*/Get* accessors are removed. Tests of truly
uninitialized locals now disable CLI local initialization; default union values
are valid inactive values.

The larger directory/provider fixture exceeds the CLI's 100,000-instruction cap
with the generated carrier initialization. Its optional trusted `--runner` uses an
explicit 10,000,000-instruction budget and a 256-object GC budget. Production limits
are unchanged; this migration does not claim equal storage or execution cost.

Generic Option, Result and TaskOutcome remain handwritten. The generic metadata
probe still compiles separate consumers, but the library projector rejects generic
families before writing a reference. It needs companion ownership, generic parameter
substitution and generated payload-method import before those carriers can migrate.
Option/Result also need preservation of Propagatable residual/output contracts;
TaskOutcome needs its terminal-state/task integration checked. Per-case helpers are
not a future requirement. Payload-bearing explicit layout is separately rejected.
These are tracked implementation boundaries, not permanent exemptions.

The Enum helper slice now supplies both TypeInfo and generic overloads on
System.Enum, typed value snapshots and boxed formatting. See the
[verified sample](../enum-helpers/README.md). The historical Neo bootstrap uses
explicit legacy carrier/BindingFlags snapshots; the Raven profile uses the migrated
unions. This compatibility boundary does not restore Is*/Get* requirements.

The applicable nongeneric empty-case migration batch is complete. Generic Option,
Result and TaskOutcome remain documented manual exceptions pending their specific
projection/propagation work. Author direction on 2026-09-25 places other additions
on hold and returns work to HTTP. The nongeneric data-bearing projection checkpoint below is complete. Public HTTP
error bindings and string/Uri BaseUri resolution are the next integration work.


## Nongeneric payload-bearing library projection

The HTTP prototype now projects a normal `HttpError` declaration into a separate
compiler reference and imports its Raven source bodies. Cases retain UriError,
DnsError, SocketError or a message string. A separate Raven consumer compiles case
construction and nested patterns against that projected reference. Native execution
uses the imported library methods directly; this is not yet a public HTTP API or
an end-to-end Raven HttpClient integration.

```sh
python3 docs/experiments/http-error-unions/verify_payload_library.py \
  --bundle /path/to/matching/development-sdk \
  --bridge docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --runner target/release/examples/measure_async
```

Admission remains bounded: nongeneric sequential fields for managed payload unions,
or the existing empty-case explicit-layout exception. Source/reference case maps,
fields and signatures must agree. The VM still sees ordinary records/interfaces;
Raven metadata is consumed by the development bridge, without a runtime dependency
or a new platform-wide convention. Generic companion projection and overlapping
nonempty payload layouts remain rejected.

The test checks payload extraction, defaults, copies and boxing under GC pressure:
101 allocations, four collections, zero live objects. Changed payload contracts and
all-value overlapping layouts are rejected before use. No invalid reference is
published on projection rejection. A private generated formatting helper exposed an
adapter visibility bug: single-argument conversions now stay at the validated
nonpublic call site. Visibility is preserved and a direct external call is rejected;
more complex nonpublic conversions still require dedicated adapter support.

This follows the existing CLI/value-copy comparison and .NET error-model research:
expected HTTP failures retain domain causes instead of recreating Exception subclasses.
The prototype cases are integration evidence, not a commitment to the complete final
HttpError taxonomy. The current published development HttpClient/handler/server
signatures still return string errors; wire the new union into those contracts next,
then add Uri/string overloads and BaseUri resolution with matching API documentation.

## Public HTTP integration

The development library now declares HttpError in ordinary union syntax and uses it
in client/handler/server contracts. The isolated payload-library probe uses the
separate ProbeHttpError name so its four-case experimental family cannot overwrite
the public family. Projected consumer binding follows the selected core metadata,
checks signatures and preserves constructors and conditional extraction. Generic
companion and overlapping-layout rejection remain unchanged. See the HTTP client,
server and JSON samples for execution through the actual public API.
