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

## Borrowed Int32 wrappers

Under --reference-arena, public ordinary nonvirtual Int32 instance wrappers now
project to private free functions with an explicit Int32& receiver. The original
CIL argument indices and body are retained; metadata parameter annotations are
adjusted for the inserted receiver. The internal selection report authorizes only
that projected receiver in the backend. Arbitrary borrowed input parameters remain
unsupported, including a free function with a primitive-looking name. Constructors,
readonly receivers, virtual members and generic wrappers are not admitted here.

This follows the existing primitive ownership/borrow contract rather than adding a
second formatting implementation or copying the receiver. Int32.ToString uses the
existing explicitly bound formatting service. ValueDisplay now checks minimum and
maximum signed values, zero, Equals and nested HttpError Content/UnsuccessfulStatus
rendering. [Evidence](../../../benchmarks/native-web/primitive-display-validation.json)
records interpreter/native parity and standalone linkage. Nine root-layout/selection
unit checks and the negative arbitrary-borrow consumer pass. The full server now
reaches the unbound asynchronous SocketConnectResult service; HTTP execution still
requires task/host-root/socket completion integration.
