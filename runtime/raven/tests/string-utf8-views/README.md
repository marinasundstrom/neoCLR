# Native string-backed UTF-8 views

The existing String.Utf8Bytes API now lowers to a readonly borrowed byte window in
JIT/AOT, sharing literal or managed text without allocating an encoding array.
The consumer returns zero only after checking literal bytes, NUL/multibyte text,
nested and empty windows, decoding split scalars, independent mutable copies and
explicit GC after the method constructing the string has returned. It exercises
the existing interpreter contract too. No source compiler or public signature change.

Native memory kernel tests also check readonly writes, persistent conversion,
provider access, array copying and unchanged allocation state for view creation.
The exact binding test rejects mutable/persistent results, non-string inputs and
invalid InternalCall bodies/flags. Run those with:

```sh
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer python3 -m unittest discover -s scripts -p test_native_memory_views.py
cargo test -p neoclr-codegen utf8_view_requires --lib
```

Build Tests.rvnproj with scripts/build-native-project.py, console profile, using a
matching native bundle and rebuilt neoclr-codegen. Retained build/report evidence
contains the complete commands, library paths and native helper identity. The JIT
host must also be rebuilt from matching sources. Evidence uses Xcode 26.2 and the
MacOSX26.2 SDK; Raven dcf7ac41f remains unchanged.

The first consumer printed a final constant Console line. AOT/JIT passed, while
the interpreter faulted in that console call after the assertions. This unrelated
standalone-console path is not fixed or silently counted as passing. The final
consumer uses its exit status and has no console dependency; both failed and final
reports are retained. Initial test setup also corrected a missing System.Runtime
import and an invalid test JSON field spelling; neither changed production behavior.

The private offset storage discriminator and trusted literal-pointer boundary are
explained in docs/raven-cli-bridge.md. This is not a stable Native API or a validator
for arbitrary external pointers. Other native element types remain unqualified.
