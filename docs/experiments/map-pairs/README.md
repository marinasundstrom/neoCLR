# Map pair contract

Map<K,V> now extends Iterable<KeyValuePair<K,V>>. The pair uses a positional Raven
record struct; the native development contract supports constructors, copying,
read-only components and Deconstruct. HashMap supplies a shallow snapshot iterator.
The imported-library consumer is in ../native-collections/Main.rvn and exercises
interface dispatch, pair deconstruction, copies and later map updates. Validation
is retained with that consumer.

Historical baseline: compiler 71cafd353 rejected raven/Main.rvn with NEOMETA001 at
the record declaration. The new explicit positional-storage compiler capability
removes that blocker, without claiming full native record support. Generated record
equality, hashing, formatting and init-only updates remain unsupported. See
../../raven-cli-bridge.md and ../../collection-contracts.md for scope and costs.
