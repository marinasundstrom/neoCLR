# SQL/PostgreSQL qualification

Development slice, 2026-10-12. See [design and limits](../../sql-data-access.md) and
[public member reference](../../../api-docs/sql.md). This provider is local-only,
trust-authenticated and bounded; it does not implement TLS or password authentication.

`Main.rvn` is a public consumer compiled against separate Runtime, Data, Networking
and PostgreSql assemblies. It exercises positional values (including injection-shaped
Unicode text), NULL/empty text, property and explicit mapping, zero-column rows,
duplicate aliases, SQLSTATE errors and recovery, commit, rollback, failed transactions,
nested/stale handles, input rejection and disposal. Tables are temporary and belong to
its session. `Contracts.rvn` checks owned snapshots, strict access and conversion
bounds. `Failures.rvn` drives ten Python protocol peers: unsupported authentication,
malformed startup, opening deadline, oversized frame, COPY, invalid cell length,
invalid UTF-8, mismatched columns, excessive rows and fragmented successful messages.
Peers verify separate Parse/Bind frames, connection retirement after failures and
scoped `use` cleanup after the fragmented successful response.

## Current native package qualification (2026-10-26)

For a native-source-core bundle, `validate.py` now compiles each consumer through
an SDK project importing `NeoCLR.ClassLibrary.props`. This preserves the matching
native core, runtime contracts and runtime-async provider instead of selecting the
historical CLI bootstrap flags. The old recipe below remains historical evidence.
Pass the extracted bundle's compiler, `lib`, runtime, JIT compiler and JIT host,
plus the disposable local database port; `--aot` includes native execution.
The Preview 18 requalification covers live PostgreSQL in all three execution modes,
SQL contracts and all ten malformed/fragmented protocol peers. See the
[release record](../../validation/release-preparation-20261025/README.md).

## Initial protocol qualification

The following recipe records the original protocol milestone; the common-query
follow-through below uses a newer compiler and adds native AOT.

### Reproduce

Use a native-enabled Raven compiler with the retained-async adapter. The qualified
compiler revision is `c1a938ccd`; exact binaries and sources are hashed in the evidence.
The consumer uses `use` for session ownership following the adjacent compiler fix.
Explicit disposal remains in tests of early-close and idempotency behavior.

Build the optional provider together with the normal class libraries (paths shown are
local build inputs, not installed product defaults):

```sh
python3 scripts/build-native-class-library.py \
  --compiler /path/to/Raven/src/Raven.Compiler/bin/Debug/net10.0/rvnc.dll \
  --compiler-revision c1a938ccd \
  --core target/encoding-array-bundle/bootstrap/Core.dll \
  --bootstrap-directory target/encoding-array-bundle/bootstrap \
  --translator tools/metadata/NeoCLR.Metadata.Translate/bin/Release/net10.0/NeoCLR.Metadata.Translate.dll \
  --output target/sql-postgresql-libraries --with-postgresql
cargo build --manifest-path tools/aot-poc/Cargo.toml
```

The native linker must include the scoped-service naming fix in this change. Both the
provider and consumer declare the existing private runtime-async adapter; neither
requires a new host service. The narrow admission regression is:

```sh
cargo test --manifest-path tools/aot-poc/Cargo.toml --test sql_linking
```

Start a disposable server, binding its published port only to loopback:

```sh
docker run --detach --rm --name neoclr-sql-test \
  --publish 127.0.0.1::5432 --env POSTGRES_HOST_AUTH_METHOD=trust postgres:18
docker port neoclr-sql-test 5432/tcp
```

Pass that assigned port and a matching runtime/JIT helper bundle. The output directory
must not already exist. The script compiles consumers, retains mapper construction
and setters in an explicit reflection manifest, runs JIT and interpreter consumers,
and writes commands, results and SHA-256 fingerprints to `validation.json`.

