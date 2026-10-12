# Release preflight — 2026-10-13

Hosted source qualification now passes on `a9adc97cb`, including the full Linux
archive suite, macOS/Windows host boundaries and the minimum-Rust matrix. Backend
repair `566fb34dc` has focused Windows object-generation and macOS execution
coverage; its Windows HTTP/package runs remain pending. This is **not a published
release**. Earlier failures remain evidence rather than being rewritten.

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

## Hosted source and callback repair

[Hosted run 38317598328](https://github.com/marinasundstrom/neoCLR/actions/runs/38317598328)
passes the canonical format/strict-Clippy/source gate on `a9adc97cb`:
[full source/archive report](hosted-source.json), [macOS host report](hosted-macos.json)
and [Windows host report](hosted-windows.json). Rust 1.85 checks pass on all three
hosts. This closes the previous source gate; the original failed reports above
remain unchanged.

Windows JIT run `38317601422` exposed a separate SSL compilation failure in the
combined host callback dispatcher. [Repair details](../jit-windows/README.md#mixed-callback-dispatcher-repair)
explain why removing its redundant union cap does not alter per-shape admission or
the callback ABI. `566fb34dc` passes two focused tests: Windows x64 COFF generation
and sanitized macOS host execution across four unit/no-result/mixed variants.
This backend-only delta follows the source-qualified revision; it does not claim
a second full source run. Windows execution must still pass its own consumer gate.

The refreshed macOS bundle passes [extracted default-JIT acceptance](callback-extracted-acceptance.json),
[SSL/HTTPS JSON in all three modes](callback-ssl.json) and
[HTTPS failure/recovery, timeout, cancellation and deadline checks](callback-https-failures.json).
[Archive hashes](callback-bundle-archive.json) and [component inputs](callback-bundle-inputs.json)
identify the bundle. It refreshes ten catalogued runtime/backend/notice files in the
existing author installation, after verifying that no installed files had drifted.
The SDK, libraries, samples and extension remain unchanged. [Installed hashes](callback-installed-refresh.json)
and [fresh installed acceptance](callback-installed-acceptance.json) pass; earlier
SDK/editor evidence still describes the unchanged compiler and extension.

The first [Windows SDK/package run 38317604005](https://github.com/marinasundstrom/neoCLR/actions/runs/38317604005)
passes on `a9adc97cb`: [default-JIT packaged samples](windows-sdk-acceptance-initial.json),
[26 VS Code checks](windows-sdk-vscode-initial.json) and [archive hashes](windows-sdk-package-initial.json).
This qualifies Raven `6b5e052ce` and SDK/VSIX `0.1.13-neoclr.20261013.1` on Windows,
but its backend precedes the callback repair. Run `38319479140` rebuilds and retests
the package with that repair; run `38319332994` qualifies the broader Windows
SSL/HTTP execution matrix.

## Remaining release gates

1. Retain the passing hosted source gate and focused backend-delta evidence above.
   Do not present the earlier source revision as testing the later callback repair.
2. Retain Windows Raven native revision `6b5e052ce3648f9247ac76aba275587a3a5a1bb4`
   qualification above. Its validation branch is `codex/neoclr-release-20261013`;
   Raven main and the native compiler line remain separate.
3. Qualify the rebuilt Windows default-JIT bundle, HTTP/HTTPS and matching SDK/editor;
   retain archive, payload and compiler hashes. Refresh packaged notices on both
   distribution targets and verify extracted payloads before publication.
4. Review the selected package and [draft release notes](../../next-preview-release-notes.md). Local installed VS Code acceptance
   now passes all [26 checks](installed-vscode-acceptance.json), including activation,
   navigation, native/.NET editing and Tasks/await runs; [hashes](installed-vscode-provenance.json)
   identify the real installed SDK/extension. Windows editor acceptance passes on the initial
   package and will be repeated with the repaired backend.

The [installed macOS instructions](../../local-preview-20261012.md) retain the same
SDK and launch paths; the runtime/backend refresh has its own evidence above. Tagging, uploading releases and website publication have
not occurred. No Windows ARM64 or Linux JIT/AOT qualification is implied.
