# Native primitive wrapper completion — development support

The sample survey exposed Boolean and Int64 member-owner rejections after scalar arrays
were admitted. The original verified load set now projects public ordinary nonvirtual
Boolean, Int64 and UInt64 instance wrappers to private free functions with an explicit
borrowed receiver, extending the existing Int32 mechanism. The original CIL body and
receiver storage remain intact. Static wrappers for these owners also lose their owner
only in the private compilation projection; original identity remains in the report.

Exact receiver type is retained, including signed versus unsigned wide integers.
Parameter contracts shift by one for the explicit receiver. There is no receiver copy,
boxing, substituted CompareTo/ToString implementation or new runtime intrinsic.
Readonly/virtual/abstract/override, interop/internal-call, generic/explicit-interface and
constructor shapes remain outside this projection. Ordinary source access/metadata
verification still precedes lowering. General interface/virtual dispatch is separate.

`primitiveInstanceProjections` reports receiver types and compiled indices; the previous
`int32InstanceProjections` diagnostic field remains for existing evidence consumers.
These are internal experiment reports, not a published stable ABI. No Raven compiler,
Runtime Contract, metadata encoding or public API change is involved.

The next rejection in Int64.CompareTo exposed an admission omission: relational and
comparison-branch instructions assumed Int32 even though native lowering uses the
operand's machine width. Admission now accepts two matching Int32 or Int64/UInt64 stack
operands; opcode signedness determines ordering. Mixed widths remain invalid. Arithmetic
beyond existing wrapping operations is unchanged, so this is not full 64-bit arithmetic
or calendar support.

## Comparison and validation

Reuse the [primitive API](../../raven-primitive-api.md),
[integer types](../../integer-types.md) and [numeric contracts](../../design/numeric-contracts.md)
research. .NET is the ergonomic baseline for primitive comparison and value receiver
behavior. This restores existing interpreter semantics in a bounded native backend;
it is not a claimed advantage or a new language-level primitive model. The private
free-function adaptation avoids duplicate primitive declarations in the backend, at
the cost of explicit projection bookkeeping and limited admitted member shapes.

A 56-case native/interpreter test checks signed/unsigned comparisons and all admitted
comparison branches at high-bit/min/max/equal boundaries. Four synthetic primitive
members write through their receiver to prove the caller's original Boolean, Int32,
Int64 or UInt64 storage changes. Sanitized native results match the interpreter.

[PrimitiveMembers.rvn](../../../benchmarks/native-web/PrimitiveMembers.rvn) checks Boolean
ordering, Int64 extrema/formatting, nested borrowed comparisons, high-bit UInt64 ordering,
static properties and existing Int32 behavior with exact expected output. Interpreted,
sanitized native and standalone runs pass; native cleanup passes and the standalone
image links only libSystem. Run `benchmarks/native-web/verify_callbacks.py` with the
normal tool/bundle arguments and `--case PrimitiveMembers` to reproduce.

The unchanged library-booleans and library-generic-collections samples now also pass.
Calendar advances to a wide arithmetic rejection, which remains deferred.
[Raw follow-up and test hashes](../aot-sample-assessment/primitive-member-followup.json).
No performance benchmark is needed or speedup claimed for this admission/correctness fix.


## Division/remainder follow-up

The next calendar boundary now admits matching Int32 or Int64/UInt64 operands for
signed/unsigned division and remainder. The shared emitter chooses the signed minimum
from the machine operand width before checking minimum/-1, preserving the interpreter's
overflow fault for both division and remainder. Zero checks precede machine operations;
checked add/subtract/multiply remain at their prior admission scope. This reuses the
integer research above and preserves the existing neoCLR behavior rather than adopting
an independent AOT arithmetic policy. Seventy-two cross-mode cases pass with sanitized
native adapters, atomic fault results and frame/heap cleanup. The Raven calendar
consumer passes; [time-sample follow-up](../aot-sample-assessment/division-followup.json)
records remaining limits. No performance change is claimed.
