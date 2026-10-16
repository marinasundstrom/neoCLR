# Recoverable arithmetic and checked contexts

Status: research and source-local prototype, 2026-10-16. Public API names,
error shape and capability interfaces are not decided or implemented.

## Current behavior and .NET comparison

Raven's .NET target follows the platform's checked overflow behavior:
[checked integer arithmetic throws OverflowException](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/statements/checked-and-unchecked).
NeoCLR has terminal ArithmeticOverflow faults, not catchable exceptions. Current
checked expressions preserve their scalar result type and use those faults.
The [consumer evidence](../validation/compound-operators/README.md#checked-unary-follow-up)
qualifies that behavior; it does not establish a recoverable arithmetic API.
The author asked whether safe methods returning a success/failure result would
better fit the exception-free platform and requested a Rust comparison.

## Rust comparison

Rust separates policies in its integer methods: checked_add and checked_neg return
Option, strict_add panics on overflow, wrapping_add wraps, saturating_add clamps,
and overflowing_add returns the wrapped value and an overflow flag. checked_div
returns None both for zero divisors and signed minimum divided by minus one.
See the shipped [integer method contracts](https://doc.rust-lang.org/std/primitive.i32.html#method.checked_add),
[division](https://doc.rust-lang.org/std/primitive.i32.html#method.checked_div),
and [overflow flags](https://doc.rust-lang.org/std/primitive.i32.html#method.overflowing_add).
Rust's unsafe unchecked_add is not equivalent to Raven/.NET unchecked wrapping;
overflow violates its safety contract.

## Recommended separation and tradeoffs

Keep faulting checked expressions separate from explicit recoverable methods.
Changing checked(expr) to return Result only on NeoCLR would change typing,
composition and overload behavior across targets. A native fault is also not
identical to Rust panic/unwinding; only the failure policy is comparable.

Option<T> is sufficient when only success matters. Result<T, ArithmeticError>
would preserve reasons such as overflow versus division by zero and compose with
existing Result handling, at the cost of an error taxonomy and wider public API
commitment. Prefer evaluating a consistent Result-based arithmetic family before
settling individual method names. Wrapping and saturating methods are additional
policies, not substitutes for recoverable failure. Future nominal numeric
capabilities should expose explicit semantics; supporting + alone does not imply
recoverable arithmetic support. No interfaces are added by this proposal.

## Implementation and validation requirements

A recoverable method must detect failure before executing a terminal operation;
it cannot catch ArithmeticOverflow and turn it into Result afterward. Managed
prechecks can establish correctness with no new opcode, but may duplicate arithmetic
and branches. Nonfaulting runtime overflow primitives can share interpreter/JIT/AOT
semantics and expose machine overflow flags, at the cost of metadata/codegen work.
Prototype one bounded integer operation and inspect generated code before choosing
that implementation boundary. Do not promise allocation-free results without
measuring the actual nominal Result representation and lowering.

Tests must cover signed/unsigned boundaries, single operand evaluation, zero
division, minimum/-1, and continuation after failure across interpreter/JIT/AOT.
Conversion work also needs NaN, infinity and truncation rules. Compare performance
of successful and failed recoverable operations against explicit prechecks and
existing faulting operations; do not benchmark terminal failures as recovery.
Use ordinary nominal library unions; broader structural work stays parked.

## Author clarification and first prototype

The author confirmed the analogy with indexing: a direct indexer may fault, while
an explicit safe method returns a recoverable result. This establishes the policy
separation, without deciding every arithmetic method name or error type.

Existing `System.Int32.Divide` already returns Result<int, IntegerDivisionError>,
checking zero and minimum/-1 before division. Reuse that precedent when reviewing
a permanent arithmetic family rather than adding a competing division API.

The [source-local addition prototype](../validation/recoverable-arithmetic/README.md)
uses nominal Result<int, ArithmeticError> and two sign-dependent bounds checks.
For positive right operands, MAX-right is representable; for negative right
operands, MIN-right is representable. Only a proven in-range sum reaches addition.
This avoids wider production arithmetic and terminal overflow operations, and
would generalize to signed widths without needing a wider primitive type.
The test oracle deliberately uses widened arithmetic independently.

No public System type or method is added. Naming, harmonizing IntegerDivisionError,
unsigned operations, other widths/operators and optimized nonfaulting intrinsics
remain follow-ups. Before adoption, measure result construction and branch costs;
the disassembly contains nominal constructors and is not evidence of zero heap
allocation or optimized machine-overflow-flag use. No .NET compiler semantics or
Runtime Contract configuration changes are needed for this experiment.
