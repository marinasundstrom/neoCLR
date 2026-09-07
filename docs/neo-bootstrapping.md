# Future Neo bootstrapping exercises

Status: possible future exercises, not requirements for the first Neo slice.
Choose additions to the language through small end-to-end workloads before attempting
a library rewrite or self-hosting compiler.

Neo remains a small companion compiler maintained alongside neoCLR for testing and
explanation. Possible self-hosting is not a requirement to turn the current compiler
into a full-fledged language implementation; reassess scope explicitly before that
exercise. See the [near-term slice plan](neo-roadmap.md).

## Port ordinary library code first

Begin with a small function whose behavior is already implemented and tested in neoIL.
Compile its Neo implementation to a normal module, replace the corresponding library
body, and run the same runtime/API tests. Expand only when that workload establishes
a need for more source-language features.

Ordinary algorithms and library contracts are candidates for Neo implementations.
Genuine runtime facilities—allocation, GC, metadata access, native calls and other
host-dependent primitives—still need runtime support. Porting the library must not
introduce special behavior based merely on a method name; retain the validated
InternalCall binding boundary and familiar public contracts.

Record missing features as each port requires them: control flow, collections,
generics, interfaces, output parameters, strings and richer module compilation.
Compare semantics, generated IL and diagnostics with the existing implementation.
Do not require a whole-library rewrite before gaining value from the exercise.

## Later, attempt a Neo-written compiler

A possible staging sequence is:

1. Keep the current Rust compiler as stage 0 and define a bounded Neo subset that
   can express the compiler's parser, binding and lowering.
2. Write a compiler for that subset in Neo and compile it with stage 0.
3. Run the resulting compiler on representative programs and its own source.
4. Rebuild again with the produced compiler; compare observable behavior and,
   where the encoding is deterministic, normalized or byte-identical outputs.

Keep source-language/compiler conformance tests and runtime validation independent.
Successful self-compilation is useful evidence but does not prove semantic correctness.
Stage 0 remains a recovery/debugging path while the exercise is experimental.
Version the language/artifact expectations and retain reproducible commands and
known-good stages rather than silently relying on whichever compiler is on PATH.

The current [Neo subset](neo.md) is not yet large enough for this exercise. Its
[grammar](neo-grammar.md) and generated neoIL provide a small starting point. The
immediate goal remains useful end-to-end programs over the memory-management
foundation, with object-model and interop work added when those programs require it.
