# Projected String call semantics — 2026-10-20

Release review item: native String wrappers checked null at method entry after
both direct `call` and `callvirt` had become static calls. This caused the native
backend to reject a null direct receiver even when the body ignored it, whereas
the interpreter entered that body. Ordinary Raven instance calls already used
`callvirt` and faulted consistently.

The intended repair preserves the original `callvirt` sites in the private backend
projection and performs their null check in the caller. Direct calls retain the
interpreter's existing behavior. Dereferencing a receiver or invoking another
checked operation can still fault inside the body. No public String API or Raven
source nullability rule changes.

## CLR comparison and scope

Microsoft documents [callvirt](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.callvirt?view=net-10.0)
as throwing NullReferenceException for null. Its
[call documentation](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.call?view=net-10.0)
requires a non-null instance reference. Therefore the low-level null direct-call
case here is an execution-mode consistency regression, not a promise of portable
CLR behavior for invalid input or a recommendation for Raven source code.

The alternative of rejecting every direct null receiver in the interpreter would
change existing behavior and add a String-specific entry rule. Preserving the
original call operation instead keeps native lowering faithful to its input. The
cost is a private call-site inventory that must remain aligned with projected
instruction positions. Native receiver-aware code generation should eventually
remove this temporary projection and its inventory.

The runtime/backend owns this change; Runtime Contract configuration, compiler
emission, public metadata and temporary CLI reference signatures remain unchanged.
No Raven main fix is needed. Existing String API documentation stays valid.


## Regression and reproduction

The baseline on neoCLR ba8c77897 fails: the interpreter succeeds for a direct null
receiver whose body returns 42, while AOT exits with a null fault. See
[baseline output](baseline.txt). The first fixture attempt used unsupported Boolean
bitwise-or operands and was corrected to branches before reproducing the runtime
issue; that fixture failure is not counted as evidence of the backend bug.

The expanded `string_instance_projection_preserves_receiver_arguments_and_null_faults`
test covers five modes: non-null virtual byte count, null virtual byte count, null
virtual receiver-independent call, null direct receiver-independent call, and
non-null direct byte count. Interpreter/AOT results and fault diagnostics are
compared; a native JIT host checks all five results/statuses independently. Original
external access to the private helper is still rejected before projection.

```sh
cargo test -p neoclr-codegen --test console \
  string_instance_projection_preserves_receiver_arguments_and_null_faults -- --exact
```

The native integration test runs on macOS ARM64 with Xcode clang and the MacOSX26.2
SDK. It uses the current System seed plus two test-only String instance methods and
one private helper; this is a focused backend regression, not a frozen SDK package
or full source-class-library qualification. No public API signatures change, and
the existing website/API descriptions remain applicable; no site build or
publication is required for this backend-only repair.


Final results: the five-mode regression passes in interpreter, AOT and managed JIT.
The JIT host uses native GC and stack-budget support and verifies that each success
or fault leaves no registered roots or live arena allocation after collection.
The existing String-interface dispatch and Object-display fault regressions also
pass. See [final output](validation.txt), [neighboring regressions](regressions.txt)
and [artifact/source hashes](artifacts.json). The first JIT harness lacked required
GC/stack support; that harness configuration was corrected before the final pass.

The recorded projection discrepancy is closed. Generic record owners, other record
component/formatting gaps and fresh release-package qualification remain separate.
