# Explicit interface properties: end-to-end gate

The native emitter authors explicit accessor mappings from Raven symbols and output-owned
references. Builders and manually authored definitions share validation. CLI output uses
MethodImpl; native output retains the existing interface_implementations format. Runtime
property metadata now accepts the qualified names emitted for explicit accessors. It still
validates accessor identities and signatures. No Runtime Contract or seed change is required.

Run `bootstrap/verify_explicit_properties.py` with `--compiler`, `--runtime`, `--core`,
`--seed`, `--base-library`, `--ownership` and a fresh `--output` directory. The checked
sources compile contracts, implementation and consumer independently on each target.
The consumer mutates through one interface reference and observes it through another;
its explicit getter returns 42. No library source is present in the consumer invocation.
Use the text-service seed and canonical Numbers ownership described in the preceding gate.
Evidence records exact commands, input/artifact hashes and repository base revisions.

The stronger .NET control exposed an independent codegen defect: imported constructed
interface setters were looked up with open generic parameter types. The lookup now uses
the substituted interface signature. This fix has a focused C# execution regression and
is isolated from native emission changes for independent main-line validation.

The actual String/Char cumulative source build passes the explicit Count property and now
fails before publication at `String.Concat: local loaded before store on some path`.
This is the next source-library blocker; this gate does not establish source-owned String/
Char storage. A public property plus an explicit property of the same short name also
produced RAV0111 during binding and remains a separate general compiler candidate.

Native CLI-projection convenience rejects explicit mappings; native semantic import does
not use that path. CLI-reader mapping materialization and generic explicit methods remain
unsupported. See the public host API reference for exact boundaries. No website build or
performance claim is associated with this slice.

Validated compiler: 0bc0a65d2; isolated general fix: 5f431d6c3. Metadata contracts:
141/141; runtime property tests: 7/7; focused .NET controls: 7/7. Both driver consumers
exit 42 with empty stdout. [Commands and hashes](explicit-properties-native-2026-10-04.json).

The subsequent pattern-expression slice resolves the recorded String.Concat emission
failure; see [the follow-up evidence](pattern-expressions-native-2026-10-04.json). The script
accepts `--scenario pattern-expression` for that paired gate. Its null case executes inside
the implementation library because imported native nullable annotations remain unsupported.
