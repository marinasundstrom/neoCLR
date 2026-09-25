# Bounded UTF-8 file I/O

The current File helpers read and write UTF-8 text synchronously, with an explicit byte limit and typed Result errors.

**Preview 9 implementation.** The current API uses `System.Storage`, renamed from `System.IO`. Use the matching Preview 9 packages; Preview 8 uses the old namespace. File behavior is unchanged and the API remains open to feedback.

<a id="example"></a>

## Bounded UTF-8 file I/O in Raven
```raven
{{FILE_SAMPLE}}
```

Load propagates a read failure or returns the decoded String. The complete sample creates or replaces neoclr-file-demo.txt in its working directory, reads it back, and checks that an oversized write leaves the previous content intact. Run it in a disposable sample directory.

[Complete executable sample →](../../samples/library-files.rvn) · [VS Code setup →](../../try/#development)

<a id="limits"></a>

## Behavior and limits
.NET File names are familiar; explicit byte limits and Result outcomes are different contracts. These are host file operations, not a complete filesystem abstraction. Writes are not transactional or guaranteed atomic, and errors can occur after opening a file.

[Detailed contract and comparisons →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/raven-file-api.md)

<a id="direction"></a>

## Planned work and open questions

**Development after Preview 9:** [FileInputStream and FileOutputStream](../../docs/streams.html) now provide bounded byte reads and writes, typed errors, exclusive creation and explicit close. They use caller-owned buffers and support partial transfers. These calls block; they do not implement asynchronous I/O or durable flush.

The [provider-bound File and Directory sample](../../docs/storage-experiment.html) runs the same byte workflow against disk and memory. Text helpers and streams now share file contents in both providers. Its memory streams deliberately transfer only two bytes at a time; the verifier checks the actual UTF-8 disk contents. All public stream members are documented in the API reference, with Flush covered in the stream guide.

The tested [System.Storage.Path value object](../../docs/api/System.Storage.Path.html) is now integrated into the development platform library. Path.Parse returns a Result after checking logical syntax; providers resolve accepted values. Future parsing is intended to support Unix and Windows formats with normalization through the object; the current parser still preserves its accepted slash-based spelling. Path is a Storage value, not a system-wide requirement: other APIs accept strings, and callers can optionally parse then pass Text. Provider string overloads remain open. The development InputStream and OutputStream interfaces are now part of System.IO; the file classes and sample memory streams implement them directly. The [System.Storage.StorageProvider interface](../../docs/storage-provider.html) now resolves logical paths through GetItem, GetFile and GetDirectory. Results use the StorageItem, File and Directory interfaces; byte opening belongs to provider-owned File implementations. Text helpers are not required by that contract. [StorageItem, File and Directory](../../docs/storage-items.html) are now platform interfaces. Providers implement File or Directory; FileSystem supplies internal host-backed item implementations, while the sample supplies memory implementations. Name and Path expose represented state without a new query. File opens streams and Directory resolves relative children. Native static text helpers now use FileText in the development Raven API; Preview 9 is unchanged. Lookup experiments distinguish a path address, a metadata observation and an open file handle. A successful lookup cannot guarantee that a later open succeeds. The development [metadata API](../../docs/storage-lookup.html) accepts native path strings; sample providers now consume the platform Path and offer typed GetFile and GetDirectory lookup. The product obtains its root Directory through the provider interface; the memory fixture exposes only its root directory. File lookup includes relative Path resolution within a directory. Absolute paths remain provider-level inputs; the directory string overload accepts one child name. Common identity, asynchronous ownership and scheduling remain open; networking follows these foundations.

Storage uses a closed StorageItem interface hierarchy with File and Directory as its two branches. Providers supply concrete implementations; StorageProvider resolves paths, and GetItems returns a bounded sequence of StorageItem values. The interface hierarchy is integrated, with closure checked by the compiler/importer; raw neoIL does not enforce it. FileSystem provider resolution, relative directory traversal and bounded mixed-item enumeration are integrated. Enumeration is a synchronous snapshot with explicit limits; asynchronous and incremental enumeration remain future work. Additional information and attributes should be queryable through storage objects while the core stays independent of a particular backing model. The query shape, supported information and freshness rules remain open; native filesystem attributes will not become requirements for every provider.

The immediate Storage POC stays small: provider-bound addresses, useful names, explicit lookup and byte access. File.Name is derived from its validated Path; descriptor construction performs no lookup. Directory.FileAt describes an address, while GetFile explicitly checks it. Windows Runtime is a design reference, but richer properties, timestamps, size snapshots and query APIs will follow concrete application needs.

Development Path now compares and hashes its exact accepted spelling through both typed equality and Object dispatch. Separately parsed equal paths work as HashMap keys with explicit equality/hash callbacks. This does not compare filesystem identity or normalize case, Unicode or native path formats. EquatableTo&lt;T&gt; accepts T; nullable operands are not imposed on value types. Object.Equals(Object?) is the separate null-aware overload.

### Console and standard streams

The development [Console API](../../docs/console.html) keeps System.Console as a static class. In exposes TextReader; Out and Error expose TextWriter. The new StreamWriter handles partial UTF-8 writes, and StreamReader adds bounded line reading. Byte-stream factories support raw input/output. The runnable sample separates a stdout greeting from stderr diagnostics. Calls are synchronous; terminal controls and async console I/O are not implemented. The [error and optional-result examples](../outcomes/#handling) show how ? and pattern binding keep input handling direct.

### File Transformer

The development [File Transformer sample](../../docs/file-transformer.html) reads a UTF-8 JSON sensor report through Storage, validates it and writes an acknowledgement. Invalid input creates no output; existing output is preserved. JSON remains experimental sample code, and a later I/O failure can leave a partial new file.

### Runnable Storage POC

The [standalone Storage POC](../../docs/storage-poc.html) resolves a directory through FileSystem, writes a file, reads UTF-8 through TextReader/StreamReader, seeks and reads again, and lists files and directories through StorageItem. Its source and expected output are available on the site. Streams, text readers and optional seekability now use System.IO; storage providers and items remain in System.Storage. This development POC is synchronous; task-returning Storage methods are future work.

```raven
{{STORAGE_POC_SAMPLE}}
```

[Related proposals and open questions →](../../proposals/#io)

<a id="feedback"></a>

## Questions and contributions

Questions, examples and documentation corrections are welcome. See [how to contribute](../../#feedback).

Report issues with a small program, the toolchain version, expected behavior and observed output. API proposals should identify the missing operation or contract.

[Discuss on GitHub ↗](https://github.com/marinasundstrom/neoCLR/issues)

## API reference

[System.Storage](xref:System.Storage) · [System.IO](xref:System.IO)

The generated reference describes development after Preview 9. Use the availability
notes above to distinguish it from the published toolchain.


## Development case representation

Storage and stream errors now use normal Raven union declarations. Match named
cases directly; handwritten per-case `Is*`/`Get*` helpers have been removed.
Rebuild applications with the matching development SDK and runtime library.

`EntryKind` is a non-flags enum: `File = 1` and `Directory = 2`. Compare these values
directly. Zero is unnamed; metadata lookup reports failure through StorageLookupError
rather than inventing a fallback kind.
