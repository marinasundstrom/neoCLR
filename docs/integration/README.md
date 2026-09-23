# Raven integration

[Documentation index](../README.md)

Integration and experiment notes describe bounded supported paths; consult each page for limitations and validation.
See also the [Raven experiment](../experiments/raven-target/README.md).

- [Try the application toolchain locally](../raven-application-local-build.md)
- [Raven application types](../raven-application-types.md)
- [Managed array shapes in the Raven target](../raven-array-shapes.md)
- [Raven backend integration map — slice 1](../raven-backend-integration-map.md)
- [Raven binary experiment: CLI container and explicit dependency closure](../raven-binary-profile.md)
- [Boolean boundaries in the Raven bridge](../raven-boolean-api.md)
- [Raven date, time and local clock APIs](../raven-calendar-api.md)
- [Raven integer clamping](../raven-clamp-api.md)
- [Try the collection library with Raven (.12)](../raven-collections-local-build.md)
- [Minimal core reference artifact for Raven](../raven-core-declarations.md)
- [Func and collection predicates from Raven](../raven-delegate-api.md)
- [Raven delegates and lambdas on neoCLR](../raven-delegates-lambdas.md)
- [Raven recoverable integer division](../raven-division-api.md)
- [Raven error-value APIs](../raven-error-api.md)
- [Raven extension methods on neoCLR](../raven-extension-methods.md)
- [Raven text files on neoCLR](../raven-file-api.md)
- [Raven Double Math APIs](../raven-floating-math-api.md)
- [Fundamental interfaces through Raven](../raven-fundamental-interfaces.md)
- [Try the generic-array Raven/neoCLR build](../raven-generic-arrays-local-build.md)
- [Closed collection APIs from Raven](../raven-generic-collections.md)
- [Imported application identities](../raven-import-identities.md)
- [Raven Int32 instance methods](../raven-integer-api.md)
- [Raven-facing library interface contract](../raven-interface-contract.md)
- [Separate Raven libraries](../raven-library-import.md)
- [Raven match forms on neoCLR](../raven-match-matrix.md)
- [Raven minimal target contract: emission evidence](../raven-minimal-target.md)
- [Native memory through Raven](../raven-native-buffer-api.md)
- [Order workflow on neoCLR](../raven-order-workflow.md)
- [Raven integer parsing](../raven-parsing-api.md)
- [Raven lexical path APIs](../raven-path-api.md)
- [Raven/neoCLR fundamental preview acceptance](../raven-preview-acceptance.md)
- [Raven primitive storage and character APIs](../raven-primitive-api.md)
- [Process APIs from Raven](../raven-process-api.md)
- [Prototype query API for Raven](../raven-query-api.md)
- [Try the query-enabled Raven build locally](../raven-query-local-build.md)
- [Reflection migration for the Raven target](../raven-reflection-api.md)
- [Existing runtime API coverage for the Raven POC](../raven-runtime-api-coverage.md)
- [Shared Raven signature projection](../raven-signature-projection.md)
- [Try the stabilized Raven/neoCLR build](../raven-stabilization-local-build.md)
- [Raven String helpers on neoCLR](../raven-string-api.md)
- [Library port validation and compiler branch audit](../raven-library-port-validation.md)
- [Authoring the foundational library in Raven](../raven-system-library.md)
- [Normal Raven compilation and independent neoCLR import](../raven-target-compilation.md)
- [Target-specific language contracts in Raven](../raven-target-contracts.md)
- [Raven target support before library migration](../raven-target-evaluation.md)
- [Raven target and binary artifact experiment](../raven-target-experiment.md)
- [Raven target profiles, symbols and emission backends](../raven-target-profiles.md)
- [Option and Result APIs from Raven](../raven-union-api.md)

- [Try the completed Raven source port locally](../raven-port-local-build.md) — isolated VS Code workspace and saved-project task.


### Storage POC I/O surface (2026-09-23)

The development Runtime Contract now exposes byte streams and text readers under
System.IO. Recompile development consumers using System.Streams imports. ReaderBindings
admits the exact TextReader/StreamReader declarations, including owning and leave-open
constructors; strict import includes reader reference locals and constructor argument
coercions (CLI Int32 Boolean values become neoIL Boolean storage). SeekableStream is
an independent interface, implemented by FileInputStream. Native FilePosition/FileSeek
carry signed 64-bit positions through the erased runtime-service boundary. Existing
runtime permission routing classifies both as file input operations.

This does not change Raven emission or its target configuration. The standalone POC
uses an object view before a SeekableStream pattern because direct unrelated-interface
conversion is rejected by the current compiler. All I/O remains synchronous; Task
return types and cancellation are future contract work. Validation lives in the
[standalone POC](../experiments/storage-poc/README.md), the provider ReaderContracts,
strict interface probes, native file-resource tests and generated runtime/API checks.


