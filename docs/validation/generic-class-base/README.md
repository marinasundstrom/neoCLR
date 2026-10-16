# Generic class over an ordinary nongeneric base

The SDK fixture GenericClassBaseChecks authors Base with an integer field and
Box<T> with its own T field. The constructor forwards the base argument, then
initializes its own storage. Main constructs Box<int>, upcasts it to Base and
sums both fields to 42. SDK/CLR checks additionally preserve object identity and
validate Box<string>, reject missing base initialization, and validate a producer
conversion through generic-to-base ancestry. Existing generic class/Object-root
checks run alongside the fixture.

```sh
python3 docs/validation/generic-class-base/run.py \
  --bundle /absolute/path/to/native-bundle \
  --output target/generic-class-base
```

Validated on 2026-10-16 against bundle variance-argument-sequence-20261016.
Interpreter, AOT and managed JIT return 42; native fault/root/collection checks
and AOT undefined-behavior/bounds sanitizers pass. Commands and artifact hashes
are retained in results.json. This is an SDK-authored consumer, not a Raven source
hierarchy. No native representation or Runtime Contract change is required.

The CLR check initially returned 2 because the generic field remained zero. Using
an open constructed-owner field reference fixes the inherited-storage offset;
integer and reference payloads now retain their values.

The AOT path currently requires a leading base call with forwarded arguments.
An earlier constant argument reproduced the existing diagnostic “reference
constructor requires an unconditional leading base constructor call with
forwarded arguments”. Constant/computed arguments remain a backend gap, not a
language restriction. Constructed generic base authoring and Raven generic
hierarchy emission are also unfinished. Structural work and Sliceable/Result
variance remain deferred.
