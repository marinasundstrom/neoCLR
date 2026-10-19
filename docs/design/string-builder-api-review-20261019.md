# StringBuilder API review — 2026-10-19

Source review at neoCLR 76b72a296, separate from the
[String semantic review](string-api-review-20261019.md). Recommendations are not
implemented changes. Swift is the author-selected model for text semantics;
mutable construction must preserve valid Unicode, grapheme behavior across append
boundaries and immutable snapshots, without copying Swift's storage ABI.

## Current contract

[StringBuilder.rvn](../../runtime/raven/src/System/Text/StringBuilder.rvn) exposes
Append(string), both AppendLine forms, Clear, ToString, Utf8ByteCount and MaxUtf8Bytes.
Appends are fluent. The constructor selects a byte quota, not initial capacity;
the default and maximum are 65536 bytes. Overflow faults before content mutation.
AppendLine uses LF. ToString caches an immutable result until a nonempty append.
Clear releases fragments and preserves the limit. The builder is not thread-safe.

It stores immutable string fragments in a geometrically grown array. Materialization
joins them once and compacts to a one-element array holding the snapshot. This
avoids a growing-string copy for every ordinary append, but has these costs:

| Path | Observed implementation | Candidate follow-up |
| --- | --- | --- |
| Append(text) | Retains text; grows and copies the reference array when full | Measure managed array/GC cost versus owned UTF-8 staging. |
| AppendLine(text) | Preflights total bytes, then allocates Concat(text, LF) | Append two fragments after atomic capacity/size preflight; preserve failure behavior. |
| ToString after mutation | Allocates final text and a one-element array | Retain immutable snapshots but avoid compulsory capacity collapse if measurements support it. |
| Append after ToString | The compact array is full, so the next append regrows it | Repeated snapshots still copy growing text; do not promise amortized linear cost for this pattern. |
| Clear | Allocates a new empty array and releases fragments/snapshot | Consider reuse with explicit reference clearing, or lazy empty storage. Retained capacity trades memory for fewer allocations. |
| Append numeric/Char/formatted data | No direct overloads or writer integration | Add typed output primitives without forcing temporary ToString results or object boxing. |
| Byte-oriented producers | No append view contract | A UTF-8 view append must validate before mutation and copy or retain an owned representation; borrowed Slice cannot be stored. |

The existing [design](string-building.md) documents these choices and
[retained benchmark](../experiments/string-building/README.md#measurement) reports
this fragment builder slower than Concat in that particular whole-process workload.
Those historical timings are not current release or HTTP performance evidence.

## Design recommendation

Keep the builder focused on construction. It need not duplicate String's searching,
comparison, replacement and indexing surface. In particular, byte capacity is not
character count: appending a combining mark can merge with the previous grapheme.
The [Swift text model](https://docs.swift.org/swift-book/LanguageGuide/StringsAndCharacters.html)
provides that semantic constraint. [.NET StringBuilder](https://learn.microsoft.com/en-us/dotnet/api/system.text.stringbuilder?view=net-10.0)
provides useful fluent construction and immutable materialization roles, without
requiring its UTF-16 storage or overload inventory.

Start with the avoidable AppendLine temporary and capacity churn around snapshots
and Clear. Preserve the existing quota-fault atomicity and snapshot independence.
Do not replace the storage implementation solely from source inspection.

Compare two bounded implementations on the same workloads:

- Retained fragments: cheap append of existing strings, fewer immediate text copies,
  but reference-array growth and retention of all fragments until materialization.
- Owned UTF-8 staging: copies bytes on append, can format directly into spare space
  and reduce fragment roots, but growth copies storage and final String must remain
  independent. A chunked variant trades fewer growth copies for more objects and
  more complicated finalization.

Use Memory/Slice only where their ownership roles fit. A builder owns writable
storage, borrows slices during immediate writes, and publishes immutable text.
A returned readonly slice is not permission to mutate its backing while readers
retain it. Capacity transfer or copy-on-write requires a real ownership contract.

Shared typed formatting should target a text/UTF-8 writer abstraction usable by
StringBuilder and stream serializers. A Format convenience can then materialize
once. Raw Stream output is not identical to text formatting: encoding validity,
partial writes, cancellation and quota errors must remain explicit. Avoid making
StringBuilder a compulsory intermediate buffer for streaming HTTP/JSON.

## Validation plan

Retain current [builder consumers](../experiments/string-building/Build.rvn), and
cover exact/zero quotas, rejected line append leaving content intact, previous
snapshots after append/Clear, empty appends, multibyte and combining fragments.
For view input, test nonzero windows, invalid UTF-8, mutation independence and GC.

Measure append-once/materialize-once, append/snapshot alternation, Clear/reuse,
small headers, numeric output and large multibyte reports. Record allocations,
bytes copied, retained roots and elapsed time separately in interpreter/JIT/AOT.
Only then use a selected implementation in HTTP or claim a performance benefit.
No new benchmark or implementation is part of this source review.
