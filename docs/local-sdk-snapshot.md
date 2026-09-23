# Local SDK snapshot — 24 September 2026

The local development workspace is
`~/.neoclr/experiments/records-storage-20260924/Platform.code-workspace`.
It pins its own Raven compiler and language server and includes a matching neoCLR
runtime, library, bridge and reference assembly. It is not a published release.
The bundle's `snapshot.json` records source revisions and artifact hashes.

Run `~/.neoclr/experiments/records-storage-20260924/open-vscode.sh` to open it
with the existing isolated experimental Raven extension profile. Select a project
and run its **neoCLR: Run** task (the default build task). Console accepts input in
the task terminal. Each Storage run creates a separate retained `storage/runs/`
folder, so its exclusive-create example can be repeated without deleting files.

Projects: integer record equality/hashing/deconstruction, Storage file read/write
and enumeration, and Console input/output/error streams. Records are currently
limited to non-generic classes with Int32 components and a direct Object base.
HashCode supports Add(int), Add(string), ToHashCode and Combine(int, int).

Validation: all three saved projects build and run; language-server protocol checks
return HashCode.Combine and System.Concurrency completions. The compiler record suite
passes 40 tests; the runtime hash suite passes three. This does not claim a manual
VS Code UI test or broad record parity with .NET. The existing SDK base version is
retained; snapshot provenance distinguishes these rebuilt local tools.
