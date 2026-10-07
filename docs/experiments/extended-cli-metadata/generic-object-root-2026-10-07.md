# Generic classes over the source Object root — 2026-10-07

Raven **d8c4923b2** enables an explicit target capability for generic reference classes
with the local source-defined Object root. The native adapter preserves this base when
creating the generic definition. Metadata readers/writers retain the existing CLI TypeDef
base token and native named-base relationship; no encoding version or runtime change is
needed. Manual definitions and the builder overload share validation.

The metadata validator now compares full open generic receiver identity during base
construction and admits initialized constructed instances to inherited Object calls.
Missing base construction remains invalid. General generic bases, ordinary local bases
on generic classes and nested derived owners remain unsupported.

Validation:

- **163/163 metadata groups** pass, including manual/builder parity, round-trip base
  identity, canonical introspection, rejection of unsupported bases/missing initialization.
- An API-authored native Box<int> initializes its base, retrieves payload 40 and dispatches
  inherited GetHashCode returning 2; the separate consumer exits 42.
- A Raven-authored Box<int> uses the existing source Object test fixture, retains payload
  42 and dispatches inherited GetHashCode returning 0; its separate API-authored consumer
  exits 42. [Evidence and hashes](generic-object-root-2026-10-07.json).
- **32 focused Raven root/constructor tests pass**, against a pre-change 31-test baseline.
- API snapshot validation passes; no unrelated website build was run.

The fixture is not the production System.Object implementation. The consumer is authored
with the metadata API because ordinary Raven imported-root consumers remain explicitly
unsupported. This proves source emission and runtime execution, not complete bootstrap.

Reproduce with `verify_generic_object_root.py --compiler <rvnc.dll>
--compiler-revision <revision> --core <Core.dll> --seed <System.neox>
--runtime <neoclr> --output <fresh-directory>`. The script records exact compiler commands
and uses a separate empty runtime seed plus explicit `--module`/`--object-root` selection.
The API-only regression can be emitted with the C# test executable's
`--generic-object-root <root-output>` switch; it also writes `<root-output>.app`.

## Full-System frontier

The 194-input full-owned-handle audit clears binding and passes Array<T>. It next stops
at the native enum FlagsAttribute ownership check, which currently identifies the core
through System.Object's owner. With a source Object root, that owner differs from the
explicit primitive bootstrap containing FlagsAttribute. Next validate core attributes
against the selected bootstrap identity rather than infer it from Object.
[Audit evidence](native-bootstrap-generic-root-2026-10-07.json).

Full System still emits no assembly; later errors remain unknown. During regression
construction, `self.value = value` with a same-named constructor parameter reported
RAV0200. That is recorded as an unresolved general binding candidate in Raven's compiler
docs, not included as a fix here. The new fixture uses distinct names; production library
sources are unchanged.
