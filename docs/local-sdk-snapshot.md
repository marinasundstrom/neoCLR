# Local SDK snapshot — 24 September 2026

The local development workspace is
`~/.neoclr/experiments/nullable-records-20260924/Platform.code-workspace`.
It pins its own Raven compiler and language server and includes a matching neoCLR
runtime, library, bridge and reference assembly. It is not a published release.
The bundle's `snapshot.json` records source revisions and artifact hashes.

Run `~/.neoclr/experiments/nullable-records-20260924/open-vscode.sh` to open it
with the existing isolated experimental Raven extension profile. Select a project
and run its **neoCLR: Run** task (the default build task). Console accepts input in
the task terminal. Each Storage run creates a separate retained `storage/runs/`
folder, so its exclusive-create example can be repeated without deleting files.

Projects: integer/string/nested record equality, hashing and deconstruction, Storage file read/write
and enumeration, and Console input/output/error streams. Records are currently
limited to non-generic classes with integer, non-null string or same-compilation
record-class components and a direct Object base. Record references may be nullable;
nullable string/value components and record structs remain unsupported.
The Object equality project also checks boxed Int32 value equality and hashes while
separate boxes retain distinct identities. Other boxed values remain unsupported.
HashCode supports Add(int), Add(string), ToHashCode and Combine(int, int).

Validation: all four samples build and run; language-server protocol checks
return HashCode.Combine, System.Concurrency and nested-record member completions,
and hover retains Person? for nullable properties. The compiler record suite
passes 46 tests; boxed/class Object equality passes 12 runtime cases and the
.NET comparison passes 22 assertions; the runtime reference-slot suite passes 32. The prior hash suite
passed three tests; the HashCode implementation is unchanged in this slice. This does not claim a manual
VS Code UI test or broad record parity with .NET. The existing SDK base version is
retained; snapshot provenance distinguishes these rebuilt local tools.
