# Release readiness review — 2026-10-20

Author direction: continue toward release and identify urgent issues/compiler gaps.
Reviewed neoCLR `9f4fdc809` and the matched Raven compiler `100f98a2f`;
Raven `db4014f71` adds bridge documentation only. This is triage, not release approval
or a passed package gate. No candidate version, tag or publication date is selected.

## Fix or resolve before freezing

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
