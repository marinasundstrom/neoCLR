# Local reflection preview — 2026-10-17

This local macOS ARM64 development installation packages Raven `codex/neoclr`
with neoCLR main's field and type-initializer reflection support. It is not a
published release. Broader structural work remains parked.

- SDK: `~/.raven/sdk/0.1.15-neoclr.20261017.1`
- Preview: `~/.neoclr/experiments/preview-20261017-reflection`
- Extension: `0.1.15-neoclr.20261017.1`, installed in the preview's
  `editor-extensions`, with a separate `editor-state` profile.
- Open: `~/.neoclr/experiments/preview-20261017-reflection/open-vscode.command`

The workspace pins the installed SDK. Global SDK launchers and the default editor
extension installation are unchanged. Host prerequisites are .NET 10/11,
Python 3 and Xcode command-line tools.

Open `samples/reflection/Main.rvn` and use **Cmd+Shift+B**. The default task compiles
an isolated copy of the edited sample, resolves schema-7 reflection roots from
fresh metadata, then checks interpreter, JIT and sanitized AOT execution. Success
prints `PASS: reflection checks returned 42 in interpreter, JIT and AOT.`
The individual non-42 returns in the source identify failed assertions.

The sample covers readonly static and instance fields, Int32 literal values,
metadata tokens, static initializer discovery without execution, initialization
on reflected access, mutable static field writes, and rejected private/readonly
access. It uses the [committed reflection consumer](validation/field-reflection/Main.rvn).
The installed preview includes its own compiler, runtime binaries, native libraries
and C host support; it does not require the source checkouts to execute.

This is a focused reflection validation workspace rather than the complete
release sample bundle. Native reflection limitations remain as documented in
[the runtime API](reflection-members.md).

[Installation evidence](validation/local-preview-20261017/installation.json)
records source revisions, installed binary hashes and validation results.
