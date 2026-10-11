---
uid: N:System.Data.PostgreSql
---
## Local PostgreSQL binding

PostgresDatabase implements System.Data.Sql.SqlDatabase over PostgreSQL protocol 3.0.
This development binding connects only to 127.0.0.1 with trust authentication. Statements
use positional `$1` parameters and return owned buffered UTF-8 rows. See the
[SQL member reference](/docs/sql.html) for limits and unsupported features.
