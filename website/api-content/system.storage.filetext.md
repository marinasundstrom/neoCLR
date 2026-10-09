---
uid: N:System.Storage.FileText
---
## Bounded whole-file text operations

ReadAllText and WriteAllText are synchronous host-path helpers with an explicit
maximum byte count. They return Result values with FileReadError or FileWriteError
for expected path, access, encoding, size and I/O failures.

Use these helpers for bounded whole-file operations. Use
[streams](/docs/streams.html) when processing data incrementally, or
[storage providers](/docs/storage-provider.html) for provider-owned resources.
