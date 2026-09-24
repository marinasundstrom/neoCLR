# API reference maintenance

RavenDoc publishes this reference inside the single neoCLR website at `/docs/`.
It reads a checked-in compiler reference assembly and the authored XML sidecar,
then renders Raven signatures with the same layout, navigation and development
notice as the Markdown guides. No DocFX build, metadata YAML or second site exists.

The site documents development after Preview 9 and may be published before a
runtime release. Generated reference pages are development documentation; per-page
release labels and published sample/toolchain versions remain authoritative.

## Build and refresh

```sh
python3 scripts/build-website.py
python3 -m unittest discover -s scripts -p 'test_build_website.py'
```

Normal CI requires only Python and .NET 10, using the pinned portable
[RavenDoc build](../tools/ravendoc/README.md). To refresh after a public API change,
first build the matching Raven-target bridge following its README, then:

```sh
mkdir -p target/api-docs/input
dotnet docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --reference-core target/api-docs/input/NeoCLR.CoreProbe.dll
python3 scripts/build-api-docs.py \
  --refresh target/api-docs/input/NeoCLR.CoreProbe.dll --check
python3 scripts/build-website.py
```

Commit `reference/NeoCLR.CoreProbe.dll`, `snapshot.json`, `types.json` and relevant
XML/Markdown changes together. The assembly is a small documentation/compiler
reference, not executable runtime code. The build copies `NeoCLR.CoreProbe.xml`
beside it locally; that generated copy is ignored. Source fingerprints and the
assembly SHA-256 reject stale/corrupt snapshots. A fingerprint is not proof that an
arbitrary input DLL was produced by those sources: always regenerate from the
freshly built matching bridge. CI never requires that compiler checkout.

## Public API coverage

Add each public type to `types.json` and provide useful XML summaries, parameter
and return descriptions, error/ownership behavior and examples. XML descriptions
are currently authored directly, not claimed to be emitted from Raven comments.
The current loader also accepts Markdown prose in those descriptions; put generic
type spellings in backticks. Update namespace navigation and the API landing page
in the same change. Validate the complete site, not just the snapshot hash.

RavenDoc now renders the previously excluded TaskQueue Post/Run, Task OnCompleted,
ITaskAwaiter OnCompleted, FileText WriteAllText and OutputStream/FileOutputStream/
TextWriter/StreamWriter Flush signatures, including Func<Void> and Result<Void,E>.
Their Markdown guides remain behavioral reference, not renderer exclusions.
Existing selected types include Tasks, Thread/ThreadPool, Storage, IO, Console,
Object/Value, HashCode, Equatable, compiler async support and Introspection descriptors.
Host Fault/FaultCode and debugger fields remain a complete manual [reference](faults.md),
not synthetic CLI types. The Rust-only StringValue payload and owned-text migration
are documented in the [Object guide](objects.md#rust-host-string-payloads) and source
rustdoc; StringValue is not a guest CLI type selected for RavenDoc.

Sequence, Collection, Iterable and Iterator now have generated reference coverage,
including String construction and its read-only grapheme indexer. String Count is
an explicit Collection implementation, visible through Sequence/Collection only.

Remaining coverage gaps are not exemptions: arrays/remaining collections, delegates, query
operators, Option/Result and TaskOutcome helpers, primitives/text/encoding/process,
time/calendar and remaining public resource/interop contracts need reference coverage
as developed. `namespaces.md` inventories implemented APIs; do not add proposed
namespaces as shipped. Compiler-reference carrier/scaffold notation may differ from
idiomatic application source; prefer the tested guide examples for application code.

The generated reference starts at `/docs/api/`. `legacy-routes.json` preserves the
former DocFX type/namespace routes and member anchors as redirects into RavenDoc.
Keep those routes stable for external links. New links should use xrefs or the
current generated routes. The build validates all local page, sample and asset links
under both root and project-path hosting. Publication remains manual and separate
from runtime releases.


`exclusions.json` records the one reference-only ThreadPool constructor that is not
an application API. This preserves the former scaffold exclusion; it is not a
renderer workaround. The combined build verifies every selected type and authored
member has a generated route, checks XML summaries, rejects missing member summaries
and validates links across the entire result. Documentation-only XML changes can
refresh the manifest against the existing matching reference assembly; source/API
changes still require regeneration from the bridge.

Char now has generated coverage for FromString, ToString, Equals(Char) and CompareTo.
Its grapheme semantics and boxed Object behavior are documented; other primitive
reference coverage remains incremental.

String's existing public text methods, properties and operators now have generated
coverage. Its reference-only parameterless scaffold constructor is excluded in
exclusions.json and explained in the [Object guide](objects.md#string-through-object-development);
it is not an executable application API. UTF-8 slice errors are described on the
method; remaining text/encoding type coverage is still incremental.


String.Intern now has generated member documentation. Its execution-owned retention
and host quotas are described there and in the manual Fault reference, including
InternPoolLimitExceeded. This does not add a public interning-pool type or IsInterned.
