# Value-type async state-machine probe

Development investigation, 2026-09-24. This is not a supported application sample or
an allocation benchmark. Run against a matching development bundle:

```sh
python3 docs/experiments/value-async/probe.py \
  --toolchain-root /path/to/bundle --output /tmp/value-async-evidence
```

The probe builds the same pending-await program in Release with the current heap
policy and with RavenHeapAsyncStateMachines=false. It checks `Pending` then `42`
for executable cases and retains compiler assemblies and build/run logs for inspection.
A blocked value case is reported, not counted as supported; failure of the existing
heap baseline fails the probe. Changing an outer MSBuild command-line property alone
is insufficient here: rvnc reevaluates the project, so the policy is written after
its props import in each temporary project.

Observed with Raven target branch d2a583386 and neoCLR b3b3cfc5:

- The existing heap policy builds, imports and runs the pending continuation.
- Value policy emits a struct extending the target System.ValueType and implementing
  IAsyncStateMachine, but import rejects `Unsupported reference cast`.
- Inspection of the emitted value assembly shows boxing at builder Start and at
  AwaitOnCompleted. The current builder takes IAsyncStateMachine by value in both
  places. Accepting its cast alone would not establish the intended allocation model.
- Raven's six existing HeapAsyncStateMachineTests pass on .NET 11, including default
  struct and opt-in class states, ready/pending awaits, retained locals and GC.
  Those tests do not prove neoCLR execution or a Release allocation benefit.

An initial scratch fixture captured queue in a nested callback and hit the previously
noted `Missing local builder for 'queue'` emission issue under both policies. Moving
Promise construction outside the callback isolates the async storage question; the
shared closure candidate remains deferred, not repaired by this experiment.

See [the assessment](../../async-state-machine-assessment.md#value-state-machine-priority--2026-09-24)
for the next implementation gates. No default, builder ABI or public Task contract
changes in this probe. Do not publish its rejected value case as a working sample.
