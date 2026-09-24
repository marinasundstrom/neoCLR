# String storage exploration

The Rust example compares an owned String baseline with production
`Value::String` shared cloning and measures the current VM's repeated String-to-Object wrapper allocations.
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
Production Value clones now request zero allocations for all measured sizes; the
owned baseline still requests one buffer per non-empty clone.

A retained private `Arc<str>` candidate checks alias/content distinction, Rust array/field ownership, last-owner
release and no assumed empty-string interning. It does not test guest tracing GC for
the candidate. The separate current-VM cast loop runs under an eight-object heap limit
and checks full reclamation of its 1,000 temporary wrappers.

See the [design investigation](../../string-storage-design.md) for recorded results,
.NET comparison, alternatives, costs and the proposed integration gates.

Validation on 2026-09-24: production Value clone allocation assertions and prototype
ownership gates pass. Five focused runtime tests cover buffer adoption, extraction,
content equality, Object display sharing and GC/host ownership; the current VM reclaims all 1,000 wrappers across 125 collections. The .NET 10
reference-identity baseline and the 510-page combined website build pass.

The production shared-storage phase also passes the compiled Raven String/Object
sample: 421 managed allocations reclaimed, nine collections, peak 64 and zero live
objects. String identity remains explicitly rejected.

The all-target Rust compile check passes after updating host constructors. Object
equality/display, arrays, graphemes, String helpers, file streams and reflection
regressions pass. UTF-8 and worker tests, including the combined result/output byte
budget, pass as well: 116 focused tests in total. API snapshot checks and the combined
510-page website build pass.
