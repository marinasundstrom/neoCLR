# Union display and hover — 14 October 2026

Both bugs are fixed in Raven main and carried separately to the native compiler
branch. [Validation record](validation.json) identifies commits and checks.

- `ToDisplayString` renders an ad-hoc union as `A | B`, including declaration
  formats. Nominal union type uses stay named; nominal declarations can still
  request their own cases. Nested alternatives do not expand recursively.
- Hovering each alternative in a nested generic annotation resolves its own type,
  range and documentation. Hovering `|` shows the compact combined union.

Main: **41 display + 124 hover/resolution tests pass**. Native: **41 + 127 pass**.
Both retain three existing skipped hover tests. New regressions cover source and
imported unions, generic nesting, cold queries and queries after diagnostics.

## Real editor

[VS Code responses](vscode.json) use the Preview 15 JSON sample's
`Task<Result<unit, StreamError | JsonError | MappingError>>` annotation with the
rebuilt native-enabled server. All three alternatives and both separators pass
before and after an edit. Each alternative's range excludes its neighbors.
The [client excerpt](client-excerpt.log) and [server excerpt](server-excerpt.log)
identify the fresh isolated process and successful requests.

The first development-server attempt lacked the native host adapter and fell back
to inferred project loading. It is not counted as successful validation. The retry
built Raven.LanguageServer with `-p:NeoClrMetadataProject=` pointing to this repo's
`tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj`,
copied the server output to an isolated folder, and used a fresh editor profile.

To repeat, copy a matched bundle's JSON sample, set `raven.languageServerPath` to
the native-enabled server, and launch VS Code with an isolated `--user-data-dir`,
`--extensions-dir`, the matching extension as `--extensionDevelopmentPath`, and
`--extensionTestsPath` pointing to `scripts/native-vscode-union-hover-acceptance.cjs`.
The probe restores its edit and writes `vscode-union-hover-acceptance.json` in the
sample folder. Do not run against a user's edited sample.

Validation ran on macOS ARM64. Runtime behavior, Runtime Contract configuration,
bridge encoding and emission are unchanged. No runtime benchmark or website build
is required for this compiler/editor-only change. Preview 15 remains frozen; these
changes belong to the next toolchain release.
