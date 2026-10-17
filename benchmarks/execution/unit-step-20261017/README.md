# Exclusive unit-step compiler experiment — 17 October 2026

Raven `368e5ca39` removes a redundant advance overflow guard for native
portable signed Int32/Int64 exclusive loops with literal unit step. The upper
bound proves the increment safe. Other steps and inclusive bounds retain guards.
The [boundary consumer](../../../docs/validation/range-iteration/README.md#exclusive-unit-step-optimization--2026-10-17)
passes interpreter/JIT/AOT; 23 focused compiler tests pass.

## Controlled comparison

Only the compiler analysis DLL is replaced in a copy of the
[fixed editor candidate](../../../docs/validation/release-candidate-20261017/README.md#fixed-packaged-editor-qualification).
The worker is recompiled; runtime, codegen and five native libraries are unchanged.
The compiler assembly is a fresh Debug/net11.0 build. This is **not a qualified
release package** and does not replace the installed SDK or published baseline.
Compiler build/configuration affects compilation, which is outside these timings.
[Identity and summaries](comparison.json) and raw reports retain inputs and hashes;
3262 distinct input paths still match after all 20 successful processes.
The diagnostic excerpt shows the removed guard; it is not an opcode-shape test.

On Apple M1/macOS 27.0, each CPU operation sums 0 through 9,999 and checks 49,995,000.
Interpreter runs use two warmups and 16 operations, five fresh processes per cohort.
Before and after cohorts run sequentially, not interleaved. Native checks use 20
warmups and 10,000 operations, five processes per mode. No builds or tests overlap
the timed runs. This remains an interactive host without CPU/thermal isolation;
batch timing includes callbacks, checks and marker/pipe scheduling, with startup
reported separately. One after round was slower; retain it rather than discard it.

| Cohort | Mode | Median operations/s | Range |
| --- | --- | ---: | ---: |
| Before | interpreter | 4.27 | 4.22–4.31 |
| After | interpreter | 6.31 | 5.41–6.33 |
| After | jit | 40303.32 | 40012.32–40558.57 |
| After | aot | 40805.63 | 40650.22–40902.32 |

Interpreter median throughput improves **47.7%**.
The earlier [Preview 15 measurement](../candidate-20261017/README.md) was 6.45
operations/s: this largely recovers the 35% regression observed in the candidate.
That earlier cohort is not a paired measurement; small differences and native
throughput differences do not establish improvement or regression. JSON and HTTP
are not remeasured here. A fresh complete SDK/library build, packaged editor
qualification and Windows execution/performance remain release work.

## Reproduce

Build the native compiler at the recorded revision, prepare a coherent bundle (or
explicitly record a compiler-only experimental override), then use:

```sh
python3 benchmarks/execution/prepare.py --bundle /bundle --output /new/execution
python3 benchmarks/framework/runner.py --manifest /new/execution/manifest.json --output /new/cpu.json --rounds 5 --timeout 300 --filter cpu.sum --warmups 2 --operations 16 --mode interpreter
python3 benchmarks/framework/runner.py --manifest /new/execution/manifest.json --output /new/native.json --rounds 5 --timeout 300 --filter cpu.sum --warmups 20 --operations 10000 --mode jit --mode aot
```
