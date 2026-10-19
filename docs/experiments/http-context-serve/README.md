# HTTP context callback sample

Development after Preview 16; use a matching compiler and native class library.
`Sample.rvn` is the canonical website server example. It listens on loopback port
8080 and returns a UTF-8 greeting. The async callback receives `HttpContext`,
configures its response and returns success. `Serve` sends and disposes that
response. Do not call `Complete` or retain the context after the callback returns.

Build with the matching native development bundle:

```sh
export NeoClrBundleRoot=/absolute/path/to/bundle
dotnet "$NeoClrBundleRoot/sdk/tools/rvnc/rvnc.dll" neoclr --project Sample.rvnproj
```

The handler has no asynchronous work in this small example; its async declaration
satisfies the callback contract and may produce a compiler warning. Real handlers
can await I/O. Awaiting `Serve` waits for the server lifetime while requests are
processed concurrently, up to the current 16-exchange limit. An application can
coordinate a separate shutdown signal, call `Close`, then await the lifetime task
to join started handlers. This minimal program runs until its process is stopped.

`Probe.rvn` is a separate lifecycle test, not application boilerplate. Its Promise
gates deliberately hold callbacks pending to check close, cancellation, admission
limits and late response disposal. The repository's existing Serve verifier can
run it with `--source docs/experiments/http-context-serve/Probe.rvn`.

The JSON interoperability walkthrough demonstrates `Accept` explicitly and retains
manual response completion. It is a different ownership model from this sample.
