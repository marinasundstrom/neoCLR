# File Transformer

**Development experiment after Preview 9.** This example connects Storage, UTF-8
text reading and the experimental JSON document code. It reads a sensor report
from `reports/report.json` and writes a new `reports/reply.json`.

Input:

```json
{"station":"Café","readings":[21,22.5]}
```

Output:

```json
{"station":"Café","accepted":true,"count":2,"note":null}
```

The app explicitly accesses the station and readings fields, requires an integer
first reading and constructs an acknowledgement. It does not use reflection or
object serialization. JSON types are sample code, not part of the platform API.

Download the [complete source bundle](/samples/file-transformer.zip), extract it,
and use matching development SDK/runtime artifacts. Create the input above in
`reports/report.json`, relative to the directory where you run the app.

```sh
dotnet msbuild file-transformer/FileTransformer.rvnproj -p:NeoCLRRoot=/path/to/neoclr -p:RavenSdkRoot=/path/to/raven-sdk
/path/to/neoclr/bin/neoclr run file-transformer/bin/neoclr/Debug/App.neoil --system /path/to/neoclr/lib/System.neoil
```

Success prints `Saved reply.json`; expected failures print `Failed: ...`. Use a
fresh output path for each successful run: existing reply files are preserved.

The app resolves File and Directory interfaces through StorageProvider. A StreamReader
owns and closes the input. Parsing, field validation and output serialization all
finish before exclusive output creation. The output loop handles short writes,
flushes and closes the stream. See [Storage POC](storage-poc.md) and
[streams/readers](streams.md) for the platform contracts.

The input and escaped JSON output are limited to 128 UTF-8 bytes; JSON is limited
to four nested containers and 32 values. Malformed UTF-8, invalid JSON and field
errors create no output. A later write/flush failure can leave a partial new file;
there is no rollback, atomic replacement or durability guarantee. The app leaves
the input unchanged. All I/O is synchronous. Task-based Storage and cancellation
remain future work.
