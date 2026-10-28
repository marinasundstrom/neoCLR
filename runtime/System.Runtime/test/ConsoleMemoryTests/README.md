# Console memory integration

Run from the repository root with a matching current compiler/runtime bundle:

```sh
python3 runtime/System.Runtime/test/ConsoleMemoryTests/verify.py \
  --bundle target/console-memory-qualified-bundle/bundle \
  --output target/console-memory-consumer
```

The async Raven consumer uses the production public console streams. The verifier
checks binary stdout (`2800ff`) and stderr (`00ff`) separately in interpreter,
JIT and AOT, including nonzero offsets, NUL/high bytes, end-position empty writes,
invalid ranges, flush, closed-before-range errors and pre-cancellation.

The public stream signatures remain array/range until their coordinated migration.
Internally output uses `ReadOnlyMemory<byte>`. Interpreter host calls snapshot
only the selected initialized window before releasing graph access. Native code
resolves a checked readonly window for the existing synchronous console provider.
Neither path retains a raw pointer after the call. This is not asynchronous OS I/O
qualification or a public Native API. The private legacy array service remains for
bootstrap fixtures; it is absent from the production Raven declaration.

Rust host-call tests cover source mutation after preparation and uninitialized bytes
outside/inside the selected window. Native binding tests reject wrong element and
capability metadata rather than recognizing only a type name.
