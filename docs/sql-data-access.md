# SQL connections and PostgreSQL — development

Author direction, 2026-10-12: implement a small SQL provider, PostgreSQL binding and
Dapper-like mapper, using SQL domain concepts rather than reproducing ADO.NET. The
author subsequently prefers submodules under `System.Data.*`. Common contracts live
in `System.Data.Sql` (System.Data.dll); the optional binding lives in
`System.Data.PostgreSql` (System.Data.PostgreSql.dll). This bounded implementation
supersedes the SQLite-first sequencing in the older [proposal](proposals/sql-data-access.md),
without selecting the rest of that proposal or permanently reordering the roadmap.

## Common provider direction

Author clarification (2026-10-12): the SQL interface must fit most SQL-like engines
and allow engine-specific extensions. SqlDatabase/SqlConnection/SqlStatement and
buffered results are the shared contract; no PostgreSQL protocol, placeholder scanner
or server quota belongs in the mapper. Providers may add derived interfaces or
concrete types for optional capabilities. No universal capability registry or opaque
provider-value escape hatch is introduced without a concrete consumer. The current
value subset, buffered rows and transaction policy remain gaps to expand where
needed, not proof of production compatibility with every SQL engine. No second
engine has been qualified. As in .NET's DbConnection/provider split, shared callers
use common operations while provider extensions remain explicit. Unlike ADO.NET,
this experimental API retains immutable statements, Task/Result operations and
owned buffered results; the narrower scope trades compatibility for a smaller surface.

## Implemented model

A `SqlDatabase` opens an owned `SqlConnection`. A `SqlStatement` snapshots its named or positional
`SqlValue` parameters. `Execute` returns a buffered `SqlResult`; `SqlRow` snapshots
survive connection disposal. Values never become SQL source. PostgreSQL uses `$1`, `$2`,
etc.; callers use SQL casts when the server cannot infer a parameter's type. Identifiers
and SQL fragments are not parameters. A statement contains one server statement; the
extended protocol rejects multiple statements rather than rewriting SQL.

`SqlMapper.Map<T>(result)` maps exact column names to writable scalar properties on a
public parameterless reference class. This is a small SQL-first object mapper, not
Dapper API compatibility. `SqlMapper.Map<T>(result, row => ...)` supports explicit constructors,
optional fields and domain conversions. Required NULL, missing/duplicate column names,
unsupported property types and conversion failures are mapping errors. Extra columns
are ignored. There is no SQL generation, relation loading, identity map or change tracking.

Author clarification (2026-10-12): the mapper should accept a connection and command,
with a string/params convenience form, without requiring consumers to construct SQL
primitives. The author selected both plain positional values and named parameter
objects. `SqlMapper.Query<T>` now composes execution and mapping; its string overloads
convert scalar values or a single named property object internally. The caller owns
the connection and transaction. See the [member contract](../api-docs/sql.md) for the
bounded types, lexical rules and retained-reflection requirements.

