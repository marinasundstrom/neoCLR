# API documentation maintenance

The `/docs/` section is built with pinned DocFX 2.80.1 (.NET 10). It describes the
development API following **Preview 9**. Public APIs must have reference coverage;
update descriptions, signatures, navigation and generated metadata in the same
change as the API. The limited async overview was the Preview 9 release baseline,
not a permanent exemption for other APIs.

For each public API change, include the namespace/type in `filter.yml`, author useful
XML documentation, refresh metadata from the current bridge, and check navigation
from the API landing page to its type and members. Renames remove stale entries and
record migration information. Renderer exclusions need a linked manual reference
entry with the exact signature and behavior. Run the combined website build; passing
snapshot hashes alone does not establish browsability or adequate descriptions.

## Namespace overview

Keep [the namespace overview](namespaces.md) aligned with implemented public APIs.
Add a namespace when its APIs land, explain its contents and link its generated
reference or on-site guide. Distinguish namespaces from types and compiler-only
support, mark missing member coverage, and keep proposed namespaces out of the
current inventory.

## Build the complete website

From the repository root:

```sh
dotnet tool restore
npm ci --prefix website --ignore-scripts
python3 scripts/build-website.py
```

Serve `target/website`; the reference is at `/docs/`. The Pages workflow builds and
uploads both sections as one artifact. Deployment remains the existing manual
workflow on main. A local build or a push does not publish either section.

## Refresh signatures and XML descriptions

Use a freshly built Raven-target bridge (see its README for prerequisites):

```sh
mkdir -p target/api-docs/input
dotnet docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --reference-core target/api-docs/input/NeoCLR.CoreProbe.dll
python3 scripts/build-api-docs.py \
  --refresh target/api-docs/input/NeoCLR.CoreProbe.dll --check
```

The DLL must come from the current declarations, not an old published bundle. The
bridge output folder may differ by configuration. Alternatively supply the freshly
generated `demo/NeoCLR.CoreProbe.dll` from a bundle built at the same source revision.

DocFX reads that compiler reference assembly and the companion
`NeoCLR.CoreProbe.xml`. We currently author this XML sidecar directly: it is **not**
claimed to be compiler-emitted from Raven comments. It uses standard documentation
IDs. Keep descriptions aligned with the Raven implementations and tested feature
samples. Moving descriptions into source comments and automatic XML emission can
follow when that path is validated for the library build.

Commit the generated `api/*.yml` and `snapshot.json`, never the DLL or generated HTML.
This lets website CI render the reference without a Raven checkout. The manifest
records the input DLL hash, source/configuration fingerprints and output hashes.
It catches stale snapshots, but does not certify that an arbitrary supplied DLL was
built from those sources. Regeneration must use the current bridge. The build checks
that each included type/member has an XML summary and treats DocFX warnings as errors.

## Current scope and limitation

`filter.yml` selects Task, Promise, TaskQueue, TaskState and the public Thread/ThreadPool
APIs in System.Concurrency. Other feature areas have
an overview linking their on-site guides. TaskOutcome and supporting types are shown
in signatures without implying complete reference coverage.

Existing reference coverage gaps (not exemptions from the policy):

- Arrays and collection interfaces/implementations, delegates, queries and operators.
- Option/Result carriers, TaskOutcome and composition/propagation helpers.
- Primitives, text/encoding and process APIs (Console is now covered).
- Time/calendar, introspection and public resource/interop contracts.

Backfill these as each area is developed. The next Storage/Streams slice must include
its public reference pages from the start. Feature guides remain useful context but
do not count as member reference coverage. Internal bootstrap/runtime services are
not public application APIs and do not belong in the application reference.

DocFX 2.80.1 fails in `YamlModelGenerator.AddSpecReference` when Roslyn sees the
neoCLR-specific `Func<System.Void>` argument. The filter excludes exactly Post, Run
and OnCompleted; `callbacks.md` documents them in Raven notation and the index makes
the omission explicit. Do not substitute `Action` or another type in the reference
assembly merely to make DocFX accept it. Remove this workaround after verifying a
renderer that supports these signatures. The XML keeps descriptions for those IDs
ready for that change; update the callback guide alongside them in the meantime.

