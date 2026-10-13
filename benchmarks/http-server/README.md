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

## JSON and database workloads

Select `--workload json` to construct a new `Person` object and serialize it on every
request. The response contains `id: 42`, `name: "Café team"`, and `active: true`.
The client parses every response and validates the complete three-field value.
This measures HTTP plus object construction/reflection/serialization and transfer,
not an isolated serializer.

Select `--workload database --postgres-port PORT --postgres-container NAME` for
one parameterized PostgreSQL SELECT per HTTP request. The query returns those same
three scalar values, maps the single buffered row to `Person`, then uses the same
JSON response function. One connection is opened before readiness and reused for
all 32 requests. There is no table, permanent dataset, transaction or pool. This is
a query/map/serialize baseline, not disk-backed database scalability.

Use a dedicated disposable database and record the image/version/resources:

```sh
docker run --detach --rm --name neoclr-http-benchmark \
  --cpus 2 --memory 512m --publish 127.0.0.1::5432 \
  --env POSTGRES_HOST_AUTH_METHOD=trust postgres:18
docker port neoclr-http-benchmark 5432
```

After PostgreSQL is ready, pass the printed host port and container name to the
benchmark. The harness records version, image digest, memory/CPU limits and query
parameters. Keep the port bound to loopback; this uses development trust
authentication. Stop only the benchmark's own container after the run:

```sh
docker stop neoclr-http-benchmark
```

JSON and database builds retain the `Person` properties/constructor through explicit
reflection roots, used by both native modes. The interpreter receives the same
library modules. Preparation happens outside timed request loops.

### Database AOT host boundary

The released PostgreSQL library uses runtime-async operations. The standalone HTTP
project build profile does not bind these operations. For the database benchmark,
the controller follows `docs/experiments/sql-postgresql/validate.py`: compile the
same assembly ahead of time with runtime-async/security bindings and link the entry
to the released shared native JIT host. The process executes precompiled code; it
does not JIT the guest. This is labeled **AOT (shared host)**, not a standalone SQL
deployment qualification. Its build commands and linked host hash are retained.

The initial standalone-profile attempt and an incomplete shared binding set failed
before timing; neither is a performance result. The complete binding set is used
for the recorded database run. JSON's original controller is preserved in
`controller-json-20261013.py` because that run predates the database-specific build
path; its SHA-256 matches the JSON report. Use `benchmark.py` for new runs.

## Composed workload runs — 2026-10-13

Each case below completed 576 validated responses with zero errors. The same
interactive-host limitations and five-round protocol apply. No background builds
were run during measurement.

| Case | Mode | Median requests/s | Round range | Median startup (s) |
| --- | --- | ---: | ---: | ---: |
| Object JSON | interpreter | 0.93 | 0.92–0.94 | 2.432 |
| Object JSON | jit | 33.36 | 30.77–33.77 | 6.095 |
| Object JSON | aot | 34.76 | 17.71–35.32 | 0.013 |
| PostgreSQL + object JSON | interpreter | 0.55 | 0.55–0.56 | 3.255 |
| PostgreSQL + object JSON | jit | 15.65 | 13.21–15.82 | 8.241 |
| PostgreSQL + object JSON | AOT (shared host) | 16.06 | 15.83–16.09 | 0.083 |

[JSON raw report](json-results-20261013.json) · [Database raw report](database-results-20261013.json).
PostgreSQL 18.6, ARM64 Docker image digest recorded in the report, 2 CPUs/512 MiB,
one reused connection per process and one parameterized row per request. No table
or persistent dataset. The temporary benchmark container was stopped after the run.
