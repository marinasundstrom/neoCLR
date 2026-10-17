# Production bootstrap removal audit — 2026-10-17

The author asked to identify the issues before fixing them in a sweep, and authorized
adding real neoCLR types/attributes needed by Raven and the samples. The initial inventory is complete and bounded migration slices are underway.
This records remaining blockers, not completed native-only production support.
Broader structural work remains parked; ordinary Raven/.NET remains separate.

The author clarified the destination: **Raven targeting neoCLR must load native
metadata references, including its core library, without a CLI bootstrap assembly.**
This applies to compiler binding and editor symbols as well as emitted artifacts.
The audit identifies obstacles to that destination; preserving the bootstrap is
not its goal. Required types/attributes belong in native libraries; compiler-required
operations need explicit native contracts. Raven itself running on .NET does not
require target reference assemblies to use CLI metadata.

## Evidence and boundaries

[Artifact inventory](production-inventory.json) inspects the unchanged four-library
`local-libraries-20261017` bundle and its CLI core, recording hashes. The public
metadata SDK reads the native definitions directly. The installed SDK remains unchanged.

| Artifact | Bootstrap dependency | Confirmed TypeRefs into bootstrap |
| --- | --- | --- |
| System.Runtime | NeoCLR.CoreProbe | System.ParamArrayAttribute |
| System.Data | NeoCLR.CoreProbe | System.ParamArrayAttribute |
| System.Networking | NeoCLR.CoreProbe | None; retained System binding calls IsValue/UnpackValue |
| System.Web | None directly | None; transitive dependencies still matter |

This inventory does **not** prove the library sources can compile without CLI input.
Removing the final emitted reference is a different gate from removing build-time
semantic dependencies. Nor does an absent TypeRef establish that an AssemblyRef is
safe to delete. The [direct-call inventory](retained-call-inventory.json) shows
Networking calls `System.Runtime.CompilerServices.RuntimeServices.IsValue` and
`UnpackValue`; its native module bindings map NeoCLR.CoreProbe to retained `System`.
Runtime also calls retained services for strings, parsing, files, scheduling, type
metadata and other operations. These need native ownership/contracts, not merely
AssemblyRef removal. The inventory records direct `call` instructions only and is
not exhaustive call coverage. It also includes canonical native `System.*` primitive
aliases: their names alone do not establish CLI bootstrap dependence.

## Confirmed migration requirements

| Layer | Present dependency or restriction | Required replacement / acceptance |
| --- | --- | --- |
| Parameter arrays | Native writer imports a marker through the legacy core binding; reader recognizes its legacy System module identity | Native-owned and separately imported marker support, shape validation, parameter introspection, compiler params import and calls, with legacy behavior retained |
| Runtime-service declarations | Native extern admission requires MethodImplAttribute from the selected binding core | Real MethodImplAttribute/MethodImplOptions declarations and an explicit native owner contract; preserve exact InternalCall admission and reject spoofed owners/invalid signatures |
| Self | Production ownership manifest selects NeoCLR.CoreProbe for Self | Native compiler/library Self contract; verify interface implementations and substituted signatures; do not reintroduce nominal delegate or structural feature expansion |
| Generic runtime services | SDK previously rejected generic InternalCall declarations and symbol-authored references could not carry the flag | Preserve generic signature/implementation metadata across SDK and compiler; validate the existing queue services and reject conflicting flags |
| RuntimeServices facade | Production sources extend/import a bootstrap static container | Port needed declarations or retarget adapters to native namespace services; preserve supported public API shapes and ensure no placeholder bodies execute |
| Type handles | Emitter recognizes RuntimeServices.TypeHandle<T>() only through BootstrapReference identity | Native intrinsic contract with the same argument/result validation and runtime descriptor identity |
| Allocation | CheckedStorage.Reserve<T>() is also admitted only through BootstrapReference | Native allocation contract; preserve checked size, element layout and GC behavior; do not replace with unchecked allocation |
| Core production | Native-only project mode rejects source-root/ownership/bootstrap-intrinsic settings used by the production build | Native source-core production path with explicit primitive/Object/unit owners and no CLI semantic reference; no fixture Object stubs |
| Async/generator integration | Bundle injects RuntimeAwait/RuntimeGenerator sources; native-only project mode rejects current helper configuration | Audit actual helper/service requirements, then provide explicit native contracts and validate await/generator consumers; a helper name containing Bridge alone does not prove CLI translation |
| Distribution | Library builder, seed preparation, documentation and editor config still consume the CLI core | Rebuild matching artifacts, API snapshots and SDK configuration only after production compile and consumer gates pass |