Generated C#/VB declarations are metadata notation, not supported application
frontends. This tradeoff reuses established .NET documentation tooling without
requiring neoCLR's type system to obey every C# restriction. A custom reference
renderer would avoid the notation mismatch but add maintenance; defer it for this
release. Primary source: [DocFX assembly input and XML documentation](https://dotnet.github.io/docfx/docs/dotnet-api-docs.html),
reviewed 2026-09-23. Task/.NET contract comparisons remain in the existing Task
feature guide and design records; no runtime contract changes are made here.


The API header uses plain neoCLR branding. Its small logo margin is supplied by
`template/public/main.css`, using the documented
[DocFX custom template hook](https://dotnet.github.io/docfx/docs/template.html).
The combined site builder converts root-relative reference links, including DocFX's
client-side navigation JSON, to page-relative links for GitHub Pages project paths.

The development stream slice includes System.IO in generated reference.
FileOutputStream.Flush and OutputStream.Flush return Result<System.Void, StreamError>, which DocFX 2.80.1
cannot render; its exact signature and full contract live in [streams.md](streams.md).

The host-facing Rust Fault/FaultCode API and debugger `fault_code` field are covered
in [faults.md](faults.md), linked from the API guide and navigation. These are host
contracts, not synthetic CLI types for DocFX metadata. Keep that complete member/code
reference synchronized with `src/fault_code.rs` and the debugger snapshot.

System.Storage.Metadata, EntryKind and StorageLookupError are now included in DocFX
selection. Their complete metadata reference and [lookup guide](storage-lookup.md)
describe development host-path lookup. StorageProvider.GetItem/GetFile/GetDirectory are the integrated lookup contract; concrete
providers remain [sample implementations](storage-experiment.md).
No renderer exclusion is needed for these new signatures. FileText and its legacy text error types are now selected too; only WriteAllText is
excluded because DocFX 2.80.1 cannot render Result<System.Void, FileWriteError>. Its
complete manual reference is [WriteAllText](storage-items.md#writealltext).
StorageItem, File, Directory and StorageProvider have generated type/member coverage.
File/Directory are provider-implemented interfaces; inherited Name/Path are documented
on StorageItem. FileText contains the renamed native static helpers. Path coverage follows.

System.Storage.Path is now a platform class with Parse, immutable properties and
lexical equality, alongside the existing static native string helpers. Path and
InvalidPathError are selected for generated reference, including both legacy methods.
The Storage sample imports this platform type rather than defining its own. Its
memory provider remains application-owned; FileSystem is now an integrated host provider.
System.Storage.StorageProvider resolves items through three documented members.
The transitional StorageLookup type and provider byte methods have been removed;
byte opening belongs to File implementations.

InputStream and OutputStream are selected with their concrete file implementations.
OutputStream.Flush shares the exact unit-valued Result renderer exclusion; its
signature and provider contract are linked from the generated type page to
[the Flush reference](streams.md#flush). Directional interface dispatch is exercised
by the disk/memory SDK sample; memory stream classes remain sample implementations.


FileSystem and the Directory traversal/GetItems members have generated API entries.
StorageLookupError.InvalidRange and LimitExceeded cover the bounded snapshot contract.
LocalFile/LocalDirectory are internal implementation classes and are not public APIs.


Development streams moved from System.Streams to System.IO. TextReader, StreamReader,
TextReadError and SeekableStream are selected for generated member documentation.
TextReader/StreamReader use strict bounded UTF-8 reading; FileInputStream adds optional
positioning. Public names and exact unit-return exclusions were migrated together.


The provisional raw worker cancellation services are not managed reference-core
members. Their exact low-level signatures, outcomes and limitations are documented
in [Pending reads](pending-read.md#experimental-per-worker-cancellation-services).
No public Thread/Task member was added by that runtime experiment.


## Console and text output (development)

Console remains a static class. Its common methods and In/Out/Error properties,
byte-stream factories, ConsoleReadError, TextReader.ReadLine and TextWriter/StreamWriter
are selected and documented from the matching reference assembly. The Console
[guide](console.md) covers ownership, bounds, Result/Option outcomes and host support.
DocFX excludes exactly TextWriter.Flush and StreamWriter.Flush because their
Result<System.Void, StreamError> signatures hit the existing renderer limitation;
that guide is their linked manual reference. Internal ConsoleInputStream and
ConsoleOutputStream are provider implementation classes, not public APIs.

## Object and Value coverage — 2026-09-23

Object, GetType, ToString, ReferenceEquals, Equals, GetHashCode and Value have generated
summaries and an [on-site guide](objects.md). The equality/identity surface is a
bounded development implementation: String identity and most boxed primitive
virtual equality/hash remain unsupported. Static Object.Equals is not yet declared or implemented.
Value has no public member API; low-level operations
and its temporary role are explained in the guide. TypeInfo and MemberInfo now have
generated type/member coverage and an [introspection guide](introspection.md).
AssemblyInfo and ModuleInfo now also have generated type/member coverage.
FieldInfo, MethodInfo and PropertyInfo now have generated member coverage.
ParameterInfo and BindingFlags now have generated member coverage too. Parameter
owner identity is internal; the guide distinguishes this from a future Member property. Object/Value source files participate in the reference
snapshot fingerprint.

The metadata refresh normalizes two DocFX core-type assumptions: Object's declaration
is rendered as `public abstract class Object`, and Value does not advertise inherited Object methods
from the reference-only ValueType scaffold. The latter is not an executable Value
member. This normalization changes documentation only, not reference metadata.

Object's protected parameterless constructor is described in the Object guide. It
is only a derived-construction contract; direct construction is rejected because
Object is abstract. The runtime's public constructor entry supports validated base
chaining; reference accessibility and constructor-state checks restrict its use.


2026-09-24: System.HashCode and the compiler-only IsExternalInit marker are included
in metadata selection with summaries for every newly exposed member. The Object guide
and namespace overview describe the bounded record/hash contract. HashCode has no generic
or comparer overloads. IsExternalInit is metadata support, not an executable service.

The development async builder reference now selects IAsyncStateMachine, ITaskAwaiter
and AsyncTaskMethodBuilder<T>. Exactly ITaskAwaiter.OnCompleted is excluded because
DocFX cannot render Func<Void>; [the manual protocol guide](async-builders.md) covers
its signature and semantics. These temporary compiler APIs may be removed when
runtime-owned suspension replaces state machines. Application-facing ref metadata
is an importer projection over bootstrap retained-owner helpers.

Development Path Object alignment includes Equatable<T> and Equals(T) in generated
metadata. Typed operands remain non-nullable; only the separate Object equality
overload accepts a nullable reference. The Path hash and overrides are documented
alongside the Storage guide.
