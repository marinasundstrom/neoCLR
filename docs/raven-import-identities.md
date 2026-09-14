# Imported application identities

Source experiment, 2026-09-14. This is groundwork for library importing; it does not
admit additional assemblies or generic library bodies. Library migration stays paused.

## Problem and baseline

The application importer previously named types, fields and static functions from
TypeDef/Field/MethodDef row tokens. Inserting an unrelated declaration could change
those names. A separate compilation could not use them as stable symbolic references.

The CLI scopes types through assemblies and uses metadata tokens within modules.
A token is not a cross-build symbol identity. This slice follows that distinction;
it is a correction toward the CLR model, not a claimed advantage over it. See
[ECMA-335, sixth edition](https://ecma-international.org/wp-content/uploads/ECMA-335_6th_edition_june_2012.pdf),
Partitions I §8.2.1 and II §§22–23 (consulted 2026-09-14).

## Current mapping

Application type names now derive from assembly full identity, namespace, name and
nesting. Static functions and value-constructor functions additionally include the
method name, receiver/calling convention, generic arity, return type and ordered
parameter types. Type signatures distinguish named types, closed generic arguments,
vectors, managed byrefs and pointers. Unsupported signature forms still fail.

The text representation uses versioned prefixes and hexadecimal UTF-8 of
length-delimited components. It is reversible and avoids relying on hash collision
probabilities or ambiguous punctuation concatenation. Assembly version changes are
identity changes; this does not introduce assembly unification or type forwarding.

Ordinary field and instance-method names remain readable. Names outside the text
identifier grammar and names starting with the reserved escape prefix are encoded.
Instance methods retain matching slot names across base/derived/interface types;
the existing owner and parameter signature still drive dispatch. Constructors remain
`.ctor`. This does not add return-type-only instance overload support to neoCLR.

The `.map.json` sidecar records the encoding version, assembly identity, original
metadata names and translated type/field/method names. Tokens remain appropriate for
source locations and work queues inside this single imported module. Private generated
adapter names and branch labels remain import-local and may change between builds.
Raven-generated closure/lambda names can also change when source is reorganized;
this slice preserves metadata identity, not source-level identity across compiler renaming.

## Alternatives, costs and boundaries

Keeping row names is compact but unstable. Using bare source names is readable but
loses assembly isolation and risks collisions in nested or generated names. The
reversible qualified encoding is a provisional choice for the text importer: it
produces longer IL and diagnostic names. Sidecar display names help inspection; a
native binary loader should retain structured identities rather than adopt this
spelling as a new public ABI. No runtime opcode or Raven compiler change is required.

Breaking for generated artifacts: rebuild application IL and its source map together.
Installed tools are unchanged. The runtime class library retains its current bindings;
this slice does not replace per-API adapters, remove Void projection, implement
namespace-function metadata contracts or permit loading a separate Raven library.
Those remain in [the target assessment](raven-target-evaluation.md).

## Validation

Build the source bridge, then run:

```sh
dotnet docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll --identity-checks
```

The Cecil fixture serializes and reloads metadata before and after inserting unrelated
types, fields and methods. It confirms all three token rows changed while translated
names stayed stable. Other checks cover TypeRef/TypeDef agreement, assembly names and
versions, overloads, return and byref signatures, closed generic arguments, nesting,
escape-prefix collisions and ordinary virtual slot spelling.

Use `verify_application.py` for execution coverage of inheritance, interfaces,
properties, closures, delegates, value copies and collections. The independent
compiler/import checks in [the compilation workflow](raven-target-compilation.md)
cover Result/Option propagation as well. Calls to closed generic APIs do not prove
that generic declaration bodies can be imported.

Validation at this checkpoint: 12 metadata identity checks, 15 application checks
and five normal compiler/import checks passed, including sidecar identity assertions.

Subsequent checkpoint: [static library importing](raven-library-import.md) now uses
these symbol identities across explicitly supplied assemblies. It replaces token-only
work queues and source-map labels with assembly-qualified records and import-local IDs.
