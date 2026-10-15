# Native callback with a structural union result

The callback has the exact shape `(int) -> (int | Other)`. Its target boxes the
integer alternative into the structural union; invocation returns that union value,
and extraction produces exit status **42**. The union is a value even when an
alternative holds a reference.

Run from the neoCLR repository root with the companion compiler and default runtime:

```sh
dotnet /Users/robert/Projects/raven-union-intersection/src/Raven.Compiler/bin/Debug/net11.0/rvnc.dll neoclr --project docs/experiments/value-unions/callback-project/App.rvnproj --run /Users/robert/Projects/neoclr/target/debug/neoclr
```

The project uses the existing local `target/tuple-names-libraries-v2` fixture bundle.
This validates a nongeneric exact callback target. Callback variance/adapters,
generic targets containing structural types and native JIT/AOT are not established
by this sample. See the parent evidence for the separate fresh library build.
