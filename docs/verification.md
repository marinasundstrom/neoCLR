# Control-flow verifier foundation

The verifier is an explicit analysis pass over validated, linked metadata. It does
not execute code, load native libraries, instantiate attributes, or allocate guest
objects. The assembler/loader and runtime retain their existing checks; verification
is not yet mandatory for execution.

```sh
cargo run -- verify examples/generic-methods.neoil
cargo run -- verify program.neo.json
```

`check` validates metadata. `verify` additionally analyzes each IL function's reachable
control flow, evaluation-stack height, and definite local initialization. The Rust
APIs are `verify(module)` and `verify_with_library(module, library)`. The first uses
the bundled System library for applications and verifies a System module directly.
The explicit-library API analyzes application and library IL together.

## Implemented checks

- Underflow for every implemented instruction, including call receivers and fields
  consumed by record construction.
- Exactly one value at each reachable ret, including Void-returning functions.
- Equal incoming stack heights at control-flow joins and loop backedges.
- No reachable fallthrough beyond the function body.
- Initialization of a local on every incoming path before ldloc reads it.
- Existing metadata checks before analysis, including slot/branch indices, method
  references, and legal targets around unaligned prefixes.

The analysis starts every function with an empty evaluation stack and uninitialized
locals. Parameters are supplied by the call contract. stloc establishes assignment;
merges intersect the initialized-local sets. A worklist revisits affected instructions
when a later incoming path weakens those guarantees. Constant branch conditions are
not evaluated: both conditional successors and switch's fallthrough are considered.

All IL definitions are analyzed, including unused methods and open methods on generic
types. Substituting types does not change the number of logical values consumed by
construction or calls. Void and zero-sized records remain actual stack values.
Native declarations are metadata-validated but have no IL bodies to analyze.

Fault and ret terminate paths. Infinite loops with stable stack height are permitted;
this analysis does not prove termination. Unreachable instructions still undergo
ordinary metadata validation, but are not checked for stack underflow or assignment.
Calls are assumed to return one value; there is no interprocedural non-return analysis.

Reports identify each IL definition by its index in the linked module, name, owner,
parameter signature, and instance flag. They contain its maximum reachable stack
height and reachable instruction count. Maximum height is per frame, not a claim
about total process memory or recursion depth. The pass does not rewrite metadata
or change configured runtime limits.

## Deliberate limits

Stack values do not yet carry abstract types. A String and an Int32 at a same-height
join therefore pass this analysis, as can an incorrectly typed return. Arithmetic,
field value types, actual call argument types, and pointer validity remain runtime
checks. Marking a local assigned does not prove that its stored value has the correct
type. This pass must not be described as a full verifier or a memory-safety boundary.

Reference lifetime/permission analysis, partial field initialization, and constructor
verification await their contracts. No checked-byref or mutable-receiver semantics
are introduced. Future typed stack states should build on this control-flow pass,
followed by field/reference initialization and escape analysis as those features
are implemented. Making verification mandatory needs an explicit compatibility
choice; current tests can still assemble malformed execution fixtures deliberately.

The test suite verifies all shipped IL examples without running them, including
native-import and terminal-Fault examples. Negative cases cover branches, switches,
loops, late-arriving uninitialized paths, malformed returns, and underflow. Generic
bodies, empty records, existing metadata checks, prefixes, source and serialized CLI
inputs, and the remaining type-checking limitation are exercised as well.
