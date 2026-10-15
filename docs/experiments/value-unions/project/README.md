# Native value union consumer

This development sample passes a scalar, a reference object and a declared value
union through structural union signatures. It extracts the scalar and the declared
union, matches the latter's payload, and returns **42** on success. It also replaces
a union through an `out` parameter and increments it through `ref`, while checking
that the earlier copied union still contains its original value.

Requires the matching local `target/tuple-names-libraries-v2` bundle, a compiler
built from Raven's `codex/neoclr-union-intersection` branch with the companion
metadata project, and the default neoCLR interpreter build. The bundle is an older
validated fixture; this sample does not establish a complete fresh library build.

From the neoCLR repository root:

```sh
dotnet /Users/robert/Projects/raven-union-intersection/src/Raven.Compiler/bin/Debug/net11.0/rvnc.dll neoclr --project docs/experiments/value-unions/project/App.rvnproj --run /Users/robert/Projects/neoclr/target/debug/neoclr
```

Exit status 42 means success. Value alternatives currently use explicit boxed
payload storage and copied extraction. Generic value-wrapper integration
and the final native layout remain incomplete.
