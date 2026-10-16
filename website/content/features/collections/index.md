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

## Recoverable slicing (development)

Array and ArrayList explicitly implement `Sliceable<T[]>`: `Slice(range)` returns
`Result<T[], InvalidRangeError>`, while their range indexers fault for invalid
bounds. ArrayList resolves boundaries against Count, not Capacity. Results are
independent shallow array copies. The capability does not imply enumeration,
mutation or indexing; a similarly named method alone does not provide conformance.
See the [API contract](../../docs/arrays.html).

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

Use structural types when data needs a shape but not a new nominal declaration.
Tuples combine values, unions represent alternatives, and intersections describe
simultaneous nominal contracts. They remain real types with native metadata and
introspection; interface implementation stays explicit.

The structural feature branches support native Raven tuples with ordered elements,
value copies and local mutation. Element names are stored in the signature and
preserved by reflection, while identity and compatibility remain positional.
Native binding requires no nominal tuple carrier. Development runtime libraries
now remove `System.Tuple` and `System.Union`; rebuild consumers with the matching
compiler. `Range` returns a named structural offset/length pair.

Structural unions support both value and reference alternatives, including scoped
nominal constituents such as `Response<T> | HttpError | JsonError`. Tested task
consumers deliver all alternatives with `Response` and `Response<int>`; a generic
method with `Response<T>` also executes. Copying a reference payload preserves its
object identity. Arrays retain their structural runtime shape and nominal
`Array<T>` representation.

This work requires matching feature-branch artifacts. Interpreter execution is
validated, and bounded tuple storage also runs through macOS ARM64 AOT using inline
slots without a heap carrier. Bounded intersection views also have AOT checks and
retain one object reference. Native reference-alternative unions likewise copy a
payload reference without a carrier while remaining structural value types. Native
value alternatives and broader AOT/JIT support are still being developed;
the temporary boxed union storage is not the intended final performance model.
Bare generic-parameter union alternatives and unrestricted combinations remain
incomplete.

[Structural families](../../docs/structural-types.html) ·
[Tuple contract](../../docs/tuples.html) · [Union contract](../../docs/unions.html) ·
[Intersection contract](../../docs/intersections.html)

## Interface variance (development)

`Iterable<out T>`, `Iterator<out T>`, `Collection<out T>` and `Sequence<out T>`
permit compatible reference views. AsyncIterable and AsyncIterator also declare
`out T`. `Comparer<in T>` and `EqualityComparer<in T>` consume compatible reference
arguments. A mutable collection can expose a wider read-only interface while its
writable interface stays invariant.

Raven imports and emits these annotations into native metadata. The runtime keeps
nominal conformance explicit; variance does not box elements or widen Result values.
Sliceable, mutable collections and array storage remain invariant. Generic metadata
must remain available independently of native executable specialization.

Development validation also covers local classes inheriting an imported base:
producer, consumer and nested variance calls preserve object identity and inherited
storage in interpreter, AOT and managed JIT. Reverse and unrelated-class conversions
remain rejected. See the [variance reference](/docs/interface-variance.html).

Interface-valued arguments can also widen to Object in the declared variance
direction; native producer and consumer paths preserve reference identity.

Array references are also supported as variant arguments: Producer<int[]> can
widen to Producer<Object>, and Consumer<Object> can narrow to Consumer<int[]>.
The calls retain the same array; element storage remains invariant. Both directions
are qualified in Raven interpreter, AOT and managed JIT.

Array arguments may also widen to their declared Iterable<int> contract: producer
and consumer conversions between int[] and Iterable<int> preserve array identity
and support iteration in interpreter/AOT/JIT. Result and Sliceable remain invariant.

String arguments support the same reference variance: producer views can expose
Object or String's nominal Iterable<char> contract, while consumer views accept
String. Text identity is preserved. Native conversions use the existing tagged
pointer; interpreter conversions use ordinary String view handles without copying
UTF-8 bytes. See the [variance reference](/docs/interface-variance.html) for costs
and limitations.

Variance also follows nominal interfaces inherited through imported class bases.
The metadata writer uses explicit dependency ancestry; it does not infer
conformance from matching members. Result and Sliceable remain invariant.

Generic dependency ancestry substitution is implemented in the metadata SDK and
validated against CLR behavior. Local native generic base signatures and host introspection are now preserved;
external constructed bases are supported by host introspection too. Layout
authoring and Raven execution still need completion; this is not yet an end-to-end supported feature.

SDK-authored generic classes over ordinary nongeneric bases now pass CLR and
native interpreter/AOT/JIT checks, including inherited storage. Constructed generic
bases still need completion. Raven now emits generic classes over local nongeneric bases;
external-base authoring and imported generic derived owners remain open; native base-constructor expressions support scalar constants/arithmetic;
helper calls and control flow in those expressions remain backend gaps.
