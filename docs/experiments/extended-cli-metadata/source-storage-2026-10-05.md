# Native source storage gate — 2026-10-05

The remaining eleven production System.Storage files compile unchanged against the
cumulative 115-input Numbers library (which already contains stream implementations).
Three internal native adapter files supply seven services. Consumers contain no library
sources, stubs or CLI-projected application/library dependencies.

The unchanged `docs/experiments/storage-poc/Main.rvn` verifies and runs with status 0:

```text
Created message.txt
Read: Hello, värld!
Re-read after seek: Hello, värld!
Existing file: AlreadyExists
Items:
directory: examples
file: message.txt
Directory: /examples
Item: message.txt
Missing file: NotFound
```

`bootstrap/storage-text-consumer.rvn` additionally verifies and runs with status 0,
prints `Native bounded file text passed`, and tests Result returns across module-function
references, UTF-8 contents, too-large reads/writes and missing-file errors. The harness
checks exact file bytes, including unchanged contents after a rejected write.

## Owning changes and comparison with .NET

Raven's portable adapter now propagates empty-stack context through lowered value
blocks, conditional branches, conversions and transparent expression wrappers. This
admits early returns inside Result match arms; pending outer operands retain explicit
rejection. Ordinary .NET's general emitter already handles these constructs. The main
branch has no portable adapter to backport. Integration compiler commits are dcc9c1c2e
and separate stale-test correction 8b8856b1d; 56 shared-body .NET tests pass.

The metadata library permits output-owned top-level value-type contracts in authored
module-function signatures, just as member signatures already do. Result carriers
retain existing scoped nominal identities and native encoding. Nested types, byrefs,
owner parameters and out parameters in this particular API remain unsupported.
No new metadata schema or implicit dependency loading is introduced.

The runtime's new marked StorageNames internal call converts its erased native string
vector to independently owned managed Array<String> storage. Array and heap limits
are checked before allocation. File/path/list/text services use the existing runtime
implementations. Unlike the legacy ordinary neoil vector helper, this ABI matches the
source compiler's managed-array contract. It does not change array semantics or claim
performance improvements. The .NET comparison is ordinary managed arrays and bounded
file services; no .NET storage-source execution is claimed by this native gate.

## Reproduction and evidence

Run `verify_source_storage.py --help`, then supply the compiler, cumulative library,
explicit ownership manifest, primitive core, retained seed and runtime executable.
The output directory must be new. It owns the fixture and records compile, verification
and execution commands, exact stdout/status, source/artifact hashes and repository
revisions in `validation.json`.

The checked-in [evidence](source-storage-2026-10-05.json) uses the cumulative type-budget
gate's dependencies. Its runtime revision is the pre-slice base; runtime and adapter
hashes identify the tested working slice. The primitive bootstrap and retained seed
remain explicit. Run with a 100-million-instruction limit; this is not a benchmark.

Focused validation: all 154 C# metadata groups pass, including authored-function
contracts and round trips; runtime snapshot
identity/invalid-payload/limit tests, exact binding tests, nine file-stream tests and
both artifact-only consumers. Existing public guest signatures are unchanged. The
host API manual is updated; the previously stale generated guest API snapshot remains
separate documentation debt. No website build is needed for this slice.

Next: DNS/socket service families and the imported Error identity discrepancy, then
full-System source/bootstrap ownership. Source-level decompilation is not a prerequisite;
a read-only assembly dump would be useful debugging infrastructure.
