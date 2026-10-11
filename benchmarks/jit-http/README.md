# HTTP execution-mode workbench

Development benchmark for macOS ARM64: the same Raven HTTP server and client
assemblies run through interpreter, JIT and AOT. Each process performs 32 sequential
loopback HTTP/1.1 requests with connection close and a ten-byte Unicode response.
The Python peer validates the response bytes or request target; the guest client
also validates the response. Any mismatch fails the run.

Run from the repository root using a passing CLI HTTP qualification recipe:

```sh
python3 benchmarks/jit-http/benchmark.py \
  --recipe target/jit-cli-http-final/validation.json \
  --compiler /absolute/path/to/native-enabled/rvnc.dll \
  --output target/jit-http-measurement --rounds 5
```

The output directory must be new. The recipe supplies the current runtime libraries,
AOT compiler, native host build/link commands and CLI invocation. The workbench
rebuilds both applications and the host, recording commands, input hashes, build
times and artifact hashes. `--compiler` selects a compatible updated compiler
without modifying the recipe's runtime bundle. Its adjacent DLL/JSON dependencies
are hashed too. Building with and without native support in one Raven output tree
may require a forced restore/rebuild to refresh compiler dependency metadata.

One complete process per mode/workload warms the machine before measurement. Each
measured process discards its first eight requests and measures the remaining 24.
Five rounds rotate mode order; startup is recorded separately. Startup includes JIT
compilation and metadata loading but excludes ahead-of-time builds. Results contain
all raw samples, median/range throughput and startup, and server p50/p95 latency.

This is a small concurrency-one iteration workload, not a saturation test. The
Python peer shares the machine; short runs, scheduling, GC policy and thermal state
can affect results. Background Rust compilation and macOS indexing were observed
during the recorded run; its timing is exploratory and must be repeated on a quiet
machine before using it for performance claims. It does not compare .NET, Go or JavaScript, establish Windows
performance, or justify a general language/runtime ranking. Run without concurrent
builds or other heavy work. JSON node/object serialization is a separate required
benchmark before scheduler work.

## Compiler regression

The server intentionally binds `server` using `let … else`, uses it before an
await, and reuses it across repeated awaits. This exposed two compiler gaps:

- Shared lifetime analysis dropped the binding at a synthetic guard block. The
  ordinary .NET regression read zero after suspension; native emission rejected
  an uninitialized local. Raven main `2059d5e5f` fixes lifetime analysis.
- The native integration portable emitter stored pattern payloads only in locals,
  even when later reads used hoisted fields. Integration commit `2535c8706` now
  initializes those fields too. The native compiler is from
  `codex/source-object-metadata-resolution`; native support is not implied on main.

These are compiler corrections, not changes to the public async contract or a
relaxation of native metadata verification. The current state-machine implementation
remains in place until the separately planned runtime suspension work.

## Recorded run — 2026-10-11

[Raw measurements and build provenance](results.json) and
[compiler regression evidence](compiler-validation.json). Large successful compiler
diagnostic dumps are represented by hashes/byte counts in the checked-in report;
raw timing samples are retained in full. All 36 processes
(6 warmups, 30 measured) completed: 1,152 validated requests in total. Input hashes
remained unchanged during measurement. The runtime is neoCLR main `33648ce6`;
the native-enabled Raven compiler matches integration revision `2535c8706`, using
the existing extension-indexer runtime bundle identified by the recorded hashes.

The following medians are **provisional under observed background CPU contention**,
not release performance claims. Each throughput sample covers 24 requests after
eight discarded requests. Startup includes JIT compilation but excludes AOT builds.

| Workload | Mode | Requests/s | Startup (s) |
| --- | --- | ---: | ---: |
| server | interpreter | 1.38 | 3.120 |
| server | jit | 53.49 | 3.874 |
| server | aot | 54.31 | 0.010 |
| client | interpreter | 0.96 | 2.377 |
| client | jit | 34.83 | 4.525 |
| client | aot | 35.66 | 0.010 |

JIT and AOT have similar request throughput in this small workload; JIT startup
is substantially higher because each process compiles its application. These
measurements do not isolate individual optimizer, GC or transport costs. Repeat
on a quiet machine before drawing quantitative comparisons. JSON and equivalent
.NET/Go/JavaScript workloads remain separate follow-ups.
