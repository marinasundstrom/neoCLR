# Sample assessment after ref/out metadata support — 2026-10-02

The small Raven-to-native execution path still works. The broad collections application
still cannot emit: Raven has not yet connected its synthesized output locals and calls
to the metadata API's new ref/out contracts. Next work should connect that shared
emission path and its target capability, then address imported value receivers as the
unchanged sample exposes them. More standalone metadata features alone will not advance
this sample.

## Evidence and scope

The rebuilt Raven probe uses compiler `532fc049f` and metadata `1c284c94` on their
respective integration branches. No Raven source or sample was changed for this run.
The existing runtime binary and reference snapshot hashes are recorded in each report.
This is a development assessment, not a release or a full compiler regression suite.

- [Inventory](readiness-after-out-2026-10-02.json): 31 attempts, comprising nine library
  groups in host and target configurations, twelve existing target applications, and
  the whole 167-file runtime library. Four library attempts emit and verify; eighteen
  stop at emission and nine at binding. No failed-emission application is executed.
- [Propagation workflow](propagation-after-out-2026-10-02.json): an additional unchanged
  existing sample; CLI control emits, native emission stops at object construction.
- [Native controls](native-profile-after-out-2026-10-02.json): all four freshly compiled
  controls verify and execute. Hello World/function, interface dispatch and array cases
  return 42; Unit entry returns 0. Wrong core name/version are rejected.
- The metadata C# producer has 81 passing contract groups; its separate ref/out
  library/consumer binaries verify and execute on neoCLR, with matching CLR execution.
  Those prove API/runtime support, not Raven adoption of that API.

The CLI control uses a declaration-only core and is **not executed**. Native inventory
emission has no registered external library dependencies; a dependency rejection is a
configuration/integration gap, not proof that the runtime lacks the requested method.
Whole-library compilation uses the consumer snapshot, not an implementation bootstrap
seed. Diagnostic counts must not be interpreted as independent defects or completeness
percentages. The inventory records the first rejection, not every later missing feature.

## Current observations

| Sources | Direct native result | CLI control / implication |
| --- | --- | --- |
| Iterator/Iterable/Disposable contracts; Language | Emit and verify in host and target profiles | Target CLI controls also emit. These are library verification checks, not executable apps. |
| Collection and Sequence contracts | Interface declaration admission rejects | Target CLI emits; constructed interface bases/indexers need further admission support. |
| Option and Result source | Source declaration admission rejects | Target binding and CLI emission succeed; union/value declaration emission remains incomplete. |
| ArrayList source | Target binding fails with three errors | Snapshot lacks System.Runtime implementation support. |
| Math and GC source | Target binding fails, including RuntimeServices imports | Need implementation intrinsic/seed declarations before emission can be assessed. |
| application-interfaces | BoundObjectCreationExpression rejected | CLI emits; constructor admission remains a gap in this sample. |
| application-inheritance, application-delegates | Root-class contract admission rejects | CLI emits; broader class/delegate contracts remain unsupported. |
| application-types | Source declaration admission rejects | CLI emits. |
| application-iterable | Generic/root-class admission rejects | CLI emits. |
| application-order-collections | Synthesized uninitialized local admission rejects | CLI emits 7168 bytes; metadata now has ref/out primitives but Raven does not use them yet. |
| library-arrays, library-math, library-strings | Unregistered NeoCLR.CoreProbe dependency | CLI emits; register the correct native library identity and member mapping rather than execute the declaration snapshot. |
| library-option; library-propagation-workflow | BoundObjectCreationExpression rejected | CLI emits; imported/value construction needs admission. |
| library-result | BoundIsPatternExpression rejected | CLI emits; pattern/union lowering remains incomplete on this path. |
| library-async-default-queue | Top-level body admission rejects | CLI emits; async body support remains outside the current subset. |
| Whole runtime library | 700 binding diagnostics; no emission | Missing CheckedStorage/RuntimeServices and mismatched typeof/other target contracts cause cascades. This is not a supported bootstrap configuration. |

All twelve selected applications pass binding and CLI control emission; none reaches
native execution. The new output metadata support therefore must not be reported as
having completed the broad application integration.

## Next bounded work

1. Preserve uninitialized local declarations, local/parameter addresses and ref/out
   call contracts through Raven's portable emission layer. Enable the corresponding
   native capability explicitly and keep ordinary .NET semantics unchanged. Prove a
   Raven source ref/out program on both targets before expanding the broad case.
2. Rerun unchanged application-order-collections; implement the next observed imported
   value-receiver/member requirement without default-initialization workarounds. Keep
   the invalid-propagation fault path and inhabited Unit mapping explicit.
3. Connect exact native System dependency identities for the existing array/math/string
   samples; preserve the temporary CLI snapshot as symbols only. Use these simpler
   consumers alongside the broad collections acceptance case.
4. Treat runtime-library bootstrap as a separate dependency milestone: construct the
   implementation seed with RuntimeServices, CheckedStorage and consistent target
   contracts before using whole-library diagnostics to select codegen work. Then return
   to Collection/Sequence declarations and union/value source emission.

The earlier October 1 assessments remain historical records. Their blanket target-profile
rejection is superseded by current native controls and target library verification;
remaining first-failure observations are captured above without claiming later coverage.


## Subsequent same-day integration

Raven `e6912a285` implements step 1's bounded ref/out path. Five native controls now
pass, including source output forwarding and mutation (42); 64 focused C# tests pass.
The [refreshed unchanged collections case](collections-after-raven-ref-out-2026-10-02.json)
advances to imported value-receiver TryGetOutput invocation admission. The inventory
above remains the pre-integration observation; other rejected samples were not rerun
or declared solved. Step 2 is now the immediate blocker.
