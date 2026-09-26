# Generic HTTP JSON client checkpoint

Development POC using `System.Web.Http.Json` extension methods. `GetFromJson<T>`
requires a successful HTTP status and reads a supported flat model; `PostAsJson<T>`
serializes a model and preserves the response, including non-2xx status. String/Uri
and optional cancellation overloads use the existing client configuration.

```sh
python3 docs/experiments/http-json-client/verify.py \
  --toolchain-root /path/to/matching/development/bundle \
  --runner /path/to/measure_async
```

The focused consumer checks typed decoding, UTF-8 content, BaseUri resolution,
default headers, response/request association, nested transport/status/JSON errors,
unsupported model rejection before sending, pre-cancellation, and acknowledged pending
cancellation through the GET/POST continuation chains. It exercises
reference-type generic serialization and primitive rejection through `box T`.
The [managed report sample](../json-object-mapping/README.md) uses both helpers
against a neoCLR server; its verifier also supports an independent Python peer.

The compiler must include Raven's generic method-group fix: main `13b9105d8`,
neoCLR branch `56083626e`. No per-call header or serializer-option API is added.

The focused public check passes with 863 allocations, 14 collections and zero final
live objects on a 256-object heap. API bridge signature validation passes 609 checks.

Matching API and library snapshots validate, including explicit dependency hashes
for all JSON helper source files. Website build and full suites are skipped.
