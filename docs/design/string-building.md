# StringBuilder and separator joins — development, 2026-10-08

The author explicitly requested this side quest while native HTTP work continues,
then selected fluent append with a fault on limit overflow and requested String.Join.
This revisits, rather than invalidates, the slower application prototype in the
[2026-09-27 evaluation](text-abstraction.md#bounded-report-construction-evaluation--2026-09-27).
It does not reprioritize the remainder of the platform roadmap.

## Contract and comparison

The ergonomic baseline is .NET 10 [StringBuilder](https://learn.microsoft.com/dotnet/api/system.text.stringbuilder?view=net-10.0):
mutable construction, fluent append, and an immutable ToString result. Its UTF-16
length/capacity vocabulary is not reused for neoCLR's UTF-8 storage. neoCLR exposes
Utf8ByteCount and MaxUtf8Bytes explicitly; appending a combining mark can change
one grapheme without adding one independently countable character. The initial
builder accepts a limit of 0..65536 bytes, default 65536. This bounded development
policy follows the current array/storage profile, not a permanent text-format limit.
The integer constructor specifies a limit, not .NET's initial capacity.

Append(string), AppendLine(string), AppendLine(), and Clear() return the same
builder. Overflow raises System.Fail's user fault before content mutation. AppendLine
preflights both the text and the LF byte, so it cannot leave half a line. Its newline
is always LF; protocol code must append CRLF explicitly. ToString caches an immutable
snapshot until the next nonempty append. Clear releases fragments and preserves the
limit; retaining buffer capacity is not promised. Instances are not thread-safe.

The author considered Result<StringBuilder, Error> with propagation. The retained
choice is fluent Append, keeping ordinary construction concise. A future TryAppend
may supply recoverable quota errors if a real consumer needs them; it is not added
speculatively. Allocation failures remain runtime faults.

[String.Join](https://learn.microsoft.com/dotnet/api/system.string.join?view=net-10.0)
provides the second .NET baseline: separators occur only between elements, including
empty elements; an empty array produces empty text. The first neoCLR overload is
Join(separator: string, values: string[]) -> string. Both separator and elements
are non-null, consistent with existing String text overloads. Unlike .NET's null-to-
empty coercion, invalid null inputs fault. No implicit object formatting, variadic,
iterable, or range overload is included. Inputs currently have at most 65536 elements;
the result has an independently checked Int32 UTF-8 byte count, not the builder's
65536-byte quota. Native heap limits still apply.

Primary documentation retrieved 2026-10-08. These are API/library policies, not new
CIL instructions or compiler syntax. No implementation or performance equivalence
with the CLR is claimed. Rust's stable [String](https://doc.rust-lang.org/std/string/struct.String.html)
API offers a useful alternative: owned growable UTF-8 storage with push_str and byte
length. Adopting that storage design would require a new mutable native buffer and
ownership contract here; immutable neoCLR String remains unchanged in this slice.

## Implementation and tradeoffs

StringBuilder is ordinary Raven CIL with a geometrically grown checked string array.
Append retains immutable fragments. Materialization performs one result allocation
and copies each fragment once through the private StringJoinParts service; then the
builder compacts its retained fragments to that snapshot. The original prototype's
balanced repeated concatenations are not promoted. Repeated ToString after every
append still copies the growing text, so applications should materialize at useful
boundaries. Small fixed concatenations can remain cheaper.

The shared service takes a checked prefix, separator, and independently verified
expected byte count. It validates the entire prefix and separators before allocation;
uninitialized slots outside the prefix are ignored. Malformed/null/uninitialized
service inputs produce RuntimeError; allocation exhaustion produces the existing
native-memory fault. Failure does not publish an output. Managed arrays and text
use the existing interpreter ownership and native GC tracing. The native C service
cannot reenter or collect; caller roots keep the array, separator and fragments alive.
The private pointer layout and C symbols are not a stable external ABI.

String.Join preflights its full array in Raven and uses the same service. The
additional size pass costs traversal but avoids repeated growing-string copies.
The service rechecks the sum rather than trusting arbitrary CIL or bridge callers.
No native mutable-buffer handle or special StringBuilder opcode is introduced.

The native backend now admits direct virtual calls whose declared owner is a verified
sealed reference class with no derived type in the complete loaded set. The current
sealing flag is descriptive, so the backend explicitly checks that no descendant
can provide a different implementation instead of trusting that flag alone. The private projection
removes the virtual flags while preserving method bodies and callvirt null checks.
Unsealed virtual dispatch remains unsupported by this path. This is required for
ordinary StringBuilder.ToString calls; it does not add general class dispatch.

## Validation and remaining scope

[Executable samples and runner](../experiments/string-building/README.md) cover
Unicode/combining fragments, chaining, exact/zero limits, immutable snapshots,
clear/reuse, empty/singleton arrays, separators and fault parity in both modes.
Kernel checks exercise reserved-prefix initialization, byte-count mismatches,
empty fragments, null input, allocation exhaustion and no output publication in
both arena and GC configurations under UBSan/bounds. Sealed-call tests compare
interpreter/native results and null faults and retain rejection of unsealed dispatch.

Benchmarks compare the same Raven construction workload using Concat, StringBuilder
and Join in both modes. Their scope includes process startup/load/verification and
is not an isolated append benchmark, HTTP throughput, or a .NET comparison.
Do not replace HTTP header construction or response streaming based on this result
alone. Insertion, replacement, numeric formatting, pooling, general collection
joining and larger builder quotas remain separate consumer-driven work.
