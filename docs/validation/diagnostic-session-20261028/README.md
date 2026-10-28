# Diagnostic session qualification — 28 October 2026

Five exporter tests pass; native attach lifecycle/configuration tests pass (two
sanitizer-backed tests, including actual CLI session output). The JSON Schema is
checked with jsonschema 4.25.1 in a temporary Python environment; unavailable/fresh/
cached payloads and lossless large integers pass, and an invalid unavailable payload
is rejected. Runtime/client production use needs only the Python standard library.
`checks.json` retains results. No performance or heap-inspection claim is made.

The root-publication investigation starts from a fresh 138-command owned-chain
baseline, including paced/worker and sanitizer configurations, retained in
`root-baseline.json.gz`. It is baseline evidence, not an optimization result.

See the [contract](../../diagnostic-session-format.md) and
[objectives](../../experiments/shared-scheduling/optimization-objectives.md).
