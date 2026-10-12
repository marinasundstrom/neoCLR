# Case: query PostgreSQL objects

Run a small Raven application that sends two queries to PostgreSQL and maps their
rows to `Person` objects. The first uses positional values; the second uses a named
parameter object. It needs no tables and changes no persistent database data.

Start with [SQL databases](/features/sql/) to read the query code.
[Download the complete sample](/samples/postgresql-people.zip).

## Start a local database

The sample requires Docker, Python 3 and a matching development neoCLR bundle with
the optional PostgreSQL provider. See [setup](/try/#development) for the toolchain.
The current provider supports local IPv4 loopback and trust authentication only.

```sh
docker run --detach --rm --name neoclr-people \
  --publish 127.0.0.1:55432:5432 \
  --env POSTGRES_HOST_AUTH_METHOD=trust postgres:18
```

Wait for PostgreSQL to report that it is ready. This trust-authenticated database
is for local testing; keep the published port bound to loopback.

## Open and own the connection

The program passes its port argument to this helper. `use` disposes the connection after
the queries complete, including when an operation fails.

```raven
{{SQL_PEOPLE_MAIN}}
```

`ShowPeople` is included in the download and shown on the [SQL guide](/features/sql/).
`SqlMapper.Query<Person>` receives the connection, SQL and plain parameter values.
Column aliases select the writable properties to populate.

## Run the sample

Extract the download and run:

```sh
python3 run.py --bundle /absolute/path/to/neoclr-bundle --port 55432
```

The runner builds with that bundle's compiler and runs using its default JIT mode.
It also retains the model metadata required for property mapping.

```text
42: Café team
43: O'Brien
```

The two queries return their bound values as rows. Replace their SELECT expressions
with a query against your tables while retaining the matching column aliases.
The runner saves its build and execution results in `build/validation.json`.

Stop the disposable database when finished:

```sh
docker stop neoclr-people
```

[SQL contracts and limits](/docs/sql.html) ·
[PostgreSQL API](xref:System.Data.PostgreSql)
