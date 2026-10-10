# Init-accessor qualification (development)

See [the contract](../../init-accessors.md) for the .NET comparison, native flag,
readonly-write rules and runtime-call boundary.

Metadata/runtime slice (2026-10-10):

- `dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- --init-accessors`
  checks authoring, native JSON/binary containers, introspection, CLI modreq,
  imported CLI calls, reflection invocation and malformed/readonly rejection.
- `cargo test --locked --test properties`: 9 passed.
- `cargo test --locked --test readonly_fields`: 5 passed, including an init setter
  writing its own readonly field and ordinary-method rejection.

Compiler/native slice: Raven `b939cd6964d57829fb00c4e058bf43309de39628` on
`codex/source-object-metadata-resolution`, with matching neoCLR tools. The 62 focused
Raven property/initializer/record tests pass. The [macOS report](macos-validation.json)
records the successful native/interpreter collections and init consumers plus the
separate-library consumer and rejected ordinary/direct-accessor writes.

`native/Main.rvn` tests automatic and implemented init accessors, a local record,
and an imported generic KeyValuePair. `library` and `raven` test a separately built
class/record library and its interpreter consumer. Reproduce both through:

```sh
SDKROOT=$(xcrun --sdk macosx --show-sdk-path) python3 scripts/validate-native-collections.py \
  --bundle target/init-accessors-pinned/bundle --output target/fresh-init-validation
```

The Windows native collections GitHub action runs the same native/interpreter and
separate-library/rejection cases. Windows init qualification is pending.

AOT specialization and selection retain only init setter associations whose bodies
are already reached by calls. They do not retain unused setters/getters merely to
preserve a property description. A focused AOT test checks two generic instantiations,
unused-setter trimming and readonly rejection after removal of the marker:
`cargo test --locked --manifest-path tools/aot-poc/Cargo.toml --bin neoclr-aot-poc init_accessor`.
Runtime/reflection invocation outside initialization remains intentionally permitted.
