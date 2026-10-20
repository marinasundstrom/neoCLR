# Class-library release soundness review

Author-directed review, 2026-10-20. The release objective includes class/struct
behavior, interface contracts, equality, hashing, display/formatting and allocation
purpose. A result or owner allocation can be appropriate; prioritize avoidable
short-lived wrappers, copied buffers and intermediate strings. Structs do not imply
zero GC impact: they may hold references, be embedded in objects or be boxed.

This is an open review record, not a completed library audit or release gate pass.
The default comparison is .NET value/reference semantics and ordinary class/struct
ABI. Keep native support gaps explicit rather than inventing identity semantics for
values or requiring special runtime record types.

## Qualified follow-ups

| Area | Concrete result | Evidence |
| --- | --- | --- |
| Generic records | Declared-type equality policy no longer allocates reflection descriptors; value boxes remain | [record policy and allocation controls](../experiments/record-generics/README.md) |
| Duration | Typed/Object equality and hashes cover all signed ticks | [Duration values](../experiments/duration-values/README.md) |
| Formatting | Culture reaches Formattable; bounded time/duration formats and all composite arities work | [culture formatting](../experiments/formattable-culture/README.md) |
| Other time values | Date, TimeOfDay, Instant, TimeOffset and LocalDateTime have matching Object equality/hash; LocalDateTime has civil equality/ordering interfaces | [time values](../experiments/time-values/README.md) |
| Enum Object values | Native Int32-backed enum equality/hash adapters retain exact type and underlying value; [general formatting](../experiments/enum-display/README.md) now supports names/flags/numeric text | [enum qualification](../experiments/enum-object/README.md) |
| Floating Object values | Native Equals/GetHashCode now preserves NaN, signed zero and exact types, including ordinary/generic struct fields | [floating Object qualification](../experiments/floating-object/README.md#native-backend-qualification--2026-10-20) |
| HTTP version | Both components participate in typed/Object equality and hashes; display dispatch matches the reference contract | [HTTP version values](../experiments/http-version-values/README.md) |

These consumers qualify their documented contracts in interpreter/JIT/AOT. They
are not exhaustive tests of every member or every class-library type. Prior evidence
should be reused when its assumptions are unchanged.

## Remaining concrete candidates

- [Index/Range and seven error display overrides](../experiments/value-display/README.md)
  now preserve direct/boxed/composite output in interpreter/JIT/AOT. This repairs
  dispatch; it does not change their existing equality or allocation policy.
- Ordinary structs now have runtime-owned fieldwise defaults; see the
  [qualification](../experiments/struct-defaults/README.md). Extend supported native
  field shapes as their boxed adapters become available; this is not full .NET
  ValueType coverage and does not synthesize EquatableTo or ordinary operators.
- Generic record display, explicit enum format specifiers and primitive
  self-equality/formatting coverage remain incomplete. Floating-point equality must
  handle NaN and signed zero consistently with hashing; do not replace it casually
  with the equality operator.
- Instant and TimeOffset display need a deliberate representation contract over
  their entire supported domains. Instant admits signed tick values outside the
  bounded civil year range; an ISO-only formatter must not silently lose that domain.
- Primitive numeric format strings, custom patterns/providers and comprehensive
  locale data remain separate from the now-qualified culture propagation.

## Families still requiring review coverage

Collections need collision/comparer/iterator/mutation and retained-reference checks;
source inspection alone is not complete validation. Snapshot allocations may be
intentional ownership contracts. Stream/text wrappers need lifecycle, capability,
buffering, cancellation and ownership evidence, including the existing distinction
between Close and finalization. HTTP/networking need retained-body, lifetime and
callback review without claiming a throughput result from allocation count. Data
and provider APIs, diagnostics/reflection, ordinary error values, unions and internal
implementation helpers remain in scope as dependencies of those public contracts.

Discovery with `rg '^public .*\b(class|struct|record)\b' runtime/raven/src
runtime/raven/providers` currently finds 149 declaration lines. This is only a source
search aid: it excludes unions and internal helpers and is not a coverage metric.
A whole-library completion claim requires closing the concrete gaps and mapping
remaining families to supported contracts and focused evidence.

## Default equality versus generated operators

Author clarification, 2026-10-20: ordinary structs need useful overridable default
value equality, and EquatableTo provides the typed comparison path. The general
fallback must be addressed separately from completing individual library types.

[.NET ValueType.Equals](https://learn.microsoft.com/en-us/dotnet/api/system.valuetype.equals?view=net-6.0)
provides ordinary struct value equality. [C# equality operators](https://learn.microsoft.com/dotnet/csharp/language-reference/operators/equality-operators)
are not supplied automatically for plain structs; record structs synthesize equality
members. Follow that distinction unless the author chooses a language divergence.
The [struct specification](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/structs)
also distinguishes typed/constrained calls from conversions to interface objects:
implementing EquatableTo does not make every interface conversion allocation-free.
neoCLR must qualify those paths independently; existing generic record boxes remain
a measured limitation, not a required semantic allocation.

## Whole-library contract pass

Author direction, 2026-10-20: extend the check to most/all types, continue through
issues, and make consistency an explicit objective. The [reproducible inventory](class-library-type-inventory-20261020.md)
covers **all 260 authored top-level public declarations**, including the optional
PostgreSQL provider: 66 interfaces, 11 static classes, 12 enums, 33 unions, two
records, two borrowed views, two persistent views, 17 intrinsic types, 19 ordinary
structs and 96 reference classes. This is a declaration/inheritance and implementation
review, not execution qualification of all methods. Internal reflection implementations
were additionally inspected because they implement the public descriptor contracts.

Consistency means typed/Object/interface dispatch agrees where supported; equal values
have equal hashes; display overrides remain virtual; and reference identity is not
replaced by content equality just because an object has fields. The ordinary shared
value fallback remains separate from user-declared EquatableTo contracts. Missing
backend support is a gap, never an alternative equality policy.

| Reviewed family | Finding and allocation assessment | Evidence / remaining scope |
| --- | --- | --- |
| Interfaces and static services | Interfaces specify behavior; comparer Equals/GetHashCode operate on arguments, not the comparer object's identity. Static crypto, console and runtime services have no instance value identity to implement. | All declarations inventoried; provider behavior is not execution-qualified by this check. |
| Mutable collections, arrays, builders and owner/service classes | Inherited Object equality/hash is appropriate for mutable owners; mutation must not change identity hash. StringBuilder's override returns a cached immutable snapshot, with allocations when changed text is materialized. | [Family consumer](../experiments/type-contracts/README.md), [collections](../experiments/native-collections/README.md), [retention](../experiments/native-collections-gc/README.md). Explicit comparer keys must remain stable; snapshot iteration intentionally allocates. |
| Index, Range and empty error structs | Shared fieldwise/empty-value equality remains; nine hidden display methods were inconsistent and are now overrides. | [Direct/boxed/composite checks](../experiments/value-display/README.md). Range and Index still form intermediate component text; no allocation improvement claimed. |
| Other ordinary structs | CancellationToken compares its source identity through shared defaults, unaffected by cancellation; NetworkDeadline stores one monotonic stamp. HashCode is a mutable accumulator, not a recommended stable key. | [Family consumer](../experiments/type-contracts/README.md) tests token copies/defaults/different sources and GC. Deadline and accumulator reviewed in source; no new typed interfaces synthesized. |
| String and Char | Canonical equality/hash agree; protocol code uses explicit ordinal policies. Original UTF-8 is preserved. | [Canonical model](string-api-review-20261019.md), [Char](../experiments/char-object/README.md). Object/interface adaptation can allocate; a struct containing references is not allocation-free by definition. |
| Numeric primitives and runtime tokens | Int32/Int64/Boolean, floating and Char adapters differ in coverage. [Narrow/unsigned/native-width adapters](../experiments/integer-object/README.md) now reuse the same scalar path and pass interpreter/JIT/AOT. RuntimeTypeHandle is intrinsic, not an empty user struct. | Integral direct/boxed/composite checks pass; numeric formatting/providers and handle boxing need separate qualification. Do not treat empty-looking intrinsic source declarations as empty-value contracts. |
| Enums | Share nominal value boxes and numeric/text services; exact type participates in equality, while hashes can collide between types. | [Equality](../experiments/enum-object/README.md), [display](../experiments/enum-display/README.md). Int32 backing only; explicit format specifiers remain gaps. |
| Records and unions | Generated record equality policies cover qualified declared components; generic records still box values. Union case/payload semantics must not be inferred from an ordinary struct source scan. | [Records](../experiments/record-generics/README.md), [union components](../experiments/record-unions/README.md). Whole-union Object boxing and broader generic formatting remain explicit gaps; not every error union is individually executed here. |
| Slice and Memory | Borrowed views cannot box/escape. Persistent view equality should describe storage region, not mutable contents; current native Memory boxing is explicitly rejected. | `runtime/codegen/src/boxing.rs`, view implementation contracts. Follow-up requires owner/offset/length equality and matching hashes, including empty views; do not infer semantics from fieldless source projections. |
| Uri, Path, HttpMethod and IPAddress | Each authored equality override has a matching hash. Uri/Path use ordinal text; HttpMethod uses ordinal-ignore-case; IPAddress compares address bytes/family. IPv4/IPv6 inherit the base contract. ToString returns stored text where available. | [Family consumer](../experiments/type-contracts/README.md) qualifies Uri/Path. IPAddress formatting currently assembles intermediate strings; a bounded buffer is a candidate, not a measured regression. |
| Date/time and globalization | Date/TimeOfDay/Duration/LocalDateTime and Instant/TimeOffset have matching value hashes. Instant/TimeOffset lack deliberate full-domain display. TimeZone, ZonedDateTime, Calendar and culture objects currently inherit identity. | [Time values](../experiments/time-values/README.md), [formatting](../experiments/formattable-culture/README.md). Review value-like classes deliberately; zone rules, aliases and database version make equality by name alone insufficient without a documented contract. |
| Reflection descriptors | Internal runtime descriptors pair every Object equality override with a hash. Type wrappers compare execution handles across their concrete wrappers; hashes use display names (collisions between different types are legal). Member identities use owner/definition or stored identity consistently. | [Reflection execution](../experiments/reflection-execution/README.md). DisplayName hashing may materialize text; caching or runtime identity hashing requires measurement and canonical-handle guarantees. |
| Streams, networking, HTTP, JSON and SQL owners | Mutable/lifecycle objects retain reference identity. A JsonValue class is not automatically structural JSON equality; SQL rows/results and HttpContent are owners/views with separate contracts. Stream capabilities do not imply blocking adapters. | Declaration/override review across every public type; existing domain tests remain evidence for their stated scope. This pass does not certify I/O timing, cancellation, disposal races, serializer throughput or provider correctness. |

The fresh family consumer exposed a concrete build-host defect: synchronous cancellation
retained a timer-cancel service without selecting the invocation-owned scheduler host.
The builder now selects that host from timer bindings too and omits default-queue
draining when no Drain function is compiled. The same source passes interpreter/JIT/AOT.
This is a host dependency fix, not a new asynchronous API or a forced allocation in
ordinary equality. Source and metadata/API snapshot checks complement these execution
checks; none is a substitute for a replacement release's platform gates.
