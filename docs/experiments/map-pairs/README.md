# Map pair contract

**Release-check update (2026-10-30):** The historical exclusions below were
superseded by mutable positional records and generated value/reference record
semantics. The collections validator now executes
[record contracts](../native-collections/record-contracts/Main.rvn) in native and
interpreter modes instead of expecting compiler rejection. Imported KeyValuePair
property mutation remains a negative check. See the
[release evidence](../../validation/release-preparation-20261030/README.md).

The remaining text describes the original bounded map-pair checkpoint.

Map<K,V> now extends Iterable<KeyValuePair<K,V>>. The pair uses a positional Raven
record struct; the native development contract supports constructors, copying,
read-only components and Deconstruct. HashMap supplies a shallow snapshot iterator.
The imported-library consumer is in ../native-collections/Main.rvn and exercises
interface dispatch, pair deconstruction, copies and later map updates. Validation
is retained with that consumer.

Historical baseline: compiler 71cafd353 rejected raven/Main.rvn with NEOMETA001 at
the record declaration. The new explicit positional-storage compiler capability
removes that blocker, without claiming full native record support. Generated record
equality, hashing and formatting remain unsupported. The init-accessor follow-up now
supports initialization-only updates; see ../init-accessors/ and
../../raven-cli-bridge.md and ../../collection-contracts.md for scope and costs.
