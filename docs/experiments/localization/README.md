# Localization foundation — 2026-10-21

## Direction and comparison

The author requested a small Localizer and JSON provider, pointed out .NET's
IStringLocalizer<T> resource association, and specified key fallback, optional
missing-key reporting, optional scope, locale/neutral/invariant filenames,
configurable directory and configurable invariant fallback.

.NET provides IStringLocalizer, typed resource association, a factory, localized
string metadata and culture fallback. Its normal resource model is RESX and
satellite assemblies. We retain familiar indexer/key fallback and typed scope,
but use an explicit configured provider and JSON as requested. No ambient culture
or DI/factory abstraction is required. The cost is explicit configuration and a
bounded culture catalogue. Result/Option preserve recoverable loading and missing
lookup outcomes without a per-lookup LocalizedString object.

Primary sources consulted 2026-10-21:
- https://learn.microsoft.com/en-us/dotnet/core/extensions/localization
- https://docs.weblate.org/en/latest/formats/json.html
- https://www.i18next.com/misc/json-format

There is no universal JSON localization standard. We select the flat string-map
format supported by Weblate and the simple string entries common to i18next.
Full i18next plural/interpolation semantics and ICU MessageFormat are deferred,
not silently approximated. Nested object keys are not flattened.

## Implementation and boundaries

See [the API guide](../../../api-docs/localization.md) for the complete contract.
The owned ordinal map is built directly, without a JsonValue DOM. Shared JSON
UTF-8, whitespace and string escape validation is reused; the internal string
reader accepts a caller-supplied byte limit while general JSON retains its existing
1024-byte default. The resource limit is 65536 bytes/4096 entries per file.

Localizer and typed Localizer<T> live in System.Runtime. JSON provider and error
union live in System.Data under the same System.Localization namespace to avoid
a Runtime-to-Data dependency. No public implementation map or mutable DOM escapes.

FileInputStream's native/JIT binding is not implemented yet. File loading instead
uses the existing, correctly synchronous FileText.ReadAllText contract. Stream
loading remains async and borrowed. Enable the existing ReadAllText binding and
its file-input helper in macOS native/JIT launch profiles; no new InternalCall or
compiler encoding is introduced. Windows native file input is still a gap.

A temporary compiler trace identified a parse failure from using the reserved
`scoped` keyword as a local identifier. The implementation uses `scopePrefix`;
the diagnostic instrumentation was removed. Robust parser recovery for that
invalid source remains a separate general Raven candidate. Type scope accesses
NominalTypeInfo.Name explicitly; structural types return InvalidScope.

## Validation

Built-in tests cover flat JSON, Unicode/escapes and ordinal keys, empty/missing
values, duplicates, invalid structure/UTF-8, larger-than-DOM catalogues, byte
quotas, file and borrowed-stream input, quiet Find and reporting indexers, scoped
and unscoped per-key fallback, optional invariant and typed resource loading.
The typed instance Load<T> exposed a stale metadata-reader rejection of generic
instance methods. Runtime/SDK emission already supported these methods; remove
that admission gate while retaining ordinary signature validation. The existing
SDK generic-instance harness now also checks direct native materialization,
instance/generic identity and byte-preserving snapshots. Its verify/run test passes
with result 42; ordinary .NET emission is unchanged. Native Raven uses the rebuilt
metadata SDK, identified in the library build evidence. No new metadata encoding
or Runtime Contract setting is introduced.

Final execution and API snapshot results are recorded alongside this document.

## Lookup-first formatting

LocalizerFormatting supplies one/two/three-argument indexers and Find overloads,
plus an argument-array form with params expansion. Resolve first; only a found
template is formatted. Missing indexer keys remain unchanged; Find returns None.
No-argument lookup returns raw text. Invalid formats retain String.Format faults.
Culture-based loading wraps the catalogue with the requested culture; fallback
changes the selected resource, not numeric formatting. In-memory and explicit-file
catalogues use invariant culture. Formatting may allocate strings and box values;
fixed overloads avoid a temporary argument array, without promising zero allocation.

Raven main 56343767e closes expanded params indexer binding, with 11 passing
extension-indexer tests. Native carry 550cbc87c retains that behavior alongside the
branch's setter/accessibility changes. Native callable admission additionally uses
the existing AllowsGenericInstanceMethods capability, and SDK generic imported
calls retain their consumer-scoped receiver type. No new metadata/ABI encoding.

## Integration fixes

Native nominal Name currently contains its metadata namespace/nesting path; the
provider extracts the last component explicitly for its documented simple-name
resource convention. Name/FullName consistency remains a separate introspection
review item; resource lookup does not redefine that API.

The native specializer now discovers boxed primitive interface implementations
before final selection, using the same implicit-interface resolver as ordinary
value types. Previously selection could discover a numeric Formattable body too
late, leaving its generic ReadOnlySlice calls unspecialized. A reduced backend
regression and the Swedish fallback numeric-format test cover this path.
Selection errors now include the containing function to make unresolved calls
traceable. This changes discovery order, not primitive equality/storage or ABI.

The native compiler source is committed as 4afc75dd8 (after carry 550cbc87c).
The library evidence's `550cbc87c-local-generic-import` label identifies the same
source before that final commit. Main Raven's independent fix is 56343767e.

## Qualified results (2026-10-21, macOS arm64)

- Nine built-in localization tests pass in native, JIT and interpreter: 27 executions.
  See [runtime report](runtime-tests.json) and [output](runtime-tests.log).
- All 23 backend specialization tests pass, including boxed scalar interface discovery;
  [output](specialization-tests.log).
- Existing JSON DOM regression: 21 interpreter tests pass after the shared
  escaped-string reader change; [report](json-regression.json).
- Metadata generic instance materialization/import/receiver and native verify/run
  regression passes; [output](metadata-tests.log).
- Raven main: 11 extension-indexer emission tests pass; native consumer coverage
  includes four-argument params indexing and typed Load<T>.
- API snapshot validation passes; the website build checks 5,294 pages and the
  website unit tests pass 23/23 (including the corrected synchronous Console label).

The test bundle composes matching development libraries/compiler and current
runtime/codegen with the recorded [JIT host recipe](jit-host-recipe.json). This is
focused qualification, not a release bundle or Windows/Linux certification.

## Foundation exit scope — 2026-10-21

The author asks to close localization alongside DI before further stabilization.
The final integration test loads real `FeatureX` JSON catalogues, registers
`Localizer<FeatureX>` as a singleton and resolves a constructor-injected scoped
consumer twice. It checks scope identity, regional lookup and Swedish formatting
of an invariant fallback template. The dedicated website page links this with DI.

The author subsequently clarifies that provider registration against `Localizer<>`
must resolve `Localizer<HomePage>` automatically. The instance-registration test is
useful coverage but does not close that requirement. Open-generic DI integration
remains required; plural rules, reload/watchers,
automatic caching, larger catalogues and additional locales remain future work.
The existing macOS native file-host limitation remains explicit. Neither this
integration nor the container implies thread-safe shared mutable services.

The final 10 localization tests pass on interpreter/native/JIT (30 executions).
The same [report](di-integration-tests.json) and [output](di-integration-tests.log)
contain two method-discovery tests per engine (six executions). API snapshot checks,
the 5,580-page website build and all 23 website tests pass. This is the closed-instance
integration checkpoint, not completion of open-generic provider registration.
