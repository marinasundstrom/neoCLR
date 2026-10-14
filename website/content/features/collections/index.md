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
Guest `TupleTypeInfo.ElementTypes` exposes ordered recursive snapshots. Raven source
tuples still use the nominal implementation while compiler migration is completed. See the
[tuple API reference](/docs/tuples.md) for the current boundary.
