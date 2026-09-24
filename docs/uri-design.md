# URI references and HTTP addressing

## Author direction — 2026-09-24

The next HTTP objective is `Uri`, `HttpError` and `HttpClient.BaseUri`. Keep string
and Uri overloads wherever request addresses are accepted; callers should not have
to construct a value object to make a request. This supersedes the earlier tentative
BaseAddress spelling and takes priority over the next cancellation exploration.

## First slice: managed Uri

`System.Uri` is an immutable reference value with `Parse(string) -> Result<Uri,
UriError>`, `Text`, `IsAbsolute`, `IsRelative`, and `Resolve(string)` / `Resolve(Uri)`.
It performs no I/O and introduces no native service. `UriError` identifies InvalidFormat,
UnsupportedAuthority, TooLong and BaseNotAbsolute. Empty text is a valid relative
reference. A scheme makes a reference absolute; it does not imply HTTP support.

The provisional grammar accepts escaped ASCII RFC 3986 URI references up to 4096
bytes, including opaque paths, reg-name authorities, user information, query and
fragment. It validates percent triplets and rejects whitespace, control characters,
backslashes and non-ASCII text. Bracketed IP literals are explicitly unsupported in
this slice. This is a bounded subset, not a claim of full RFC or .NET Uri coverage.
Port syntax is digits; scheme-specific range/default-port policy belongs to a
consumer. No implicit escaping, percent decoding, IDNA or IRI conversion occurs.

Resolution follows RFC 3986 section 5.2 with an absolute base. It replaces authority
for network-path references, replaces the last base path segment for relative paths,
removes literal dot segments, retains a base query only when both reference path and
query are absent, and takes the reference fragment. Absent and empty query/fragment
are distinct. Encoded dots and slashes are preserved. Resolution revalidates the
result and its size. Resolving against a relative base returns BaseNotAbsolute,
even when the supplied reference is absolute.

Equality, hashing and ToString use exact Text, following the existing Path value
object. This deliberately does not implement .NET Uri resource equivalence: case,
default ports, escaping and fragments remain significant. Never use this lexical
identity as an origin/access-control comparison. The benefit is a transparent,
stable contract while canonicalization is unresolved; the cost is that multiple
spellings of a resource are distinct keys. Parse preserves text; Resolve performs
only the documented reference-resolution transformations. All storage is ordinary
managed references; there are no native handles or new GC roots. Copying and repeated
UTF-8 slicing make this a POC parser, not a throughput claim. The initial boundary
probe exposed repeated runtime array-resource scans while a temporary byte array
was live. The parser now reads one-byte string slices instead of retaining an
encoded array. Runtime array-accounting cost remains a separate investigation; this
is not a change to GC policy or a general performance result.

## Comparison and alternatives

Primary sources reviewed 2026-09-24:

- [.NET 10 Uri.TryCreate](https://learn.microsoft.com/en-us/dotnet/api/system.uri.trycreate?view=net-10.0)
  provides nonthrowing construction and base/reference overloads. neoCLR uses Result
  instead of bool/out and retains string conveniences.
- [.NET 10 HttpClient.BaseAddress](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpclient.baseaddress?view=net-10.0)
  requires an absolute base and disallows changing it after requests start. Its URI
  resolution replaces the last base segment; directory bases need a trailing slash.
  BaseUri configuration/error policy is the next slice, not implemented here.
- [RFC 3986 sections 5.2–5.4](https://www.rfc-editor.org/rfc/rfc3986.html#section-5.2)
  supplies the resolution algorithm and executable examples. This avoids inventing
  string concatenation rules for HTTP paths.
- [dotnet/runtime issue 78381](https://github.com/dotnet/runtime/issues/78381) is an
  open documentation issue reporting permissive relative parsing on .NET Framework
  4.8. It is experience evidence, not proof of a current .NET defect. Our comparison
  probe records .NET 10 behavior separately; strict escaped input is an explicit
  neoCLR choice with reduced convenience for human-entered URLs.
- [Rust url::Url](https://docs.rs/url/latest/url/struct.Url.html) exposes parse/join
  Results but implements the WHATWG URL model. Adopting it would bring a mature
  normalization model, but not this generic URI-reference contract, and introduces
  native-library representation or wrapper work. No runtime parser is necessary
  for the current small managed contract.
- [Flurl URL building](https://flurl.dev/docs/fluent-url/) offers mutable building
  and string conveniences. Its append/combine operations serve a different purpose
  from RFC reference resolution. We retain overload convenience without treating
  path appending as resolution or adopting a builder yet.

Language/compiler/metadata remain unchanged: ordinary class methods, Equatable,
Result and the existing closed error carrier representation are sufficient. The
neoCLR bridge explicitly maps the new types and checked methods. Future URI schemes,
IP literals, canonicalization and convenience members need their own use cases.

## Following HTTP slices

1. Replace provisional string request/transport errors with HttpError. Preserve
   inspectable lower-level causes where useful; malformed protocol data, unsupported
   features, resource limits and exchange timeout must be distinguishable. Content
   decoding failures remain separate from obtaining the response bytes.
2. Add BaseUri and string/Uri request overloads. Validate configuration and resolve
   before dispatch; handlers receive a resolved request. Check ordinary relative,
   root-relative, query-only and absolute overrides with a fake handler and a real
   loopback peer. Keep parameterless HttpClient construction.
3. Update the sample and generated API reference together with each contract change.

These following APIs are not implemented by the Uri slice. Public signatures and
configuration policy will be recorded with executable evidence when integrated.

## Comparison result

The local .NET 10.0.0 probe agrees on ordinary relative paths, parent paths,
query-only and fragment-only resolution. It differs on four stored RFC cases:
rejects `g:h`, adds a slash to `//g`, resolves `http:g` as a same-scheme relative
reference, and normalizes encoded dot segments in `%2e%2e/g`. It also accepts several
strings this strict parser rejects, including spaces and malformed percent escapes.
These are recorded compatibility differences, not assertions that .NET is defective.

## Later encoding utilities — author direction, 2026-09-24

The author requests URI/URL encode utilities later. Keep this separate from the
current parser and HTTP addressing milestone. Future design should distinguish
encoding a path segment, a query value and form data; a single escaping operation
cannot silently serve all of them. Compare .NET escaping APIs and RFC/WHATWG rules
with executable Unicode, reserved-character, percent-escape and double-encoding
cases before selecting names or contracts. These utilities are not implemented;
Uri.Parse continues to require already-escaped input.

## Integrated validation — 2026-09-24

The [probe](experiments/uri/README.md) passes 46 stored resolution pairs through
both overloads plus three base-path cases, invalid grammar and length checks.
It verifies Object dispatch, Equatable, collection elements and equal hashes, with
504 allocations, nine collections and zero final live managed objects under a
256-object limit. Matching bootstrap and API snapshots validate; the combined
986-page site and 17 website tests pass. Boundary parsing is still expensive in the
interpreter; no performance or cross-platform result is claimed.
