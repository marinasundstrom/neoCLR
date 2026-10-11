# SQL databases

Development work adds SQL-first data access under `System.Data.Sql`, with an optional
`System.Data.PostgreSql` provider. Databases open owned connections; statements carry
positional parameters; results contain buffered rows that survive connection disposal.
A small mapper copies columns to scalar properties or calls an explicit row mapper.
There is no SQL generation or change tracking.

The first PostgreSQL binding is for local development: IPv4 loopback, trust
authentication, UTF-8 text results and bounded result sizes. TLS, password authentication,
remote servers, pooling and streaming are not implemented. This is unreleased work.

Transactions have awaited commit/rollback. Disposing an active transaction closes its
connection and lets PostgreSQL roll back on disconnect; explicitly rolling back permits
connection reuse. Server errors retain SQLSTATE. Values are bound separately from SQL
using PostgreSQL `$1` placeholders, with no SQL interpolation.

See the [SQL API reference](/docs/sql.html) for all member contracts and limits, and the
[compiled consumer](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/sql-postgresql/Main.rvn)
for property mapping, explicit maps, NULL values, transactions and error recovery.
