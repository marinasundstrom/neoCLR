# Next experimental preview — release candidate notes

**Draft for review; no version, publication date or download assets are selected.**
Follow the [qualification record](experiments/release-20261013/README.md) for exact
revisions and remaining gates. These notes describe the candidate, not Preview 13.

The candidate makes default JIT execution usable for the HTTP client/server
showcase, including typed JSON. It also adds shared managed TLS streams and an
optional PostgreSQL provider with object mapping. The platform remains experimental:
rebuild applications with the matching compiler, runtime and libraries.

## Try the showcase

The [installed macOS preview](local-preview-20261012.md) has the SDK and VS Code
extension configured side by side. Open `http-json-server`, start its **neoCLR: Run**
task with request count `2`, then open `http-json-client` and supply the server's
printed URL. The client retrieves a typed report, posts it back and prints
`{"accepted":true}`. Collections, Tasks and JSON are standalone packaged examples.

The package verifier checks hashes, compiles the examples and exercises default JIT:

```sh
python3 tools/verify-native-bundle.py --report ../acceptance.json
```

Use `python` on Windows. Keep the compiler, extension, runtime, native JIT helper
and libraries together. The Raven compiler/editor still require the matching .NET
SDK prerequisites described in the [package procedure](experiments/raven-target/RELEASING.md).

## Changes worth trying

- **JIT and native execution:** larger native type and heap-object budgets admit
  the HTTP async state machines. Mixed unit/no-result host callbacks retain the
  same ownership and fault behavior when their combined target count exceeds 32.
  Interpreter execution remains available explicitly; AOT remains a separately
  qualified publishing experiment with bounded capabilities.
- **HTTP, TLS and JSON:** asynchronous streams compose through managed `SslStream`
  and shared certificate/security foundations. The SSL consumer checks HTTPS
  nested-object exchange, and separate failure cases cover rejection/recovery,
  deadlines and cancellation. Qualification is recorded per platform and mode;
  local success does not establish Windows success.
- **SQL mapping:** the optional PostgreSQL library supports query mapping with plain
  positional values or named parameter objects. The shared mapper takes a
  connection and statement; PostgreSQL placeholder handling belongs to its provider.
  The current provider is managed Raven over shared sockets, not a libpq binding.
  See the tested [PostgreSQL example](experiments/postgresql-people/README.md).
- **Tooling and examples:** installed sample tasks accept HTTP arguments, repeated
  reflection preparation preserves caller-owned files, and the API website uses
  its committed native snapshot rather than stale local audit output.

## Migration and limits

Use `System.Concurrency.Tasks` for task APIs. Stream operations return tasks by
convention; update older synchronous stream consumers to await them. The package's
`neoCLR: Run` task now uses installed default JIT; use explicit interpreter selection
when that execution mode is required. Native bounds are implementation limitations,
not language rules or a promise of unrestricted native compilation.

The selected qualification targets are macOS ARM64 and Windows x64. Windows ARM64,
Linux JIT/AOT, general boxed Object formatting and unrestricted reflection remain
outside this release qualification. PostgreSQL has local macOS evidence; fresh
Windows SQL and standalone SQL deployment qualification remain open. Source
bootstrap still retains the documented primitive core/seed dependency.

Recorded benchmarks describe named development configurations. No new cross-runtime
performance claim, stable ABI, production support guarantee or website deployment
is implied by this candidate. Publication requires review of the final artifacts,
checksums and [remaining gates](experiments/release-20261013/README.md#remaining-release-gates).
