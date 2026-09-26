# neoCLR SQL Data Access

## Status

Proposal

## Summary

neoCLR should provide a small, provider-oriented abstraction for accessing SQL databases without reproducing the full ADO.NET architecture.

The platform should define a common SQL programming model covering the operations that are broadly shared between relational database systems:

- Opening and managing connections
- Executing SQL statements
- Binding parameters
- Reading result sets
- Managing transactions
- Representing common SQL values and metadata

Actual database implementations should be supplied separately by providers such as SQLite, PostgreSQL, MySQL, and SQL Server.

Providers may expose functionality beyond the common abstraction. The platform API therefore represents the portable SQL capability rather than attempting to model every feature supported by every database.

## Goals

The SQL data-access model should:

- Provide a small and understandable abstraction over SQL databases.
- Avoid reproducing the historical structure and complexity of ADO.NET.
- Allow applications and libraries to operate against different SQL providers.
- Keep provider-specific functionality accessible.
- Work naturally with neoCLR's provider and capability-oriented platform design.
- Support both synchronous and asynchronous implementations where appropriate.
- Make SQLite a practical first implementation without designing the abstraction specifically around SQLite.
- Allow the API to evolve independently from individual database providers.

The abstraction should represent SQL database access rather than databases in general.

Document databases, key/value stores, graph databases, and other storage technologies should not be forced into a SQL-shaped abstraction.

## Architecture

The architecture consists of two layers:

```text
Application
    │
    ▼
System.Data.Sql
    │
    ├── SqlDataSource
    ├── SqlConnection
    ├── SqlCommand
    ├── SqlParameter
    ├── SqlReader
    ├── SqlRow
    └── SqlTransaction
    │
    ▼
Provider
    │
    ├── SQLite
    ├── PostgreSQL
    ├── MySQL
    └── SQL Server
    │
    ▼
Native / external database implementation
```

`System.Data.Sql` defines the common programming model.

Providers implement that model and bridge it to the underlying database technology.

The underlying implementation may be native code, a system library, a network protocol implementation, or another external component. These implementation details should not leak into the common API unless they represent meaningful SQL capabilities.

## Data Sources

A data source represents the configured means of connecting to a SQL database.

Conceptually:

```raven
interface SqlDataSource
{
    OpenConnection() -> Result<SqlConnection, Error>
}
```

A data source may encapsulate:

- Connection configuration
- Credentials
- Provider configuration
- Connection pooling
- Provider-specific initialization
- Other resources shared between connections

This separates database configuration and connection creation from the connection itself.

For example:

```raven
dataSource = SqliteDataSource("application.db")

use connection = dataSource.OpenConnection()?
```

A networked provider could instead expose:

```raven
dataSource = PostgresDataSource {
    Host = "database.example.com"
    Database = "application"
    User = "app"
}

use connection = dataSource.OpenConnection()?
```

Both can subsequently be consumed through the common `SqlDataSource` abstraction.

## Connections

`SqlConnection` represents an active connection or logical session with a SQL database.

Its common responsibilities should remain relatively small.

For example:

```raven
interface SqlConnection
{
    CreateCommand(String sql) -> SqlCommand

    BeginTransaction() -> Result<SqlTransaction, Error>
}
```

Connection establishment belongs primarily to `SqlDataSource`.

This distinction leaves providers free to implement pooling or other connection-management strategies without exposing them as properties of every connection.

## Commands

`SqlCommand` represents a SQL statement together with its bound parameters.

For example:

```raven
command = connection.CreateCommand("""
    SELECT id, name
    FROM users
    WHERE active = @active
""")

command.Parameters.Add("active", true)
```

A command can expose operations such as:

```raven
command.Execute()
command.ExecuteScalar<T>()
command.ExecuteReader()
```

The exact result types should follow neoCLR's normal error model rather than introducing a database-specific exception model.

Conceptually:

```raven
Execute() -> Result<int, Error>

ExecuteScalar<T>() -> Result<Option<T>, Error>

ExecuteReader() -> Result<SqlReader, Error>
```

Asynchronous variants can follow the platform's task model where the provider or underlying database operation can benefit from asynchronous execution.

## Parameters

Parameters should be first-class values rather than requiring applications to manually construct SQL literals.

For example:

```raven
command.Parameters.Add("name", name)
command.Parameters.Add("age", age)
```

The common API should define parameter binding semantics while allowing providers to expose additional parameter metadata when required.

Providers may need to support concepts such as:

- Database-specific types
- Parameter directions
- Explicit sizes
- Precision and scale
- Native type identifiers

These should not necessarily become requirements of the base abstraction merely because one provider supports them.

## Reading Results

Query results should be exposed through a forward-oriented reader abstraction.

For example:

```raven
use reader = command.ExecuteReader()?

for row in reader {
    id = row.Get<int>("id")?
    name = row.Get<String>("name")?
}
```

Rows should support access by both column name and position where practical:

```raven
row.Get<String>("name")
row.Get<String>(1)
```

The API should distinguish SQL `NULL` from ordinary values.

For example:

```raven
name = row.Get<Option<String>>("name")?
```

or through another explicit nullable/optional retrieval mechanism.

SQL nullability should not require nullable value types in the runtime. It can instead map naturally onto `Option<T>`.

## Transactions

Transactions should be represented explicitly:

```raven
use transaction = connection.BeginTransaction()?

// execute operations

transaction.Commit()?
```

Disposal without a successful commit may roll the transaction back where supported.

More advanced transaction functionality may remain provider-specific unless a sufficiently portable abstraction emerges.

This includes features such as:

- Savepoints
- Isolation modes
- Deferred transactions
- Distributed transactions
- Provider-specific locking behavior

