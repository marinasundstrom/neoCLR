# neoCLR Preview 10 — HTTP applications

Version **0.1.0-preview.10** · tag **v0.1.0-preview.10**.

**Release candidate: validation in progress.** This file will record the selected
runtime/compiler revisions and evidence before publication.

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
Build diagnostic/stale-output qualification and the final known-defect disposition
will be recorded with the candidate evidence before publication.

## Validation

The release uses one comprehensive canonical source/archive run, focused OS/ABI
checks on other supported source hosts, and per-OS minimum-Rust compilation.
Each shipped binary target needs extracted-package checks. A manual full stable
matrix remains available. Source CI is separate from SDK/editor/package evidence.
Final results, revisions, hashes and any remaining limitations will be linked here.
