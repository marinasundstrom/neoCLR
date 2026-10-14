# Source structural union Task delivery

Development experiment, 2026-10-14. Companion Raven branch
`codex/neoclr-union-intersection` emits `Response | HttpError | JsonError`
directly as native signatures. The source checks parameters, returns, local storage,
all three awaited alternatives, UnionTypeInfo constituents and Task generic identity.
There are no generated nominal ad hoc carriers or token substitutions.

Run `python3 docs/experiments/source-structural-unions/verify.py --compiler /path/to/rvnc.dll
--library /path/to/native-library --output target/source-union-verification` with
an experimental-compositions debug interpreter. The matching native library must
include UnionTypeInfo/IntersectionTypeInfo. See results.json for artifact hashes.

The project explicitly disables the temporary generic Wait<T> bridge and uses native
async state machines. Complete uses Promise<T>; generic-method specialization such as
Task.FromResult<union> remains unsupported. This probe covers completed awaits;
existing imported Task probes retain pending/GC evidence. CLI projection, mixed
nested bounds, open/value alternatives and JIT/AOT remain outside this profile.
