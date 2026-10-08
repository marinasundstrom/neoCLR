# Native value diagnostic members — work in progress

The HTTP error path calls the ordinary Raven StreamError.ToString override. Its
metadata carries a virtual override flag but a direct by-reference value receiver.
After verification of the complete original load set, the private AOT projection
now removes the slot flags from closed value-record ToString overrides. It retains
the original CIL body, borrowed receiver, branch behavior and returned text.

Admission is restricted to the verified instance/byref String-returning no-argument
ToString override on a closed value record without a base. Abstract, generic,
readonly-receiver and reference-class methods do not use this projection. Existing
class dispatch/projections remain separate. Callvirt admission is not broadened,
and general boxed-value display or interface dispatch is not implied. Inspection
records valueDisplayProjections so the private transformation is reviewable.

The [object-model review](../../object-model-review.md) provides the .NET baseline:
values may provide ToString overrides, while reference/boxed dispatch has a distinct
receiver and runtime contract. This restores existing neoCLR direct-call semantics,
not a new API or an improvement over .NET. Preserving the original body avoids a
second formatting implementation; the cost is a bounded projection rather than a
general native object-slot implementation. Metadata and CLI bridge formats do not
change. The interpreter requires no change.

[ValueDisplay.rvn](../../../benchmarks/native-web/ValueDisplay.rvn) checks Closed,
InvalidEncoding and IoFailure descriptions from the real runtime library. The same
artifact passes interpreted and native execution, sanitized adapters and standalone
libSystem-only linkage. [Evidence](../../../benchmarks/native-web/value-display-validation.json)
records the full server advancing to a primitive member-owner admission failure.
Native HTTP remains unqualified. No formatting performance claim is made.
