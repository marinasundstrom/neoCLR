# Number and concrete parsing contracts

Development slice; not part of Preview 10. `Main.rvn` exercises a nested generic
arithmetic/ordering consumer for eight fixed-width integers, Single and Double,
then successful concrete parses (including Byte payloads 1/2), format/range errors
and Boolean parsing. Int32/Int64 share NumberParseError; JSON Int32 conversion
is checked as a dependent consumer. Native `numeric_parse::tests` cover every integer boundary,
full-input grammar precedence, floating special values/rounding/underflow and Boolean
spellings. `static_number_contracts` checks runtime static conformance and rejection.

The numeric importer specializes closed static application functions with Number<T>
constraints and the ten supported primitive arguments. The subsequent
[ordinary helper slice](../generic-helpers/README.md) also admits unconstrained
closed static application methods. It retains the
normal checked import of each resulting body; generated internal helpers carry no
invented source metadata token. Debug mappings refer back to the original method.
User-defined numeric types, generic classes, other constraints and static interface
defaults are separate work. See [design and comparisons](../../design/numeric-contracts.md)
and [general interface direction](../../tracking/runtime-language.md#interfaces-as-a-platform-capability).

Run `verify.py` with a freshly built bridge, runtime, matching generated Raven System
and compiler reference. It executes the consumer and rejects a Boolean numeric
argument and unsupported additional constraints. No website or full platform suite
is needed for this slice.


Recorded results: 15 focused Raven compiler tests, 3 native parser groups,
2 static contract tests, 12 existing default-interface regressions, 4 archived
parser checks and [210 exact signature checks](number-signatures.json). The
[consumer evidence](validation.json) pins sources and artifacts. The updated
[casing/Int64 consumer](../casing-integer/README.md) also validates the shared-error
migration. No full suite or website build was run.
