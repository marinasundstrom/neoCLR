---
uid: N:System.Data.Sql
---
## SQL connections and mapping

SQL-domain contracts: configured databases open connections, statements bind
positional values, and execution returns owned buffered rows. SqlMapper maps rows to
scalar properties or through explicit callbacks. Transactions have explicit completion
and lifetime. This is not ADO.NET or Dapper API compatibility.

See [SQL member reference](/docs/sql.html) for signatures, errors, ownership and limits.
The optional System.Data.PostgreSql binding currently supports local trust-authenticated
PostgreSQL only; TLS, password authentication and remote access remain unimplemented.
