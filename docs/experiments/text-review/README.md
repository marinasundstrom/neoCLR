# Text foundation review observations — 2026-09-27

This is evidence for the [existing text design review](../../design/text-abstraction.md#systemtext-foundation-review--2026-09-27),
not a new implementation milestone or cross-platform test suite. The probes inspect
current behavior; they do not prototype proposed APIs. No runtime/compiler code changes.

`Main.rvn` compiled, imported, verified and ran using the development bridge/library
from `730bcdc2` and Raven `a108df82a`, on macOS arm64. It checks counts and indexing,
scalar-boundary slicing inside a grapheme, ordinal prefix/search, resegmentation on
construction/concatenation, exact equality, simple folding and strict UTF-8 failure.
It prints `Text foundation observations passed`. The first attempt needed an explicit
byte cast in the malformed-input fixture; the corrected probe passes.

The other four programs ran locally on .NET 10, Swift 6.2.3, Rust 1.95.0 and Go
1.26.3. [Recorded observations](results.json) capture these runs, not claims that
all Unicode versions/platforms give identical answers. The corpus uses older stable
characters. Rust and Go probes do not add an external grapheme library. Framework
reference versions reviewed on the web can be newer than locally installed tools;
the design separates documentation from executed evidence.

Run the independent probes from the repository root:

```sh
dotnet run --project docs/experiments/text-review/TextReview.csproj
swift docs/experiments/text-review/probe.swift
rustc docs/experiments/text-review/probe.rs -o /tmp/neoclr-text-review-rust
/tmp/neoclr-text-review-rust
go run docs/experiments/text-review/probe.go
```

For Raven, place `Main.rvn` in a saved project with the matching development reference,
then use `docs/experiments/raven-target/run_project.py` with the matching bridge,
System library and native runtime. The preceding
[string-comparison evidence](../raven-target/string-comparison-validation.json)
identifies those artifacts and supplies the broader folding/hash contract checks.
No website, full runtime suite or performance benchmark was run for this review.
