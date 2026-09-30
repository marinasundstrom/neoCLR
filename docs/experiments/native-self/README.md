# Generic native Self cloning (development)

`Main.rvn` declares a nongeneric application Clonable contract and calls
`Copy<T>(value: T) -> T where T: Clonable` for both Cell and Box. Mutating the
original Box after cloning leaves the copied Box unchanged. This is a separate
consumer; it does not replace the existing System.Clonable<T> library API.

The Raven Runtime Self Contract settings match the numeric probe. The importer
admits this bounded shape: closed static generic helpers, one exact cloning bound,
a nongeneric application interface with one public abstract Clone method returning
Self, and direct public class/struct implementations returning their own type.
It retains `callself borrow` in native code. Ordinary CLR execution is not claimed.
Erased receiver calls, missing bounds and wrong result implementations are rejected.
Inherited conformances, default clone bodies and general Self method shapes remain
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
and compiler rejection cases on 2026-09-30. Native tests separately prove slot
mutation, null rejection, no boxing and readonly receiver restrictions.
