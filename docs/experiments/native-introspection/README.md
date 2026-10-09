# Public Raven introspection probe

This compiles `typeof(Report)` through the existing public `NominalTypeInfo` wrapper,
prints `Report`, checks `IsNominalType`, constructs the model through `CreateInstance()`, checks its constructor-initialized
property, and compares the result’s `GetType()` with `typeof(Report)`. Native macOS ARM64 and the matching bundle
interpreter both exit 0 with exactly `Report\n`. The lower-level descriptor/shape
consumer is separately qualified on Windows x64; this Raven project has not yet
been run on Windows.

```sh
python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/native-introspection/Native.rvnproj \
  --reflection-roots docs/experiments/native-introspection/reflection-roots.json \
  --bundle /absolute/path/to/development/bundle \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --output target/my-native-introspection
```

Use a fresh output directory and a matching development bundle. This is synchronous
console work: the HTTP host expects task-pump exports not retained by this program.
`Name` is a member of NominalTypeInfo/MemberInfo, not the general TypeInfo interface,
so the explicit nominal cast is intentional. The probe also discovers properties, checks accessor and setter-parameter metadata,
invokes GetValue/SetValue, filters nonpublic properties, creates/reads vectors and
checks element and runtime array identities.

The versioned roots file selects the exact source module/revision/type index from
the matching compiler output and requests constructor, property metadata and getter/setter invocation retention. Its
identity was verified against the native selection report; update it deliberately
if the project’s assembly identity/type ordering changes. It is private development
configuration, not a Raven annotation or a public preservation API. An earlier
GetType-only probe required an explicit Object cast to bridge an implicit-upcast
projection limitation; reflection construction already returns Object here.
