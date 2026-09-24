# String storage exploration

This is an executable investigation, not a production String representation change.
The Rust example compares actual `Value::String` cloning with a private `Arc<str>`
prototype and measures the current VM's repeated String-to-Object wrapper allocations.
The .NET baseline tests the reference semantics that a later migration should preserve.

```sh
cargo run --release --example measure_string_storage
dotnet run --project docs/experiments/string-storage/dotnet/Baseline.csproj
```

The allocator probe counts allocation requests and requested bytes only during clone
loops. Input creation, formatting, runtime setup and candidate construction are
excluded. Clones are immediately dropped; the output is not peak live memory or a
speed benchmark. The example asserts the observed owned-string baseline so a future
representation change must deliberately update this experiment.

The candidate checks alias/content distinction, Rust array/field ownership, last-owner
release and no assumed empty-string interning. It does not test guest tracing GC for
the candidate. The separate current-VM cast loop runs under an eight-object heap limit
and checks full reclamation of its 1,000 temporary wrappers.

See the [design investigation](../../string-storage-design.md) for recorded results,
.NET comparison, alternatives, costs and the proposed integration gates.

Validation on 2026-09-24: Release allocation assertions and prototype ownership gates
pass; the current VM reclaims all 1,000 wrappers across 125 collections. The .NET 10
reference-identity baseline and the 510-page combined website build pass.
