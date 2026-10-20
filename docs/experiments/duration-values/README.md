# Duration value contract — 2026-10-20

Duration already compared typed values by signed 100 ns ticks, but lacked Object
value equality and a declared hash. A record containing Duration consequently failed
native admission; generic hashing could fault. [Baseline](baseline.txt) retains the
compiler rejection against the preceding bundle.

Duration now overrides Equals(Object?) and GetHashCode. Both typed and boxed equality
compare the entire signed tick count. Null and other types are unequal. The hash folds
the two 32-bit halves, using the existing integer hash policy; collisions are valid and
hashes are not persistent IDs. Default(Duration) is zero ticks. Representation and ABI
are unchanged. This follows [.NET's elapsed-duration value equality](https://learn.microsoft.com/en-us/dotnet/api/system.timespan.equals?view=net-10.0)
and corresponding hash contract (TimeSpan), without claiming a stable cross-runtime numeric hash.

[Program.rvn](Program.rvn) covers typed/interface/Object calls, distinct boxes, wrong
types, null, zero/default, negative ticks, both Int64 extrema, equal low halves with
different high halves, and an intentional hash collision. It also covers nongeneric,
generic and Object-typed record fields plus imported KeyValuePair keys. [Results](validation.json)
retain interpreter/JIT/AOT execution and artifact hashes. This is a correctness change;
no throughput or zero-allocation claim is made, and no new benchmark is needed.

The record-components MissingHash fixture now uses Date, which still lacks a declared
hash in this cohort; the compiler's separate synthetic missing-hash test remains.
Other value types are being reviewed under the broader release audit, not certified by
this Duration fix. No compiler or runtime backend change is required. The tested compiler
is the retained `4c051e332-dirty-record-policy` binary (source committed as Raven
`8da3a45f8`), with fresh `target/duration-value-libraries` and
`target/duration-value-bundle/bundle`. API snapshots use the matching libraries.
