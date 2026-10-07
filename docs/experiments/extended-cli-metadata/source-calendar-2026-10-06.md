# Source-built calendar gate — 2026-10-06

Unchanged TimeZone, TimeZoneError, ZonedDateTime, LocalTimeMapping and DateTime sources
now compile into Calendar.dll using five internal runtime-service adapters. A separate
consumer imports Calendar.dll and Numbers.dll with their sources absent, verifies and
executes with exact stdout `Native source calendar passed` and exit 0.

The test checks Stockholm's skipped spring time and two ordered autumn candidates,
their distinct offsets, UTC's unique mapping, invalid-zone/range errors, and conversions
and type patterns for the optional DateTime union. Prefer specific temporal types in
public APIs; this union exists for contracts intentionally accepting either form.
System-zone discovery remains host-dependent; this consumer does not qualify it.

## Import and runtime changes

Raven `befb8c1e1` accepts constructor unions through the existing UnionAttribute,
public single-value constructors and Object Value getter. It retains their alternatives
and generic scopes without inventing named cases. The existing named-case path remains
in use for LocalTimeMapping and Result. No reader object is reused during emission,
and the .NET backend/importer is unchanged. Provider-interface-only unions are not
added by this bounded importer change.

TimeZoneMapLocal admits managed Int64 array returns, with exact signature validation
and array/heap limits. Each runtime buffer becomes a managed array reference at the VM
boundary, sharing the Environment snapshot allocation path. Legacy transport still
loads; it does not establish inline value arrays as a supported platform feature.
The calendar API continues to use the existing IANA rules and civil-time policy.
See [contracts and .NET comparisons](../../time-zones.md); no new rule-resolution
policy, metadata format, performance claim or public service capability is introduced.

## Validation and remaining bootstrap work

- Artifact-only calendar execution, including DateTime conversions/type patterns.
- C# native constructor-union probe: plain alternatives and open/constructed generics.
- Eleven focused ordinary .NET union tests pass.
- Five runtime tests pass: four Environment controls and the managed mapping boundary.
- Two bundled time-zone rule tests pass, including gaps/overlaps, historical seconds,
  exact ticks and range limits.
- API snapshot and website rendering pass. Existing API-summary coverage warnings
  remain; this slice does not claim to fill those unrelated documentation gaps.

The full-owned-handle audit falls from 48 to **35 diagnostics** over 186 source inputs.
No full System artifact is published. Five missing time-zone services and their
associated binding cascades are removed. Next prioritize Console/native-width service
inputs; then missing RuntimeFailure/NativeAllocation imports, let-else propagation and
Http task-return errors. These are observed diagnostics, not proof of independent root
causes. Existing ownership/bootstrap restrictions remain in force.

## Reproduce

```sh
python3 docs/experiments/extended-cli-metadata/verify_source_calendar.py \
  --compiler /tmp/calendar-compiler1006/rvnc.dll \
  --compiler-revision 2ea6e36be+constructor-union \
  --core /tmp/preview12-bootstrap/Core.dll \
  --seed /tmp/preview12-bootstrap/System.neox \
  --library /tmp/native-poc-libraries1005/Numbers.dll \
  --ownership runtime/raven/native/poc-ownership.json \
  --runtime target/debug/neoclr --output /tmp/calendar-fresh
```

[Execution evidence](source-calendar-2026-10-06.json) records commands and artifact,
source, compiler-component and runtime hashes. The compiler snapshot was built before
commit befb8c1e1; its label records the base plus pending union change. Runtime revision
881c4d01 likewise precedes the managed mapping change, whose source hashes are recorded.
The [audit](native-bootstrap-calendar-2026-10-06.json) uses the same compiler snapshot.
These are development-branch results, not a new release or main-branch qualification.
