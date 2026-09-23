# Byte Copy: managed storage before stream APIs

**Development experiment, 2026-09-23.** First in-memory checkpoint in the
[platform roadmap](../../platform-roadmap.md), partial S1 evidence. The program runs
on neoCLR through the matching Raven target. It does not add public System APIs,
new instructions, a Memory/Span representation or an asynchronous stream contract.

## What the sample explains

[Main.rvn](Main.rvn) transfers the four bytes of `Hi!\n` from a memory input into a
separate destination. A three-byte scratch buffer sits between them. Input returns
at most two bytes per read; output accepts one byte per write. This makes partial
transfer visible without a file, socket or scheduler. UTF-8 decoding happens only
after byte transfer, using the existing library conversion.

[ByteCopy.rvn](ByteCopy.rvn) separates the small pieces:

- `ValidRange` checks nonnegative offset/count, permits an empty range at the end,
  and uses `count <= length - offset` after validating offset. It avoids overflowing
  an unchecked `offset + count` calculation.
- `CopyBytes` validates both ranges before writing, then preserves overlapping bytes
  by selecting a copy direction. It does not allocate temporary payload storage.
  Successful copies preserve bytes outside the destination range. Recoverable range
  errors leave the entire destination unchanged.
- `MemoryInput` retains an ordinary managed array and a cursor. Invalid read ranges
  do not advance it. Zero from a nonempty read means EOF; an empty request also returns
  zero and must not be interpreted as EOF.
- `MemoryOutput` has fixed capacity and a cursor. Full output returns an error for
  a nonempty write. An empty valid write succeeds even when full. It never grows the
  destination or silently discards unwritten bytes.
- `CopyAll` handles both short reads and short writes. It rejects empty scratch space,
  and contains a no-progress guard for writing. An output failure can leave a copied
  prefix: this is not a transaction, and its Result does not yet report partial count.

These concrete endpoints are a fixture, not a chosen stream hierarchy. Their limits
of two and one byte are deliberate test behavior. Names and string-valued errors are
provisional; choose typed error families and general interfaces when another consumer
makes the requirements clearer.

## Storage and lifetime contract

Arrays are borrowed by ordinary managed reference. The endpoints retain them; they
neither clone nor free them. A caller can still mutate an alias, so no readonly or
exclusive-ownership guarantee is claimed. The copy is synchronous on one invocation.
It does not retain a raw pointer or a frame reference.

`CopyBytes` supports aliasing within one copy, including both overlap directions.
`CopyAll` requires distinct source, scratch and destination storage: per-call memmove
behavior cannot protect bytes that a later read has not consumed yet. This experiment
states that precondition rather than pretending to enforce it through nominal types.
General stream consumers, external writes, close/flush and pending-operation lifetime
remain open. The current source never exposes native memory.

## Comparison and placement

Primary references checked 2026-09-23:

- [.NET 10 Array.Copy](https://learn.microsoft.com/en-us/dotnet/api/system.array.copy?view=net-10.0)
  supplies the baseline of copying a selected range and handling overlapping source
  and destination. Our byte-only function keeps that behavior, while Result replaces
  argument exceptions. It does not reproduce general element conversions or all
  Array.Copy failure guarantees.
- [.NET 10 Span.CopyTo](https://learn.microsoft.com/en-us/dotnet/api/system.span-1.copyto?view=net-10.0)
  provides another overlap-preserving copy model. A checked view could package offsets
  more conveniently, but establishing a new view/lifetime contract is unnecessary for
  testing byte movement. Repeated range arguments are the cost of this smaller step.
- Reuse [stream research](../../stream-design.md) for partial I/O and the
  [generic managed-array contract](../../generic-managed-arrays.md) for reference
  identity, runtime bounds checks and metadata mapping.

Range validation and transfer loops fit ordinary library code over existing checked
array operations. The runtime remains responsible for element storage, type checking,
bounds and tracing references. No missing runtime primitive has been demonstrated by
this sample. This is not evidence that future native or suspended I/O needs no runtime
work, nor a claim that interpreted element-by-element copying is fast.

The checks use an independent expected-value formula over known original bytes,
not a second implementation of the directional copy algorithm. A pinned executable
.NET comparison and a wider span/memory design review remain future work if this
becomes a public API; no performance comparison has been run.

## Run and verify

Use a matching local development bundle (the September 23 Tasks bundle was used):

```sh
python3 docs/experiments/byte-copy/verify.py --toolchain-root /absolute/path/to/development-bundle
```

The verifier builds the saved project in temporary directories, executes the
[teaching sample](Main.rvn) against [expected output](expected.txt), then runs
[Checks.rvn](Checks.rvn). The range matrix runs in fresh invocations to respect the
normal 100,000-instruction limit; it does not disable runtime budgets.

For the sample alone:

```sh
export NeoCLRRoot=/absolute/path/to/development-bundle
export RavenSdkRoot="$NeoCLRRoot/raven-sdk"
dotnet msbuild docs/experiments/byte-copy/ByteCopy.rvnproj -nologo
"$NeoCLRRoot/bin/neoclr" run docs/experiments/byte-copy/bin/neoclr/Debug/App.neoil --system "$NeoCLRRoot/lib/System.neoil"
```

Use environment variables here so both MSBuild and the nested compiler project
reader see the same toolchain paths. A command-line MSBuild property alone is not
forwarded to the nested project evaluation by this local toolchain.

Validation covers 1,024 small range/alias combinations, extreme counts/offsets,
empty ranges, unchanged destinations after invalid ranges, partial transfers, empty
input, cursor preservation after errors, full output, and a retained array whose
creator returned before allocation pressure triggers real guest collection.
The verifier requires nonzero `--gc-stats` collections; this is synchronous retention,
not proof of rooting a buffer during native I/O.

Related direct-IL validation:

```sh
cargo test --test generic_array_shape --test object_reference_arrays --test managed_arrays
```

These existing 32 tests cover intrinsic/generic identity, bounds, element compatibility,
reference retention and historical array paths. They are not 32 new Byte Copy tests.

## Open work and compiler observation

S1 remains partial: no public stream interfaces, ReadExactly helper, close/flush
contract or injected no-progress backend is established. The guard in CopyAll cannot
be reached by this fixed fixture and is not claimed as independently validated.
Next introduce a second small consumer before selecting reusable contracts, and keep
failure and overlap cases as those contracts evolve. Pending I/O GC belongs to the
later Delayed Copy checkpoint.

An initial `while true` copy loop with no trailing return compiled but emitted an
unresolved InvalidOperationException construction on the local target. Adding a
trailing return was admitted but warned as unreachable. The sample now uses an explicit
EOF flag and normal return, which expresses termination without that fallback. This
is a deferred general Raven emission candidate requiring an isolated .NET/CLI repro;
no Raven compiler change or integration is included here.
