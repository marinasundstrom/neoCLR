# Generic HTTP JSON client checkpoint

Development native `GetFromJson<T>` returns `Task<Result<T, HttpError | JsonError>>`;
`PostAsJson<T>` returns `Task<Result<HttpResponse, HttpError | JsonError>>`.
These helpers compose independent transport and serialization error sources, so an
ad-hoc union preserves both causes without an aggregation-only wrapper or converters.
GET requires successful HTTP status before decoding. POST preserves the response,
including non-2xx status. String/Uri and cancellation overloads retain client settings.
Cancellation remains task cancellation, not an error alternative.

This replaces the previous HttpJsonError return signatures. Rebuild consumers and
replace Http/Json wrapper patterns: extract `Error(let error)`, narrow to HttpError
or JsonError, then match that type's nominal cases. The nominal HttpJsonError type
remains available for explicitly declared contracts. HttpError and JsonError retain
their own domain cases; this migration does not flatten them.

## Native validation (2026-10-13)

```sh
python3 docs/experiments/http-json-client/verify-native.py \
  --library /path/to/matching/bundle/lib \
  --compiler /path/to/matching/rvnc.dll \
  --runtime /path/to/neoclr \
  --output /path/to/new/validation-directory
```

The gate uses the bundle's runtime-await project configuration and native metadata:

- `Main.rvn` executes typed decoding, UTF-8, BaseUri/default headers, request association,
  direct status/transport/JSON errors, unsupported-model rejection before sending,
  scalar serialization, pre-cancellation and pending GET/POST cancellation.
- `PropagationChecks.rvn` executes async composition of two different error streams
  using `?`, preserving success/error alternatives, skipped later work and use cleanup.
- `ApplicationChecks.rvn` executes the intentional AppError domain mapping in both
  flat and nested report models, including successful payload preservation.
- Both report clients and servers compile and verify. This gate does not rerun their
  network interoperability matrix or claim native-GC JIT/AOT HTTP qualification.

The compiler includes the general inline-union constructor/generic-call binding fix
on Raven main and its deliberate carry to the neoCLR branch. No alias is needed for
`Promise<Result<HttpResponse, HttpError | JsonError>>()`. A separate native fix expands
propagation before runtime-await completion wrapping, so early returns complete the
task and run cleanup. The temporary System.Union bridge and its two-to-five-alternative
limit remain; native structural union values are still planned. No Runtime Contract
setting changes are required.

The report applications retain AppError because its Body case is application-specific;
`AdaptClientResult` explicitly maps the original alternatives at that boundary.
See the [style guide](../../raven-conventions.md#propagating-several-existing-error-types).

## Earlier checkpoint evidence

The old `verify.py` uses the legacy CLI importer and is retained as historical tooling;
it is not qualified for these new System.Union signatures. The former CLI checkpoint
reported 863 allocations, 14 collections and zero final live objects on a 256-object
heap, with 609 API signature checks. Those figures do not describe this migration.
Earlier generic method-group fixes were main `13b9105d8` and native `56083626e`.
The [managed report sample](../json-object-mapping/README.md) retains its earlier
network interoperability evidence. Website publication remains separate.


Current evidence: [native validation](validation-20261013.json),
[main tests](raven-main-tests-20261013.log) and
[native-branch tests](raven-native-tests-20261013.log). Raven main `2b372ddc7`
and native `cb4721e7d` each pass 16 focused tests. All eight native programs compile
and verify; four execute (HTTP JSON, async propagation/cleanup, flat/nested adapters).
The matching reference snapshot and 11 focused website/API checks pass. No full
website build, publication or live report-network rerun was performed.