```sh
python3 docs/experiments/sql-postgresql/validate.py \
  --compiler /path/to/Raven/src/Raven.Compiler/bin/Debug/net10.0/rvnc.dll \
  --libraries target/sql-postgresql-libraries \
  --runtime target/default-jit-prepared-v2/bundle/bin/neoclr \
  --jit-compiler tools/aot-poc/target/debug/neoclr-aot-poc \
  --jit-host target/default-jit-prepared-v2/bundle/bin/neoclr-jit-host.dylib \
  --postgres-port PORT --output target/sql-postgresql-validation
docker stop neoclr-sql-test
```

The fixture uses up to 60-second network deadlines to accommodate the interpreter.
The timeout peer deliberately uses 100 ms. `--skip-interpreter` is available for a
focused native rerun. This is functional qualification, not a throughput benchmark.
Windows and a separately emitted AOT SQL executable are not qualified by this fixture.

## Documentation qualification

`python3 scripts/build-api-docs.py --check` passes; the native snapshot archive checks
pass (`python3 -m unittest discover -s scripts -p test_native_api_snapshot.py`). The
native renderer includes the optional provider and all 22 selected public SQL types.
Both the SQL member guide and feature page have working current-page navigation.
The full website build reaches reference coverage but stops on the pre-existing
`System.Introspection.ModuleInfo.GetFunctions` missing member entry; full-site success
is not claimed. Four existing API guides missing sidebar entries were added alongside
the SQL entry so rendering could reach that coverage check.

## Recorded outcome (2026-10-12)

**Passed on macOS ARM64:** contracts in JIT; the real PostgreSQL consumer in JIT and
interpreter; all ten protocol/deadline cases in JIT. The fragmented success peer also
verifies connection closure through `use`. The native linker admission regression
passes (one test), and the API snapshot checks pass (two archive tests plus the
aggregate fingerprint check). No Windows or separate AOT SQL-executable result is
inferred from these checks.

[Validation evidence](validation.json) records the final `SqlConnection` consumers,
commands, outcomes, elapsed wall time including compilation, and exact input hashes.
[Build evidence](build.json) records selected source/compiler/artifact hashes and the
PostgreSQL 18.6 server image. Timings are qualification diagnostics, not benchmarks.

[The initial row-limit failure](initial-row-deadline.json) is retained: per-fragment
async work exhausted the operation deadline before the row limit. Socket buffering
alone did not resolve it. The final provider parses buffered messages synchronously,
awaits only buffer refills, and passes the unchanged 1025-row rejection case within
the shared deadline. The disposable test server was removed after qualification.

## Common query and provider boundary follow-through

The expanded Main consumer adds SqlMapper.Query with statements, plain positional
values and named parameter objects, including 64-bit values and a reflected Int64
getter. It checks repeated named parameters, quoted/dollar-quoted text and nested
comments, separate Unicode/injection-shaped values, missing/mixed parameters,
mapping failures and SQLSTATE recovery. The common mapper preserves SQL text;
PostgreSQL alone performs named-placeholder rewriting. Contracts also uses a
recording SqlConnection to verify unchanged `SELECT :id`/`SELECT ?`, snapshot
ownership, one execution and borrowed connection lifetime without a database.

Use Raven native compiler `7b48ce117` with the 1,024-local metadata writer and
native Int64 boxing/property adapters. `--queries-only --aot` runs the changed
contracts and real PostgreSQL consumers in interpreter/JIT, then emits an AOT
application linked against the same native host library. It skips unchanged hostile
wire peers; their initial evidence remains applicable. The AOT executable still
depends on that shared host and is not a self-contained deployment qualification.
Windows execution and remote/password/TLS SQL support remain unqualified.

**Qualified on macOS ARM64 (2026-10-12):** both contracts and live PostgreSQL
queries pass interpreter/JIT; the live query consumer also passes shared-host AOT.
[Query evidence](query-validation.json) records all ten successful build/execution
steps and unchanged input hashes; [environment](query-environment.json) records the
PostgreSQL 18.6 container and host. The compiler is frozen at Raven `7b48ce117`;
the later literal-admission guard at `af994c8d1` has its own compiler regressions
and does not change this consumer's literal lowering. Native Int64 reflection
get/set adapters additionally pass ten focused success/fault modes.

The API snapshot and complete website build now pass. This supersedes the earlier
website limitation above, without rewriting the historical SQL qualification.
