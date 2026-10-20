# HTTP version value contracts

Development follow-up, 2026-10-20. HttpVersion retains its two nonnegative integer
components and ordinary struct layout. It now implements EquatableTo<HttpVersion>,
overrides Object.Equals/GetHashCode, and overrides its existing ToString method.
Typed and boxed equality compare both components; null and other types are unequal.
The hash combines both fields and is not a persistent identifier. Display remains
major.minor, including the full supported Int32 range. Default remains 0.0.

This applies the same explicit value-contract repair as the
[time structs](../time-values/README.md), following .NET's typed/Object equality and
hashing consistency rules. The existing HttpVersion struct represents only the two
HTTP protocol components, unlike the general-purpose .NET System.Version class;
this change does not introduce a new representation or transport capability.
HTTP/1.1 remains the supported transport version. Constructing a 2.0 value does not
enable HTTP/2. No new ordering, formatting specifiers or compiler/runtime ABI are added.

Typed equality/hash operates on fields. Formatting allocates display text, and
Object/interface conversions can box. This correctness fix does not claim an
allocation-free Object API or a measured throughput improvement.

`Program.rvn` covers typed/interface/boxed dispatch, both components, defaults,
maximum component values, null/wrong types, display/composite formatting, imported
and generic record components, and versions exposed by local request/response
objects. The baseline records the missing interface on the previous bundle.
`check.py` records interpreter/JIT/AOT results and matching library/compiler hashes.
No network requests are sent by this consumer.

The reference assembly previously advertised a ToString override that the Raven
source did not implement. The source now honors that declaration; reference and
native snapshots also include the new equality interface and Object methods.
The broader [class-library review](../../design/class-library-soundness-20261020.md)
remains open; this is not a release qualification or whole-HTTP-stack benchmark.

Validation completed: the consumer passes interpreter, JIT and AOT; four native API
snapshot checks pass. The matching website build passes (see api-validation.txt).
Publication remains separate.
