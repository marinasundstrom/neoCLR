# Boolean boundaries in the Raven bridge

The bounded importer projects Boolean.CompareTo and handles Raven's Boolean
literals, local stores, parameters, returns, equality and negation. Boolean payloads
can also pass through admitted generic constructors and collection methods.

The [sample](experiments/raven-target/samples/library-booleans.rvn) checks both
Boolean argument positions around an integer argument, mixed literal/value calls,
returned equality, Option<bool> and ArrayList<bool>. It is part of the saved-project
suite. Signatures remain Boolean in metadata: an Int32 member signature does not
become an overload match merely because both use the CLI integer stack category.

[ECMA-335](https://ecma-international.org/publications-and-standards/standards/ecma-335/)
separates metadata/storage types from evaluation-stack categories. neoCLR currently
retains a distinct Boolean stack value. The bridge inserts conversions at admitted
storage, return, equality and call boundaries; it emits small typed call adapters
when a Boolean argument is below another argument. These preserve argument order
and conditional-output declarations rather than rearranging caller evaluation.

This is a bounded projection of Raven's canonical 0/1 Boolean output. An integer
other than 0 or 1 crossing into Boolean faults. It is not a claim of general support
for arbitrary CLI Boolean bit patterns, every Boolean opcode/stack merge or numeric
conversion. Boolean-to-Int32 projection produces 0/1. The runtime's own Boolean
representation and instruction set are unchanged.

This approach avoids changing the interpreter ABI during API projection but adds
adapter overhead. A general CLI loader should normalize stack categories centrally
and revisit these bounded restrictions. It reuses the tradeoff recorded in
[integer storage](integer-types.md) rather than describing this adapter as a new
runtime optimization. Installed SDK/extension packages still require a refresh.
