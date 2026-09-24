# Object identity and class equality

Development sample for a matching neoCLR bundle:

```sh
python3 verify.py --toolchain-root /path/to/bundle
```

Cell uses Object's default reference equality and identity hash. Mutation through an
alias changes its field without changing its hash. Key overrides equality and hash
using one property: two equal keys remain distinct objects. Calls through Object
exercise virtual dispatch. The verifier checks all twelve labeled output lines.
Hash numbers are deliberately not printed or used as unique IDs.

This is a handwritten class prerequisite for the later Raven record-syntax case,
not a record implementation. Boxed Int32 now compares and hashes its stored value through Object; separate boxes
still have distinct identities and source mutation does not change the copy. String
identity, other boxed virtual equality/hash and static Object.Equals remain unsupported. Raw tests additionally cover
nulls, GC, arrays, explicit base calls and separate box identity.

## Record acceptance history

`RecordProbe.rvn` expresses the same distinction using `record class Key(Number:
int)`, typed/Object Equals, generated operators and hashes. It is deliberately not
part of the passing sample or downloadable archive. To try it, copy this directory
to a scratch location, replace Main.rvn with RecordProbe.rvn and run the same MSBuild
project against a matching bundle. The desired output is `Record identity, equality
and hashes agree`.

Observed 2026-09-23: compilation reaches emission, then fails with `Failed to resolve
EqualityComparer<T>.` Raven record hashing also references System.HashCode.Add<T>
and ToHashCode. These dependencies are not implemented by this Object slice. This
probe is an acceptance target, not a passing test or a promise of record support.

2026-09-24: the integer record-class probe passes with the opt-in target contract.
See [the maintained record sample](../records/README.md) for current scope and checks.


Nullable reference arguments are part of the development Object contract:
Equals(Object? other) and ReferenceEquals(Object? left, Object? right). The sample
checks literal nulls and nullable locals through default equality, an explicit class
override and boxed Int32 equality. Null is still rejected for a non-nullable Object
local (RAV1509), checked by verify.py. Reference annotations are Raven compatibility
metadata; runtime types/slots and identity semantics are unchanged.
