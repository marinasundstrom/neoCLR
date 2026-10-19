---
uid: N:System.Security.Cryptography
---
## Hashing, authentication and secure random bytes

Static APIs provide SHA256.HashData, HMACSHA256.HashData/VerifyData
and RandomNumberGenerator.Fill over caller-owned byte arrays. Recoverable errors
use CryptographyError and Result; invalid storage remains a terminal fault.

See [cryptography contracts](/docs/cryptography.html) for buffer ownership, error
behavior, .NET differences and the bounded initial API surface.

Development buffer contracts use `ReadOnlySlice<byte>` inputs and `Slice<byte>`
destinations. Arrays convert implicitly; range slicing selects a zero-copy window.
