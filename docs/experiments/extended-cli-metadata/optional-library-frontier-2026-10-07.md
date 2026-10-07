# Separate optional-library frontier — 2026-10-07

[Compilation evidence](optional-library-frontier-2026-10-07.json) tests unchanged
Data and Networking sources against the independently built System.Runtime. No
Runtime source participates in these invocations. Both builds reject without output;
this is not executable optional-library acceptance.

The first Data attempt failed while importing flags enums because the native symbol
adapter looked for FlagsAttribute in the selected Object assembly. Raven `eab5b3e7d`
fixes that lookup to use the explicit primitive bootstrap identity. The C#
`FlagsSymbolChecks` probe passed before modification for the ordinary root, reproduced
the imported-root failure, and now passes both configurations. The focused compiler
build passes. No shared .NET loader/emitter behavior changes, so this native-adapter
fix remains on the integration branch rather than a general main backport.

The subsequent ordinary driver results are:

- **Data:** RAV0117 for ReflectionArrayLength, ReflectionArrayGet and
  ReflectionArrayCreate, plus downstream conversion diagnostics. ObjectMapper calls
  an internal IntrospectionRuntimeServices extension that was available inside the
  aggregate. Existing public reflection extensions cover construction and properties,
  but not these array operations. Add a supported bounded reflection API with error
  contracts and runtime validation; do not publish the entire internal service facade.
- **Networking:** the metadata builder throws when attaching an override. Its bounded
  Object override validator admits a local native root or bootstrap Object, but the
  imported source Object belongs to System.Runtime. Extend explicit imported-root
  authoring/identity validation and preserve exact signatures, including Equals(Object).
  The current driver exception is a known diagnostic gap; no output is published.

Imported-root authoring is the next shared unlock for optional assemblies that
implement Object operations. It must carry an output-owned reference derived from
Raven symbols/host identities, without accessing importer objects, with definition/
builder parity and native round-trip/runtime evidence. Do not allow any arbitrary
same-named System.Object to satisfy the contract. Then close the array-reflection
boundary and resume the existing JSON object and network cancellation consumers.
Web and project/editor catalogs remain later gates.

## Reproduce

```sh
python3 scripts/audit-optional-libraries.py \
  --compiler /tmp/flags-compiler1007/rvnc.dll \
  --compiler-revision eab5b3e7d --core /tmp/failure1006b/Core.dll \
  --runtime-library-directory /tmp/runtime-split1007/runtime-owned \
  --output /tmp/optional-split-new
```

The tooling records commands, diagnostics and source/compiler/dependency hashes. It
expects the explicit Runtime candidate and finalized retained seed from the Runtime
split audit. It records rejected builds as frontier results; a failed build that
publishes an artifact invalidates the audit. Successful emission alone would still
require a separate execution gate. Existing aggregate/orders evidence is unaffected.
No guest public API changes or website capability claims are introduced here.
