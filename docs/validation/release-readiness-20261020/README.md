# Release readiness review — 2026-10-20

Author direction: continue toward release and identify urgent issues/compiler gaps.
Reviewed neoCLR `9f4fdc809` and the matched Raven compiler `100f98a2f`;
Raven `db4014f71` adds bridge documentation only. This is triage, not release approval
or a passed package gate. No candidate version, tag or publication date is selected.

## Current stabilization checkpoint

After the author-directed globalization and enum work, neoCLR `386e1c7b0` and
Raven `08c0bce8c` are the latest functional/documentation checkpoints. The compiler
binary is still the separately hashed `4c051e332-dirty-record-policy-metadata-function-bound`
cohort; Raven HEAD alone does not reproduce that compiler bundle. Freeze requires
reconciling that provenance and constructing a fresh matched package.

Completed evidence superseding earlier gaps below:

- Numeric primitives now declare explicit EquatableTo and Formattable contracts;
  floating NaN/signed-zero Object equality and hashes and all supported scalar
  adapters have focused qualification. [Numeric globalization](../../experiments/numeric-formats/README.md)
  passes 23 built-in tests per engine (69 executions). Custom patterns/providers,
  localized digits/sign policies and additional locales remain deferred.
- [Enum parsing](../../experiments/enum-parsing/README.md) passes four built-in tests
  per engine (12 executions), with matching API snapshots and a 5,036-page website
  build. Missing native generic-method struct/Enum constraints mean runtime type
  validation today; this is an explicit compiler/metadata gap. Existing Int32-only
  storage and ASCII member names remain bounded preview support.
- Ordinary struct defaults, scalar Object adapters, persistent Memory region
  equality and explicit value display overrides now have evidence in the
  [class-library review](../../design/class-library-soundness-20261020.md).
  This does not certify every record component or every library method.

Source-gate cleanup has resumed: formatting drift is corrected and the configured
root-package Clippy gate passes, alongside 16 focused numeric/inheritance regressions,
three host-validator tests and notice verification (139 packages, 256 files).
[Retained source-gate evidence](source-gates-20261020.json) records the limited scope.
The subsequent [backend cleanup](backend-gates-20261021.json) passes strict Clippy
for all backend targets and 114 library unit tests; the existing multi-million-call
performance test remains opt-in. Five argument-count exceptions remain documented
at explicit compiler/GC lowering boundaries. Native consumer validation also caught
and repaired a no-std build regression in the shared numeric formatter.

The next bounded stabilization tasks are the remaining record/display and
generic-constraint scope before package freeze. Continue to
extend native method constraints through producer, reader, importer and emission
without a Parse-specific exception; then qualify the remaining generated record
formatting shapes. Each needs focused positive and rejection tests. Rebuild and
pin the actual compiler/library/runtime cohort before extracting a candidate SDK.
No new public API expansion is required to run those checks. The original failures
below remain historical evidence; do not mistake them for fresh failures or use
focused development successes to close the cross-platform package gates.

## Original review: fix or resolve before freezing

1. **Native record-generated equality.** A fresh compile of
   [RecordGap.rvn](../../experiments/canonical-text/RecordGap.rvn) still rejects
   generated `==` with NEOMETA001: unregistered dependency on the current assembly.
   [Exact result](record-equality.json). Generated hashing/formatting are also
   unqualified; this run proves equality rejection only. Public record-valued APIs
   make normal value comparison a practical compiler requirement. Default .NET/Raven
   record semantics supply value equality and corresponding hashes; neoCLR should
   support those semantics, including canonical String/Char identity, rather than
   silently omit them. First fix emission/dependency registration and independently
   test equality, unequal values, hashing, boxing/interface paths and formatting.
   Keep .NET-general fixes on Raven main when applicable, then carry deliberately
   to the native line. Do not assume this native-only rejection needs a main change.
