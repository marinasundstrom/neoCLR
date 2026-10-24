# Warm native call layout — 2026-10-24

This controlled follow-up separates two hypotheses from the remaining owned-heap
JIT/AOT plaintext HTTP gap: long-range call lowering and dynamic helper linking.
It changes no production code or default. The temporary compiler patch is retained
as `experiment.patch`; both the compiler source and normal binary were restored.

## Method and admission

`measure.py` reuses the native-host-ownership experiment's application, exclusively
owned 1 MiB paced heap, helper source, wrapper and checked Go client. Configurations
run forward then reverse, with one second warmup, five seconds measured load,
32 connections and client GOMAXPROCS=2. Every request opens a new connection and
checks the ten-byte response. CPU is the server `ps time` delta after READY through
warmup and measurement, divided by completed plus drained responses. Its resolution
is approximately 10 ms; it excludes compilation and teardown. These development
entry hosts are not the optimized multiworker host or a Kestrel comparison.

The four admitted configurations are ordinary static AOT, PIC AOT with all imported
function references marked non-colocated, ordinary AOT linked to the exact owned
helper dylib used by JIT, and ordinary owned JIT. The baseline AOT object is checked
byte-for-byte against the preceding experiment. Commands and input hashes are in
`results.json`. Absolute target paths record local provenance, not portable inputs.

A fifth configuration attempted non-PIC AOT with non-colocated libcalls, matching
more of JIT's flags. macOS rejected illegal text relocations before any load trial.
`rejected-nonpic.json.gz` retains the failed build and diagnostics. This is not a
runtime failure or evidence against JIT's supported in-memory relocation scheme.
The first admitted cohort overlapped disassembly and is retained separately in
`overlapped.json.gz`; use only the subsequent clean cohort for conclusions.

## Results

| Configuration | Mean requests/s | Range | CPU µs/response |
| --- | ---: | ---: | ---: |
| baseline | 2,937.7 | 2,906.6–2,968.8 | 338.5 |
| long-range | 2,911.1 | 2,898.8–2,923.4 | 344.4 |
| dynamic | 2,954.9 | 2,940.4–2,969.4 | 339.0 |
| jit | 1,781.2 | 1,737.4–1,825.0 | 549.9 |


Only two observations per configuration are available on this macOS ARM64 desktop.
Small differences are exploratory; no confidence interval, universal call penalty,
or peer-server parity is established. Longer representative workloads remain useful.

`instructions.json` counts instructions in 1,133 generated functions in the final
executables. Static baseline has 3,815 direct `bl` calls and 18,223 indirect `blr`
calls; long-range has zero and 22,038 respectively. Thus linker relaxation did not
remove the manipulated distinction. Dynamic AOT retains baseline call counts but
has more loads. Counts are static, not executed frequency. Selected prologues are
retained; `inspect.py` regenerates counts/excerpts with Apple's llvm-objdump.

The PIC long-range and dynamic-helper changes do not reproduce the large JIT gap
in this workload. This narrows the investigation without proving these costs zero.
Non-PIC generated code, actual JIT allocation/address locality, and runtime call
frequency remain unisolated. Next compare generated JIT machine code and hot helper
call counts under symbolized profiling; do not remove required range protections.

## Safety, comparison and reproduction

Cranelift 0.121.2's `cranelift-jit/src/backend.rs` explicitly configures non-colocated
libcalls and non-PIC code because independently allocated definitions may be too
far apart for short-range calls. neoCLR's `compiler.rs::prepare_calls` additionally
clears imported-function colocation for JIT. AOT can rely on linker-managed reach.
The temporary patch applies that latter policy to PIC AOT for a safe counterfactual.
This does not establish that ordinary JIT functions can use direct calls safely.

The existing [CLR/runtime comparison](../../../docs/design/allocation-metadata-runtime-comparison.md)
remains the design basis: distinguish loading, generated execution and shared
runtime services. This experiment changes no .NET-like API or GC contract and
makes no claim that CoreCLR uses the same code-placement implementation.

To reproduce with the retained prerequisite artifacts, build a separate compiler
with `experiment.patch`, save it as `target/call-layout/experiment-codegen`, then
restore the source and normal compiler before running `measure.py`. It intentionally
refuses an existing output directory. The rejected non-PIC attempt is evidence only;
the final driver selects the admitted configurations. Run `inspect.py` **after**
measurement, then `summarize.py target/call-layout/results/results.json`.
The prerequisites are listed and hashed in the report, including the generated
owned-host source from the earlier experiment. No globally installed bundle changes.

Four aggregation tests check arithmetic, failed servers and incomplete cohorts:
`python3 -m unittest discover -s benchmarks/execution/call-layout -p 'test_*.py'`.
Eight checked live load trials validate the diagnostic configurations; no production
runtime change requires a broad suite or website rebuild.
