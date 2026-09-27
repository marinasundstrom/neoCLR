# neoCLR Preview 10 — HTTP applications

Version **0.1.0-preview.10** · tag **v0.1.0-preview.10**.

**Qualified 2026-09-27.** Runtime artifacts use neoCLR `59f9f4a7` and Raven
`a108df82a` on `neoclr`. The final tag also contains these release notes and evidence.

neoCLR is an experimental application platform. This preview's bounded HTTP POC
lets Raven client and server applications exchange typed JSON over real sockets,
with cancellation and explicit exchange ownership. APIs remain experimental.

## Run the HTTP application

Prebuilt tools target **macOS arm64**. Use matching runtime, Raven SDK and VSIX
assets; .NET SDK 11.0.100-rc.1.26425.128 and Python 3.9+ are build prerequisites.
The runtime and built guest applications do not require .NET.

Extract the runtime and SDK separately. From the runtime bundle:

```sh
python3 configure.py --sdk /absolute/path/to/extracted/raven-sdk
python3 samples/http/http-json/verify.py --toolchain-root "$PWD" \
  --sdk /absolute/path/to/extracted/raven-sdk --runner tools/http-runner --mapped
python3 samples/http/http-stream-upload/verify.py --toolchain-root "$PWD" \
  --sdk /absolute/path/to/extracted/raven-sdk --runner tools/http-runner
```

The first command checks the typed JSON application with neoCLR and independent
Python peers. Omit `--mapped` for the JSON DOM variant. The second checks upload
bytes, short reads, source ownership/errors, cancellation and one-shot use.
These fixtures use the bundled runner with the documented HTTP instruction budget.
Install the matching VSIX and use the explicit neoCLR MSBuild tasks for editing;
Raven's ordinary .NET toolbar is not the neoCLR execution path.

## Included behavior

- IPv4 DNS and TCP connect/listen/accept/send/receive, partial transfers, bounded
  deadlines and cancellation with acknowledged cleanup. Typed address values
  include IPv6 parsing, but IPv6 transport is not implemented.
- HTTP/1.1 GET, HEAD, POST, PUT, PATCH and DELETE; typed errors, URI/base resolution,
  headers/default headers, status values, request association and handler pipelines.
  Server Accept/HttpContext owns the request/response and explicit completion.
- Buffered bodies up to 1,024 bytes; known-length client uploads up to 65,536 bytes,
  read in chunks after headers. Streams are one-shot with explicit owned/borrowed
  lifetime. Responses and server request bodies remain buffered.
- JSON DOM, strict UTF-8, flat string/int/bool object mapping and generic JSON
  client helpers. JSON remains bounded to 128 UTF-8 bytes, four nested containers
  and 32 value occurrences; broader model shapes are not supported.
- Provider-bound Storage, synchronous file/memory byte streams, text readers/writers
  and standard console streams. Tasks and cooperative cancellation tokens support
  operation completion; runtime-owned suspension remains future work.
- Standard Raven Option, Result and TaskOutcome unions with generated IUnion,
  boxed active cases, HasValue and typed TryGetValue. Option/Result support propagation;
  task cancellation remains distinct from a completed error value.
- Object/reference/value contracts, record classes/structs, shared immutable String
  storage, grapheme indexing and explicit execution-local interning; bounded metadata
  discovery and constructor/property reflection support the JSON mapper.

Rebuild applications with the matching SDK, reference metadata, importer and library.
Preview APIs and artifact formats can change; old generated outputs are not compatible
by assumption. See the feature/API guides for exact supported signatures and limits.

## Limits and known observations

This is a cleartext, one-connection-per-exchange HTTP POC. TLS, pooling, redirects,
response streaming, unknown-length uploads, informational responses/trailers and
HTTP/2/3 are deferred. Upload source reads are synchronous and cannot be interrupted
mid-read. HTTP retains provisional fixed deadlines; serial controlled correctness
checks do not establish latency or throughput guarantees under load.

Compiler-generated state machines implement async. Ordinary worker joins can block
the invocation queue; general async cleanup, shared worker heaps and runtime-owned
suspension are not included. The importer supports a bounded CLI subset, not arbitrary
.NET applications. Raven source debugging on neoCLR is not claimed.

The HTTP samples retain explicit workarounds for recorded compiler issues, including
an expression-bodied conditional getter. Earlier transport timeouts under competing
work remain unexplained; this release does not claim those observations repaired.
Packaged build diagnostics and stale-output rejection pass. A boxed Completed
pattern has a compiler binding limitation; use an explicit closed-case cast when
inspecting IUnion.Value. Ordinary task outcome patterns and the actual boxed
contract pass. The historical source API checklist also needs a separate refresh;
the on-site metadata API inventory and snapshot are current.

## Validation

The six-job source CI split passed at `1404454e`: the slowest job took 8.88 minutes,
compared with the earlier 68-minute job. That run predates the union migration. At
the author's direction, later corrections use focused validation rather than a
repeated full platform matrix: the combined union contract, 24 task pipeline checks,
103 affected runtime tests and ten Raven metadata tests pass.

The final extracted macOS arm64 runtime/SDK passes union and editor checks, typed
JSON with independent peers and managed pairs, and all 11 upload cases. All report
zero live objects. Twenty-two packaged MSBuild diagnostic/build checks passed with
the same compiler, bridge source, reference and build assets before the final native
reflection correction. Interactive VS Code hover/build/run printed 42 from the new
TaskOutcome union; the final packaged checks cover the rebuilt reflection path.

The API snapshot, 18 website tests and 1,025-page website build pass. Prebuilt binary
support is macOS arm64 only. Source CI does not establish packaged binary support
on other platforms. Website publication is a separate manual workflow.

See [validation evidence](preview-10-validation.json). Download matching runtime,
SDK, VSIX and companion notices from this release. `release-manifest.json` records
exact revisions and asset hashes; `validation-evidence.tar.gz` contains reports and
logs. Verify downloads with `SHA256SUMS`. The separately named source archive matches
the final tag, including release documentation added after runtime packaging.
