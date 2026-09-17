# Interface-based introspection migration probe

This is an **isolated executable experiment**, not a replacement System library.
It exercises the [2026-09-17 proposal](../../../introspection-design.md) using the
existing Raven target and runtime without changing their production identities.

The bounded hypothesis is that Raven-authored TypeInfo and MemberInfo interfaces
can describe runtime-backed entities, including a DeclaringType reference returning
the same TypeInfo interface, without exposing the backing class to consumers.

- TypeInfo currently contains only Name and Namespace.
- MemberInfo contains Name and DeclaringType: TypeInfo.
- Internal RuntimeTypeInfo and RuntimeFieldInfo adapters delegate to today's
  runtime descriptors. Their constructors accept legacy types only as transitional
  implementation plumbing within this experimental assembly.
- Main demonstrates interface-typed consumers and a nonpublic field acquired with
  BindingFlags. The existing flags API and filtering semantics are retained.

The importer preserves assembly-scoped identity: these experimental interfaces do
not replace same-named classes from the old reference core. That isolation makes
the probe executable but does **not** solve the production type-identity migration.
No RuntimeContext, invocation, Emit, metadata loading, new equality contract or
complete member model is claimed. Adapters may allocate on access. Current Name
behavior is preserved, including qualified names.

From the repository root, with the bridge built against the Raven feature branch:

```sh
python3 docs/experiments/raven-target/verify_introspection_v1.py \
  --bridge docs/experiments/raven-target/bin/Release/net11.0/Probe.dll \
  --runtime target/debug/neoclr
```

The check compiles the Raven files, verifies the emitted interface shapes and closed
DeclaringType signature, verifies and executes neoIL against the existing System
library, checks output and rejects Invoke and legacy Info access on the new TypeInfo
contract. Existing introspection consumers are tested separately with
`verify_introspection_namespace.py`.

This reuses the existing .NET-inspired descriptor comparison linked from the proposal.
The experiment tests interface dispatch and structural closure, not .NET cross-context
Emit compatibility. No compiler changes or Runtime Contract configuration changes
are needed; general inheritance/provider loading support is not inferred from this
bounded result.

Next: coordinate runtime/reference descriptor identity and type-acquisition lowering,
then extend the minimum contracts and RuntimeContext discovery. Keep BindingFlags
while migrating query operations. The POC must not leak System.Type into the final
public structural model.

The checked library importer now supports nongeneric interface declarations,
validated against the existing Clock reference contract by
`verify_interface_library.py`. This prerequisite is integrated into the Raven
runtime profile; the Info interfaces in this probe still use application-scoped
identities and are not yet production library declarations.
