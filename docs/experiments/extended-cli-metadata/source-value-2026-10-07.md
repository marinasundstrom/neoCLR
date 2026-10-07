# Source-owned erased Value — 2026-10-07

The Environment failure exposed a representation mismatch: source System.Value
was an ordinary empty struct while retained IsValue/UnpackValue helpers used the
runtime erased carrier. The explicit bootstrap ownership manifest now selects
System.Value's source library. Raven validates its fieldless nongeneric value shape
and marks it with `SetNativePrimitive(PrimitiveType.Value)`.

The CLI signature remains the owning nominal System.Value definition. Native
signatures use the existing Value storage category and Runtime representation.
Compared with CLR object boxing, this is an explicit existing neoCLR erased-carrier
contract; it is not Object inheritance, a new cast rule or arbitrary same-name type
equivalence. Metadata flow recognizes a marked source carrier and the exact selected
core/System seed alias as the same runtime storage. Unmarked local structs do not
receive that treatment. The core reference must remain bound to the explicit System
seed, and its Value facade must be nongeneric, top-level, fieldless and value-shaped.
A selected local Value owner then removes the need for a competing seed declaration.

## Evidence

- [Native consumer](source-value-2026-10-07.json): ordinary Raven emission of source
  Value and real EnvironmentCurrentDirectory internal call, with an explicit
  ownership manifest and minimal seed containing only IsValue/UnpackValue helpers.
  The runtime verifies the artifact and executes a metadata-API-authored consumer
  with exit 42 and no output. It rejects the integer type test, accepts the string
  type test, unpacks the payload and checks nonempty text through the real text
  runtime service. A fielded source carrier rejects before output publication.
- C# metadata groups: 163 pass, including manual/builder Value representation and
  canonical nominal signature round trips, forbidden bare enum signatures and
  invalid record storage. Ownership/unit/profile tests in Raven: 17 pass, including
  wrong owner rejection and keeping Value outside CLI special-type dictionaries.
- [Full audit](native-bootstrap-source-value-2026-10-07.json): 197 inputs, no binding
  errors, no output published. Source Value replaces the seed declaration. The next
  encoding failure is `invalid native array backing storage contract`.
- Compiler snapshot: e24b6b9ae plus the source-owner changes committed in ca4aeccfb.
  neoCLR base: 8abef3a0 plus this slice. Actual binary/input hashes are in the evidence;
  the compiler directory contains the updated metadata DLL. Runtime is unchanged.
- API snapshot check passes. No public guest signature changes or website build.

This is the bounded source bootstrap path. Generic helpers still live in the
explicit retained seed; moving their instruction bodies to source/metadata authoring
and qualifying an ordinary Raven consumer that imports source Value remain separate
work. Full System and complete public Environment execution remain blocked by the
next array contract. Do not treat this focused payload test as full bootstrap.
