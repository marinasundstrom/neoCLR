# Source-owned String integration

The normal native compiler builds the cumulative numeric library plus String,
StringComparison and Utf8SliceError. The checked string-ownership.json assigns String
to Numbers and keeps Char with the runtime seed. verify_source_string.py constructs the
retained seed from checked inputs, removes exactly its String declaration, and compiles
an artifact-only consumer. Required inputs: --compiler, --runtime, --core and --output.
Use the text-service primitive core from the preceding text-service gate.

Raven's host contract now permits an explicit native String provider. Missing providers
never fall back. Source compilation uses the core signature while emission designates
runtime-owned reference storage. Intrinsic m_value reads use the receiver directly;
stores and field addresses reject. Imported primitive methods reached through remaining
bootstrap signatures are rebound to the exact configured provider using semantic facts,
with signature validation. No metadata reader is reopened by the emitter. Ordinary .NET
selection and Reflection/Emit remain unchanged.

Runtime interface dispatch accepts immutable String reference bodies alongside existing
readonly-byref bodies. Interface views unwrap to the original String payload for reference
calls. Canonical member-name resolution uses the validated metadata origin as it does for
numeric primitives. No format-version change. The source equality operators use the
existing exact text equality service, matching the already documented bootstrap API.

The consumer checks grapheme Length, UTF-8 byte count, Unicode casing, containment, equality,
explicit collection Count and grapheme iteration. Char and its ToString remain bootstrap-
owned. Numeric execution and missing/duplicate ownership rejection are controls. This is
not full String/Char dual-target class-library completion. Nullability import, source-owned
Char and broader text class-library expansion remain open.

The text model is Unicode text; UTF-8 is its storage encoding. Unlike .NET char (a UTF-16
code unit), native Char is one grapheme. Unicode scalar APIs still use uint. This slice
preserves those established contracts, not a new semantic divergence.

Validation: 142 metadata groups, 23 runtime interface tests and 19 focused .NET tests pass.
The API snapshot check still reports the recorded stale full-bridge snapshot; the two
source operators already have reference signatures/XML. No website build was needed.

[Recorded commands and hashes](source-string-native-2026-10-04.json).

Matching Raven compiler implementation: `6c5b43ff6`.
