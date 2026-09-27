# Files, storage and streams

System.Storage describes provider-bound paths, files and directories. System.IO
supplies byte streams, text readers and writers. Operations are synchronous and
return typed Result errors. Use matching Preview 10 runtime and SDK artifacts;
see [setup](../../try/#development) for published package availability.

<a id="example"></a>

## Resolve a directory and work with files

The [Storage POC](../../docs/storage-poc.html) resolves a directory through FileSystem,
writes a file, reads UTF-8 through StreamReader, seeks and reads again, and lists
files and directories. Its source and expected output are available with the guide.

```raven
{{STORAGE_POC_SAMPLE}}
```

StorageProvider resolves paths through GetItem, GetFile and GetDirectory. StorageItem
is a closed interface hierarchy with File and Directory branches. FileSystem supplies
host-backed implementations; the [memory provider sample](../../docs/storage-experiment.html)
uses the same interfaces with in-memory contents. Name and Path describe represented
state without an implicit lookup. File opens byte streams; Directory resolves children.

## Paths and lookup

Path.Parse validates logical slash-based syntax and preserves accepted spelling.
Equality and hashing use that spelling, not filesystem identity, case folding or
Unicode normalization. Providers decide how a valid Path resolves. Other APIs can
accept text without requiring callers to adopt Path universally.

Directory.FileAt describes an address; GetFile explicitly checks it. A successful
lookup cannot guarantee that a later open succeeds. Relative paths resolve within
a directory; absolute paths belong to provider-level lookup. The directory string
overload accepts one child name. GetItems produces a bounded synchronous snapshot.

[Path API](../../docs/api/System.Storage.Path.html) · [Storage items](../../docs/storage-items.html)
· [Provider contract](../../docs/storage-provider.html) · [Lookup](../../docs/storage-lookup.html)

## Byte and text I/O

FileInputStream and FileOutputStream provide bounded reads and writes, exclusive
creation and explicit close. InputStream and OutputStream express separate
capabilities; optional seekability is separate. MemoryStream provides in-memory
byte storage. Callers own buffers and must handle partial transfers.

StreamReader decodes strict UTF-8; StreamWriter retries partial UTF-8 writes.
In development after Preview 10, ReadToEnd decodes chunks incrementally and reports
malformed input before EOF when detected. It still returns one accumulated String;
errors return no partial text. See the [reader contract](../../docs/streams.html#text-readers).
Callers flush and close explicitly. FileText supplies bounded whole-file text
helpers. Console exposes the same text and byte interfaces through standard streams.

[Streams and ownership](../../docs/streams.html) · [Console](../console/)

<a id="limits"></a>

## Selected text encodings (development)

StreamReader and StreamWriter accept an [Encoding](xref:System.Text.Encoding), with
UTF-8 as the default. `Encodings.Ascii` selects strict ASCII. Both adapters retain
the leaveOpen option. Lines are recognized after decoding; unsupported ASCII output
fails before writing. Development StreamWriter uses bounded encoder output and
provides Finish to observe final conversion bytes/errors. Flush handles the stream;
Close releases ownership. Call Finish and Flush explicitly when required before
Close. See [selected encoding contracts](../../docs/streams.html#selected-encodings-development).
Use matching development artifacts; Preview 10 does not include these overloads.

## Behavior and limits

Compared with .NET Stream capability flags, separate interfaces express readable,
writable and seekable operations at the cost of more contracts and adapters.
Storage paths describe provider addresses rather than universal native file identity.

Calls block; async and incremental enumeration are not implemented. Writes are not
transactional or guaranteed atomic, and a failure after opening may leave partial
output. Flush does not promise durable storage. Memory and host providers can have
different bounds and supported operations. Match named storage/stream error cases.
EntryKind is a non-flags enum with File = 1 and Directory = 2; zero is unnamed.

The [File Transformer](../../docs/file-transformer.html) is an application experiment
using its own bounded JSON parser. The separate public [JSON API](../web/#json-dom)
supports DOM and flat object mapping; it does not change that sample's contract.

<a id="direction"></a>

## Possible directions

A file catalog can guide richer metadata queries, freshness rules and enumeration.
Async stream ownership and provider-neutral information need concrete use cases.
Windows/Unix path normalization and broader attributes remain open design work.

[Related proposals](../../proposals/#io)

<a id="feedback"></a>

## Reference and feedback

[System.Storage](xref:System.Storage) · [System.IO](xref:System.IO)

Report a small program, toolchain revision, expected behavior and observed output.
See [how to contribute](../../#feedback).
