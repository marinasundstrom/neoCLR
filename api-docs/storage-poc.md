# Storage POC

**Development after Preview 9.** This runnable app demonstrates the platform APIs
working together, without defining its own filesystem provider:

1. Construct FileSystem and use it through StorageProvider.
2. Resolve a Directory and describe a new File address.
3. Write UTF-8 bytes through OutputStream, then close it.
4. Read text through TextReader, implemented by StreamReader over InputStream.
5. Discover SeekableStream, rewind, and read again.
6. Enumerate mixed StorageItem values and traverse a child directory.
7. Observe AlreadyExists and NotFound as expected errors.

Download [Main.rvn](/samples/storage-poc/Main.rvn),
[StoragePoc.rvnproj](/samples/storage-poc/StoragePoc.rvnproj) and
[expected output](/samples/storage-poc/expected.txt). Use matching development SDK
and runtime artifacts; these APIs are not in the published Preview 9 downloads.

```sh
mkdir -p storage-demo/examples
dotnet msbuild StoragePoc.rvnproj -p:NeoCLRRoot=/path/to/neoclr -p:RavenSdkRoot=/path/to/raven-sdk
/path/to/neoclr/bin/neoclr run bin/neoclr/Debug/App.neoil --system bin/neoclr/Debug/System.neoil
```

Run from the directory containing `storage-demo`. Start each run with a fresh
fixture: creation deliberately refuses to replace an existing message.txt.

The key boundaries are StorageProvider → Directory/File → InputStream/OutputStream.
TextReader adds text decoding; SeekableStream is optional. Callers depend on these
interfaces while FileSystem supplies internal implementations. The separate
[disk/memory comparison](storage-experiment.md) exercises the same contracts with
a bounded memory provider and short reads/writes.

The sample explicitly closes streams. Its readers use leaveOpen so the input can
be repositioned after the reader closes. The current compiler requires an object
view for discovering an unrelated interface capability; the sample uses a checked
pattern through that view. Byte positions are not decoded-character positions.

See [Storage providers](storage-provider.md), [item interfaces](storage-items.md)
and [System.IO streams/readers](streams.md) for member contracts and limits.
All operations are synchronous; task-returning Storage operations remain future
work. Listing is bounded and non-atomic, and root mapping is not a security sandbox.

The [File Transformer](file-transformer.md) builds on this example to validate a
JSON sensor report and save an acknowledgement with an explicit failed-save policy.
