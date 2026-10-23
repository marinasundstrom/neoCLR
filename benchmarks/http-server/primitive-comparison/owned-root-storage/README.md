# Generated storage ownership — 2026-10-23

The native compiler creates each function's storage-descriptor table once in its
prologue. Slot values change during execution, but descriptor addresses, widths
and flags do not. Previously every root publication rescanned those descriptors
on every ancestor, despite that compiler-owned lifetime.

Native-GC codegen now imports `neoclr_gc_enter_owned_v1`. That entry validates the
complete table once, enters the frame and sets an ownership bit in its existing
reserved field. Subsequent publication skips only that frame's storage metadata
scan. Collection still reads the current slot values. Transient tables remain
checked because codegen rewrites them at publication sites. Fault contexts, point
plans, root publication frequency and GC pacing are unchanged.

Ordinary native callers and existing generated objects still use
`neoclr_gc_enter_v1`, retaining mutable-descriptor checks at every publication.
Mixed chains preserve those checks per frame. The new contract requires exclusive
ownership of the table and fixed frame storage pointer/count, addresses, widths
and flags until leave or parked-stack expiry. Native callers must not forge the
marker or mutate owned descriptors. This is a trusted native contract, not memory
protection against arbitrary native writes. The compiler owns the table; runtime
entry checks and frame lifetime establish its validity. There is no global cache
keyed by table address or image registration/unloading protocol.

The worker adapter preserves the ownership bit alongside its existing exclusive
boundary state. Park/resume retain it with the frame; leave and new entry clear
old state. The private frame stays 104 bytes, no object headers grow and there is
no new allocation or TLS table. Costs are one storage-validation pass on each
owned entry and a per-ancestor ownership branch. Invalid generated metadata now
fails at entry, before publication. New objects require the new runtime symbol;
old objects continue to link and run on the checked path.

## Comparison and boundaries

[CoreCLR stack walking](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/stackwalking.md)
describes compiler-controlled frames and per-method metadata for managed stack
walking, alongside explicit runtime Frames for unmanaged code. The applicable
principle is using compiler/runtime knowledge of execution metadata. neoCLR's
implementation deliberately retains explicit per-invocation tables and frame
chains rather than CLR's unwind/GC encoding. Its benefit here is avoiding repeated
validation; its costs include table construction and entry validation. Shared
binary maps and image lifetime registration remain future work.

Raven CLI metadata, semantic behavior, Runtime Contract configuration and .NET
behavior are unchanged. The tested Raven bundle remains codex/neoclr revision
1128b98cf; only neoCLR's native backend changes generated entry imports. Both
repositories' bridge docs record the contract and compatibility requirement.
There is no public managed API change or API snapshot update. Website feature
claims remain accurate; no website build is needed for this internal change.

## Validation and reproduction

New native cases reject malformed owned entry tables, preserve validation of an
ordinary ancestor beneath an owned child, and reject mutated transient metadata
on an owned frame. A live-root case changes slot values, collects through a parked
owned frame, verifies the old allocation is reclaimed and the new one retained,
then clears the root and reuses the frame without retaining ownership state.
The complete optimized fixture has 94 cases across GC and probe-only modes.
Native GC sanitizer validation and all existing mutable-descriptor cases pass.

Eleven focused native-codegen tests pass, including Windows object-generation
checks (not Windows execution qualification). The rebuilt HTTP object imports
only the owned GC entry; the retained baseline object imports the ordinary entry.
Forced-GC HTTP completes eight checked responses over two invocations. Compiled
shared-worker optimized/UBSan tests pass with graph and fault collection on the
new entry path. UBSan does not establish race freedom; thread synchronization is
unchanged. Reports retain commands and tested-source hashes.

```sh
CARGO_INCREMENTAL=0 cargo test --release -p neoclr-codegen \
  --test root_probes --test windows_stack
python3 scripts/validate-native-gc.py --output target/owned-root-check
python3 benchmarks/http-server/primitive-comparison/owned-root-storage/validate.py \
  --output target/owned-root-optimized
python3 benchmarks/http-server/primitive-comparison/owned-root-storage/measure.py \
  --build target/primitive-comparison-build/build.json \
  --client target/primitive-comparison-eof/client \
  --object target/plain-slot-metadata/candidate.o --baseline 522382b9c \
  --output target/owned-root-build-final --build-only
python3 benchmarks/http-server/primitive-comparison/owned-heap/forced.py \
  --build target/primitive-comparison-build/build.json \
  --object target/owned-root-build-final/candidate.o --output target/owned-root-forced
```

For capacity trials repeat the measure command without `--build-only`, using a
fresh output directory. The runner regenerates the candidate object before timing.
Both objects use the same current runtime, owned heap/base cache, 1 MiB per owner,
16 Serve lanes and paced GC. One/four isolated HTTP owners serve checked ten-byte
responses through EOF with 32 connections, two seconds warmup and six measured,
forward/reverse. CPU/response includes process lifetime and warmup/window/drain
completions, subtracting client CPU. Builds/tests do not overlap capacity trials.
Use the same Xcode SDK and bundle as the preceding experiments.

The first build-only attempt rejected a changed root-probe.h hash before compiling;
the harness whitelist was corrected to record this deliberate contract change.
The successful build and all validation are retained separately. This was not a
runtime failure. These short local trials do not qualify sustained capacity,
shared-heap HTTP or parity with other web servers.

## Results and remaining work

| Variant | Owners | Requests/s (two trials) | CPU µs/response | p99 ms |
| --- | --- | --- | --- | --- |
| baseline | 1 | 1,096.3, 968.7 | 911.9, 982.3 | 33.59, 75.66 |
| baseline | 4 | 3,487.2, 3,640.7 | 1,108.0, 1,030.8 | 18.48, 12.97 |
| candidate | 1 | 1,650.8, 1,631.3 | 606.9, 616.6 | 23.50, 38.06 |
| candidate | 4 | 4,992.7, 4,302.8 | 753.3, 782.9 | 8.51, 22.55 |

Mean one/four-owner throughput rises 58.9%/30.4%; estimated CPU per response falls
35.4%/28.2%. Both candidate throughput and CPU samples improve over both respective
baselines. Desktop variation remains substantial, particularly at four owners,
and four-owner tail latency is mixed. The magnitude is local evidence of useful
work removal, not a precise portable or sustained capacity claim. Forced-GC and
timed candidate objects are byte-identical (hashes retained in both reports).

Shared-worker optimized/UBSan runs each complete 32 healthy and 32 fault invocations,
24,544 graph collections and 448 fault collections, reaching four active graph
participants. Shared asynchronous HTTP integration/performance remains separately
unqualified. Remaining costs include transient validation, repeated plan checks,
frame traversal and constructing per-invocation tables. Extending ownership to
mutable transients or replacing tables with shared binary maps requires separate
lifetime and validation work; no such broader contract is implied here.

After validation/timing, only the helper comment was corrected to describe its
entry-time and publication-time callers. validation-source-note.json records the
before/after hashes and identical comment-stripped source; the tested runtime
implementation is unchanged.

Paired Raven documentation commit: `b805b5367` on `codex/neoclr`; Raven main and
the compiler binary remain unchanged.
