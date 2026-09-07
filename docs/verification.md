# Typed control-flow verifier

The verifier is an explicit analysis pass over validated, linked metadata. It does
not execute code, load native libraries, instantiate attributes, or allocate guest
objects. The assembler/loader and runtime retain their existing checks; verification
is not yet mandatory for execution.

```sh
cargo run -- verify examples/generic-methods.neoil
cargo run -- verify program.neo.json
```

`check` validates metadata. `verify` additionally analyzes each IL function's reachable
control flow, evaluation-stack types, and definite local initialization. The Rust
APIs are `verify(module)` and `verify_with_library(module, library)`. The first uses
the bundled System library for applications and verifies a System module directly.
The explicit-library API analyzes application and library IL together.

## Implemented checks

- Underflow and typed operands/results for every implemented instruction.
- Exactly one value compatible with the declared return type at reachable ret.
- Equal incoming stack heights and stack types at joins and loop backedges.
- No reachable fallthrough beyond the function body.
- Initialization of a local on every incoming path before ldloc reads it.
- Argument/local stores, call arguments and receiver types, record construction,
  substituted fields, numeric categories, and typed pointer operands.
- Existing metadata checks before analysis, including slot/branch indices, method
  references, and legal targets around unaligned prefixes.

The analysis starts every function with an empty evaluation stack and uninitialized
locals. Parameters are supplied by the call contract. stloc checks storage compatibility
and establishes assignment; merges intersect the initialized-local sets. A worklist
revisits affected instructions when a later incoming path weakens those guarantees.
Constant branch conditions are not evaluated: both conditional successors and switch's
fallthrough are considered. There is no implicit widening or common-base-type inference
at joins; incoming abstract stack types must match exactly.

All IL definitions are analyzed, including unused methods and open methods on generic
types. Void and zero-sized records remain actual stack values. Native declarations
are metadata-validated but have no IL bodies to analyze. Calls resolve the signature
and substitute generic owner arguments before checking their inputs and return value.
An instance receiver in an open generic definition has the symbolic constructed owner,
for example Box<!0>, rather than an uninstantiated Box name.

Fault and ret terminate paths. Infinite loops with stable stack types are permitted;
this analysis does not prove termination. Unreachable instructions still undergo
ordinary metadata validation, but are not checked for stack types or assignment.
Calls are assumed to return one value; there is no interprocedural non-return analysis.

Reports identify each IL definition by its module-local definition identity, index
in the linked module, name, owner,
parameter signature, and instance flag. They contain its maximum reachable stack
height and reachable instruction count. Maximum height is per frame, not a claim
about total process memory or recursion depth. The pass does not rewrite metadata
or change configured runtime limits.

## Storage types and generic normalization

The verifier follows the current interpreter's storage and evaluation-stack rules:
small integers and UInt32 normally load as Int32, UInt64 as Int64, and Single as Double.
Stores check permitted conversions back to the declared storage type. Thus Byte and
Int32 locals can contribute the same stack type to a join without becoming the same
storage type. Nominal record identity includes every generic argument.

A loaded open parameter !n is represented symbolically as its normalized stack form.
It is distinct from a raw stored !n value: loading Byte, for example, changes the
runtime stack type to Int32. The symbolic loaded form can be stored back to the same
parameter or forwarded through a compatible signature. Operations requiring a known
numeric category or inferring a container type from an unresolved normalized parameter
are rejected until a suitable proof/constraint mechanism exists.

Ordinary Option/Result constructors and accessors follow the same signature-based
storage conversions and stack normalization as other members. `heap.load` still
returns its stored payload without normalization, and `heap.store` requires an exact
type. Loading through a typed local can normalize a raw small-integer payload.

## Managed references and output contracts

The call-scoped `T&` subset checks exact slot types, explicit reference receivers
and interface views. Metadata rejects reference locals, fields, returns and native
boundary signatures; runtime guards also reject forbidden reference values without
requiring this optional analysis pass. See [reference contracts](reference-slots.md).

For a directly addressed local, `ldloca` preserves its origin in the abstract stack.
`stobj` establishes initialization, while `ldobj`, interface formation and ordinary
reference arguments require initialization. All ordinary input preconditions are
checked before a call's output promises are applied, including when output and input
arguments alias the same local.

| Callee parameter | Caller-side initialization proof |
| --- | --- |
| T& | The directly addressed local must already be initialized |
| out T& | A normal return establishes initialization |
| out(true) T& | The direct brtrue success target or brfalse fallthrough establishes initialization |

Conditional proof follows the Boolean evaluation-stack value. Storing it in a Boolean
local or comparing it does not preserve that proof. Stack joins require identical
abstract reference origins and conditional-output facts; they can reject otherwise
valid programs. The analysis does not infer arbitrary aliases or prove each callee's
output assignment obligation from its body.

Execution independently checks output assignments on every invocation, including
forwarding and interface dispatch. A preexisting slot value is insufficient: a
successful typed write after entry is required. One write through an alias can satisfy
multiple output references to the same slot. A miss from a conditional-output callee
does not satisfy an enclosing unconditional output promise. Missing writes on a
required normal return produce a Fault in the concrete callee before control returns
to the caller. Interface implementations must match receiver mode and output contracts.

## Deliberate limits

Passing verification is not a memory-safety guarantee or a promise of successful
execution. Pointer validity/lifetime/alignment, memory initialization, active union
cases, allocation sizes, numerical overflow, divide-by-zero, and resource limits
remain runtime checks. For example, a load through a correctly typed null pointer
passes type analysis and Faults at execution. Type-dependent native layout and generic
specialization constraints can still fail. Calls bind definition identities before
specialization so ordinary generic overload collisions do not change their targets.
The pass does not specialize and reanalyze every possible closed generic body.

The analysis is conservative: an unconstrained generic arithmetic operation can be
rejected even when one particular closed instantiation would execute successfully.
Generic constraints and more expressive joins remain separate work. Invalid metadata
is rejected regardless of reachability, but typed analysis applies only to paths
reachable from each function's entry under the conservative branch model.

Whole-value constructor initialization and call-scoped references are implemented.
Partial field initialization, returned/stored managed references, readonly permissions
and broader lifetime analysis remain future work. Making verification mandatory needs
an explicit compatibility choice; current tests can still assemble malformed execution
fixtures deliberately.

Tests cover every shipped IL example without execution, including native-import and
terminal-Fault examples. Negative cases exercise joins, loops, assignment propagation,
returns, calls, receivers, fields, numeric categories, and pointer operands. Generic
normalization, raw bootstrap payloads, small-integer/floating storage conversions,
source/JSON CLI inputs, and dynamic pointer failures are covered as well.
