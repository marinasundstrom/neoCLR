# Dynamic member parameter consumer (development)

`sample/Main.rvn` forwards `typeof(Message)` through an annotated TypeInfo parameter
to `JsonSerializer.Deserialize`. No retention file is present. The native result
prints `retained` and exits zero after reflection constructs the class and sets its
Text property.

Build with a development bundle containing the annotated libraries:

```sh
python3 scripts/build-native-project.py --profile http \
  --project docs/experiments/dynamic-access/sample/App.rvnproj \
  --bundle <development-bundle> --aot target/debug/neoclr-codegen \
  --output target/dynamic-access-sample --command-timeout 600
target/dynamic-access-sample/app
```

The HTTP service profile supplies socket bindings currently reached by JSON's
selected library graph; the program itself opens no listener. The console profile
fails on SocketClose in this qualification. Nested JSON mapping emits unproven-flow
warnings; this simple model needs only the inferred Message root. See
[qualified controls and limitations](../../validation/dynamic-access-20261028/README.md).

The `closure` project adds reflection inside `Message`'s retained constructor. It
maps `Other` through the same annotated `Read` function and prints `retained` twice.
Use `--project docs/experiments/dynamic-access/closure/App.rvnproj` to exercise the
retained-body follow-up. The pre-follow-up backend fails at runtime because Other's
property metadata is missing; see [comparison](../../validation/dynamic-access-closure-20261028/README.md).
