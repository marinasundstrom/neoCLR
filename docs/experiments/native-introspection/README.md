# Public Raven introspection probe

This compiles `typeof(Report)` through the existing public `NominalTypeInfo` wrapper,
prints `Report` and checks `IsNominalType`. Native macOS ARM64 and the matching bundle
interpreter both exit 0 with exactly `Report\n`. The lower-level descriptor/shape
consumer is separately qualified on Windows x64; this Raven project has not yet
been run on Windows.

```sh
python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/native-introspection/Native.rvnproj \
  --bundle /absolute/path/to/development/bundle \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --output target/my-native-introspection
```

Use a fresh output directory and a matching development bundle. This is synchronous
console work: the HTTP host expects task-pump exports not retained by this program.
`Name` is a member of NominalTypeInfo/MemberInfo, not the general TypeInfo interface,
so the explicit nominal cast is intentional. Property discovery and reflection
invocation are not exercised or qualified by this probe.
