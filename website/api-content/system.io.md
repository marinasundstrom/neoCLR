---
uid: N:System.IO
---
## Byte streams and text streams

InputStream and OutputStream define byte-oriented capabilities; SeekableStream adds
optional seeking. File streams and MemoryStream provide concrete storage, while
TextReader, TextWriter, StreamReader and StreamWriter layer text operations over streams.

See [streams](/docs/streams.html) for encoding, bounds, closing and typed failures,
and [Console](/docs/console.html) for standard streams. System.Storage supplies
paths and storage-item lookup separately from open stream operations.
