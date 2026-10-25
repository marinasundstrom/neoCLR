# GC benchmarking and profiling procedure

Our intention is to make concurrent GC the general-purpose default, with better
overall application performance. Sync GC remains useful for small,
resource-constrained or single-thread environments. We compare each collector with
its own previous results and also compare both on equivalent application inputs.

## The standard samples

| Sample | What it distinguishes |
| --- | --- |
| Plain HTTP | Basic request and response cost |
| HTTP JSON | Serialization and additional request-path work |
| Compute only | Long-running scalar work with few allocations |
| Reusable buffer | Many writes to existing storage with few allocations |
| Steady allocation | Short-lived objects throughout execution |
| Allocation bursts | Short bursts separated by allocation-free work |
| Retained window | Replacement while a working set remains reachable |
| Mixed phases | Longer alternating allocation-heavy and compute-only periods |

Non-HTTP samples are ordinary Raven applications with deterministic inputs and
checked checksums. They do not force GC. Work counts differ across samples;
compare each row with itself, not the absolute speed of unrelated applications.
The allocation cadence follows work phases, rather than a wall-clock load generator.

## Benchmark and profile separately

**Benchmark mode** uses minimal common counters to measure throughput, CPU,
managed allocation totals and arena occupancy. Five alternating rounds are the
standard. Plain HTTP and HTTP JSON run with checked response bodies and EOF.

**Profile mode** runs the same samples with owner-pause histograms and, for the
standalone cases, GC phase costs. It explains where work occurs. Instrumented
results are kept separate and are never used to claim a throughput improvement.
General CPU sampling is a complementary tool when the cost is outside the collector.

The first configuration uses sync allocation/publication budgets of 256/4096 and
concurrent budgets of 1024/16384 with a 50% mutation-log watermark. Policies may
differ: concurrent work needs its own memory/headroom and scheduling evaluation.
Every report records its configuration; these profiles do not change production
defaults.

## Run the developer suite

The current harness requires a qualified macOS ARM64 development checkout and its
retained compiler/native build artifacts. It is not an installed benchmark command.
See the [setup and exact sample definitions](https://github.com/marinasundstrom/neoCLR/blob/main/benchmarks/gc-suite/README.md)
for toolchain prerequisites and artifact preparation.

```sh
python3 benchmarks/gc-suite/run.py benchmark --output target/gc-runs/baseline

python3 benchmarks/gc-suite/run.py profile \
  --build target/gc-runs/baseline/build --output target/gc-runs/profile \
  --rounds 3 --requests 1024
```

A later benchmark can use `--baseline target/gc-runs/baseline/run.json` to compare
sync with sync and concurrent with concurrent. The comparison rejects mismatched
sample definitions, hardware, workload sizes or diagnostic runs. Policy changes
are reported explicitly. Keep raw reports, failed runs and build hashes; changing
workloads creates a new series rather than rewriting the old one.

## Read the result before changing defaults

Throughput, CPU, pauses and memory are separate measurements. A concurrent collector
can reduce interruptions while using more CPU, and a smaller-resource workload may
favor sync. Few allocations can still expose runtime store/barrier overhead. A
collector should not be selected solely from one HTTP response benchmark.

[Current results](/benchmarks/gc/) show both improvements and remaining gaps.
[Historical results](/benchmarks/history/) preserve earlier conditions. The default
changes only after broad correctness, ownership, resource and execution-mode
qualification—not because the intended direction has already been chosen.
