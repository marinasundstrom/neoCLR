# File Transformer: sensor report acknowledgement

Development experiment, 2026-09-23. This M1 file-consumer slice connects the existing
JSON document experiment to the platform Storage and System.IO APIs. It introduces
no public API or runtime mechanism. JSON remains sample code, not a shipped library.

The app reads `reports/report.json`, validates a sensor report, and writes a new
`reports/reply.json`. For example:

```json
{"station":"Café","readings":[21,22.5]}
```

produces:

```json
{"station":"Café","accepted":true,"count":2,"note":null}
```

It checks that the first reading is an integer; it does not compute an average or
validate every reading. The acknowledgement carries the number of readings.

Run against matching development artifacts:

```sh
python3 docs/experiments/file-transformer/verify.py --toolchain-root /path/to/development/bundle
```

The verifier compiles once and runs isolated fixtures, checks exact UTF-8 bytes and
independently parses the output with Python. It checks successful transformation,
existing output, malformed UTF-8, oversized input, malformed JSON, missing fields,
a non-integer first reading, missing source and a directory in place of the source.
Every fixture checks that the source remains unchanged and no output escapes the
provider root. It does not rerun the complete parser or platform suite: those are
covered by the linked component experiments.

## Layering and failure policy

StorageProvider resolves a File interface; StreamReader owns and closes the opened
InputStream. ReadReport closes the reader before propagating either text or errors.
Acknowledge uses the existing explicit JSON tree, and serializes the entire reply
before the app calls CreateNew. OutputStream writes handle short progress and flush
before closing. Main displays an expected error and returns normally; the printed
outcome, rather than a nonzero exit code, is the sample's error report.

Input and output retain the JSON experiment's 128-byte bound, four nested containers
and 32 values. Text is strict UTF-8. The whole-text reader preserves a BOM, which the
JSON parser rejects. Reading/validation/serialization failures create no destination.
Exclusive creation preserves existing output; there is no existence-check/open race.
Once creation succeeds, an I/O failure can leave a partial destination. The sample
closes it but does not remove or roll it back. Flush is not a power-loss durability
or atomic-save guarantee. The fixtures do not simulate disk-full or device failure.

Compared with [.NET 10 FileMode.CreateNew](https://learn.microsoft.com/en-us/dotnet/api/system.io.filemode?view=net-10.0)
(primary documentation checked 2026-09-23), exclusive creation has the same useful
no-replacement policy. neoCLR reports expected failure through Result rather than
IOException. This is library/host-file policy, not a new CLI feature. The existing
[JSON/.NET comparison](../json-document/README.md#comparison-and-choice) and
[reader comparison](../../proposals/streams-api.md#minimal-text-reader-for-the-storage-poc--2026-09-23)
apply unchanged.

Buffering before creation delays output effects and simplifies validation, at the
cost of bounded retained input, tree and output allocations. Streaming would reduce
some retained data but expose partial output earlier. Temp-file-and-rename saves
would require replacement/cleanup contracts absent from the POC; they remain a
future alternative rather than silently approximating atomic save.

All operations are synchronous. This does not settle Task-returning Storage,
cancellation races, pending buffer ownership or remote provider behavior. The
existing delayed-copy experiments remain the starting evidence for those questions.
