# Library-based signals

`SignalsDemo` exercises ordinary property exposure, synchronous ordered delivery,
nullable arguments, bound receivers, duplicate subscriptions, idempotent disposal,
empty dispatch, zero-argument handlers and reentrant registration/removal snapshots.

`SignalReflectionDemo` independently checks MethodInfo.FunctionType and the ordinary
parameter/result view of a closed SignalSource method.

`AsyncSignalsDemo` observes a timer-delivered signal using the existing `Promise<int>`
and `await`. A second timer settles the promise with a failure sentinel after five
seconds. This is a test coordination layer: SignalSource still invokes its handlers
synchronously and performs no Task unwrapping. A resettable wait primitive is not
needed for this one-shot observation.

Build each `.rvnproj` using the matching development `rvnc neoclr --project` command.
Run with the matching System runtime seed, System.Runtime and Neo.Signals assemblies.
See [the extension README](../../extensions/Neo.Signals/) for its API and limitations.
