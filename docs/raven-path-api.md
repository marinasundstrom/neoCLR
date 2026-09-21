# Raven lexical path APIs

The experimental target exposes both existing `System.Storage.Path` methods:

| Method | Return | Behavior |
| --- | --- | --- |
| `Combine(string left, string right)` | `string` | Join lexical paths using the host separator; a rooted right operand replaces the left |
| `GetFileName(string path)` | `string` | Final lexical component; empty for empty input or a trailing separator |

These are calls into the runtime library, not the declaration assembly's placeholder
bodies. A shared member catalog produces the metadata declarations and admits the
matching static signatures. Raven uses ordinary static invocation and string values;
no explicit managed-reference syntax or compiler modification is needed.

The [existing path contract and .NET comparison](path.md) apply unchanged: this is
lexical manipulation, not file access or canonicalization. Inputs are nonnullable,
`.` and `..` are preserved, and invalid filesystem characters are left for file APIs
to handle. Windows-specific behavior still needs Windows execution evidence; this
projection was tested on macOS. There is no claim of the full .NET Path surface.

The [sample](experiments/raven-target/samples/library-paths.rvn) exercises ordinary
joins, empty operands, trailing separators, Unicode and rooted paths. With a fresh
collections probe and editor project prepared using the
[integration instructions](experiments/raven-target/README.md), copy the sample into
that project's `Main.rvn` and run its **neoCLR: Run saved project** task. The saved-project checker
runs the sample and rejects incompatible argument types and unprojected methods.

```sh
python3 docs/experiments/raven-target/verify_project.py /tmp/PROBE/editor/Demo.rvnproj \
  --collections --raven /path/to/Raven --runtime /path/to/neoclr
python3 docs/experiments/raven-target/verify_editor.py /tmp/PROBE/editor \
  --collections --files --strings
```

The editor check includes Path type/member completion and confirms that unsupported
host APIs such as `GetFullPath` do not appear. Focused signature checks also reject
malformed parameter and receiver signatures. This source bridge change does not
refresh the installed SDK or VSIX; use a newly generated declaration assembly.
