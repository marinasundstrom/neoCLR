# Path Object contracts

Development validation, 2026-09-24. Run against a matching development bundle:

```sh
python3 docs/experiments/path-object/verify.py --toolchain-root /path/to/bundle
python3 docs/experiments/path-object/verify.py --toolchain-root /path/to/bundle --audit
```

The main fixture checks separate parsed objects, typed and EquatableTo<Path> equality,
Object dispatch, null/wrong-type rejection at the Object boundary, ordinal Unicode
spelling, equal hashes, display, duplicate map keys, replacement, collisions and
rehashing. EquatableTo<int> exercises a value type without Nullable<int>.

EquatableTo<T>.Equals accepts T, not T?. Path's typed overload accepts Path.
Object.Equals(Object?) remains an explicitly null-aware reference boundary; this
does not change generic equality into an absence protocol. Path.Parse reports parse
failure through Result. Reference identity remains separate from value equality.

The optional audit reports RuntimeTypeInfo's current typed/Object behavior instead
of asserting that an inconsistency must remain. It is an investigation of the next
library candidate, not a newly promised reflection contract. Generic Object map keys
currently fail importer admission; the collision fixture uses Path keys and routes
its comparison through an Object-typed function. This limitation remains explicit.

A diagnostic probe found that Raven currently accepts a null literal passed through
EquatableTo<Path>.Equals despite the non-nullable T contract (including explicit
reference metadata annotations). Do not treat that compiler acceptance as a promise
that typed implementations handle null. Generic-argument nullability diagnostics
need a separate Raven investigation; this slice changes neither T into T? nor the
nullable-value policy.

The archived Neo/bootstrap profile lacks the Raven platform's Object and HashCode
classes. Generated Path.bootstrap fragments preserve that older lexical surface;
collection_library.py selects the complete Path implementation for Raven. This is
the same separation already used for String, not a promise of Object dispatch in
the archived profile.