### Bootstrap worker cancellation adapter (2026-09-23)

RuntimeServiceBindings admits RequestWorkerCancellation(Int32)->Boolean and
JoinWorkerResult(Int32)->Value only in the bootstrap RuntimeServices contract.
Normal core generation still omits those services. The isolated worker adapter
checks erased String/Void and completes/cancels its Promise after notification and
acknowledged join. Public Thread/Task signatures, Raven Runtime Contract configuration,
compiler emission and the checked-in worker library are unchanged. The API snapshot
was refreshed from the matching normal reference assembly.

The [sample](../experiments/worker-task-cancellation/README.md) tests dedicated/pooled
cancelled awaits, successful siblings, actual GC, genuine producer Fault propagation
and rejection of ordinary bootstrap-service calls. An async parameter named `state`
exposed a generated-field collision in the imported artifact; the consumer uses
`destination`. A reduced general CLI-metadata case and responsibility analysis are
still required before changing Raven or the importer. This remains a deferred
integration candidate, not a user-facing parameter-name restriction.


### Queue-ownership fixture findings (2026-09-23)

The worker adapter's Affinity.rvn fixture uses a synchronous owner method for
bookkeeping after await and a separate synchronous Register helper for result
observation. Initial versions exposed two unreduced integration failures:

- A callback created inside the async Observe function and capturing its parameters
  failed strict import with a receiver/state-machine stack-type mismatch.
- A callback nested inside the Main queue callback captured the local result Task,
  but its later GetResult call faulted with a null object reference.

These observations do not yet establish compiler versus importer responsibility.
Reduce them into independent CLI-metadata/emission cases before changing Raven;
retain them as deferred general integration candidates. Runtime Contract settings,
compiler code and target policy are unchanged. See the
[fixture notes](../experiments/worker-task-cancellation/README.md#queue-ownership-characterization).

## Console integration — 2026-09-23

neoCLR now projects System.Console as a static class with text reader/writer
properties and standard byte-stream factories. The reference/importer catalogs
admit TextReader.ReadLine, TextWriter, StreamWriter and the internal Console stream
providers. Internal constructor access is allowed for verified library exports
inside their module; these providers are not public application contracts.
Runtime Contract configuration is unchanged: unit uses System.Void. The neoCLR
importer retains legacy Console value-Void exports, inserts/discards their value
at static call boundaries, and preserves no-result instance provider methods.
This is target adaptation, not a Raven compiler emission change.

Validation uses matching regenerated reference/library snapshots, a greeting,
short-write/ownership tests and both Result/Option patterns: `?` plus
`let Some(input) = ... else` and `if let Some(input) = ... { ... } else { ... }`.
The else guard returns before later use of input. The samples pass normal input,
EOF, empty lines and malformed UTF-8. APIs are development-only after Preview 9.

Deferred general compiler candidates, observed on the integration branch and
requiring independent reduction/testing before any main-branch fix:

- Expression-bodied static Console properties emitted null instead of the factory
  result; explicit getter bodies work in this slice.
- A nested match returning configured unit produced incompatible branch stacks;
  a propagation helper avoided that shape.
- Expression-bodied custom stream methods returning Result emitted a case rather
  than its carrier in this experiment; block-bodied returns work.

No general compiler fix was made or merged as part of this work. These observations
are not intended language restrictions. Existing neoCLR-specific configuration
and experiments remain isolated on the integration feature branch.

See [Console contracts](../../api-docs/console.md) and [tested samples](../experiments/console-streams/README.md).

## Object display integration — 2026-09-23

The neoCLR importer now admits the implemented virtual Object.ToString body and
preserves core Object ancestry for ordinary application classes. Virtual reference
calls and explicit base calls use separate adapters, retaining CIL callvirt versus
call semantics. Object's GetType remains nonvirtual. Runtime Contract settings,
Raven's value/reference classification and compiler emission are unchanged.
Rootless nominal classes/arrays use the runtime's default Object slot; overrides
require declared ancestry. Boxed-value and intrinsic-string Object virtual dispatch
remain unsupported; typed formatting and existing GetType paths are separate.
No Raven compiler code change or general fix is included. The neoCLR object-display
sample and raw dispatch tests cover the bounded contract; these changes require
matching development references, bridge and generated System library.

Object is now abstract by author direction. Its reference constructor is protected;
the importer preserves base chaining into an empty runtime constructor entry.
Ordinary derived classes remain constructible; direct Object construction is rejected
by source compilation and raw runtime allocation. No synchronization API is implied.

Migration: application classes now retain Object as their metadata base. A class
that supplies ToString should declare an override; same-name hiding is rejected by
the current importer/runtime profile. Use matching reference/library artifacts.
