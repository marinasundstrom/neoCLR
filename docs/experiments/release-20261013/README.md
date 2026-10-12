# Release preflight — 2026-10-13

Local macOS ARM64 preflight closes notice-inventory and stale-test blockers.
This is **not a published release or fresh Windows qualification**, and the initial
full-suite report remains failed rather than being rewritten after repairs.

## Results and exact revisions

- `874c36f85`: [source audit](source-before-notices.json) stopped before compilation
  because 20 TLS-related locked packages were absent from the notice manifest.
- `501532e2a`: [full extracted-source run](full-suite-before-fixture-fixes.json)
  completed all 250 test targets: 1,956 passed, three failed, three ignored. The
  failures were the old rejection of generic lexical owners and two reflection
  fixtures that treated physical CLI scopes as logical modules.
- `90bceafcd`: [fresh source archive smoke check](source-smoke.json) passes with
  99 registry packages, 198 verified notice files, the pinned native seed, all 25
  source/artifact smoke programs, native FFI and embedding examples. It explicitly
  records `full_tests: false`; it is not a replacement full-suite report.
- [Focused checks](focused-checks.json): all six nested-type tests and all 26
  reflection tests pass after correcting their fixtures, with seven tooling tests.
  Strict all-target Clippy and format checks pass; newer-toolchain deprecations are
  narrowly allowed to preserve the Rust 1.85 APIs. No runtime semantics changed.

The nested-type fix is `90bceafcd`; the reflection fixture fix is `091fa7647`.
The latter changes only tests and changelog after the smoke archive. Tests now
preserve malformed ownership rejection and assembly-scoped module identity while
accepting the already implemented generic-owner and logical-namespace contracts.
The broad test evidence is reused with these focused repairs; do not describe it as
one successful clean full-suite run on the final revision.

The source notice repair is `501532e2a`. Development JIT preparation now refreshes
notices and records their input hashes (`e2e03589f`) instead of inheriting stale
bootstrap notices. It also retains backend/TLS inventories and lockfiles and the
Cranelift license. This preparation behavior has a focused packaging regression;
the previously installed author-testing bundle has not been replaced in this slice.

## Remaining release gates

1. Obtain a clean hosted source/format/Clippy gate on the selected candidate, including
   focused macOS/Windows boundaries and the minimum-Rust matrix.
2. Qualify Raven native revision `6b5e052ce3648f9247ac76aba275587a3a5a1bb4` on Windows.
   It is now available on `codex/neoclr-release-20261013`, and HTTP/JIT and SDK
   qualification select it. A passing older run does not qualify this compiler.
3. Qualify the rebuilt Windows default-JIT bundle, HTTP/HTTPS and matching SDK/editor;
   retain archive, payload and compiler hashes. Refresh packaged notices on both
   distribution targets and verify extracted payloads before publication.
4. Complete installed editor acceptance and review the selected package/release notes.
   Local SDK language-server probes are not proof of interactive extension activation.

Reuse the [installed macOS HTTP evidence](../../local-preview-20261012.md) for its
unchanged binary behavior. Tagging, uploading releases and website publication have
not occurred. No Windows ARM64 or Linux JIT/AOT qualification is implied.
