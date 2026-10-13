# PostgreSQL mapper benchmark — development

Two async process-isolated cases use the public mapper API and a reused connection:

- `postgres.positional-1`: one generated row, plain positional integer parameter.
- `postgres.named-32`: 32 generated rows, a named parameter object, reflected result objects.

Each operation executes a parameterized SELECT, buffers and materializes every row,
and verifies count, scalar properties and the sum of IDs. SQL is read-only and needs
no permanent schema. Connection setup and parameter-object construction precede READY;
all execution, parameter binding, mapping and checks run between BEGIN and END. Setup,
JIT compilation and connection opening are reported separately as process startup.
This is end-to-end latency/throughput including PostgreSQL, Docker networking and
result checks, not a measurement of mapper CPU in isolation. The two cases differ in
both binding and row count; do not interpret their difference as named-binding overhead.

Use a disposable, loopback-only trust-authenticated PostgreSQL server, as described in
[SQL qualification](../../docs/experiments/sql-postgresql/README.md). Do not use an
application database. Record the image ID, server version and host/container settings
with results. The server must remain running at the recorded port through all rounds.

First obtain a passing query qualification recipe with `--queries-only --aot`.
Then build and run using the existing [benchmark runner](../framework/README.md):

```sh
python3 benchmarks/postgresql/prepare.py \
  --recipe target/sql-query-validation/validation.json \
  --postgres-port PORT --output target/postgresql-benchmark
python3 benchmarks/framework/runner.py \
  --manifest target/postgresql-benchmark/manifest.json \
  --output target/postgresql-benchmark/results.json \
  --rounds 3 --warmups 2 --operations 8 --timeout 300
```

Preparation verifies qualification input hashes and fingerprints the worker, generated
port configuration, reflection roots and native artifacts. The runner verifies its
manifest inputs before/after all rounds and retains every sample. This is currently a
macOS host recipe; AOT emits the application ahead of time and links the same shared
native host used by JIT. It is not a self-contained deployment qualification. Windows and .NET/Go/JavaScript comparison programs remain work. The separate
[HTTP benchmark](../http-server/README.md) records the composed HTTP+JSON+SQL workload;
its results are not measurements of these mapper-only cases.
The worker is explicitly registered through the runner's marker protocol because
attribute discovery does not yet support Task-returning benchmark methods.

## Validation status — 2026-10-13

The worker compiles with the Preview 14 bundled Raven compiler and reference libraries;
the preparation script passes Python syntax checks. No mapper-only timing run or full
preparation run is claimed for this snapshot. Run the qualification recipe and both
cases before using results as performance evidence.

The worker uses separate bounds checks and awaits a selected query operation in a
local binding. Compound comparison parsing, await in a value-producing block, and
await inside compound assignment exposed compiler limitations during preparation;
these source forms avoid those limitations without changing the measured queries.
They are compiler gaps, not permanent Raven or neoCLR contracts.
