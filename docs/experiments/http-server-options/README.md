# Configured HTTP server probe

See [the contract and .NET comparison](../../http-server-options.md).

```sh
python3 scripts/prepare-http-foundation-bundle.py --bundle /path/to/bundle \
  --output target/http-options
python3 scripts/build-native-project.py --profile http \
  --project docs/experiments/http-server-options/Native.rvnproj \
  --bundle target/http-options/bundle --aot /path/to/neoclr-aot-poc \
  --output target/http-options-build
python3 docs/experiments/http-server-options/verify.py \
  --bundle target/http-options/bundle --build target/http-options-build \
  --runtime /path/to/neoclr --output target/http-options-validation.json
```

The peer exercises both interpreted and native macOS ARM64 consumers. Each process
also checks invalid option ranges before binding and changes every option after
Listen, ensuring the active server uses its validated snapshot. Body boundaries
include small decimal lengths because signed division's truncation toward zero
must not allow a single digit above a configured limit below nine.

The native host uses its default 1 MiB heap for these small messages. This is
functional evidence, not a throughput/load or write-backpressure claim. No Windows
run is claimed. The existing foundation fixture covers default options and general
HTTP framing; this probe focuses on the new configuration behavior.

Local macOS ARM64 qualification on 2026-10-11 passes all 18 checks (nine each
in interpreter and native modes). [Recorded results](validation.json) retain
source, runtime, bundle and build-evidence hashes. Read expiry returns the existing
`HttpError.TimedOut` contract.
