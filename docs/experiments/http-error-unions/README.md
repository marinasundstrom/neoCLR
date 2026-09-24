# Standard-syntax HTTP error union investigation

This reduced probe authors cases, a computed property and an override directly in a
Raven `union`. It is not the public HttpError API. Standard syntax is the
[class-library default](../../raven-conventions.md); manual carriers require a
specific bootstrap/runtime reason and a condition for revisiting that exception.

## Reproduce

Build `../raven-target/Probe.csproj` with the matching Raven checkout, and use a
matching installed neoCLR bundle containing that bridge, core metadata and Raven SDK.
The shape reporter uses Mono.Cecil from that local bridge build.

```sh
python3 docs/experiments/http-error-unions/verify.py \
  --bundle /path/to/matching/neoclr-bundle --expect-import-rejection
```

The script compiles in a temporary directory, prints CLI metadata and constructor /
TryGetValue instructions, and checks the exact known importer rejection. Omit the
explicit rejection flag when testing importer support; import failures then fail the
command. Neither mode claims successful runtime execution. Generated files are not
reference snapshots, and the observed field layout is not asserted as a permanent ABI.

## Observed on 2026-09-24

The source compiles with the matching experimental Raven SDK. The carrier is a
sequential value type with a byte tag and typed fields for all three cases. The
string payload requires managed-reference-aware layout. Nested case types hold
payload fields. The compiler emits constructors, TryGetValue methods, Value and
HasValue, alongside the authored property and ToString override. It also emits an
IUnion interface into this application because the target core lacks that interface.

The baseline bridge rejects `System.Networking.Sockets.SocketError&`. A temporary,
subsequently reverted admission of that byref exposed the next diagnostic:
`Only local initialization is admitted.` Generated carrier constructors use
`ldarg.0; initobj` to initialize their receiver. TryGetValue writes its output on the
matching path only. The probe has **not executed on neoCLR**.

Raven's language spec currently describes Value as the only instance storage; the
emitted artifact here has tagged typed fields instead. That discrepancy needs a
separate compiler-documentation review. Do not infer an approved .NET-compatible
physical ABI from the metadata member convention. See the existing comparisons in
[union conventions](../../union-convention.md) and
[unions and enums](../../unions-and-enums.md).

## Next bounded work

Admit generated initialization and case extraction deliberately, with reduced tests
for receiver initialization and nonmatching output behavior. Validate default and
inactive states, copies, boxing, Object members and GC tracing/cleanup for active and
inactive managed payloads before using this form in the runtime library. Review the
synthesized IUnion boundary separately. The benefit is compiler-owned case machinery
and ordinary source members; the cost is closing these importer/runtime gaps before
migrating existing hand-authored carriers. No new VM union opcode is implied.

HttpError and BaseUri remain pending. Existing manual carriers are migration
candidates, not automatically permanent exceptions or an instruction to rewrite
all working unions in one change.
