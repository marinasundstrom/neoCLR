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
