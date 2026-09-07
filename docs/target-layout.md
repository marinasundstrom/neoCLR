# Explicit target data layout

TargetLayout describes the scalar inputs used by the existing sequential record layout
engine. It separates layout calculation from the machine hosting the interpreter.
LoadedProgram::layout_of resolves a closed, scoped type and checks root module visibility
before calculating its layout. memory::layout_for is the lower-level equivalent for
already resolved signatures. The original memory::layout delegates to TargetLayout::host().

The descriptor supplies pointer size/alignment and Int64, Single, and Double alignments.
The current subset accepts four- or eight-byte pointers. Scalar alignments must be
nonzero powers of two no larger than the scalar size. Byte/Boolean, Int16/Char, and
Int32 retain fixed 1/2/4-byte sizes and alignments. Void is zero bytes with alignment one.
Native integers and pointer fields use the target pointer size and alignment.

Record packing caps field placement alignment; each nested layout retains its own
alignment and offsets. Minimum size is rounded to the record alignment. Closed generic
field substitution uses the same target recursively. Pointers do not expand pointee
layouts, so recursive pointer-bearing records remain finite. Existing recursion, depth,
complexity, and prototype Int32 size limits still apply. String, Error, Ref, and bootstrap
union byte layouts remain unsupported; this does not prevent their interpreter value use.

Descriptors are explicit numeric inputs, not named CPU profiles or a claim of matching
an entire C ABI. Pointer width alone does not determine Int64/Double alignment. Endianness
does not affect these size/offset calculations; byte encoding and target calling
conventions are separate work. Native code generation and cross-target execution are
not added by this slice.

Querying another target changes no global state. Interpreter sizeof/alignof, allocation,
indirect access, and native interop continue using host layout and host pointer widths.
Non-host results describe foreign storage for planning; they must not be used as native
interpreter allocation or marshalling layouts. The current Layout is a structural report,
not a target-tagged executable allocation capability.

`cargo run --example target_layout` compares the same Packet under explicit four-byte
and eight-byte pointer/alignment choices. Its offsets are [0,4,8] and [0,8,16], with total
sizes 16 and 24 respectively. This calculation performs no allocation or guest execution.
