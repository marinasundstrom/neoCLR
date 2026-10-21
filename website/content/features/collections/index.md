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

<a id="fluent-mutable-access-development"></a>

## Fluent mutable access

`map.Entry(key).AndModify(value => value + 1).OrInsert(2)` modifies an existing
value or inserts the fallback. `OrInsertWith` evaluates its factory only when absent.
The reusable handle resolves current map state on each operation and exposes no
storage reference. `set.GetOrInsert(candidate)` returns the stored equivalent element
or inserts the candidate, preserving stored identity. These extensions target
MutableMap and MutableSet; they add no synchronization or atomicity.

See [member contracts and callback rules](/docs/collection-entries.html).

<a id="recoverable-slicing-development"></a>

## Recoverable slicing

Array ranges now return borrowed `Slice<T>` values; `ToArray` requests a copy.
ArrayList range operations still return owned arrays. The former `Sliceable<TSlice>`
interface has been removed because its Result-based contract does not fit borrowed
views. Growable collections need explicit invalidation rules before exposing them.
See the [view contract](../../docs/memory-views.html).

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


## Structural data composition

Tuples combine values, unions represent alternatives, and intersections describe
simultaneous nominal contracts. These are runtime types with native signatures and
introspection, not implicit interface conformance. Tuple element names are retained,
while compatibility remains positional. Arrays have a structural runtime shape and
a nominal `Array<T>` representation of the same storage.

The integrated implementation covers bounded native forms; further structural work
is parked. Open-generic combinations and broader backend coverage remain incomplete.
Use the individual contracts to check supported storage, calls and reflection before
relying on a composition. Function types have their own [supported API](../functions/).

[Structural families](../../docs/structural-types.html) ·
[Tuple contract](../../docs/tuples.html) · [Union contract](../../docs/unions.html) ·
[Intersection contract](../../docs/intersections.html)

<a id="interface-variance-development"></a>

## Interface variance

`Iterable<out T>`, `Iterator<out T>`, `Collection<out T>` and `Sequence<out T>`
permit compatible reference views. AsyncIterable and AsyncIterator also declare
`out T`. `Comparer<in T>` and `EqualityComparer<in T>` consume compatible reference
arguments. A mutable collection can expose a wider read-only interface while its
writable interface stays invariant.

Raven imports and emits these annotations into native metadata. The runtime keeps
nominal conformance explicit; variance does not box elements or widen Result values.
Mutable collections and array storage remain invariant. Generic metadata
must remain available independently of native executable specialization.

Variance follows explicit nominal interfaces, including those inherited through
imported class bases. Compatible producer and consumer views preserve reference
identity for classes, interfaces, arrays and strings. For example, an array can
widen to its declared Iterable contract without changing its invariant element
storage. Strings can expose Object or `Iterable<Character>` views without copying UTF-8
bytes. Reverse and unrelated conversions are rejected.

See the [variance reference](/docs/interface-variance.html) for conversion rules,
costs and backend limits.

## Generic inheritance and constraints

Generic base signatures preserve their arguments through native metadata and host
introspection. Raven's neoCLR target supports local and external constructed
bases, including reordered parameters, recursive arguments such as
`Node<T> : Base<Node<T>>`, and separately compiled dependencies. Classes remain
invariant; inherited interface conformance comes from explicit nominal contracts.

Inherited storage and generic interface dispatch support value and reference
arguments. Concrete and abstract generic bases support local and imported virtual
slots and direct base calls. Generic virtual methods remain unsupported.
Base-constructor expressions support scalar constants and arithmetic; helper calls
and control flow remain backend gaps.

Native dependency imports preserve reference/value/default-constructor requirements
and nongeneric nominal class bounds. Constrained methods, constructed/interface
bounds and external source-bound authoring have remaining gaps. Result
variance remain postponed.
