# Compiler observations outside entry-point adaptation

Both reproducers use an ordinary synchronous Main and an async helper, isolating
these observations from the new startup adapter. Recorded 2026-09-27 with the
matching development compiler/reference used for the entry-result matrix.

- `AsyncParameterCapture.rvn`: import rejects MoveNext at IL_007a because a callback
  capture of the async parameter emits a state-machine receiver where String[] is
  expected. The original Main-specific reproduction failed similarly at IL_0061.
- `AsyncInterpolation.rvn`: compilation and execution return success, but stdout is
  empty instead of `Value 1`. The interpolated Console.WriteLine call is absent
  from the imported method. An ordinary integer WriteLine and explicit String.Concat
  after await execute correctly in the validated consumers.

These are deferred general compiler candidates. Reduce and test them independently
against ordinary CLR metadata before proposing Raven-main fixes; do not integrate
neoCLR policy wholesale. No claim of a fix is made here. The entry examples use
established target operations and test observable output, not compilation alone.

## Current-main reassessment — 2026-09-27

At neoCLR a79d0b34 with the current bridge and matching reference/library,
AsyncInterpolation still compiles and exits 0 with empty output. Inspection of the
raw compiler DLL finds no WriteLine call or `Value ` literal, before neoCLR import.
Treat this as a compiler emission candidate; ordinary .NET reproduction is still
needed to decide general versus target-specific ownership. AsyncParameterCapture
now compiles and runs with exit 0; its historical MoveNext import rejection no
longer reproduces. See the [release assessment](../../../tracking/toolchain-release.md#pre-release-assessment--2026-09-27)
for current checks. A strengthened Main returning the helper Task exits 1 for one
argument, confirming the captured length reaches the result. This does not certify all
async callback/capture shapes.


## Release repair — 2026-09-27

The interpolation member gap is now repaired: the reference and Raven library expose
String.Concat(Object?, Object?), with null-as-empty and virtual ToString conversion.
The original async consumer prints `Value 1`; the focused verifier also checks boxed
numbers/Boolean, custom ToString, nulls and the existing string overload. General Raven
now diagnoses missing Concat overloads (main 42834d791, neoclr 0d5aa83b9). Two reduced
ordinary CLI-reference regressions failed before repair; all eight focused Raven
interpolation/recovery checks pass. No Runtime Contract setting or async lowering
policy changes. See [the String contract](../../../raven-string-api.md#interpolation-runtime-contract--development).
