# SQL databases

Write SQL, pass plain values, and map rows to ordinary Raven objects.
`System.Data.Sql` provides common connections, statements and the `SqlMapper`;
`System.Data.PostgreSql` supplies the optional PostgreSQL provider.

The provider is currently for local development: IPv4 loopback and trust
authentication. Password authentication, TLS, remote servers, pooling and streaming
are not implemented. Use a matching development bundle containing the provider.

## Query objects with PostgreSQL

This model matches the returned column names:

```raven
{{SQL_PERSON_MODEL}}
```

Pass positional values alongside the SQL. `Query<Person>` returns a task containing
a Result; `?` propagates a `SqlError` to the enclosing async function.

```raven
{{SQL_POSITIONAL_PEOPLE}}
```

Values are bound separately from SQL text, including quotes and Unicode. The mapper
copies columns to scalar properties; it does not generate SQL or track changes.

## Use named parameters

A plain object can supply the values for `@name` placeholders:

```raven
{{SQL_NAMED_PEOPLE}}
```

PostgreSQL owns the placeholder translation. The common mapper passes SQL to the
selected provider without imposing PostgreSQL syntax on other database engines.

[Run the PostgreSQL people example →](/cases/postgresql/)
The walkthrough includes connection setup, a disposable Docker database and the
complete downloadable source.

## Connections, errors and transactions

Open a connection with `await database.Open()` and own it with `use`. Query results
are buffered snapshots and remain readable after the connection closes. Failures
are `SqlError` values; server errors retain SQLSTATE.

Transactions support awaited commit and rollback. Explicit rollback permits
connection reuse. Disposing an active transaction closes the connection, allowing
PostgreSQL to roll it back on disconnect.

[SQL API and mapping rules](/docs/sql.html) ·
[PostgresDatabase](xref:System.Data.PostgreSql.PostgresDatabase) ·
[SqlMapper](xref:System.Data.Sql.SqlMapper)
