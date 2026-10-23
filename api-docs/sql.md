# SQL data access

Development API, not part of Preview 13. Common types are in `System.Data.Sql`
(System.Data.dll); `System.Data.PostgreSql.PostgresDatabase` is in the optional
System.Data.PostgreSql.dll. These are SQL-domain contracts, not ADO.NET interfaces.

## Database, connection and transaction

| Type/member | Contract |
| --- | --- |
| `SqlDatabase.Open() -> Task<Result<SqlConnection, SqlError>>` | Opens one owned connection; caller disposes it. |
| `SqlConnection.Execute(SqlStatement) -> Task<Result<SqlResult, SqlError>>` | Executes one statement with bound values; buffered, owned result. Single operation at a time. |
| `SqlConnection.Begin() -> Task<Result<SqlTransaction, SqlError>>` | Starts a transaction; rejects nesting or an unavailable connection. |
| `SqlConnection.Dispose()` | Idempotently closes the connection; active server work/transactions are abandoned. |
| `SqlTransaction.Commit()/Rollback() -> Task<Result<unit, SqlError>>` | Await completion. A failed transaction cannot commit. Repeated/stale operations fail. |
| `SqlTransaction.Dispose()` | If still active, closes its connection to force rollback on disconnect. Await Rollback to reuse the connection. |
| `PostgresDatabase(database: string, user: string, port: int, timeoutMilliseconds: int)` | Configures local trust-only access at 127.0.0.1; port 1–65535, timeout 1–60000 ms. Validation happens in Open, with InvalidInput before connecting. |
| `PostgresDatabase.Open()` | Implements SqlDatabase.Open; accepts PostgreSQL AuthenticationOk and UTF8 only. Unsupported authentication is an error and closes the socket. |

No synchronous network methods, pooling, cancellation token, password or TLS option is
implemented. Do not use this local development binding as a remote database connector.
Use the transaction methods for transaction ownership; manual transaction-control SQL,
AND CHAIN and savepoints are outside the initial handle-lifetime contract.
Each opening or statement has a shared deadline; no automatic retry is performed.
A transport error can follow a committed statement. No exactly-once guarantee exists.

## Statements and parameter values

`SqlStatement(text: string)` binds no parameters.
`SqlStatement(text: string, values: Sequence<SqlValue>)` copies the sequence, preserving
order. `SqlStatement(text, values: Sequence<KeyValuePair<string, SqlValue>>)` copies
named values without interpreting the SQL. `HasNamedParameters: bool` distinguishes
the constructors even for empty snapshots; `ParameterName(index) -> Optional<string>`
returns Some(name) or None for positional values. Invalid indexes fault; inactive
SqlValue payloads in a named pair fault. Names are preserved exactly; each provider
validates its naming rules. `Text: string`, `ParameterCount: int` and `Parameter(index: int) -> SqlValue`
expose the snapshot. Parameter indexes are zero-based; an invalid index is a programmer
fault. PostgreSQL SQL uses one-based `$1` placeholders. This lower-level SQL is passed unchanged; use
explicit casts when inference is ambiguous. Empty SQL, NUL text and excessive input
are rejected before network output. A PostgreSQL statement cannot contain a batch.

`SqlValue` is a standard Raven union with `Null`, `Text(value: string)`,
`Integer(value: long)` and `Boolean(value: bool)`. Null is distinct from empty text.
The PostgreSQL provider uses UTF-8 text parameter format; other types can be passed as Text
with an explicit SQL cast. Parameter values cannot stand for identifiers or SQL syntax.

## Rows, results and mapping

`SqlRow(columns: Sequence<string>, cells: Sequence<Optional<string>>)` copies both
sequences. Mismatched lengths are a programmer fault. `Count: int` is the column count.
`Column(index: int) -> Result<string, SqlError>` returns an ordinal column name.
`Text(index: int)` and `Text(name: string)` return `Result<Optional<string>, SqlError>`.
None means SQL NULL; Some("") is empty text. Name access is ordinal, case-sensitive;
missing names, duplicate names and bad indexes are Mapping errors.

`RequiredText(name) -> Result<string, SqlError>` rejects NULL.
`Int32(name) -> Result<int, SqlError>`, `Int64(name) -> Result<long, SqlError>` and
`Boolean(name) -> Result<bool, SqlError>` parse required text. Integer parsing uses the
platform integer parser and checks range. Boolean accepts only `t`, `f`, `true`, `false`.
These conversions inspect text, not database type identifiers. Result type metadata
and provider-specific codecs remain gaps.

