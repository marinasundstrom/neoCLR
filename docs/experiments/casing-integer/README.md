# Unicode casing and Int64 report APIs

Implemented in development, 2026-09-27. The author selected these two bounded slices
after the encoding foundation. [Design and tradeoffs](../../design/text-casing-integer.md)
compare .NET 10, Unicode 17 and Rust. This does not reopen HTTP or the builder project.

`Main.rvn` is the executable API consumer: full casing expansion/context, unchanged
emoji, no normalization, simple-fold independence, signed 64-bit limits, strict
format/overflow errors, decimal round trips and a small report. The helper functions
are public to use the existing bridge's argument-conversion profile. Raven string
escapes use `\u0000` for embedded NUL.

```sh
python3 docs/experiments/casing-integer/verify.py \
  --runtime target/release/neoclr \
  --bridge docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --system /path/to/matching/System.neoil \
  --reference /path/to/matching/NeoCLR.CoreProbe.dll \
  --evidence /tmp/casing-integer-validation.json
cargo test --lib string_casing::tests
cargo test --test casing_integer
dotnet docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --casing-integer-checks /tmp/casing-integer-signatures
dotnet run --project docs/experiments/ordinal-text-dotnet/ordinal-text.csproj \
  -- --casing-integer
```

`validation.json` records the consumer and artifact hashes. `dotnet-results.json`
records the actual .NET 10/macOS baseline; differences are intentional, not a claim
of exact .NET compatibility. `casing-integer-signatures.json` records 16 exact
metadata admissions/rejections. Native unit checks visit every generated mapping,
property boundaries and contextual examples; native integration checks exercise
the erased parse protocol, service reachability and wrong native return signatures.
No full suite, website build or performance claim is part of this slice.

Validation also reruns the four archived Int32 parse tests after separating the new
Int64 union API from the archived bootstrap. A check of the separate historical
source checklist remains blocked by its pre-existing caller-discovery drift
(`UInt64ToString`); see [tooling tracking](../../tracking/toolchain-release.md).
The current public metadata inventory and API snapshot are independently checked.
