# Executable native introspection subset

`type-descriptors.neoil` exercises the existing TypeName, TypeArgumentCount and
TypeEquals runtime services. It prints semantic names, checks generic arity and
verifies that Model<Int32> and Model<String> have distinct identities despite
sharing the definition name Model. The host checks collection and buffer bounds.

From the repository root, using a freshly built native compiler:

```sh
python3 scripts/validate-native-type-tokens.py \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --output target/my-type-descriptors \
  --descriptor-queries
```

Use a fresh output directory. On Windows x64, run from the MSVC x64 developer
shell and give the compiler's `.exe` path. The validator retains `host` (macOS) or
`host.exe` (Windows), the generated guest object, logs and a hashed report.

Running the standalone host prints:

```text
System.Int32
Account
Model
Model
Type tokens: 42
```

The final line means every identity/arity assertion, collection check and buffer
canary check passed. Failure produces a nonzero exit status. The focused Rust
consumer also checks interpreter output/value parity against the same System seed.

This tests the runtime service layer directly in neoIL. It does not yet qualify
Raven's full TypeInfo wrappers, property discovery, reflection invocation or JSON.
Only primitive and closed nominal token producers are admitted for these queries.
See [the implementation plan](../../native-json-plan.md) for scope and the planned
transition from generated dispatch to shared descriptor tables.