[Dapper's query API](https://github.com/DapperLib/Dapper) accepts an SQL string and a
parameter object; this is the ergonomic baseline (primary source reviewed 2026-10-12).
neoCLR keeps its existing SqlConnection contract and uses Query without an Async suffix
under the author-directed async-first policy. Positional params additionally avoid a
parameter class for simple calls. The cost is overload dispatch: one nonscalar reference
object means named properties, while scalar arguments mean positional values. Explicit
SqlStatement overloads preserve control over already-bound requests. No ADO.NET or
Dapper binary dependency is introduced.

Named binding follows PostgreSQL 18 [lexical structure](https://www.postgresql.org/docs/18/sql-syntax-lexical.html)
(reviewed 2026-10-12), including nested comments and dollar quotes. Translation belongs to the PostgreSQL provider. The common mapper never parses
SQL: it reads an ordinary parameter object into named values and passes the original
text to SqlConnection. Each provider owns its placeholder syntax and native binding. Ordinary backslash-containing strings are rejected during
named rewriting to avoid assuming the server's standard_conforming_strings setting;
explicit E strings express escaping. All readable parameter properties are evaluated once before execution, even if
unreferenced by SQL; this snapshot separates reflection from dialect handling.
Reflection and scanning add allocations and CPU work; no cached-plan or throughput claim is made by this API slice.

`SqlConnection.Begin()` produces a `SqlTransaction`. Commit and rollback are awaited
operations. An aborted transaction cannot report a successful commit; it requires
rollback. Completed/stale transaction handles reject operations. Disposing an active
transaction closes its connection, so PostgreSQL rolls back on disconnect; the connection
cannot then be reused. This intentionally simple ownership rule avoids unobserved async
rollback in synchronous Dispose. Explicit rollback keeps the connection reusable. Connection
disposal is idempotent. Transaction handles require the transaction methods; manual
transaction-control SQL (including AND CHAIN and savepoints) is outside this initial
handle-lifetime contract. The executable consumer uses `use` for connection cleanup with Raven
native revision `c1a938ccd`, following the adjacent thread's compiler fix. Explicit
Dispose calls remain where the fixture tests early close and idempotency.

## Implementation placement review — 2026-10-12

The author asks whether foreign calls should replace runtime implementations,
identifying PostgreSQL as a possible candidate if it uses native code. Source review
finds the current provider in `runtime/System.Data.PostgreSql/src/System.Data.PostgreSql/System/Data/PostgreSql`:
`PostgresConnection` implements protocol framing and SQL behavior in Raven and
awaits shared Socket operations. It declares no extern/InternalCall methods and
has no libpq dependency. There is no database-specific runtime service to extract.

**Recommendation:** retain this optional managed provider for the preview. Keep SQL
protocol, parameter binding and mapping outside the runtime. Runtime ownership is
justified for shared socket readiness, task suspension/resumption and managed lifetime;
these are execution services used by many libraries, not database policy.

The .NET baseline is a library provider exposing async connection/command operations,
as illustrated by [Npgsql's documentation](https://www.npgsql.org/doc/index.html)
(reviewed 2026-10-12). neoCLR's smaller SqlConnection interface intentionally differs
from ADO.NET, as documented above; it does not require a different runtime boundary.

A future **optional libpq provider through FFI** is a reasonable alternative when its
broader authentication/protocol support justifies another dependency. Prefer that
placement over adding PostgreSQL InternalCalls. Its ABI needs owned opaque connection
and result handles, explicit release, copied error/text data, and bounded buffer
marshalling. No borrowed result pointer may outlive its PGresult or cross an await
without an ownership rule. A generic readiness integration must avoid conflicting
ownership of libpq's socket and preserve deadlines/cancellation during shutdown.

PostgreSQL 18's [asynchronous libpq contract](https://www.postgresql.org/docs/18/libpq-async.html)
(reviewed 2026-10-12) requires input consumption and busy checks before retrieving
results without blocking. Calling a synchronous query function behind a Task would
not provide cooperative I/O. Library discovery, ABI versions, native notices and
macOS/Windows distribution would also become provider responsibilities. Current
[FFI execution-mode gaps](native-interop.md) must be closed and tested before adopting
this route for a JIT/AOT preview; they do not justify permanent database runtime APIs.

The managed implementation avoids another native dependency and exercises common
runtime contracts, but leaves us responsible for protocol/authentication correctness.
Libpq could reduce that implementation burden at the cost of marshalling, packaging
and async integration. Neither option has demonstrated a performance advantage here.
Before changing providers, compare the same parameter/NULL/error/transaction cases,
fragmented I/O, timeout/cancellation, disposal/GC and concurrent HTTP progress in all
three execution modes on both hosts. Benchmark matched workloads separately.
This review recommends placement; it does not implement libpq or expand the current
local trust-authentication provider's supported scope.

## PostgreSQL binding and bounds

The binding implements protocol 3.0 over existing asynchronous sockets. It sends
Startup with UTF8, then Parse / Bind / Describe portal / Execute / Sync for every
statement. Results are text format; NULL is the wire's -1 length, not empty text.
The unnamed statement/portal avoids a prepared-statement cache. There are no SQL
interpolation, psql subprocess, libpq dependency or database-specific runtime services.

The first binding deliberately connects only to `127.0.0.1`, using an explicitly
configured database, user, port and 1–60000 ms timeout. It accepts trust authentication
and rejects password/SASL authentication. **TLS, SCRAM/password authentication and remote
servers remain implementation gaps. This is a local development provider.** No database
installation or server configuration is changed by ordinary Open calls.

Each Open/Execute has one monotonic deadline shared by all network reads/writes.
A bounded 4096-byte receive buffer coalesces protocol fragments; buffered messages
are parsed synchronously and only refills await I/O. Short reads/writes are completed; EOF, malformed messages, unsupported protocol modes,
encoding changes and limits retire the connection. Server SQL errors retain SQLSTATE and
message, drain through ReadyForQuery and leave the connection reusable when the server
permits. No automatic retry is safe to infer from a transport failure: a statement
may already have committed. Overlapping operations are rejected (single task-queue
ownership, not a promise of cross-thread synchronization).

Initial bounds: 64 parameters, 64 columns, 1024 buffered rows, 65536-byte request and
message bodies, and a 1 MiB response budget including protocol lengths. These are
implementation quotas, not new SQL semantics. COPY, streaming, notifications, pooling,
binary values, decimal/date codecs, cancellation requests and provider-specific type
metadata remain gaps. Text access preserves other PostgreSQL text values; automatic
mapping supports string, Int32 and Boolean only. No performance claim is made.

## Comparison and decision evidence

Primary sources read 2026-10-12; these describe shipped APIs/protocols unless noted:

- [.NET 10 DbConnection](https://learn.microsoft.com/dotnet/api/system.data.common.dbconnection?view=net-10.0)
  and [Npgsql basic usage](https://www.npgsql.org/doc/basic-usage.html): configured data
  sources, owned connections, transactions, parameters and async operations already
  exist. ADO.NET is not inherently deficient at those concepts. The deliberate
  divergence is the author's smaller SQL-domain surface: no mutable command/reader
  hierarchy, provider factories, DataSet or disconnected update machinery. Cost:
  no direct ADO.NET consumer/provider compatibility and a much narrower feature set.
- [Dapper](https://github.com/DapperLib/Dapper): SQL-first execution and buffered typed
  rows motivate the library layer. Dapper itself uses ADO.NET. Our initial property
  mapper uses existing reflection rather than generated IL; no CLR/compiler mechanism
  is added. Its exact-name matching and scalar-only mapping are narrower. Explicit
  maps work with constructors and AOT-visible calls at the cost of more application code.
- [Dapper issue 722](https://github.com/DapperLib/Dapper/issues/722) is an open design
  discussion about attribute mapping, not an adopted universal contract. It illustrates
  the distinction between result-column mapping and SQL/table naming. This slice uses
  SQL aliases and explicit maps; it makes no claim to resolve that discussion.
- [Go database/sql](https://pkg.go.dev/database/sql) separates database handles,
  connections/transactions and explicit row scanning. Its driver interface and blocking
  goroutine semantics do not directly transfer to Raven; explicit mapping does.
- [PostgreSQL 18 message formats](https://www.postgresql.org/docs/18/protocol-message-formats.html)
  and [flow](https://www.postgresql.org/docs/18/protocol-flow.html) define separate
  parameter payloads, error recovery through Sync/ReadyForQuery and transaction status.
  [libpq](https://www.postgresql.org/docs/18/libpq-exec.html) was considered: it provides
  mature authentication/TLS but current neoCLR FFI lacks managed string marshalling
  and native JIT admission for this surface. A small Raven protocol binding reuses
  supported async sockets and avoids adding host-specific SQL services. The cost is
  protocol maintenance and explicitly incomplete authentication/features. Revisit a
  libpq adapter or a fuller native protocol implementation before remote-server use.

Buffering simplifies lifetime and keeps mapping independent of I/O; its memory/copy
cost and finite quotas make it inappropriate for large result sets. Future streaming
must define backpressure, disposal, errors and transaction ownership first. No benchmark
is needed for this functional slice, and none is used to imply a speed advantage.

## Compilation and validation

See [the executable fixture](experiments/sql-postgresql/README.md) and the
[member reference](../api-docs/sql.md). Production runtime contracts, ordinary .NET
behavior and compiler branch separation are unchanged. The optional provider uses the
existing opt-in retained-async compiler helper; this is not a new runtime/compiler ABI.
Propagation inside retained async bodies and value-type captures had compiler gaps
in the original tested bundle. Generic async and synchronous use were subsequently
fixed on the native compiler line; this slice adopts use without expanding its API.
The library keeps materialization synchronous after awaited execution, uses Result propagation,
and owns each NetworkDeadline in a reference wrapper. Dynamic native buffers use the
existing explicitly admitted CheckedStorage intrinsic. These are implementation
workarounds to revisit, not permanent SQL API semantics.

The separately compiled provider exposed a native-linker gap: after two assemblies'
identical InternalCall declarations were verified in their original scopes, lowering
them to ordinary host-bound placeholder bodies put both names in one private module.
The native linker now gives already-admitted free service bodies distinct private
names, just as it already did for services in the bundled System seed. Calls remain
bound by exact definition ID; source identity remains in the report, and original
access/signature/registry checks still run before relocation. No source metadata or
native host ABI changes. The [focused regression](../tools/aot-poc/tests/sql_linking.rs)
requires explicit runtime-async admission and rejects the unadmitted version.
