# Stack traces and Fault diagnostics

Status: owned logical stack snapshots for interpreter execution Faults are implemented.
The Rust API exposes Fault.stack_trace, StackTrace, StackFrame, and CodeLocation. Guest
System.Diagnostics.StackTrace now provides current capture and printing in the development
native class library; structured StackFrame access remains pending. See the
[owned snapshot contract and validation](experiments/stack-trace/README.md).
The immediate platform milestone emphasizes simple runnable programs, representative
primitive/String APIs, and Error/Fault behavior; richer source diagnostics follow.

Faults are unrecoverable runtime and system errors. Guest applications cannot catch or
resume them. A host receiving a Fault does not imply rollback or recovery from arbitrary
native failures. Capturing a trace adds diagnostics, not guest exceptions. Recoverable
application errors continue to use Result.

## Implemented interpreter snapshot

Execution Faults carry an optional owned StackTrace, innermost first. Each frame retains
a FunctionRef with the selected module/revision/member row, name, closed owner, parameter
signature, and CodeLocation::IlInstruction index. The snapshot survives dropping the
loaded program and contains no arguments, locals, interpreter frame pointers, or guest
allocation handles. Fault Display includes the frames, and the legacy function/instruction
fields identify the innermost location.

Capture is centralized around interpreter execution, including instruction faults,
resource limits, fallthrough, and cancellation. Caller frames retain the call instruction,
not the next instruction after the call. Instruction-budget exhaustion and cancellation
identify the next instruction to execute. Stack-limit checks identify the active frame's
last instruction; after a completed return this is the caller's call site. Pre-cancellation
or a zero execution limit records the requested root at instruction zero. Loading,
verification, root resolution, entry selection, and invalid input values do not fabricate
guest stacks.

Capture keeps up to 64 innermost frames and marks truncation explicitly. The frame vector
uses fallible reservation; failure yields an empty truncated snapshot without replacing
the original Fault. This bounds frame count, not metadata string bytes. Cloning identity
strings/types still uses ordinary Rust allocation, so catastrophic host allocation failure
or native process failure is not guaranteed to yield a trace.

The same snapshot behavior applies in interpreter debug and release builds. There are no
source files/lines yet, and no fake native frames: native-call failures retain the guest
call site. Symbols, inlining, and native unwinding remain separate capabilities. Adding
stack_trace changes Rust Fault struct literals; existing literals need stack_trace: None.
This experimental API change does not alter guest metadata or IL.

`cargo run --example stack_trace` formats a three-frame Fault after dropping the program.
`cargo run -- run examples/stack_trace.neoil` demonstrates the CLI's terminal Fault output.

## Capture first, resolve and format afterward

Capture the active logical guest frames before interpreter teardown loses them. Preserve
innermost-to-outermost ordering and distinguish the failing instruction from caller call
sites. Use module/revision/member identity and the closed declaring type, so overloaded
and generic functions are unambiguous. Current IL positions are instruction indices,
not CLI byte offsets; record the position kind explicitly when additional backends arrive.

Resolve display names and optional source locations from that snapshot and matching debug
metadata. The snapshot must remain usable after frames and the loaded program are dropped;
it must not borrow interpreter frames or retain guest allocations. Record enough owned
identity/display information for a useful fallback when symbols are absent. Source line,
column, and file information is optional and must not be fabricated.

Debug information belongs to the guest artifact/build profile. A debug application must
not depend on the interpreter itself having been compiled as a Rust debug build. Release
artifacts may omit source mappings while retaining useful method/IL locations. The exact
artifact debug-info encoding and retention policy remain to be specified.

## Runtime library API

Development (2026-10-12) implements `StackTrace()` and `ToString()` as an owned text
snapshot of the active logical guest chain. It supports interpreter and the native
root-published JIT/AOT profile. The remaining surface below is still planned.

Provide familiar ordinary library types named System.Diagnostics.StackTrace and
System.Diagnostics.StackFrame. StackTrace should support explicit current-stack capture,
FrameCount, indexed GetFrame access, and formatting. StackFrame should expose method
identity and available code/source locations. Optional metadata follows neoCLR's Optional
conventions rather than using null to mean unavailable information.

Fault formatting uses the snapshot captured at failure. Constructing or explicitly
capturing a StackTrace inside running guest code captures that current stack instead.
It must not capture the later host reporting stack or overwrite the original Fault trace.
The contextual StackTraceCapture binding now captures active frames in the VM/native
root chain. It does not yet expose a structured snapshot or suspended causal parents.

These are ordinary platform types; the names do not introduce reference semantics,
mandatory GC, or a special union category. The backing representation and snapshot
ownership must be explicit. Do not expose raw interpreter frame pointers or freeze the
Rust frame/vector layout as a guest ABI. Indexed access provides an initial API without
requiring reflection or the final collection API first.

## Implementation sequence

1. Implemented: owned logical frame snapshots for interpreter execution Faults, including nested
   calls, generic identities, cancellation, and resource-limit failures. Loader and
   host-input faults have no invented guest stacks. Capture is bounded and reports truncation.
2. Add artifact-level debug/source mappings and resolve snapshots for debug applications.
   Keep a method/IL fallback for missing or mismatched symbols.
3. Add contextual runtime capture and the StackTrace/StackFrame library surface, once the
   snapshot representation and lifetime contract are concrete. Test current capture
   separately from preserved Fault capture.
4. Define JIT/AOT native-code mappings, inlined logical frames, and interop boundaries.
   No native unwinding, cross-thread capture, or arbitrary foreign-frame recovery is
   implied by the first interpreter implementation.

New execution Fault paths must retain capture coverage, including failures outside an
instruction body. Trace collection must not obscure the original Fault; further
allocation/resource-exhaustion hardening remains possible. Capturing arguments or local values is
outside this initial contract.


## Compact fault-name correction (2026-10-30)

The interpreter's `FaultDiagnostic` formatter now applies the existing bounded
`diagnostic_function_name` decoding used by guest stack capture and native
presentation. Reserved valid `NeoMetadata_<64 hex digits>` helper names display
their logical function/type names, so a terminal fault reports `Divide` and `Main`
rather than encoded transport identifiers. Unrecognized, malformed or invalid UTF-8
encodings remain unchanged; no fabricated source locations are introduced.

Only compact host presentation changes. Captured `FunctionRef` names, definition
identities, ordering, instruction indexes and legacy detailed Fault formatting remain
unchanged. Consumers parsing compact frame text should use logical names instead
of relying on temporary metadata helper spellings. Formatting allocates display strings; the existing 64-frame limit does not bound
metadata-name bytes. No fault-path performance claim is made.
The compiler, Runtime Contract configuration, metadata and native ABI are unchanged.
Structured native-origin/debug naming remains the eventual replacement for this
transport fallback. [Release evidence](validation/release-preparation-20261030/README.md)
includes valid/malformed identity controls and the retained Windows native/interpreter
parity case.
