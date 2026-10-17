---
uid: N:System.IO.Compression
---

gzip support wraps async byte streams with `GZipStream`. Compression
writes incrementally; await `Finish` for the final block and trailer before closing.
`Flush` does not finish gzip, and `Close` only handles disposal and ownership.

Decompression validates complete gzip content before exposing bytes. Stored, fixed
and dynamic DEFLATE blocks, optional headers and concatenated members are supported.
The initial decoder is bounded to 64 KiB of compressed input and 64 KiB of decoded
output. These are current limits. Malformed data returns `StreamError.InvalidData`.

See the [compression guide](/docs/compression.html) for factories, cancellation,
ownership and errors. Automatic HTTP content encoding is future integration work.
