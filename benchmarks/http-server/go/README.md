# Go HTTP comparison fixture

A retained native Go application for the [HTTP comparison matrix](../platforms.md).
It uses a direct `net/http.HandlerFunc` on `http.Server`, with no ServeMux or external
framework. Each request creates a plain UTF-8 byte body or marshals a fresh Person
using encoding/json. Unlike the Raven/.NET/Node text-then-encode path, Go's standard
JSON API returns bytes directly; this supported API difference is intentional and
must remain visible in comparisons. No response-body cache is used.

Build with the local toolchain (automatic download disabled):

```sh
python3 benchmarks/http-server/go/prepare.py --output target/go-comparison-build
```

The source declares Go 1.26.3, the installed version used for this pilot. The report
records the actual toolchain/environment, command and hashes. There are no external
dependencies. `http-comparison plain|json` prints a port, serves 32 requests, waits
for their connections to close, then prints DONE. Use the common controller rather
than running unrelated load generators. Ordinary Go GC and scheduler defaults remain.

Primary contracts: [net/http.Server](https://pkg.go.dev/net/http#Server) and
[encoding/json.Marshal](https://pkg.go.dev/encoding/json#Marshal).