`SqlResult(commandTag: string, values: Sequence<SqlRow>)` copies the row sequence.
`Count: int`, `CommandTag: string` and `Row(index: int) -> SqlRow` expose results.
An invalid row index is a programmer fault. PostgreSQL CommandTag preserves such tags
as `INSERT 0 1`, `SELECT 2`, `COMMIT`; no inferred affected-row count is fabricated for
DDL. Rows remain usable after connection disposal.

`SqlMapper.Map<T>(result: SqlResult) -> Result<Sequence<T>, SqlError>` maps exact
column names to public writable, non-indexed instance properties. T must be a
nongeneric reference class with a public parameterless constructor; supported
property types are string, int and bool. Extra result columns are ignored.
Required missing/ambiguous/NULL values, unsupported properties, construction or setter
failures return Mapping. Empty results still validate property shape but perform no
construction. Constructors/setters may have application side effects; mapping errors
do not roll back those effects or executed SQL. Native JIT/AOT callers must retain
reflection construction/setters through an explicit reflection manifest.

`SqlMapper.Map<T>(result, map: (SqlRow) -> Result<T, SqlError>)` applies an explicit
mapper in row order. It supports optional values, constructors and domain validation
without reflection. Both overloads stop on the first error and return no partial
collection.

`SqlMapper.Query<T>(connection: SqlConnection, statement: SqlStatement)` and
`Query<T>(connection, statement, map: (SqlRow) -> Result<T, SqlError>)` return
`Task<Result<Sequence<T>, SqlError>>`. They execute exactly once, await the buffered
result and apply the corresponding Map overload. Execution errors pass through
unchanged. They borrow the connection: no opening, closing, retry or transaction is
implicit. Mapping errors do not undo executed SQL.

For ordinary application code, `Query<T>(connection, text: string, params parameters: Object?[])`
accepts plain string, int, long, bool and null values in positional order. The
provider defines placeholder syntax (`$1`, `$2`, etc. for PostgreSQL). No SqlValue or SqlStatement construction is required. Unsupported values
return Mapping before executing. Zero parameters is valid. SQL casts may still be
needed for PostgreSQL type inference.

Passing one reference object instead of scalar values selects named binding: public
readable non-indexed instance properties supply `@name` parameters. Names are ASCII
letters/underscore followed by letters/digits/underscore and match property names
exactly. The mapper reads every public readable instance property once, including
unused properties, before execution; avoid side-effecting getters. Indexed/ambiguous
properties and unsupported values return Mapping. PostgreSQL resolves referenced
names from this snapshot and reports missing names as Mapping. Arrays and generic
parameter objects are outside this first reflection contract. Retain parameter getters
as well as result constructors/setters in native reflection manifests.

The PostgreSQL provider (not the common mapper) translates named placeholders to `$1`, `$2`, etc., preserving quoted identifiers,
single/dollar-quoted text and line/nested block comments. Do not mix `$n` and `@name`
parameters. Unterminated quotes/comments and ordinary string literals containing
backslashes return InvalidInput; use explicit PostgreSQL E strings for escapes.
Other providers receive the original text and named/positional values, and define
their own placeholder syntax and supported bindings. Engine-specific interfaces or
types may extend SqlConnection without changing the common mapper. This lexical
policy is PostgreSQL-specific. Values remain separate protocol parameters, never interpolated SQL.

`Query<T>(connection, text: string, map: (SqlRow) -> Result<T, SqlError>, params parameters: Object?[])`
provides the same plain-value/named binding with explicit result conversion. Its
callback precedes the variadic arguments. The statement overloads remain available
for callers who already have prepared statement values.

## Errors and limits

`SqlError` cases: `InvalidInput(message)`, `InvalidState(message)`, `Transport(message)`,
`Protocol(message)`, `Unsupported(message)`, `LimitExceeded(message)`,
`Server(sqlState, message)` and `Mapping(message)`. All payloads are strings.
Server errors preserve SQLSTATE and are drained through ReadyForQuery. Other wire/I/O
failures close the connection. Error messages can contain server-provided SQL details;
applications decide what to log or disclose.

The PostgreSQL development provider accepts at most 64 parameters/columns, 1024 rows,
65536 bytes per request/message body and a 1 MiB response budget. COPY, notifications,
multiple result sets and binary result formats are unsupported. Limits are explicit
implementation bounds. The standard union-generated case constructors/properties and
pattern forms follow Raven's normal union contract.

See [design and comparison](https://github.com/marinasundstrom/neoCLR/blob/main/docs/sql-data-access.md) and the
[compiled executable](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/sql-postgresql/Main.rvn) for the tested usage.
