# Binary native metadata loading experiment — 2026-09-30

Feature branches: neoCLR `codex/extended-cli-metadata`, Raven `codex/metadata-consumer`.
The required execution section now has a binary schema 2 alongside compatible JSON
schema 1. See the [profile and tradeoffs](../../design/extended-cli-metadata.md#binary-native-execution-profile--2026-09-30).

The initial local release-mode comparison uses one C#-produced constant-return module
and a synthetic version with 65 functions (64 unused arithmetic helpers). Both encodings
decode to identical complete runtime metadata, verify, and return 42. The examples
measure seven batches and report median time per iteration. Data is preloaded; file IO,
process startup, compiler emission and initial System parsing are excluded. Results are
exploratory, sequential, single-machine samples with no randomized ordering or confidence
intervals. They do not establish cold-start or general execution throughput.

| Case | JSON PE bytes | Binary PE bytes | JSON PE binding + decode | Binary PE binding + decode |
| --- | ---: | ---: | ---: | ---: |
| One function | 16,384 | 16,384 | 8.66 µs | 8.75 µs |
| 65 functions | 200,704 | 139,264 | 2.43 ms | 2.08 ms |

The larger fixture's container is about 31% smaller and binding/decoding is about 14%
faster in this run. The tiny case shows no decoding benefit and PE alignment hides
payload size savings. Binary is not automatically faster: this profile still walks
maps twice (strict profile guard followed by typed decoding), allocates model objects
and repeats field/identity strings. Indexed native tables/heaps remain a candidate.

Warm-System linking/preparation costs approximately 24–25 ms in these fixtures,
verification 6–8 ms and prepared-program execution 0.16–0.18 ms. These are distinct
experiments, not additive stages of a single cold start. In particular, the existing
`load` API already links against bundled System as part of validation; timing load
and then preparation would count some work twice. Variation in full-load measurements
is dominated by that shared work, so it is not evidence of an encoding-only gain.
Class-library preparation/reuse deserves attention as real class-library artifacts
become the Raven symbol-loader input.

Reproduce from the neoCLR worktree:

```sh
cargo build
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- \
  --container-integration target/debug/neoclr target/extended-cli-metadata/binary-container
cargo build --release --example metadata_loading
target/release/examples/metadata_loading target/extended-cli-metadata/binary-container
```

The local default SDK did not link with the selected Apple compiler. The successful
release build selected `SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.2.sdk`
for that invocation only; no repository/global SDK setting was changed. Raw timings,
platform and tested artifact/executable hashes are in [binary-loading.json](binary-loading.json).
Projection MVIDs change when regenerating the fixtures, so file hashes need not match
although native decoded metadata must. The benchmark is scoped to this format decision
and is not a CI performance threshold or a claim about the full runtime class library.
