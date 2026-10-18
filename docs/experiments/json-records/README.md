# JSON value records (development, 2026-10-18)

The author requested JSON record-struct serialization with interpreter/JIT/AOT
parity and separately committed slices. Reflection getter execution is the first
slice; JSON mapper admission and end-to-end evidence follow.

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
present in runtime/raven/core/Foundations.rvn. Struct ancestry/query behavior is a
separate follow-up; the property/JSON repair is needed independently of it.