Source anchors: Raven `NeoClrRuntimeServiceDeclaration.cs`, `Int32Emitter.cs`
(`IsTypeHandleIntrinsic`, `IsCheckedReservation`, enum marker handling),
`NeoClrProjectMetadataProvider.cs`; neoCLR
`runtime/raven/projects/System.Runtime/ownership.json`, `runtime/raven/native/`,
`scripts/build-native-class-library.py`, and `NativeAssemblyWriter.cs`.

## CLI-only declarations: classify before porting

The inventory lists every declaration present only in the selected CLI core. That
list is a candidate set, **not** a list of required new public APIs.

- Port needed nominal marker contracts deliberately: ParamArrayAttribute,
  MethodImplAttribute/MethodImplOptions, FlagsAttribute and compiler-generated or
  reference-only markers where actual producers/consumers require them. Validate
  constructors, constants, targets and ownership; preserve .NET semantics where used.
- Audit System.ValueType, System.Enum and nongeneric System.Array as compiler base
  assumptions. Existing native primitive/array contracts may need compiler changes
  rather than dummy public classes. System.Array<T> already exists.
- Keep Action/Func/Delegate/MulticastDelegate separate from native function contracts.
  Do not port them merely to satisfy a CLI-oriented lookup.
- Do not add NotImplementedException to satisfy a placeholder body. neoCLR faults
  are not CLR exceptions. Likewise, assess System.Type against the existing TypeInfo
  mapping instead of exposing an empty compatibility type.
- Nullable, IsExternalInit, RefSafetyRules, TupleElementNames and other emission
  markers require a use-site audit. Their presence in the CLI core alone does not
  establish a native need. Tuple expansion remains parked.

## Completed use-site audit

The native emitter consumes MethodImplAttribute for `neoCLR.Runtime` internal calls,
FlagsAttribute for enum flags, and explicit source custom attributes. No direct native
emitter use was found for CompilerGenerated, IsExternalInit, Nullable/NullableContext,
RefSafetyRules, ReferenceAssembly, TopLevel or TupleElementNames transport markers.
Their CLI availability is not a requirement to port them. Nullable annotations and
native namespace functions already have native metadata representations. Shared binder
requirements remain subject to the production compilation gate rather than an assumption
that every CLI type must exist.

`RuntimeAwait.rvn` uses native Task/Promise/TaskOutcome and function types, plus internal
calls authorized by MethodImpl/InternalCall: operation creation, managed start, owner
retention, completion, await, release, status and scheduler submission.
`RuntimeGenerator.rvn` builds on that provider and native async iterator contracts; it
has no additional direct CLI service dependency. These are source adapters, not CLI
metadata references, and need the native service-owner and project-helper contracts.

The entry-point emitter also resolves RuntimeServices to drain pending tasks. Migrate
that lookup alongside TypeHandle and CheckedStorage, not only the source-facing facade.
The retained direct-call evidence confirms the production service categories; changing
a call's ownership requires validating its resolved signature and runtime implementation.
No declaration-only stub or dependency stripping is sufficient.

The inventory is sufficient to start bounded migration. Production compilation and
consumer/editor gates may expose additional shared binder assumptions; those are
validation discoveries, not evidence that the reference migration is complete.

## Proposed order and validation gates

1. Artifact and compiler/helper use-site inventory completed above; retain it as
   the baseline for migration and update it when production validation exposes gaps.
2. Port required attributes and implement native local **and imported** metadata
   handling. Validate rejection paths, parameter expansion and introspection.
3. Replace runtime-service and intrinsic authorization with explicit native owners;
   port required declarations, preserving validation and avoiding placeholder bodies.
4. Complete native source-core production and rebuild Runtime, then Data/Networking/Web.
   Assert all semantic references and transitive artifacts are native, with no
   NeoCLR.CoreProbe dependency. Missing contracts must remain diagnostics.
5. Validate reflection/initialization, params, collections, constrained numerics,
   callbacks and async consumers. Use interpreter/JIT/AOT for changed execution paths.
   Check LSP completion/hover/diagnostics against the production bundle, not only the fixture.
6. Refresh matching API documentation and SDK/VSIX, then install and open the samples.

The sequence may change when the unresolved inventory produces evidence. No blanket
port of every CLI type, no completion-only filtering, and no metadata dependency
stripping are accepted substitutes for the above gates.

## Validated migration progress

- SDK local/imported and symbol-authored parameter-array markers are validated in
  `99eabe63f`/`609fff182`; Raven symbol selection is validated in `a6006e458`.
