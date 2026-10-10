# Guest module-function discovery

Development interpreter consumer, 2026-10-10. Uses the real source-included
NeoClr.Testing.TestAttribute from the runtime test framework. RuntimeContext traverses
the executing assembly's modules, and ModuleInfo.GetFunctions enumerates the two
functions in Example.Tests. Type-owned methods, child namespaces and an empty module
are checked separately. Each method has an absent DeclaringType, static signature,
module owner and readable parameters. Attribute queries compare exact TestAttribute
type identity and read constructor/named descriptions. Test bodies terminate with
Fail if invoked; discovery must not execute them. The parameterized marked function
is intentionally valid metadata but not a runnable framework test signature.

```sh
python3 docs/experiments/guest-functions/verify.py \
  --bundle /path/to/matching/bundle \
  --compiler /path/to/rvnc.dll \
  --runner /path/to/neoclr \
  --output target/guest-functions-check
```

The harness copies inputs, compiles a fresh consumer, verifies output and zero live
GC objects, and records input/compiler/runtime/runner/image hashes. Use a freshly
built library and interpreter containing the matching services. The checked
[local evidence](validation.json) qualifies macOS ARM64 interpreted execution. The
Windows collections workflow runs the same interpreter gate; no Windows result for
this revision is claimed. The public API snapshot is refreshed from the matching
reference bridge; legacy CLI implementation regeneration still has the pre-existing
Map-contract blocker and is not claimed here.

GetFunctions exposes all visibility levels in loaded metadata order, following the
host ModuleInfo.GetFunctions contract. Unlike .NET physical Module.GetMethods, its
scope is the assembly's exact logical namespace; this deliberate namespace model
is documented in the module design. It neither loads assemblies nor grants invocation.
Open generic signatures currently fail explicitly rather than silently disappearing.
Method-level attribute lookup checks the exact module and method token and rejects
missing/ambiguous matches. Parameter/return attribute queries for ownerless functions
remain unsupported. Native enumeration, retention and callable registration are
separate work; existing generated typed test registrations remain the execution path.

Author priority after this discovery slice: make the runner useful through test
filtering before introducing grouping attributes. Existing IDs and display names
provide selection inputs; exact CLI/filter semantics still need focused design and
validation. Manual registration must remain supported.
