---
title: Cryptography
---
# Cryptography

Development APIs in `System.Security.Cryptography`, owned by `System.Runtime`:

| Type/member | Contract |
| --- | --- |
| `SHA256.HashData(data: ReadOnlySlice<byte>, destination: Slice<byte>)` | `Result<int, CryptographyError>`; writes exactly 32 bytes and returns `Ok(32)`. |
| `HMACSHA256.HashData(key: ReadOnlySlice<byte>, data: ReadOnlySlice<byte>, destination: Slice<byte>)` | The same output contract for an HMAC tag. |
| `HMACSHA256.VerifyData(key: ReadOnlySlice<byte>, data: ReadOnlySlice<byte>, tag: ReadOnlySlice<byte>)` | `bool`; validates a complete tag using the provider's timing-safe comparison. |
| `RandomNumberGenerator.Fill(destination: Slice<byte>)` | `Result<unit, CryptographyError>`; fills the entire destination. |
| `CryptographyError.DestinationTooSmall` | Hash destination is shorter than 32 bytes; nothing is written. |
| `CryptographyError.ProviderFailure` | Randomness failed; the entire requested destination is cleared. |

Inputs must be initialized readonly byte slices. Only bytes within each view are read. Hash outputs may overlap inputs; bytes
beyond the digest remain untouched. Empty data and HMAC keys are accepted. Tags
of the wrong length return false. Invalid storage remains a terminal runtime fault.
Buffers are borrowed for the call, never retained, and callers must coordinate
concurrent writes to shared buffers.

The static names follow .NET's SHA256, HMACSHA256 and RandomNumberGenerator. neoCLR
uses borrowed slices as its single buffer contract, without array compatibility
overloads or AsSlice adapters, and returns `Result`
for recoverable errors instead of exceptions or a Boolean plus out parameter.
Arrays and persistent Memory values can be borrowed implicitly at calls. Slicing
selects a window without copying; destination writes preserve bytes outside it.
The benefit is consistent borrowing and existing managed error handling; the cost is a
source-level difference from .NET. These are synchronous CPU/provider operations,
not waiting I/O. Async stream hashing can compose the stream API later.

The initial surface deliberately omits allocating overloads, stream hashing,
stateful algorithm instances, encryption and certificate APIs. These are support
gaps, not permanent restrictions. The shared ring provider supplies SHA-256, HMAC
and operating-system randomness on macOS and Windows. No FIPS certification or
secret-zeroization guarantee is implied.

See [the tested runtime suite](https://github.com/marinasundstrom/neoCLR/tree/main/runtime/raven/tests/cryptography)
and [provider design and evidence](https://github.com/marinasundstrom/neoCLR/blob/main/docs/shared-security-provider.md).

Development migration (2026-10-19): the four methods replace their former array
signatures. Whole-array callers continue through implicit conversion; explicit
method signatures must be updated. Native execution currently admits array-backed
byte slices. String-backed UTF-8 views remain an interpreter-only capability.
All methods complete synchronously and retain no view or pointer. On random
provider failure only the requested destination window is cleared. The native
provider adapter checks storage, capability and input initialization before
passing temporary pointers; it neither allocates nor collects.
