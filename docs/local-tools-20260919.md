# Local native UTF-8 development snapshot — 2026-09-19

A fresh workspace is prepared at
`/Users/robert/.neoclr/experiments/native-utf8-20260919/demo`.
It contains the strict UTF-8 sample in Main.rvn, current System.Runtime reference
metadata and a matching runtime/library/bridge/compiler/server. Earlier workspaces
are preserved. This is a machine-local experiment, not a published release.

Open that folder in the prepared experimental VS Code profile, then run
**Tasks: Run Build Task → neoCLR: Run saved project**. Save Main.rvn before running.
Use this task rather than Raven's ordinary host-.NET run/debug commands.

```sh
"/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code" \
  --user-data-dir /Users/robert/.neoclr/vscode/raven-port-20260919 \
  --extensions-dir /Users/robert/.neoclr/vscode/raven-port-20260919/extensions \
  --new-window /Users/robert/.neoclr/experiments/native-utf8-20260919/demo
```

The sample prints six successful round trips, a byte count of 7, No preamble, AB,
and six InvalidUtf8 results. Its source and expected output are in
[the UTF-8 sample](experiments/raven-target/samples/library-utf8.rvn) and
[expected output](experiments/raven-target/samples/library-utf8.expected.txt).
The fresh workspace builds/runs that sample and passes target completion checks.
Its parent manifest.json records source revisions and binary hashes; it identifies
neoCLR 10b68e3 and Raven 2479755e7. Later website-only commits do not alter its runtime.

String storage and comparison are native UTF-8. Char still has the legacy 16-bit
representation pending the selected scalar migration. This snapshot does not claim
that migration is complete. For release gaps, see [the readiness review](preview-readiness-2026-09-19.md).

## Scalar Char follow-up

A separate workspace is prepared at
`/Users/robert/.neoclr/experiments/scalar-char-20260919/demo`, with neoCLR `0c36087`
and Raven `3bd17488a`. Its manifest pins the binaries; the earlier UTF-8 snapshot
above is preserved. Open this folder in the same experimental VS Code profile
and run **neoCLR: Run saved project** against Demo.rvnproj.

```sh
"/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code" \
  --user-data-dir /Users/robert/.neoclr/vscode/raven-port-20260919 \
  --extensions-dir /Users/robert/.neoclr/vscode/raven-port-20260919/extensions \
  --new-window /Users/robert/.neoclr/experiments/scalar-char-20260919/demo
```

Main.rvn now contains the scalar Char sample. Expected output is:

```text
127757
128512
66560
Earth
Smile
Supplementary letter
Supplementary symbol
```

The saved project builds, imports, verifies and runs on neoCLR; the matching
language server passes the target completion checks, including absence of the
removed surrogate predicates. The tools are local development artifacts, not a
new public SDK release. Scalar String access remains unimplemented.

## Grapheme text snapshot

The fresh workspace at
`/Users/robert/.neoclr/experiments/grapheme-text-20260919/demo` supersedes the
scalar-Char snapshot for current text work. Earlier workspaces remain intact.
Open it using the same experimental VS Code profile above, save Main.rvn and run
**Tasks: Run Build Task → neoCLR: Run saved project**.

[Main's source](experiments/raven-target/samples/library-grapheme-strings.rvn)
counts 4 graphemes, 12 scalars and 37 UTF-8 bytes in one string. It also exercises
literals, matching, interface iteration, arrays, fields, copying and NUL defaults.
The copied toolchain builds/runs it with the checked-in expected output, and 20
completion sections pass against the copied language server/reference metadata.

Char is no longer numeric. The workspace uses RavenGraphemeChar with matching
compiler, reference core, System.Runtime library, importer and runtime. Its parent
manifest records source revisions and binary hashes. Iteration currently allocates
snapshots; integer string indexing, normalization and specialized encoding string
types remain deferred. See [the text contract](design/text-abstraction.md).

.NET is required by the tooling and language server, not neoCLR or its guest
programs. This is a machine-local development snapshot, not a published SDK/VSIX.
