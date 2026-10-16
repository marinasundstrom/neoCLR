# Recoverable integer arithmetic prototype — 2026-10-16

This source-local experiment follows the author's safe-method/faulting-indexer
analogy. It adds no public System API, runtime instruction, compiler behavior,
Runtime Contract switch or temporary CLI representation. Broader structural work
remains parked. See the [design and Rust/.NET comparison](../../design/checked-arithmetic.md).

`SafeArithmetic.Add(int, int)` returns the existing nominal Result with a local
ArithmeticError.Overflow case. Positive and negative right operands use safe
subtractions to check the bound before adding. Failure returns normally; it never
executes a terminal checked-add instruction. The existing Int32.Divide API supplies
the public precedent and distinguishes DivisionByZero from Overflow.

The consumer checks all 81 pairs drawn from signed minimum, minimum+1, -2^30,
-1, zero, one, 2^30, maximum-1 and maximum against an independent long-sum oracle.
It also checks both operands are evaluated once on failure/success, continuation
after failure, zero division, minimum/-1 overflow and later successful division.
Every native backend returns 42: interpreter, managed JIT and AOT on macOS ARM64.
The AOT host uses undefined-behavior/bounds sanitizers. This is signed Int32
coverage, not qualification of all widths or operators. Normal .NET behavior is
unchanged; this consumer uses a NeoCLR-only existing library API.

## Reproduction and evidence

```sh
python3 docs/validation/compound-operators/run.py \
  --compiler ../Raven/src/Raven.Compiler/bin/Debug/net10.0/rvnc.dll \
  --bundle target/function-reflection-library-validated-20261016 \
  --source docs/validation/recoverable-arithmetic/Main.rvn \
  --output target/recoverable-arithmetic-check
```

Use the matched Raven native compiler `371363a67` and runtime/SDK baseline
`1f1e99af3`. The qualification used neoCLR source `219704fa7` plus this fixture.
`results.json` retains exact commands, successful backend outcomes and hashes of
source, compiler, runtime tools, runner and bundle manifest. The bundle is unchanged.
`addition-disassembly.txt` is the extracted Add body from native metadata
inspection of the exact tested artifact (`neoclr disassemble <artifact> <output>`).
It shows guarded subtractions, comparisons, ordinary addition and nominal result
constructors. Metadata instructions alone do not establish heap allocations or
machine code performance; no allocation-free claim is made.

## Next bounded work

Measure nominal Result construction and inspect final machine code before choosing
managed prechecks versus a nonfaulting overflow primitive. Then propose a coherent
public arithmetic family using the existing division contract, including unsigned
and wider integers. Error naming and capability interfaces remain open. Existing
terminal checked arithmetic remains a separate, scalar-valued operation.
