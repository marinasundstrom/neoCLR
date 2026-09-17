# Interface-based introspection migration probe

This is an **isolated executable experiment**, not a replacement System library.
It exercises the [2026-09-17 proposal](../../../introspection-design.md) using the
existing Raven target and runtime without changing their production identities.

The bounded hypothesis is that Raven-authored TypeInfo and MemberInfo interfaces
can describe runtime-backed entities, including a DeclaringType reference returning
the same TypeInfo interface, without exposing the backing class to consumers.

- TypeInfo contains Name, Namespace and GetFields(flags).
- MemberInfo contains Name and DeclaringType: TypeInfo.
- FieldInfo extends MemberInfo with Type: TypeInfo and IsPublic/IsPrivate/IsStatic.
- Internal RuntimeTypeInfo obtains the existing runtime snapshot and wraps its
  descriptions. RuntimeFieldInfo stores only values and shared TypeInfo references;
  consumers never need the legacy FieldInfo class. Type acquisition alone still
  passes a legacy System.Type to RuntimeTypeInfo inside the experiment.
- Main queries a TypeInfo using BindingFlags and passes each FieldInfo to the same
  MemberInfo consumer. Both structural type references stay in the shared model.

The experimental GetFields returns Iterable<FieldInfo> backed by an eagerly populated
ArrayList. This is a bounded demo choice, not the final collection contract or a lazy
metadata query. Each call delegates flags unchanged, then allocates new wrappers and
the list. Runtime ordering and filtering are retained; unsupported bits still fault.

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
library, checks field/declaring type closure, private/public/static/declared filtering,
unsupported flag faults, and rejects Invoke, legacy Info and field GetValue access
on the descriptive contracts. Existing introspection consumers are tested separately with
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

The author requires runtime-owned acquisition: type-of/handle resolution and
RuntimeContext must yield the same or value-equivalent TypeInfo, backed by hidden
Runtime*Info implementations. Main currently constructs internal adapters only
because this is a single-assembly bootstrap probe. That is not a proposed public
constructor API or a completed acquisition design. Production reference metadata
must hide these concrete implementations; identity, equivalence and context lifetime
must be validated before replacing the current runtime type path.

The checked library importer now supports nongeneric interface declarations,
validated against the existing Clock reference contract by
`verify_interface_library.py`. This prerequisite is integrated into the Raven
runtime profile; the Info interfaces in this probe still use application-scoped
identities and are not yet production library declarations.