The common abstraction should grow from demonstrated cross-provider requirements rather than attempting to anticipate every transaction model.

## Provider-Specific APIs

The common SQL abstraction is deliberately not intended to hide the underlying database.

A provider can expose a richer concrete type:

```raven
connection = SqliteDataSource("application.db")
    .OpenConnection()?

connection.EnableExtensions()
connection.SetBusyTimeout(...)
connection.CreateFunction(...)
```

The same connection may still satisfy the common abstraction:

```raven
SqlConnection connection = sqliteConnection
```

Portable libraries can therefore depend on `SqlConnection`, while applications that intentionally depend on SQLite can use SQLite-specific functionality directly.

This avoids reducing every provider to the lowest common denominator.

## Provider Packages

Providers should live outside the core SQL abstraction.

Possible packages include:

```text
System.Data.Sql

NeoCLR.Data.Sqlite
NeoCLR.Data.Postgres
NeoCLR.Data.MySql
NeoCLR.Data.SqlServer
```

The exact package naming convention can be decided separately.

`System.Data.Sql` contains contracts and common SQL concepts.

Provider packages contain concrete implementations and provider-specific extensions.

## SQLite as the Initial Provider

SQLite is a suitable first implementation because its native programming model exposes the essential SQL operations with relatively little additional machinery:

```text
open
prepare
bind
step
read
finalize
close
```

This provides enough functionality to validate the neoCLR abstraction without first implementing a network protocol, authentication system, or connection pool.

However, the abstraction should not simply expose SQLite's native API under different names.

After the initial SQLite implementation, the design should be validated against at least one client/server database such as PostgreSQL.

That exercise should reveal assumptions that are SQLite-specific.

## Relationship to ADO.NET

This API is not intended to reproduce ADO.NET.

In particular, neoCLR does not initially need equivalents for the entire historical ADO.NET object model, including concepts such as:

```text
DbProviderFactory
DbDataAdapter
DataSet
DataTable
DataRelation
DataView
CommandBuilder
```

Some of these abstractions solve problems that are either historical, specific to disconnected data models, or consequences of ADO.NET's provider architecture.

If neoCLR eventually needs equivalent capabilities, they can be introduced independently.

The initial API should instead model the direct interaction:

```text
DataSource
    ↓
Connection
    ↓
Command
    ↓
Reader / Result
```

with transactions alongside commands.

## Relationship to Higher-Level Data APIs

`System.Data.Sql` should remain a relatively low-level SQL interface.

Higher-level facilities can be built separately:

```text
ORM
Query API
Object mapping
Migration framework
Schema tooling
Repository abstractions
```

These systems can depend on `System.Data.Sql` without becoming part of the fundamental database provider contract.

This separation also allows multiple higher-level data models to coexist.

## Design Principle

The central principle is:

> Standardize the portable SQL operation, not every feature of every SQL database.

The abstraction should be sufficiently capable that ordinary SQL-oriented libraries can operate without knowing their concrete provider.

At the same time, it should not prevent an application from taking full advantage of PostgreSQL, SQLite, SQL Server, or another database when portability is not the objective.

The provider abstraction establishes a common foundation rather than attempting to erase differences between database systems.

## Open Design Questions

### Naming

It remains to be decided whether the common types should use the `Sql` prefix:

```text
SqlDataSource
SqlConnection
SqlCommand
SqlReader
SqlTransaction
```

or whether the `System.Data.Sql` namespace provides sufficient context to permit shorter names:

```text
DataSource
Connection
Command
Reader
Transaction
```

The explicit `Sql` names make the scope of the abstraction clearer and avoid suggesting that these concepts apply universally to every form of data storage.

### Reader and Row Model

The relationship between `SqlReader`, iteration, and `SqlRow` needs further exploration.

A reader could itself behave as a sequence of rows:

```raven
for row in reader {
    ...
}
```

This would provide a more natural Raven interface than directly exposing a traditional `Read()` state machine.

The underlying provider could still implement this efficiently using a cursor.

### SQL Types

The boundary between ordinary neoCLR types and SQL-specific types needs to be defined.

Common mappings such as strings, integers, floating-point values, binary values, dates, times, and UUIDs should ideally use ordinary platform types.

Database-specific types should remain available through provider extensions.

### Prepared Commands

It needs to be determined whether preparation is:

- An explicit common operation,
- An implementation detail of `SqlCommand`, or
- A provider capability.

SQLite naturally exposes explicit statement preparation, while other providers may cache or prepare commands differently.

The common abstraction should not expose preparation merely because SQLite requires it internally.

### Async Operations

Networked databases naturally benefit from asynchronous operations, while SQLite generally performs local synchronous calls.

The API should determine whether synchronous and asynchronous operations coexist on the same interfaces or whether asynchronous behavior emerges through separate capabilities.

This should align with the broader neoCLR task and asynchronous programming model.

### Error Representation

Database failures should integrate with the general neoCLR `Error` model rather than introducing an independent exception hierarchy.

A SQL provider may supply richer error information such as:

```text
Error
 └── SqlError
      └── provider-specific error
```

or equivalent interface/capability-based composition.

This would allow propagation through `Result<T, Error>` while preserving the concrete provider error and its additional context.

The design should therefore avoid reducing database failures to strings or generic status codes during propagation.

### Provider Discovery

It remains to be determined whether applications normally instantiate provider-specific data sources directly:

```raven
SqliteDataSource(...)
```

or whether neoCLR should eventually provide a provider-discovery/configuration mechanism.

Direct construction should be sufficient for the initial design. A provider registry or configuration mechanism should only be introduced when there is a demonstrated use case for selecting providers dynamically.