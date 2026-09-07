# Ordinary Option and Result inputs

System.Option<T> and System.Result<T,E> use ordinary record validation at the host
boundary. Their private System.Value field contains an explicitly erased ordinary
case record. The VM does not recognize union markers or enforce permitted cases.
See [erased inputs](erased-inputs.md) for shape, depth and complexity limits.

Import validates exact declared record types, field counts and concrete stored payloads
before guest execution. Supported returned carriers can be imported again; cloning
preserves independent value copies. This does not prove constructor invariants.

Pointer and Ref payloads remain unsupported at this boundary. Because an erased field
is checked using its actual payload, Option<Int32*> containing None can be imported,
while a Some case containing a pointer is rejected. Resolution does not inspect an
imaginary inactive alternative. This differs from the removed bootstrap union schema.

`cargo run --locked --example union_inputs` constructs a successful Result<Void,Error>
in guest IL and imports it into another invocation, printing `String("Succeeded")`.
Extraction uses ordinary predicates and checked case accessors. Format 4 removes the
old Value::Union host representation and special union instructions; reassemble sources.
