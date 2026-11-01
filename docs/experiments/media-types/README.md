# Media type contracts and allocation validation

Development 2026-10-23. [Contract and .NET comparison](../../http-client-design.md#structured-media-types--development-2026-10-23).
The public API is implemented in System.Web; the parser does not require new
compiler capabilities or runtime encodings. The catalog uses public classes in
System.Web.Http.MediaTypes rather than unsupported native nested static classes.

## Results

- [Contract tests](contract-tests.json): 12 cases in interpreter, JIT and native
  execution on macOS ARM64 (36 executions). Includes custom type structure,
  quoting/escaping/UTF-8 parameter round trips, equality/hash, parameter view
  immutability, syntax errors, bounds, typed/legacy content and message headers.
  Message tests check cached identity and invalidation when a retained header list
  changes; typed content and requests preserve the supplied value.
- [Allocation tests](allocation-tests.json): two passing interpreter tests, each
  repeating its operations 128 times with zero managed allocations. One measures
  warmed preset/format/hash/parameter/content/request reads and value equality.
  The other validates a raw quoted/multipart header through the exact production
  scanner source without constructing a MediaType. The scanner and limit source
  files are linked into the test assembly to measure internals without adding
  public test hooks or a friend-assembly grant. Wire tests exercise the actual
  built System.Web implementation separately.
- [Wire tests](wire-tests.json): 20 cases across interpreter/native. Independent
  Python peers check typed outbound headers with quoted semicolons, incoming
  boundary case preservation and typed responses. Seven malformed forms fail
  before DNS; conflicting HttpStringContent charset declarations fault in isolated
  processes. JIT execution is covered by the contract suite, not the wire harness.
- [Snapshot](validation.json): regenerated native reference includes all five new
  public types and changed content/message members. No native exclusions. Four
  native snapshot tests and the legacy reference fingerprint check pass. The
  legacy scaffold intentionally does not project the new native-only APIs.

The first allocation probe used Object.ReferenceEquals on strings inside the
measurement loop. That conversion allocated two object adapters per iteration;
separate operation counters isolated the cost to the probe. String identity is
still tested outside the measured region; measured formatting uses ordinary string
comparison and a retained expected string. This is not a claim about all runtime
boxing paths or native allocation counts: native/JIT GC counters remain unbound.
Parsing and changed parameters still allocate; iteration can allocate an iterator.
The bounded duplicate-name scan trades extra comparisons for no temporary index
collection. No throughput improvement or timing benchmark is claimed.

## Reproduction

Use a matching development bundle and fresh output directories:

```sh
python3 scripts/test-runtime-library.py --bundle PATH --suite media-type-values \
  --mode interpreter --mode jit --mode native --output target/media-contracts
python3 scripts/test-runtime-library.py --bundle PATH --suite media-type-allocations \
  --mode interpreter --output target/media-allocations
python3 scripts/validate-media-types.py --bundle PATH --output target/media-wire
dotnet run --project docs/experiments/media-types/DotNetComparison.csproj
python3 scripts/build-api-docs.py --check
python3 -m unittest discover -s scripts -p test_native_api_snapshot.py
```

The retained reports identify the exact compiled compiler/library bundle and
source hashes. The final bundle is an HTTP-only rebuild over the previously
validated Optional bundle; unchanged libraries keep their original provenance.
The snapshot updater now prioritizes that later HTTP rebuild's evidence over the
base library report, preserving the actual source identity of the changed library.

The [.NET 10.0.0 output](dotnet-comparison.json) records parsing/equality differences;
its probe is independently runnable. Website source content is updated, but a full
website build and deployment were not part of this focused slice.


## NeoClr.Testing wire-fixture migration — 2026-11-01

Client invalid-content rejection, client response type/length and server quoted
boundary parsing now use three named NeoClr.Testing cases. The seven invalid inputs
still run in separate client processes. Main explicitly awaits each selected
lifecycle and reports its Result through TestSuite and ConsoleReporter, returning
the report's exit code. The server records callback assertions and exposes them
after ServeOne settles; setup failures and the conflicting-charset probe remain
terminal. Automatic async/parameterized discovery is not added.

Python retains independent request/response header and body checks, the server's
initial port-line protocol, the charset-conflict subprocess and existing budgets.
Successful guest output must exactly match the named framework result and summary,
with empty stderr. Interpreter/AOT scope is unchanged; public contract/allocation
suites and historical reports above remain separate and unchanged.

[Migration evidence](../../validation/testing-migration-20261101/README.md#media-type-wire-fixtures)
records the matching-bundle run and a boundary-capitalization negative control whose
wire response still passes the independent peer checks. No production API or parser
behavior changes are introduced.
