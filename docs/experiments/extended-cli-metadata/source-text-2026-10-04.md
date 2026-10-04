# Source-owned Char and String — 2026-10-04

The native compiler now builds the real Char, String and UnicodeScalar Raven sources
with the cumulative numeric/collection library, imports only that assembly into consumers,
and executes the text cases on NeoCLR. Both Char and String are removed from the retained
seed. Native metadata is used for the rebuilt library and consumers; the explicit CLI
primitive core remains a bootstrap signature dependency.

## Contracts

Char is a Unicode grapheme value, not a numeric or CLI UTF-16 code-unit primitive.
The metadata facade supplies NativeGrapheme; Raven records System_Char in its symbol
model. The host ownership catalog selects the provider. NeoClrEmitOptions.ImplementsGrapheme
selects source ownership separately from PrimitiveType. The emitter validates the sole
private m_value field, omits physical record storage and reads the managed grapheme receiver.
External references are authored from compiler symbols and host identity/digest values.

The explicit System bootstrap's CLI Char signatures are transport aliases to the selected
local/external native grapheme declaration. Without that binding ordinary CLI Char remains
unchanged. Missing providers and duplicate seed declarations reject before publication.

Portable lowering emits Char.FromString for single-scalar and multi-scalar literals,
Char.Equals for equality and literal patterns, and ordinary boxing/interface conversion
for value-to-interface assignments. Native interface dispatch resolves the canonical
Char member using the encoded contract's declaration origin, as for numeric/String owners.

String source now declares the already documented Sequence<char> constructor directly.
Definitions, native reader and writer preserve its ordinary .ctor metadata. Existing
newobj.ctor execution creates a private String receiver slot initially containing empty
text; the constructor replaces that slot and returns the completed immutable String.
The compiler permits intrinsic m_value writes only inside its constructor; metadata
permits starg 0 only in an owned String constructor. Numeric/grapheme constructors remain
unsupported. No public mutable string, new instruction or metadata version was added.

Compared with .NET, constructor metadata and allocation/call syntax retain CLI shape;
the platform String still stores Unicode text in UTF-8, and Char denotes a whole grapheme.
Construction snapshots the supplied sequence and concatenates its text; later source
mutation is independent. Adjacent graphemes can merge. These are semantic differences,
not performance claims. Existing text design/research remains authoritative.

## Reproduce

```sh
python3 docs/experiments/extended-cli-metadata/bootstrap/verify_source_string.py \
  --compiler /path/to/rvnc.dll --runtime target/debug/neoclr \
  --core /path/to/TextCore.dll --output /tmp/native-text-fresh \
  --ownership docs/experiments/extended-cli-metadata/bootstrap/char-ownership.json \
  --consumer docs/experiments/extended-cli-metadata/bootstrap/source-char-consumer.rvn \
  --text-samples
```

Use a fresh directory and the matching explicit TextCore bootstrap from the recorded text
service gate ([bootstrap construction/evidence](text-services-native-2026-10-04.json)). The manifest records every source-built type. The script records source,
bootstrap and artifact hashes, revision bases, commands, stdout and exit codes.

The gate includes the unchanged grapheme sample (exact checked-in output), String
comparison/HashMap sample, UTF-8 slice/Result sample and sequence construction sample.
Additional assertions cover Char comparison through an interface, String equality through
an interface, Unicode casing, interning identity, Object alias identity, named arguments,
array mutation independence, character patterns and equality. Numeric control exits 99;
Char/String assertion consumers exit 42. Missing-provider and duplicate-owner checks
publish no assembly.

[Captured evidence](source-text-native-2026-10-04.json).
This completes this text ownership/emission gate; it does not claim the entire System
library compiles. The older library-strings sample also needs its Math namespace provider,
which is separate from text ownership. Wider source groups, imported nullable annotations
and ordinary cross-assembly class inheritance remain separate work.

Validation also covers C# metadata contracts (143 groups), the explicit native String
binding test, focused ordinary .NET Char/core-import tests (39), and native boxed-interface
and String-construction regressions. Guest API signatures already included the sequence
constructor in the matching bridge reference. The pre-existing full API snapshot is still
stale under build-api-docs.py --check; this subset assembly is not a replacement for the
full documented reference. The manual host metadata API reference is updated in this change.

The portable boxing/literal fixes are generally useful within the new backend, but those
portable components do not exist on Raven main. They remain explicit deferred integration
candidates, not a reason to merge the experimental target or change ordinary .NET emission.

Tested compiler commit: Raven `1bf03e88c`; NeoCLR parent `00262d15` plus this change.
The evidence captures the rebuilt runtime and compiler component hashes, not just driver
revision labels. Both feature checkouts retain the existing .NET backend; Raven main
requires no new target-specific backport.
