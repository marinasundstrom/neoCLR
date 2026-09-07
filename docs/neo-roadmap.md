# Upcoming Neo slices

Status: control flow and union matching are implemented on the [Neo foundation](neo.md).
The console calculator remains planned. The [implemented grammar](neo-grammar.md)
tracks shipped syntax only.

Neo is a small companion compiler for testing and explaining neoCLR. Maintain it as
the runtime changes: update affected lowering, runnable examples, tests and grammar
in the same slice. Prefer ordinary CLR-like IL and existing library contracts; add
runtime instructions only when a concrete semantic need justifies the deviation.
The current compiler is not intended to become a complex, full-fledged compiler.
Every addition should make a runtime capability easier to exercise or explain.

## 1. Structured control flow — implemented

Implemented `if`/`else`, `while`, integer-range `for`, and unconditional `loop`, with `break`
and `continue`. Introduce Boolean conditions, primitive comparisons and short-circuit
Boolean operators as needed. Lower these to existing branch instructions.

Keep Raven-inspired range spelling: `0..9` includes both endpoints, while `0..<10`
excludes the upper endpoint. General iteration protocols, custom steps and broader
collection syntax can wait. Specify empty ranges and Int32 boundary behavior before
lowering so the final increment cannot wrap into an unintended infinite loop.

Block names do not escape or shadow active names. Branch return checking is
implemented; loop termination analysis stays conservative. Storage remains frame-lived,
so taking addresses of block-local values is rejected. References to outer locals and
heap objects remain available; deterministic block cleanup is not claimed.

Acceptance: small sum/search programs exercise each construct, both branches,
zero-iteration loops, nested break/continue, and boundary ranges. Verify that the
runtime instruction limit still stops an unbounded loop. Include negative scope,
condition-type and missing-return cases.

## 2. Patterns and union-aware match — implemented

Add match expressions and match statements so callers can handle ordinary union
results explicitly. Begin with case patterns, payload bindings and a wildcard,
covering the library's Option and Result contracts. Add only the closed generic type
and library-call binding support needed to reach those APIs.

Implemented expression spelling:

```text
let value = parsed match {
    Ok(let number) => number,
    Error(let error) => 0
}
```

An expression produces a value with a consistent arm result type; statement arms
perform actions and allow ordinary control flow. Start with exhaustive coverage in
both forms, including a wildcard where needed. Define arm-local bindings, single
evaluation of the scrutinee, duplicate/unreachable cases and safe payload extraction.
Coverage is derived from bundled System union markers, constructors and typed public
accessors. User-declared unions are not yet exposed.
Guards, arbitrary destructuring, subtype patterns and open hierarchy coverage can wait.

Use existing union/library operations where sufficient. Audit their representation
and verifier contracts before deciding whether any new metadata or opcode is needed.
Pattern bindings must obey ordinary copy/reference rules; matching must not silently
box values or allow references into expired scrutinee/arm storage.

Acceptance: handle successful and failed `System.Int32.Parse` results, then nested
Option/Result cases for input and end-of-input. Test non-exhaustive matches, mismatched
expression arm types, invalid payload access and lifetime failures independently of
the happy-path examples.

## 3. A bounded console calculator

Combine input, parsing, matching and loops in one small program: read input, report
recoverable parse errors, calculate a result, repeat, and terminate on end-of-input.
Add only missing string/input and arithmetic support that this program actually
needs. Document one command to run it and automate representative input/output cases.

Use it to check that recoverable union results remain distinct from runtime faults,
and that repeated work respects memory and execution limits. Expand GC monitoring
only if this workload reveals a concrete diagnostic gap.

## Later platform slices

Use the completed programs to select the next runtime capability and add a small Neo
demonstration with it. The [optional object hierarchy](object-hierarchy.md) remains
the next planned platform area after the memory foundation: first establish base
layout/views and tracing of the complete derived allocation, then dispatch and common
library methods as needed. No mandatory Object root; value equality and hashing must
remain independent of whether a value is accessed as T or T&. Reference identity is
a separate operation. Do not add inheritance syntax before those contracts are ready.

[Pinning and native interop](pinning.md) remain a separate bounded design effort once
an interop scenario requires them. [Library ports and compiler bootstrapping](neo-bootstrapping.md)
are possible later exercises, not prerequisites or reasons to grow every compiler
feature now.
