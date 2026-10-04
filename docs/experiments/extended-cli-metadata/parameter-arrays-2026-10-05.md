# Native parameter arrays — 2026-10-05

The production ReflectionExtensions uses `params object?[]`. The native emitter now
preserves this call-site fact using the ordinary ParamArrayAttribute and Param tokens,
without changing array storage, calling convention or runtime instruction semantics.
Introspection supplies IsParameterArray to Raven symbols; emission uses symbol facts.
Primitive core and runtime seed must both declare the canonical marker. The checked-in
bootstrap seed and core declarations include it. Application references remain native.

Validation: 151 C# metadata groups pass; the additional `--parameter-arrays <core> <seed>`
mode checks native roundtrip, facade and projection plus malformed targets. The driver
`bootstrap/verify_parameter_arrays.py` compiles a provider, imports only its artifact,
verifies and executes a consumer with empty/expanded/existing-array calls (exit 42).
Evidence: [parameter-array-evidence-2026-10-05.json](parameter-array-evidence-2026-10-05.json).
The production JSON build advances past params declarations; JSON mapping remains open.
The guest API snapshot remains stale from the previously recorded incomplete bridge;
these host C# APIs have the explicit manual reference above. No website build was run.
