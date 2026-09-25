# Request header construction checkpoint

Development after Preview 9. Use a matching toolchain bundle:

```sh
python3 docs/experiments/http-request-headers/verify.py \
  --toolchain-root /path/to/bundle \
  --runner target/release/examples/measure_async
```

`Sample.rvn` constructs a POST, propagates preparation failures with `?`, sets Accept
and X-Request-Id, then uses Send. It uses an absolute URL; BaseUri resolution belongs
to verb helpers, not direct Send. The sample preserves non-success response data.

The fixture separates ordinary application code from assertions. It checks replacement
across casing, preservation of the original request and content, empty values, invalid
names/values, reserved fields, replacement at the field limit, excess fields, and total
header-byte rejection before DNS. A fake handler observes the fields. An independent
Python peer checks exact GET/POST headers, UTF-8 byte length, body and connection close.
Run it serially because the provider retains fixed timing bounds. It requires no website
build or all-platform sample matrix.

The new request shares content. WithHeader copies the field sequence and moves the
replacement to its end. Treat exposed headers/content as stable during Send. This is
not an append/remove collection or a deeply immutable message. Limits: 13 stored fields,
2,048 encoded head bytes, existing 1,024 body bytes; Content-Type remains content-owned.
See [design and .NET comparison](../../http-client-design.md#request-header-construction-checkpoint--2026-09-25).

Validation on 2026-09-25: the final header fixture passed (1,988 allocations, 46
collections, zero live objects). Signature admission and API/bootstrap snapshots passed.
The first oversized POST check did not observe LimitExceeded; isolated oversized GET
returned it. The exchange now starts its network timer after preflight and completes
validation errors directly, preventing Finish from replacing them with TimedOut.
The complete fixture passes with that boundary correction. This is not a benchmark or
proof of the original timing; unchanged native deadlines still apply to network phases.

The existing HTTP client fixture's single `trickling body` case also passes with the
new boundary: Request deadline exceeded, 590 allocations, 11 collections and zero
live objects. Its independent .NET Content-Length/UTF-8 baseline passes. Broader
network/platform matrices and the website build were not repeated.
