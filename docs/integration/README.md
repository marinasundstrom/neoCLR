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
