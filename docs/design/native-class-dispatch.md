# Native class dispatch — development, 2026-10-11

The HttpClient handler pipeline needs cross-assembly class inheritance and virtual
calls in both interpreted and AOT execution. It also needs direct base calls inside
async middleware. These follow CLR semantics: [call selects the declared method](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.call?view=net-10.0),
whereas [callvirt selects the implementation for the receiver and checks null](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.callvirt?view=net-10.0).
Primary contracts checked 2026-10-11. The implementation below is a bounded development
backend, not a claim of general CLR or NativeAOT feature parity.

## Layers and decisions

Raven shared main fixes override lookup and preserves the original receiver through
async lowering. The native compiler separately imports complete nongeneric external
base contracts, including private storage counts. Metadata keeps fields in the
original dependency and uses those counts to place derived fields after base storage.
See [the bridge record](../raven-cli-bridge.md) for compiler revisions and gaps.

The runtime already resolves class slots by verified ancestry. AOT now uses the same
resolver during specialization and reachability selection, retaining base declarations
needed to validate overrides. Selected nongeneric reference-class slots get separate
generated dispatch methods. Their receiver checks run from most-derived types toward
bases; target calls use the original method bodies. Only virtual call sites are
redirected. Direct base calls keep their original target and cannot recurse through
the dispatch method. Caller-side null checks, typed arguments/results and ordinary
GC root planning remain active. Generated dispatch frames are hidden in diagnostics.

This deliberately reuses the existing typed IL and code generation rather than adding
a second native method-table ABI. The benefit is shared verification and root handling;
the cost is additional generated code, a linear receiver-test chain and an extra native
call frame. This is a provisional implementation choice, not a performance improvement
claim. A method-table implementation or dispatch specialization may replace it after
measurement, without changing source semantics or metadata slot identity.

The original load set is verified before private projection. Abstract declarations
receive terminal bodies only in that projection; their virtual callers use generated
dispatch. Artifact metadata remains unchanged. Sealed classes reject derived definitions
throughout the load set, including unreachable ones, matching their CLR meaning.

## Scope and evidence

Public nongeneric top-level external class bases, virtual/abstract methods, inherited
interfaces and direct concrete base calls are supported by the current compiler contract.
Generic/nested bases and protected ordinary methods remain gaps. The AOT profile retains
its existing object-size, function-count and closed-world limits; dynamic loading is not
added. Existing specialized Object and interface dispatch remain separately qualified.

- `class-virtual-calls.neoil` checks overrides, inherited implementations, base/derived
  fields, direct base calls, null receivers and callee-fault stack traces against the
  interpreter on macOS ARM64 with native GC.
- Nearby virtual, inherited interface/Object, sealed-base and runtime class tests cover
  prior paths and rejection behavior.
- `target/http-delegating-validation-v8/report.json`: ten HTTP lifetime/pipeline cases
  pass in both interpreted and macOS ARM64 AOT execution.
- `target/http-delegating-faults-v3/report.json`: eight HTTP guard/fault cases pass in
  both modes. Windows execution remains pending; CI exercises the same HTTP suite.

The retained local reports describe the tested binaries and inputs. Later changes to
diagnostic-frame handling have focused fault-trace qualification; they do not imply
that an earlier report tested a later binary. The website is not deployed by these checks.
