# Records and hash components

Development sample for a matching neoCLR bundle and experimental Raven compiler:

```sh
python3 verify.py --toolchain-root /path/to/bundle
```

Key and Pair cover integer components. Person adds a non-null string, and Entry
contains a Person record. Assignment shares class identity; separately allocated
records compare by their components. The sample checks typed, Object and Equatable
Equals, operators, equal hashes, nested display, deconstruction and independent
HashCode value copies. Numeric hashes are not unique IDs or persistent identifiers.

The opt-in RuntimeRecordContract supports non-generic record classes with Object as
their direct base, and record structs. Components can be Int32, non-null String, or another supported
record class or record struct declared in the same compilation. Strings compare by contents without
normalization; nested records use typed Equals and GetHashCode. This is recursive
component comparison, not general graph/collection equality or implicit boxing.
Nullable string/value components, externally compiled record components and
generic/inherited records remain outside this slice (RAVT004).

The importer recognizes init-only property metadata, checks readonly-field writes,
and supports the integer, string and application-reference output parameters used by generated Deconstruct. Property
initialization restrictions are checked by Raven; runtime reflection over application
properties and a persistent runtime init-only field flag are not provided here.

OptionalEntry models a nullable reference at a boundary: its Owner: Person? can be
null, and OptionalPair exercises mixed/multiple null arguments. Equality, hashes,
display and deconstruction preserve the absent reference. This does not add nullable
strings or nullable integer components. Prefer Option<T> for domain absence; Option
record components remain a separate capability. The verifier also rejects passing
null through the unsupported intrinsic-string argument path.


Coordinate demonstrates record-struct syntax with two integer components. Counter
shows ordinary struct assignment and boxing copying independent payloads. The sample
checks typed, boxed Object and Equatable<Coordinate> equality, distinct box identity,
other-type rejection, matching hashes, virtual display and deconstruction.
Default Coordinate has zero integers; default OwnedCoordinate has a null record
reference. NamedCoordinate checks constructed string components. Struct reference
fields copy references, not the referenced objects. Nullable value boxing and generic struct components are not established here.


`Nested.rvnproj` is a separate small program with Point, Rectangle and Drawing.
It checks nested struct defaults, typed/Object/interface equality, hashing, display,
and independent construction/deconstruction copies. Point deliberately has mutable
properties so the copying behavior is observable. These projects are included in the
website download and run by verify.py. Keeping them separate also stays within the
importer's existing per-application method limit. No limit has been raised.


`Defaults.rvnproj` covers zero-initialized string and class-reference fields, including
non-nullable declarations. Generated equality/hash/display must handle their null
values without dereferencing them. Null contributes zero to HashCode and empty
component text; deconstruction preserves it. The verifier runs all three projects.
Before this repair, the default string hash reached Utf8Encode with null and raised
RuntimeError. This does not widen the direct HashCode.Add(string) contract or admit
explicit nullable-string record declarations.
