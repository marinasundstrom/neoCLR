# Collections and queries

Choose a collection by the operations you need. Interfaces describe capabilities:
Sequence adds count and indexed reads, MutableSequence adds replacement, and List
adds growth. A read-only view can still observe changes through another alias.

## Choose a collection

| Need | Implementation | Contract |
| --- | --- | --- |
| Grow an ordered list | ArrayList | List / Sequence |
| Look up values by key | HashMap | MutableMap / Map |
| Keep unique elements | HashSet | MutableSet / Set |
| First-in, first-out work | ArrayQueue | Queue |
| Last-in, first-out work | ArrayStack | Stack |

These are ordinary mutable collections, not concurrent collections. Struct elements
are copied; class elements retain shared identity. Collection constructors accept
iterables for initialization or copying where the contract provides that overload.

## Use capabilities

```raven
{{COLLECTION_SAMPLE}}
```

The complete sample reads arrays and lists, replaces elements through MutableSequence
and grows a List. [Download it](/samples/library-collection-capabilities.rvn).

## Filter and transform

Import the query operators to compose iterable work:

```raven
{{QUERY_NAMES_SAMPLE}}
```

Filter and Map correspond to .NET LINQ's Where and Select. Lazy operators defer work
until iteration; ToList and ToMap materialize a new collection. Any and All
short-circuit. First and Last return Option; Single returns a Result that distinguishes
empty and multiple matches.

[Complete query example](/samples/library-query-names.rvn) ·
[Query API](xref:System.Linq)

## Index and slice carefully

Bracket access is a convenience that can fault on invalid indexes or ranges.
Checked Get and Slice methods return Option or Result. ArrayList provides its own
indexing and slice implementation; Sequence access uses the appropriate capability
or its iterator fallback. A slice owns independent array storage with shallow-copied
elements. No concurrent snapshot is promised.

[Arrays](/features/arrays/) · [Index and Range contracts](/docs/sequence-access.html)

## Map keys and comparison

A Map iterates deconstructable KeyValuePair records. MutableMap supports assignment;
reading a missing key through an indexer can fault, so prefer checked lookup when
presence is uncertain. HashMap can initialize from key/value pairs. Selector-based
ToMap builds keys and values from input elements; duplicate behavior is explicit in
the selected API.

Use a comparer whose equality and hash rules agree. For strings, StringComparer.Ordinal
is an exact policy; OrdinalIgnoreCase supplies consistent case-insensitive comparison
and hashing. Avoid changing a key in ways that affect equality while it is stored.

[Collection API](xref:System.Collections) ·
[String comparison](/features/strings/) ·
[Option and Result](/features/outcomes/)

<a id="example"></a>

<a id="query-names"></a>

<a id="basic-operators"></a>

<a id="dotnet-mapping"></a>

<a id="limits"></a>

<a id="direction"></a>

<a id="feedback"></a>

<a id="comparer-policies-development"></a>

<a id="value-tuples-development"></a>


Development direction: structural tuple signatures will replace the nominal tuple
family. The feature branch currently preserves ordered tuple signatures in native
metadata and provides opt-in native interpreter values with positional element
access. Host inspection exposes resolved identities and ordered element descriptors.
Guest `TupleTypeInfo.ElementTypes` exposes ordered recursive snapshots. An explicit
experimental Raven host option emits scalar/nested source tuples natively, including
source-local named reads. Explicit native structural binding now needs no nominal tuple declaration. Native Rust
signatures and host descriptors preserve element names without affecting type
equality. C# authoring/import and Raven emission preserve those names. Guest `ElementNames` exposes per-signature names with positional defaults; names
do not affect equality. Local tuple element mutation supports nested writeback while preserving earlier
copies. Nine-element tuples execute without a nominal carrier. Open-element
emission and migration away from legacy runtime declarations remain pending. An explicit Raven host API
imports named scalar/nested tuples across library boundaries. See the
[tuple API reference](/docs/tuples.md) for the current boundary.

Development branch update (2026-10-15): native Raven projects enable structural
tuples, unions and intersections together, and the default runtime build admits
them in the interpreter. A separate-library sample preserves named tuple copies
and returns 42. JIT/AOT, full legacy tuple removal and broader element forms
remain incomplete; these changes are not yet a published release.

Development follow-up: structural unions are intended for both value and reference
alternatives. The interpreter now has explicit boxed scalar/nominal value storage
and copied extraction; native compiler exact conversions and metadata host authoring now support these bounded alternatives. Direct native ref/out slots now support replacement and reads; generic value-wrapper integration is still incomplete. Intersections
retain nominal interface and extendable-reference bounds.

Development follow-up (2026-10-15): exact native callbacks can return a structural union value; the executable callback sample returns 42. All four source libraries now generate, with full bundle execution still requiring qualification.

Development qualification (2026-10-15): a consumer now compiles and executes HTTP-header checks against the fresh Runtime/Data/Networking/Web bundle. Focused checks cover structural arguments in nominal value wrappers and generic callback substitution; this does not establish all endpoint or backend combinations.

Development qualification (2026-10-15): a fresh-bundle sample awaits `Task<Result<Response, HttpError | JsonError>>` through completed and queued tasks, checking success and both imported error alternatives. Native union extraction from the nominal result carrier is validated; whole-union boxing and final native layout remain incomplete.

Development qualification (2026-10-15): direct `Task<Response | HttpError | JsonError>` and closed `Task<Response<int> | HttpError | JsonError>` samples now execute all alternatives through completed, queued and async-forwarded tasks. Open `Response<T>` union constituents still fail native metadata encoding.