2. **Raw direct-call null String parity.** The interpreter can enter an instance
   body for a hand-authored direct call with null while native wrapper projection
   rejects at entry. This is documented in the [bridge review](../../raven-cli-bridge.md#private-string-helper-projection-2026-10-19).
   Ordinary Raven callvirt null behavior already has passing evidence. Reproduce
   the direct-call discrepancy on the frozen cohort and settle the intended runtime
   behavior; do not present it as a source-language null crash or silently redefine
   it as a permanent contract. This review did not rerun that raw-IL case.
3. **Replace stale candidate scope.** The October 19 candidate was superseded before
   publication and Windows validation was canceled. Text equality/search/formatting,
   sync text capabilities and ReadKey now exist; the original notes and old package
   identity cannot certify them. Refresh scope, pin exact runtime/backend/compiler/
   libraries/editor revisions and build a new package identity after fixes.

## Follow-up: bounded record value semantics

The [record-values experiment](../../experiments/record-values/README.md) now closes
the original canonical-text record rejection for supported positional records and
adds verified native boxed value equality/hash dispatch. This does not close item 1
in full. The subsequent [record-unions experiment](../../experiments/record-unions/README.md)
qualifies closed tagged-union components, including ConsoleKeyInfo's Option<int>,
across all three engines. Generic allocation overhead, unsupported/floating-point
components, automatic imported record discovery and formatting remain.
[Explicit imported value contracts](../../experiments/record-components/README.md)
now admit nongeneric Index/Range components using their public typed methods.
[Automatic equality interfaces](../../experiments/record-auto-interfaces/README.md)
now close ConsoleKeyInfo's missing self-equality interface and imported component
use. Nongeneric records with unsupported components reject at declaration; generic
equality/hash now has bounded support with allocation overhead, as recorded below. [Duration now has typed and boxed value equality with consistent hashing](../../experiments/duration-values/README.md). The earlier explicit-interface qualification is retained as historical
regression evidence, not the solution to automatic synthesis.
Native Int32/Int64/Boolean Object-dispatch adapters are now qualified; other primitive
adapters remain incomplete. The
original failure JSON above remains historical evidence. No new release identity or
platform/package gate is established by this fix.

## Follow-up: bounded record formatting

[Scalar and nested record struct display](../../experiments/record-formatting/README.md)
now has generated native bodies and targeted diagnostics for unsupported formatting.
[Closed declared union components](../../experiments/record-union-formatting/README.md)
are now included. This does not qualify enum/imported/floating-point components, generic records
or record classes. Empty positional syntax also exposed an earlier frontend rejection;
keep it separate from emitted formatting. Item 1 is still only partially closed.

## Follow-up: String call projection

The [focused backend repair](../string-direct-call-20261020/README.md) closes item 2's
recorded wrapper-entry discrepancy. Native projection preserves null checks at
original callvirt sites and retains interpreter behavior for direct calls. This
does not promise portable CLR semantics for null direct receivers or change Raven
source nullability. The baseline failure is retained with the new regression.
Fresh matched-package qualification below is still required.

## Required release qualification

- Canonical committed/extracted-source checks, including relevant format/Clippy,
  tests, notices and host gates. Focused development tests are supporting evidence.
- Fresh extracted macOS ARM64 and Windows x64 SDK/VSIX acceptance, matching runtime
  and source class libraries, native metadata, editor diagnostics/inlays/recovery,
  Tasks/await, JSON and HTTP consumers. Qualify Windows terminal acquisition as
  explicitly unsupported; do not advertise working Windows ReadKey.
- Current performance evidence for the selected package. Separate HTTP early
  response, delayed upload and complete-body workloads; retain collector settings
  and artifact identity. Experimental concurrent GC remains experimental. Do not
  infer HTTP throughput from avoided allocations or additional continuations.
- API snapshot and website checks on final scope. Publish the website separately.
  The latest ReadKey cohort already passes 15 public cases, six tool-sample cases,
  24 focused transport/binding tests and a 4,892-page site build. Reuse unaffected
  evidence where justified, but do not relabel development bundles as release assets.

## Can remain documented preview limitations

- Text-to-key handoff, native terminal worker attachment, Windows terminal input,
  async terminal readiness and general shutdown-request/veto events. Existing
  serialized POSIX entry-thread keys are the qualified scope.
- String shared-storage views (explicitly deferred), richer search/splitting,
  culture/format-provider and primitive-format-specifier integration. These are
  API coverage gaps, not evidence of a new runtime corruption bug.
- Broader generic native view element support outside qualified byte-buffer paths.
- Full-screen console capabilities and richer terminal protocols.

Do not expand these into new release requirements merely because they are possible
improvements. Prioritize ordinary value semantics and runtime consistency, then
freeze and validate the matched package. No new memory-safety or GC corruption bug
was established by this documentation review; that is not a whole-runtime safety
certification.

Generic-record follow-up: [bounded equality/hash support](../../experiments/record-generics/README.md)
now passes interpreter/JIT/AOT, including imported pairs, and Index/Range Object
hashing is corrected. The generic path adds seven managed allocations per tested
integer equality plus two hashes. Keep optimization, unsupported value adapters,
generic formatting and the exact matched-package gates open; this does not authorize
or claim a replacement release.


The [generic record allocation follow-up](../../experiments/record-generics/README.md#descriptor-free-policy-and-corrected-allocation-control)
now removes policy descriptor allocations. A corrected matched harness measures eight
versus four additional allocations per integer comparison and two hashes. Value boxing
and broader record gaps remain; this focused development evidence does not close the
matched-package release gates above.
