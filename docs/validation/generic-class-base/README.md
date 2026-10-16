# Generic class over an ordinary nongeneric base

The SDK fixture GenericClassBaseChecks authors Base with an integer field and
Box<T> with its own T field. One constructor computes its base argument with addition; another passes a
constant. Each then initializes its own storage. Main constructs both Box<int>
forms, upcasts them to Base and sums their fields (20 + 2 + 18 + 2) to 42. SDK/CLR checks additionally preserve object identity and
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

The original AOT path rejected constant/computed base arguments. The updated
consumer reproduced that failure before the fix. Native qualification now admits
an unconditional leading base call preceded by independent argument loads, scalar
constants and supported arithmetic/conversions. Receiver access/duplication,
stores, helper calls and branching before initialization remain excluded. Branches
back into the initialization prefix reject. Those remaining expression forms
are backend gaps, not intended language restrictions. Constructed generic base
authoring and Raven hierarchy emission are also unfinished. Structural work and
Sliceable/Result variance remain deferred.
