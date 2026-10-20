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
| Enum Object values | Native Int32-backed enum equality/hash adapters retain exact type and underlying value; enum formatting remains unsupported | [enum qualification](../experiments/enum-object/README.md) |
| Floating Object values | Native Equals/GetHashCode now preserves NaN, signed zero and exact types, including ordinary/generic struct fields | [floating Object qualification](../experiments/floating-object/README.md#native-backend-qualification--2026-10-20) |
| HTTP version | Both components participate in typed/Object equality and hashes; display dispatch matches the reference contract | [HTTP version values](../experiments/http-version-values/README.md) |

These consumers qualify their documented contracts in interpreter/JIT/AOT. They
are not exhaustive tests of every member or every class-library type. Prior evidence
should be reused when its assumptions are unchanged.

## Remaining concrete candidates

- Index/Range and several ordinary error structs declare ToString without override.
  Check direct versus Object/interface dispatch before changing declarations; do
  not conflate display repair with default structural value equality.
- Ordinary structs now have runtime-owned fieldwise defaults; see the
  [qualification](../experiments/struct-defaults/README.md). Extend supported native
  field shapes as their boxed adapters become available; this is not full .NET
  ValueType coverage and does not synthesize EquatableTo or ordinary operators.
- Generic record display, boxed enum formatting and primitive
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