- Native System.Runtime now owns ParamArrayAttribute. Staged preparation removes its
  CLI and retained-seed duplicates. A fresh five-library documentation bundle builds;
  an expanded-call consumer returns 42 using the fresh four-library bundle.
- Native-only LSP completion includes the marker without CLI delegate leakage. See
  [qualification details](README.md#native-parameter-array-migration).
- The initial duplicate-ownership failure is resolved. Retained runtime-service
  references, intrinsic ownership and source-core production remain open. Neither
  the installed SDK nor production editor configuration has been replaced.

- Generic InternalCall metadata authoring/import (`bdbe3298d`) and Raven native
  symbols (`ff4cff61b`) preserve runtime service identity. Production TaskQueue and
  workers now call native source-owned generic queue services, with their staged
  CLI/seed declarations removed. Registration accepts Raven's no-result ABI; exact
  nominal queue validation remains in place. Other services and native core
  construction are still open.

- Math/environment callers now use native declarations directly; 18 forwarding
  methods are removed. Fresh production compilation, interpreter execution and Math
  LSP completion pass. Native-integer migration is deferred after the consumer found
  unresolved IntPtr/UIntPtr conversion/member binding; its existing adapters remain.

- Native intrinsic ownership is now qualified independently: SDK-authored generic
  Reserve and TypeHandle bodies are imported and called with no BootstrapReference.
  Interpreter/JIT/sanitized AOT pass and negative reservation faults. The existing
  native operations suffice; new compiler interception is unnecessary for that path.
  Production ownership still needs to move when constructing its native core.

- IntPtr/UIntPtr member ownership is fixed by declaring both source types and native
  primitive owners in the production manifest. Their forwards are removed; a fresh
  five-library build, imported comparisons and LSP completion pass. Integral numeric
  casts remain a distinct Raven language/backend gap, not a reason to keep the facade.

- The integral native-integer cast gap is now addressed by a main-first Raven fix
  and explicit neoCLR lowering. Native comparisons and high-bit conversions pass
  interpreter/JIT/AOT; LSP member completion passes. Target-typed native literals
  and full float/decimal conversions remain open. See the conversion gate in README.

- Time-zone and heap public callers now bind existing native namespace declarations
  directly; 13 forwards and their source-list entries are removed. A fresh five-library
  bundle, interpreter consumer and TimeZone/GC completion pass. Payload services,
  other forwards and production core construction remain open.

- Console, file-text and storage callers now bind native namespace declarations
  directly; 12 forwards are removed. Five-library compilation, interpreter input/
  output and filesystem consumer, plus Console/FileText completion pass. IsValue/
  UnpackValue and other retained core services remain explicit migration dependencies.

- Object identity/equality/hash and string joining now call native declarations
  directly. Four adapters are removed; fresh five-library compilation, interpreter
  consumer and String.Join completion pass. No public contract changes.

- Native-only IsValue<T>/UnpackValue<T> ownership now passes compilation, interpreter
  membership/extraction/EOF and wrong-type rejection, and LSP completion. No compiler
  intrinsic interception is needed for these imported bodies. Production owner
  migration and native backend payload qualification remain open.

- Native-owned byte/Int32/unit payload checks and wrong-type extraction now pass
  interpreter, managed JIT and sanitized AOT, plus native-only project compilation
  and LSP completion. The production blocker is definition/body construction: native
  System.Value exists, but RuntimeServices helper declarations/implementations remain
  CLI/seed-owned. Their managed opcode bodies cannot simply become InternalCalls.
  See the explicit ownership boundary in README; no production migration is claimed.

- Production payload helper ownership is now migrated: Raven eef6c6e3c emits the
  exact source-owned managed bodies under the explicit System.Value implementation
  contract; staged preparation removes CLI/seed duplicates. Networking has a narrow
  documented friend grant. Five-library compilation, interpreter/JIT/AOT consumption
  and public member completion pass. This supersedes the construction blocker above
  for these two methods only; other core/bootstrap work remains open.

- Networking now calls all 21 native DNS/socket declarations directly; its
  RuntimeServices forwarding extension is removed. Fresh five-library compilation,
  interpreter loopback/deadline and cancellation consumers, Socket completion and
  API fingerprints qualify this slice. Remaining introspection and source-core
  bootstrap work stays open.

- Remove the 49-method introspection facade: callers select 48 native declarations
  and the source-owned RuntimeTypeInfo factory directly. Fresh five-library build,
  field/construction/function interpreter regressions and TypeInfo completion pass.
  Retained type-handle primitives and other core services still need migration;
  removing these forwards does not remove all bootstrap dependencies.
