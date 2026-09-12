# Raven Int32 instance methods

The experimental target now exposes the three existing Int32 instance methods:

| Method | Contract |
| --- | --- |
| `Equals(int other)` | Numeric equality |
| `CompareTo(int other)` | -1, 0 or 1 for less than, equal or greater than |
| `ToString()` | Existing decimal integer formatting |

Together with [Parse](raven-parsing-api.md) and [Divide](raven-division-api.md), this
projects all five methods declared directly in the current runtime Int32 source.
It does not imply complete primitive support or general Comparable/Equatable interface
dispatch in Raven. Culture, format-string overloads and inherited Object APIs are not
added by this slice. Inherited metadata members such as GetHashCode can still appear
in completion; that visibility does not imply an executable binding.

## Receiver mapping

The [runtime API design](api-design.md) and
[signature projection](raven-signature-projection.md) comparisons apply. Raven emits
ordinary CLI value-type instance calls using managed addresses. The importer admits
local Int32 receivers and `ldarga` for by-value Int32 parameters. Argument-address
provenance is retained through stack analysis and is admitted only as the receiver
of these three methods; this is not general byref argument or address-escape support.

Adapters pass the address to neoCLR's readonly Equals/CompareTo receivers. ToString
loads the value snapshot expected by its existing runtime method. No boxing, new
opcode, runtime change or Raven compiler change is needed for these concrete calls.
The reference metadata has ordinary nonvirtual methods. This does not implement
Object overrides or constrained generic calls.

## Example and verification

Copy the [integer example](experiments/raven-target/samples/library-integers.rvn)
into `Main.rvn` in a fresh prepared target project, then use
**neoCLR: Run saved project**. See the
[integration setup](experiments/raven-target/README.md).

The saved-project suite checks local and parameter receivers, Int32 extrema,
negative and zero formatting, and comparison/equality results. Signature checks
validate receiver mapping and reject a static/instance mismatch.
`verify_editor.py --parsing` also checks completion on an inferred integer binding.
The SDK and extension are not rebuilt by this source bridge slice.
