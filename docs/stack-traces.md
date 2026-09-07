# Stack traces and Fault diagnostics

Status: next implementation priority, recorded after the runtime-service planning slice.
The current Fault stores one function name and instruction index. It does not yet carry
a stack snapshot, and System.Diagnostics.StackTrace/StackFrame are not implemented.

Faults are unrecoverable runtime and system errors. Guest applications cannot catch or
resume them. A host receiving a Fault does not imply rollback or recovery from arbitrary
native failures. Capturing a trace adds diagnostics, not guest exceptions. Recoverable
application errors continue to use Result.

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

Provide familiar ordinary library types named System.Diagnostics.StackTrace and
System.Diagnostics.StackFrame. StackTrace should support explicit current-stack capture,
FrameCount, indexed GetFrame access, and formatting. StackFrame should expose method
identity and available code/source locations. Optional metadata follows neoCLR's Option
conventions rather than using null to mean unavailable information.

Fault formatting uses the snapshot captured at failure. Constructing or explicitly
capturing a StackTrace inside running guest code captures that current stack instead.
It must not capture the later host reporting stack or overwrite the original Fault trace.
The implementation needs a contextual runtime binding: the existing InternalCall registry
currently receives values/output but has no guest-frame capture context.

These are ordinary platform types; the names do not introduce reference semantics,
mandatory GC, or a special union category. The backing representation and snapshot
ownership must be explicit. Do not expose raw interpreter frame pointers or freeze the
Rust frame/vector layout as a guest ABI. Indexed access provides an initial API without
requiring reflection or the final collection API first.

## Proposed slices

1. Add owned logical frame snapshots to interpreter execution Faults, including nested
   calls, generic identities, cancellation, and resource-limit failures. Keep loader and
   host-input faults without invented guest stacks. Bound capture and report truncation.
2. Add artifact-level debug/source mappings and resolve snapshots for debug applications.
   Keep a method/IL fallback for missing or mismatched symbols.
3. Add contextual runtime capture and the StackTrace/StackFrame library surface, once the
   snapshot representation and lifetime contract are concrete. Test current capture
   separately from preserved Fault capture.
4. Define JIT/AOT native-code mappings, inlined logical frames, and interop boundaries.
   No native unwinding, cross-thread capture, or arbitrary foreign-frame recovery is
   implied by the first interpreter implementation.

All execution Fault paths need review, including failures outside an instruction body.
Trace collection must not obscure the original Fault; allocation/resource exhaustion
needs a bounded best-effort diagnostic path. Capturing arguments or local values is
outside this initial contract.
