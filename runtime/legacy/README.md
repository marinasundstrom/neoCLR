# Historical Neo bootstrap carriers

These snapshots preserve the historical Neo error-carrier and BindingFlags ABI from commit
`00c1828a`. The default `runtime/System.neoil` manifest still serves that bootstrap
profile, which has neither the Raven Object/interface profile nor its generated
union protocol. Importing the newly migrated union bodies there breaks loading.

The Raven profile builder replaces these includes with the current normal union
implementations in `runtime/raven`. Do not regenerate these snapshots from the new
union sources, add them to the current API reference, or use them as a convention
for new unions. Remove them when the historical bootstrap profile is retired or
explicitly migrated. No per-case Is*/Get* requirement is restored for Raven.
