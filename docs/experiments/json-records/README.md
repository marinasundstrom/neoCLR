# JSON value records (development, 2026-10-18)

The author requested JSON record-struct serialization with interpreter/JIT/AOT
parity and separately committed slices. Reflection getter execution is the first
slice; JSON mapper admission and end-to-end checks now pass.

.NET serializes public properties of supported classes and structs; see
[Microsoft's supported-type contract](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/supported-types)
and [default serialization behavior](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/how-to).
neoCLR keeps its existing bounded JSON quotas, reflection-retention policy and
Result-based errors. This closes value-record serialization gaps without changing
those policies or claiming full System.Text.Json compatibility.

The low-level reflection fixture exercises a mutating getter twice: the boxed
payload changes from 40 to 42, while the original record stays 40. Interpreter and
GC-enabled native execution agree. Existing class property checks also pass.

The author raised `ValueType : Object` as a familiar semantic model while leaving
its long-term implementation open. Source inspection found ValueType already
present in runtime/System.Runtime/src/System.Runtime/System/Foundations.rvn. Struct ancestry/query behavior is a
separate follow-up; the property/JSON repair is needed independently of it.

## JSON validation

`Main.rvn` passes with identical output in interpreter, JIT and AOT using the
Preview 16 compiler (`c0854cdda`) and source-built development libraries/runtime.
The nine checks cover flat and nested record structs, empty values, mixed
class/record properties, UTF-8 and escaping, unsupported Int64 and nominal unions,
output quota enforcement and explicit unsupported record deserialization.
The quota fixture uses a literal so string construction does not consume the
interpreter instruction budget. Earlier checks exposed unsupported Int64 mapping
to `{}`; explicit primitive admission now prevents that regression.

[Three-mode commands and hashes](three-mode-results.json.gz),
[library build evidence](library-build.json.gz), and
[existing class-mapping regression](class-regression.json.gz) retain exact inputs
and results. The library evidence includes a System.Data rebuild after the final
scalar guard. AOT links a compiled object to the matching service host; it does not
JIT the application. Both native paths enable GC. This is macOS ARM64 evidence,
not an installed Preview 16 update or a cross-platform claim.

Reproduce with `python3 docs/experiments/json-records/verify.py --library <bundle>
--compiler <rvnc.dll> --runtime <neoclr> --backend <neoclr-codegen>
--host <neoclr-jit-host.dylib> --output <new-directory>` using Xcode's SDK for linking.
The library comes from `scripts/build-native-class-library.py`; all tool and module
hashes are retained. API snapshot validation and four snapshot helper tests pass.

Only public nongeneric models with supported public getters are admitted. Native
reflection roots remain explicit. Value-record deserialization/setters, custom
record collection elements and enum/union mapping remain unsupported. Broader
array/method reflection admission is unchanged. Ordinary `record` is a reference
type; the separate type-semantics investigation must preserve that distinction.
