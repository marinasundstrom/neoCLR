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
their direct base. Components can be Int32, non-null String, or another supported
record class declared in the same compilation. Strings compare by contents without
normalization; nested records use typed Equals and GetHashCode. This is recursive
component comparison, not general graph/collection equality or implicit boxing.
Nullable components, externally compiled record components, record structs and
generic/inherited records remain outside this slice (RAVT004).

The importer recognizes init-only property metadata, checks readonly-field writes,
and supports the integer, string and application-reference output parameters used by generated Deconstruct. Property
initialization restrictions are checked by Raven; runtime reflection over application
properties and a persistent runtime init-only field flag are not provided here.
