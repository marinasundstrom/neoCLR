# String interface variance

The separately compiled library owns Producer<out T> and Consumer<in T>. The
consumer widens Producer<string> to Producer<Object> and Producer<Iterable<char>>,
and narrows the corresponding consumers to Consumer<string>. It checks repeated
result identity, argument identity, UTF-8 grapheme iteration and nullable views.
Rejected.rvn checks reverse covariance/contravariance, an unrelated Iterable<int>
view, value boxing and two unsafe nullability directions. Nominal interface conformance remains explicit.

```sh
python3 docs/validation/interface-variance/strings/run.py \
  --compiler /absolute/path/to/rvnc.dll \
  --bundle /absolute/path/to/native-bundle \
  --output target/string-variance
```

The runner uses one emitted consumer for interpreter, AOT and managed JIT. Native
hosts check faults, roots and collection; AOT enables undefined-behavior/bounds
sanitizers. String iteration uses the existing UTF-8/character bindings and matching
native text archive. Commands and hashes are retained in results.json.

String argument/result adaptation preserves the UTF-8 owner. The interpreter uses
ordinary heap-budgeted String view handles, with no payload copy. Native dispatch
uses the existing null-preserving tagged text pointer and only converts successful
results. No generic metadata encoding or Runtime Contract setting changes.

Validated on 2026-10-16 using Raven codex/neoclr `e17fb32c4`, rebuilt against the
matching SDK changes, and the unchanged `variance-argument-sequence-20261016`
bundle. Interpreter, AOT and managed JIT all return 42; six RAV1503 rejections,
native cleanup and AOT sanitizer checks pass. The nullable compiler correction
was developed on Raven main (`f8679d6f3`) and independently validated on both lines
with 51 conversion tests each. Runtime tests also cover heap-budget failure,
producer faults and implementation-typed writable argument slots.
