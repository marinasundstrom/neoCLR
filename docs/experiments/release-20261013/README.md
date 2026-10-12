# Release preflight — 2026-10-13

Hosted source qualification now passes on `a9adc97cb`, including the full Linux
archive suite, macOS/Windows host boundaries and the minimum-Rust matrix. Backend
repair `566fb34dc` has focused Windows object-generation and macOS execution
coverage and passing Windows SDK/package qualification. The later Windows AOT
heap repair also passes the broader Windows HTTP execution gate on `a77e20dba`. This is **not a published
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
a second full source run. Windows SSL subsequently passes interpreter/default JIT,
but standalone AOT exposes the host's old 1 MiB allocation cap. [The report](windows-ssl-before-heap-fix.json)
preserves that failure; `a77e20dba` aligns Windows heap admission with the common
build contract and adds 8/32 MiB guarded-allocation coverage. Its hosted run passes the guarded allocator and complete Windows consumer matrix.

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
but its backend precedes the callback repair.

[Windows package run 38319479140](https://github.com/marinasundstrom/neoCLR/actions/runs/38319479140)
now passes on callback-fixed `566fb34dc`: [packaged default-JIT samples](windows-sdk-acceptance.json),
[26 installed editor checks](windows-sdk-vscode.json) and [package construction](windows-sdk-package.json).
Downloaded archive and manifest hashes match; [component hashes](windows-sdk-components.json)
identify the runtime, backend, helper, compiler, language server, extension and base
libraries. This Windows bundle does not include the optional PostgreSQL provider.
The Windows JIT host uses its existing allocated heap, not the AOT host's guarded
allocation entry; the later heap repair is qualified separately as source AOT work.
[Run 38321019998](https://github.com/marinasundstrom/neoCLR/actions/runs/38321019998)
passes that repair and the broader SSL/HTTP matrix. It confirms the 18 guarded
heap lifecycles, managed SSL/HTTPS JSON, rejection/recovery and cancellation/deadline
cases, independent TLS interoperability, public cryptography, default-JIT HTTP,
framework suites, and both ordinary/generic async cleanup consumers. Interpreter,
JIT and AOT controls pass where selected by the gate. See the retained final reports
below for exact cases and hashes.

[Final summary](windows-final-summary.json) verifies all 27 stages, twelve
downloaded HTTP DLL/object/executable hashes and the three changed AOT host source
hashes (accounting for Windows checkout line endings). Retained reports cover:

- [Complete gate](windows-final-gate.json), including both async cleanup consumers.
- [SSL/HTTPS JSON](windows-final-ssl.json) and [failure/recovery cases](windows-final-https-failures.json)
  in interpreter, default JIT and AOT.
- [12 independent TLS 1.2/1.3 client/server interoperability cases](windows-final-tls-interop.json).
- [8 default-JIT HTTP cases](windows-final-http.json).
- [29 framework execution/filter cases](windows-final-framework.json) across
  collections, JSON DOM, encoding/streams, Index/Range, zoned-time parsing and runner contracts.
- [Public cryptography](windows-final-cryptography.json) in interpreter, JIT and AOT.

## Publication review

1. Retain the passing hosted source gate and focused backend-delta evidence above.
   Do not present the earlier source revision as testing the later callback repair.
2. Retain Windows Raven native revision `6b5e052ce3648f9247ac76aba275587a3a5a1bb4`
   qualification above. Its validation branch is `codex/neoclr-release-20261013`;
   Raven main and the native compiler line remain separate.
3. Windows guarded-allocation and HTTP/HTTPS execution pass on `a77e20dba`.
   The matching SDK/editor and default-JIT package gate pass on `566fb34dc`;
   the later source delta only changes Windows AOT heap admission/diagnostics and
   validation/docs. Refreshed notices and extracted payload hashes are verified
   on both package targets. These are separate exact-revision results, not a claim
   that every workflow ran on one commit.
4. Review the selected package and [draft release notes](../../next-preview-release-notes.md). Local installed VS Code acceptance
   now passes all [26 checks](installed-vscode-acceptance.json), including activation,
   navigation, native/.NET editing and Tasks/await runs; [hashes](installed-vscode-provenance.json)
   identify the real installed SDK/extension. Windows editor acceptance also passes on the
   callback-fixed package.

The [installed macOS instructions](../../local-preview-20261012.md) retain the same
SDK and launch paths; the runtime/backend refresh has its own evidence above. Tagging, uploading releases and website publication have
not occurred. No Windows ARM64 or Linux JIT/AOT qualification is implied.
