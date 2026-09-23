# Storage POC: file access through platform interfaces

Development after Preview 9. This ordinary Raven/MSBuild app uses only platform
Storage and I/O APIs. FileSystem resolves a Directory interface; FileAt describes
a destination; File.CreateNew writes UTF-8 through OutputStream; TextReader reads
it back; SeekableStream permits a second read. GetItems lists a directory and a
file through the closed StorageItem hierarchy. Expected errors remain values.

The memory-provider comparison and short-read/error/ownership tests live in the
sibling storage-provider experiment. This app contains no provider implementation.

```sh
python3 docs/experiments/storage-poc/verify.py --toolchain-root /path/to/development/bundle
```

The verifier builds in a temporary directory, creates `storage-demo/examples`,
checks exact output and file bytes, and removes the fixture. To run manually, build
StoragePoc.rvnproj with matching NeoCLRRoot/RavenSdkRoot properties, create that
empty fixture beneath your working directory, then run the generated App.neoil
with the matching System.neoil. Use a fresh fixture for each run: creation is
exclusive and intentionally does not overwrite an existing message.txt.

The text reader owns no input when leaveOpen is true; close it before independently
seeking, then create another reader. This compiler requires an object view when
testing an unrelated interface capability, so the sample uses that intermediate
view before binding SeekableStream. This is a compiler limitation, not a requirement
of the stream model. No unchecked cast or provider-specific class is needed.

All I/O remains synchronous. Enumeration has a count bound and can fail on concurrent
changes. FileSystem's native root mapping is not a symlink-safe security sandbox.
FileAt/CreateNew remain the deliberately small POC creation model. Metadata queries,
async scheduling/cancellation, line readers, other encodings and production provider
breadth remain future work.
