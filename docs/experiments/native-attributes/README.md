# Retained native custom attributes — development 2026-10-10

The public Raven `GetCustomAttributesData()` API now executes in a standalone
macOS ARM64 binary as well as the interpreter. It uses the same metadata-only
recipes, allocating ordinary traced descriptors. Inspection never instantiates the
attribute or invokes its constructor. This follows the existing
[.NET CustomAttributeData comparison](../../attribute-introspection.md#comparison-and-tradeoffs-primary-sources-reviewed-2026-09-27).

The checked consumer covers String/Int32/Boolean and Int32-backed enum fixed
arguments, null strings, repeated annotations, constructor signatures, copied argument
sequences, and property/accessor/parameter targets. Enum `ArgumentType` retains its
nominal identity; `Value` is the underlying boxed Int32. Empty results for a retained
unannotated type differ from a fault for an unretained type. Both user constructors
contain deliberate faults; the selection report also proves their bodies are absent.
The executable depends only on the platform system library. Interpreter cleanup
reports zero live objects. No performance claim is made.

```sh
cargo build --bin neoclr
cargo build --manifest-path tools/aot-poc/Cargo.toml
python3 docs/experiments/native-attributes/verify.py \
  --bundle target/library-scopes-final/bundle \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --runner target/debug/neoclr \
  --output target/native-attribute-validation
```

Choose a fresh output directory. Windows x64 uses the same script under an MSVC
x64 environment, with `.exe` tool paths; the collections workflow now runs this
gate. Windows execution remains unqualified until its run/artifacts are checked.
Windows ARM64 is not qualified by this gate.

## Retention and representation

Private reflection-roots schema 3 adds the required Boolean `customAttributes`
policy to each schema-2 root. `construct`, `properties`, `getters` and `setters`
remain independent. Schema 1/2 default to no attribute retention. Identities still
select an exact source module/revision/type index; this is not a stable public
preservation API. The fixture writes its roots from the actual emitted artifact.

The backend snapshots original metadata before lowering, then builds descriptor
factories only for retained attribute targets. It preserves source tokens independently
of executable type ordinals. User attribute constructors remain descriptive references,
not executable roots; runtime descriptor constructors are separately trusted factories.
This reuses the interpreter recipe instead of adding a second attribute decoder.
It costs generated factory code and per-query allocations; an indexed native metadata
store is a future alternative if code size or broader discovery warrants it.

A constructor descriptor previously received the extra identity field used only by
method descriptors. This slice fixes that layout mismatch for both interpreter and
AOT materialization against the source runtime library.

## Bounds and provenance

This gate isolates native retention from the remaining compiler work:
`NativeAttributeAotFixture` attaches native annotations to the compiled unannotated
Raven application. It does not exercise source `[Note(...)]` emission. The marker
uses the existing metadata record profile, without claiming validated native
`System.Attribute` inheritance. The bundled compiler is still `494dede84`;
Raven `51da30ea7` independently qualifies native import/usage binding, not this fixture's
annotation emission. No Runtime Contract configuration or CLI bridge encoding changes.

Named-argument inspection remains unsupported and fails explicitly when building a
retained recipe. Type/array constants, wider primitives, general generic reflection,
assembly/module discovery, automatic test discovery and attribute instantiation are
not added. Field, constructor and other member annotations reuse the same token lookup,
but this gate does not add general native field/method/constructor enumeration.
Next qualify compiler-produced source annotations and the TestAttribute discovery adapter.

See [validation.json](validation.json) for tested artifact hashes and cases. Detailed
command logs, native selection reports and binaries are generated in the output directory.
