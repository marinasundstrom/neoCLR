# PostgreSQL: query people

This small Raven application sends positional values and a named parameter object
to PostgreSQL, then maps the returned columns to ordinary `Person` objects. It
requires a development native JIT bundle containing `System.Data.PostgreSql.dll`,
Python 3, the matching Raven SDK prerequisites, and a local PostgreSQL instance.

Start a disposable database on loopback only:

```sh
docker run --detach --rm --name neoclr-people \
  --publish 127.0.0.1:55432:5432 \
  --env POSTGRES_HOST_AUTH_METHOD=trust postgres:18
```

Once PostgreSQL is ready, run from this extracted sample directory:

```sh
python3 run.py --bundle /absolute/path/to/neoclr-bundle --port 55432
```

Expected output:

```text
42: Café team
43: O'Brien
```

The runner compiles the project and retains the model metadata used by the mapper.
`build/validation.json` records compilation and execution. The queries need no
schema or persistent tables; PostgreSQL returns the bound values as rows. To use a
real table, keep the matching column aliases and replace the SELECT expressions.
The connection is disposed by `use`, and values are bound separately from SQL text.

```sh
docker stop neoclr-people
```

This provider currently supports local IPv4 loopback and trust authentication only.
It does not yet provide SQL TLS, passwords, pooling or streaming. Do not expose
this trust-authenticated development container to other machines.
