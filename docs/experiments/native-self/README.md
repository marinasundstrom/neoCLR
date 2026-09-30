# Generic native Self cloning (development)

`Main.rvn` uses the actual nongeneric System.Clonable library contract and calls
`Copy<T>(value: T) -> T where T: Clonable` for both Cell and Box. Mutating the
original Box after cloning leaves the copied Box unchanged. This consumer now
validates the migration of System.Clonable<T> to Clonable.

The Raven Runtime Self Contract settings match the numeric probe. The importer
admits this bounded shape: closed static generic helpers, one exact cloning bound,
System.Clonable (or the bounded equivalent application interface) with one public
abstract Clone method returning Self, and direct public class/struct implementations
returning their own type, plus explicit System.Clonable class mappings.
It retains `callself borrow` in native code. Ordinary CLR execution is not claimed.
Erased receiver calls, missing bounds, the obsolete generic arity and wrong result
implementations are rejected.
The inheritance consumer also checks Base-returning inherited clones, Base virtual
overrides and explicit derived Clonable implementations. Redeclaring Clonable
requires a derived result; an inherited-only conformance cannot satisfy a derived
generic Self bound. Default clone bodies and general Self method shapes remain
outside this importer slice. Runtime receiver rules are in [native Self](../../self-types.md).

Run with freshly built matching runtime, bridge, reference and Raven collection
profile (see the target integration README):

```sh
python3 docs/experiments/native-self/verify.py \
  --runtime target/debug/neoclr \
  --bridge docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --system target/native-self/System.neoil \
  --reference target/native-self/NeoCLR.CoreProbe.dll \
  --evidence docs/experiments/native-self/validation.json
```

[Validation evidence](validation.json) records the successful class/struct consumer
and rejection cases on 2026-09-30. The obsolete generic bound is rejected by the
checked importer; missing bounds, wrong results, erased calls, inherited-only
derived bounds and redeclared base results are compiler diagnostics. Native tests separately prove slot
mutation, null rejection, no boxing and readonly receiver restrictions.

## Integration baseline

This consumer is now qualified against a Self-only extraction on neoCLR main,
with nominal delegates retained. Its project explicitly selects the NeoCLR target.
The paired compiler rejects Self configuration on .NET. Rebuild all artifacts
from this line; a structural Function feature bundle is not interchangeable.
The validation JSON records artifact hashes from this integration run.
