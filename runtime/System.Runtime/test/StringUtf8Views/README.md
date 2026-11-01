# Native string-backed UTF-8 views

The existing String.Utf8Bytes API now lowers to a readonly borrowed byte window in
JIT/AOT, sharing literal or managed text without allocating an encoding array.
Four discovered `NeoClr.Testing` cases check literal bytes and NUL/multibyte text,
nested and empty windows, invalid split-scalar decoding, independent mutable copies
and collection after the method constructing the string returns. The lifetime test
keeps decoding, copying, invalid decoding and a second collection together, preserving
the original combined sequence. The same suite runs in interpreter, JIT and AOT.
No source compiler, public signature or storage contract changes.

```sh
python3 scripts/test-runtime-library.py --suite string-utf8-views \
  --mode interpreter --mode jit --mode native \
  --bundle /absolute/matching-bundle --output /fresh/output
```

The project uses generated test discovery and the shared test entry point. Behavior
failures return `TestFailure` with named diagnostics instead of panicking from a
single handwritten Main. Python retains build/process and transcript checks.
[Migration evidence](../../../../docs/validation/testing-migration-20261101/README.md#string-backed-utf-8-views).

Native memory kernel tests also check readonly writes, persistent conversion,
provider access, array copying and unchanged allocation state for view creation.
The exact binding test rejects mutable/persistent results, non-string inputs and
invalid InternalCall bodies/flags. Run those with:

```sh
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer python3 -m unittest discover -s scripts -p test_native_memory_views.py
cargo test -p neoclr-codegen utf8_view_requires --lib
```

## Historical native implementation qualification

The original standalone consumer used a matching native bundle and rebuilt codegen
and JIT host. Retained build/report evidence contains commands, library paths and
native helper identity. That evidence used Xcode 26.2 and the MacOSX26.2 SDK;
Raven dcf7ac41f remained unchanged.

The first consumer printed a final constant Console line. AOT/JIT passed, while
the interpreter faulted in that console call after the assertions. This unrelated
standalone-console path is not fixed or silently counted as passing. That final standalone
consumer used its exit status and had no console dependency; both failed and final
reports are retained. Initial test setup also corrected a missing System.Runtime
import and an invalid test JSON field spelling; neither changed production behavior.

The private offset storage discriminator and trusted literal-pointer boundary are
explained in docs/raven-cli-bridge.md. This is not a stable Native API or a validator
for arbitrary external pointers. Other native element types remain unqualified.
