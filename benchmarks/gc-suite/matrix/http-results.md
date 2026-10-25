# HTTP execution/collector results — 2026-10-25

Three rounds, 90 trials, 297,504 checked responses including warmup; all pass.
Concurrent native runs complete 25,697 epochs with no invalid-state reports.
These are single-owner loopback results from the private persistent driver,
not worker scaling, saturation, public API enablement or TechEmpower ranking.

## Native throughput

Median requests/second; 4,096 measured requests per invocation, 16 warmup,
four clients, pipeline depth 16. Compilation and readiness are outside the
measured traffic window. Client CPU and trial ranges remain in the archive.

| Payload | Connection | AOT sync | AOT concurrent | JIT sync | JIT concurrent |
| --- | --- | ---: | ---: | ---: | ---: |
| plain | close | 5,344 | 4,090 | 3,205 | 2,647 |
| plain | keepalive | 6,328 | 5,134 | 3,707 | 3,346 |
| plain | pipeline | 7,395 | 6,038 | 4,432 | 3,749 |
| json | close | 3,555 | 2,819 | 2,165 | 1,822 |
| json | keepalive | 3,809 | 3,254 | 2,321 | 2,094 |
| json | pipeline | 4,117 | 3,540 | 2,484 | 2,216 |

## Native invocation CPU

Median CPU seconds, including warmup/readiness/output/cleanup and collector
worker CPU, excluding JIT compilation. Do not divide by measured requests alone
and label that an exact request-only CPU cost.

| Payload | Connection | AOT sync | AOT concurrent | JIT sync | JIT concurrent |
| --- | --- | ---: | ---: | ---: | ---: |
| plain | close | 0.7596 | 1.0036 | 1.2696 | 1.5366 |
| plain | keepalive | 0.6435 | 0.8162 | 1.1068 | 1.2502 |
| plain | pipeline | 0.5537 | 0.6949 | 0.9280 | 1.1130 |
| json | close | 1.1399 | 1.4619 | 1.8898 | 2.2467 |
| json | keepalive | 1.0828 | 1.2932 | 1.7719 | 1.9864 |
| json | pipeline | 1.0023 | 1.1876 | 1.6531 | 1.8748 |

## Interpreter and interpretation

Interpreter uses mark-sweep only, 64 measured requests plus 16 warmup per
invocation. Median whole-process CPU seconds (close/keepalive/pipeline) are
44.75/40.66/38.36 for plain and 73.19/70.78/65.08 for JSON. These include startup
and have different work counts from native trials; the archive retains exact
throughput and output, rather than rounding these into native comparison ratios.

Connection reuse and pipelining reduce work in both collectors. Concurrent AOT
plain improves from 4,090 to 6,038 requests/s and CPU falls from 1.0036 to 0.6949
seconds (about 48% more throughput and 31% less CPU). Sync remains faster at
7,395 pipelined requests/s. This is an HTTP path improvement, not proof that
concurrent GC beats sync. JIT remains slower than AOT with either collector.

The three plaintext pipeline AOT-sync trials vary; inspect raw trial ranges before
interpreting the ratio as stable capacity. Native policies remain sync 256/4096
and concurrent 1024/16384 with a 50% watermark and index-only lookup. Both share
the same fixture/JSON model. No detailed pause profiling was active. The author
has requested separate worker/scheduling profiles to test the architecture.

Evidence: `http-results.json.gz`, matching `http-build.json.gz`, qualification
`http-smoke.json.gz`; reproduction and CPU scopes are in [README](README.md).
