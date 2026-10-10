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

Compiler and native-executable evidence is added in the next slice. A native
metadata fact alone does not qualify Raven's public surface or Windows execution.
