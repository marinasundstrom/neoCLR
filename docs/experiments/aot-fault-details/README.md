# Shared host fault diagnostics and native capture (2026-10-07)

The author directs faults to carry code, message and stack trace consistently across
interpreter and native execution. The host receives the fault and chooses how to present
it. A separate utility prints it. An unhandled standalone fault exits with **1**; normal
completion retains the application's exit code. Fault categories are not process exit codes.

## Shared contract

`FaultCode::standard_message()` is the shared runtime-message catalog. UserFault has no
standard message: its supplied UTF-8 text is preserved exactly. `Fault::diagnostic()`
returns a borrowed Rust host view (`code`, `message`, `stack_trace`), whose Display prints
`Code: message` followed by innermost-first frames. Original Fault fields and legacy
Display retain their detailed site-specific diagnostics for compatibility; this view is
the common presentation path. The interpreter CLI now uses it for captured execution
faults. Pre-execution loader/verifier details and existing debugger/host Display remain
compatible; this is not a claim that every legacy host/debugger migrated.
See the [host API reference](../../../api-docs/faults.md).

Both captures use `StackTrace::MAX_FRAMES` (64), with explicit truncation. Native capture
records managed method names and neoIL instruction indices; it excludes synthetic
InternalCall implementation frames, matching the interpreter. These are not machine
addresses or CLI byte offsets. Native source file/line information, structured method
identities and richer symbol display remain future work. The interpreter's richer owned
StackTrace is retained. Raven's current lowered names can appear in the native trace.

## Experimental native ABI

`--fault-details` selects value-profile lowering and exports **neoclr_entry_v3**:

```c
int32_t neoclr_entry_v3(int32_t input, int32_t *result, neoclr_aot_fault *fault);
```

The [header](fault-details.h) defines the caller-owned fault record. Codes retain the
experiment's explicit mapping (0 success, 1 DivideByZero, 2 ArithmeticOverflow,
3 RuntimeError, 4 UserFault), not Rust enum ordinals. Runtime messages are compiled from
the shared catalog; UserFault keeps its supplied message. Text points into immutable
image storage and remains valid while that image is loaded. The host may copy it for
longer ownership. Each invocation resets the record header; only success publishes its
result. The buffers must be aligned, writable and nonoverlapping. Separate buffers allow
concurrent calls without globals or TLS. Capture allocates no heap storage or native
runtime dependency. The checked acyclic call graph bounds execution depth; a separate
capture bound marks traces beyond 64 frames as truncated.

The [C printing utility](render.c) accepts a stream and record. It does not exit the
host; output errors return -1. The [standalone adapter](host.c) prints faults to stderr
and exits 1. Legacy v2 remains available for existing status-only consumers; it is not
the new fault-experience target. ABI v3 is experimental and must not be treated as the
future stable native metadata/hosting ABI.

`--bind-user-fault` requires `--compile-system` and explicit runtime context. It also
enables fault details. After original-scope verification, it binds only exact admitted
InternalCall contracts: legacy `System`'s `neoCLR.Runtime.Fault(String) -> Void`, or the
source-owned `neoCLR.Runtime.Fail(String) -> noresult` in an explicitly supplied module.
The source-owned contract is recognized by the existing runtime too. Managed wrappers
remain selected and compiled normally; managed functions with matching names are not
substituted. Binding reports retain original module/revision/member IDs. Private names
avoid collisions with the bundled verification context without changing source identity.
Native console input remains unsupported.

## Validation

All 86 isolated AOT tests and 13 interpreter fault/stack tests pass. The API snapshot
check passes. The focused tests compare native renderer output exactly with interpreter
`Fault::diagnostic()` for explicit faults, arithmetic and erased-value failures, and the
bound failure service. They check first-fault precedence, unchanged results, reuse after
failure/success, a 128-function call chain truncated to 64 frames with storage canaries,
UTF-8/NUL messages, normal wrappers, native-service identity and configuration rejection.

[The Raven consumer](fail.rvn) forwards a message through two functions to ordinary
System.Fail in the pinned source library. [Recorded evidence](validation.json) compiles
it freshly, compares exact diagnostic text (including message and managed frames) with
the current interpreter CLI, and verifies exit
1 from an otherwise empty directory/environment with only libSystem linked dynamically.
The native image has no unresolved runtime imports. No shared managed framework is used.

```sh
python3 docs/experiments/aot-fault-details/verify.py \
  --compiler /absolute/path/to/rvnc.dll \
  --runtime /absolute/path/to/neoclr \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --bundle /absolute/path/to/neoclr-native-poc \
  --output target/aot-fault-details
```

## Comparison and tradeoffs

.NET [Exception.ToString](https://learn.microsoft.com/en-us/dotnet/api/system.exception.tostring?view=net-10.0)
combines exception type, message, inner exception and available stack trace (primary
reference reviewed 2026-10-07). neoCLR adopts the diagnostic experience of message plus
trace, retaining its terminal-fault model: no guest catch/unwind/cleanup or inner-exception
hierarchy is introduced. Expected errors still use Result. Unlike process termination via
FailFast, an embedding host can keep running after a guest fault.

Caller-owned storage keeps capture predictable and independent of allocation/GC. Its
cost is an explicit lifetime and fixed trace capacity. Capturing logical frames at native
failure propagation avoids an OS unwinder and debug-symbol requirement; it requires each
backend to preserve the same frame and first-fault rules. Shared code messages and parity
tests keep those implementations aligned. The interpreter CLI now uses this presentation for execution faults, with exit 1.
Native input binding is the next slice; broad legacy host/debugger migration remains explicit.
