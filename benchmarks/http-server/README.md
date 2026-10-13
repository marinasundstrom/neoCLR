# HTTP server request-processing baseline

This controller builds the existing Raven plain-text server once, then runs the
same assembly through interpreter and default JIT and its native compilation through
AOT. It uses the exact qualified Preview 14 macOS ARM64 bundle and the repository's
HTTP native host build procedure. It validates the bundle manifest before and after
measurement and retains build commands, input hashes, artifact hashes and raw samples.

```sh
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
python3 benchmarks/http-server/benchmark.py \
  --bundle /absolute/path/to/neoclr-native-poc \
  --output /fresh/path/to/results --rounds 5
```

Native compilation requires Xcode's matching SDK/toolchain. Use the bundle's AOT
compiler; the script does not substitute a newer backend. The repository native host
sources and revision are recorded separately from the bundle. Run on a quiet host
without concurrent builds. The output directory must not already exist.

## Measurement contract

- GET `/greeting`, ten UTF-8 response bytes (`Café 🌍`), HTTP/1.1, no TLS.
- Sequential loopback requests, concurrency one, a new connection per request.
  The Python client runs on the same machine as the Raven server.
- Every response must have status 200, the expected byte body and Content-Length,
  and Connection: close. The process must print its completion marker and exit
  successfully without stderr. A mismatch aborts the run; failed runs are not scores.
- One full 32-request process per mode warms the host. Five measured rounds rotate
  interpreter/JIT/AOT order. Each process discards eight requests and measures 24.
- Throughput is 24 divided by the sum of those 24 request durations. Publish the
  median and min/max across rounds, not a ratio of unrelated runs.
- Latency includes connection setup, request transfer, server processing, response
  validation and EOF. Percentiles pool the 120 measured requests per mode and use
  nearest rank. At this sample size p99 is a coarse exploratory estimate.
- Startup measures process launch to the emitted listening port, separately from
  throughput. It includes JIT compilation and metadata loading; builds are excluded.
- Native heap and byte-array admission match the JIT bundle configuration. Interpreter
  allocation/GC policy differs; equal admission limits would not imply equal GC work.

This fixed-count run is a functional performance baseline, not sustained throughput,
saturation, a concurrency test or an isolated handler/serializer measurement. Host
load averages and a process CPU snapshot are retained. There is no CPU/RSS measurement
of the guest yet. JSON response serialization, JSON plus database access, Windows and
.NET/Go/JavaScript comparisons remain distinct unmeasured workloads.

## Recorded baseline — 2026-10-13

[Raw samples and provenance](results-20261013.json) record 576 validated requests
across 18 processes (three full-process warmups plus five rounds per mode), with
zero errors. Each mode contributes 120 measured requests. The controller was first
introduced with this report; its input hash identifies the exact script. The report's
repository revision identifies the existing native host sources, not a prior commit
of this controller.

Apple M1, macOS 27.0, Preview 14 macOS bundle, 32 MiB native heap admission. Both
manifest and component hashes match the released bundle. Desktop and macOS indexing
were active; this is an **exploratory local baseline**, not a controlled comparison.

| Mode | Median requests/s | Round range | Median startup (s) |
| --- | ---: | ---: | ---: |
| Interpreter | 1.71 | 1.61–1.83 | 2.510 |
| JIT | 60.96 | 44.52–61.30 | 4.313 |
| AOT | 63.43 | 57.77–64.11 | 0.018 |

Startup includes JIT compilation and is excluded from request throughput. Do not
infer a general runtime speed ratio or compare these numbers with older workbench
runs. A quiet-host repeat and longer runs are needed before stronger claims.
