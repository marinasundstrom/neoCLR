# Existing runtime API coverage for the Raven POC

The existing runtime-library member surface developed with Neo is now projected for
the Raven proof of concept. The source demonstrations and signature checks pass.
This is the existing library, not the .NET class library or unrestricted CLI import.
Neo remains outside this migration. Package validation is recorded separately.

## Audited surface

The [source inventory](experiments/raven-target/runtime-api-inventory.json) contains
744 visible declaration candidates from 133 files, including the manifest. Accessors,
fields and properties can describe the same source operation; these are not 744
independent APIs. Generated fragments and adapters contribute to these counts;
they are not a count of Raven-authored APIs. The [coverage record](experiments/raven-target/runtime-api-coverage.json)
assigns every declaring source a disposition and evidence. Its checker fails if a
new source is unaccounted for or a runtime service loses its reviewed library caller.

```sh
python3 docs/experiments/raven-target/inventory_runtime_api.py --check
python3 docs/experiments/raven-target/audit_runtime_api.py --check
```

| Area | Implemented projection and evidence |
| --- | --- |
| Primitives, Boolean, Char, String | Existing conversion/comparison/character/string methods, storage and numeric API boundaries; [primitives](raven-primitive-api.md), [Boolean](raven-boolean-api.md), [String](raven-string-api.md), [parsing](raven-parsing-api.md) |
| Math and Console | All existing Math methods, WriteLine overloads and ReadByte; [Math](raven-floating-math-api.md), [process APIs](raven-process-api.md) |
| Option, Result, Void, Propagatable | Case/carrier constructors, predicates, extraction, residual/output flow, factories and completion; [unions](raven-union-api.md), including existing reference payloads |
| Errors, File, Path | Existing error case/predicate/message APIs, bounded UTF-8 reading/writing, propagation and both Path methods; [errors](raven-error-api.md), [files](raven-file-api.md), [paths](raven-path-api.md) |
| Date, Time, LocalDateTime, Clock | Existing factories, validation, components, equality/ordering and local system clock; [calendar](raven-calendar-api.md) |
| Environment | All three APIs, copied argument arrays, current directory and variable Results/Options; [process](raven-process-api.md) |
| Managed arrays and collections | Constructors, capacity/count, indexers, Add/Copy, foreach/iteration, predicates and ForEach; [collections](raven-generic-collections.md), [array shapes](raven-array-shapes.md) |
| Managed Array<T> / native memory | Generic managed array identity, Length/Item and declared Iterable<T>; separate NativeMemory.Alloc/Free. The old native descriptor is removed. Raven native calls have conversion limits; typed access is covered in direct IL. See [array and native API contracts](generic-managed-arrays.md) |
| Fundamental interfaces | Comparable/Equatable value and class implementations, collection contracts and Disposable; [interfaces](raven-fundamental-interfaces.md). Clonable/Closable declarations and calls are available, but have no concrete implementations in the existing library |
| Func | All five existing arities, static application targets, invocation, callbacks and stored delegates; [delegates](raven-delegate-api.md) |
| Type, TypeHandle and reflection | Type tokens/queries, descriptor hierarchy, all public getters, query arrays/options and BindingFlags; [reflection](raven-reflection-api.md) |
| Signature markers | Void remains valid in generic signatures. Value is an opaque erasure identity with no public methods. RuntimeTypeHandle is produced by tokens. UnionAttribute is compiler metadata, not a union execution API |
| neoCLR.Runtime functions | Reviewed implementation services reached through the public library callers listed in the coverage record; not a second application API |

The saved-project suite includes the executable API examples and rejection cases.
Focused scripts additionally check live clock/process behavior, file bytes, wrong-case
and invalid-default faults, callback faults and native buffer lifetime. Editor checks
exercise target completion. A source coverage record does not substitute for running
these checks; it provides a reviewable checklist when the library changes.

## Subsequent Raven-target APIs

The original inventory above follows `runtime/System.neoil`; its 744 candidates do
not include later target-specific files. The generic Where/Select/ToList surface in
`runtime/raven/Linq.neoil` has separate [query contract and executable coverage](raven-query-api.md),
including custom Raven Iterable implementations. This distinction keeps the existing
library migration audit separate from subsequent API additions.

## Projection decisions and remaining boundaries

- Types and arrays use the target's CLR-like value/reference categories. Reflection
  snapshots and ArrayList are ordinary classes. Value-to-interface conversion boxes
  a copy; direct value calls do not require boxing.
- BindingFlags uses CLI enum literals/casts/operators instead of the old wrapper
  factories and combinators. The [migration mapping](raven-reflection-api.md#bindingflags-enum-metadata)
  explains each replacement.
- Duplicate field/property spellings for union payloads project as read/write Value
  properties, preserving the original case-field mutation behavior. Native
  descriptors retain writable Data/Length fields; Length also serves the legacy
  getter's read behavior. This is a preview API change, not binary compatibility.
- Reserved ArrayList capacity supports non-defaultable carriers. Ordinary newarr
  still requires valid defaults. Generic mapping is bounded and preserves invariant
  arguments. The post-Preview-4 source bridge also supports bounded application
  classes/value types and class interfaces; see [application types](raven-application-types.md).
  Generic application definitions, rectangular arrays and covariance remain outside it.
- The post-Preview-4 source bridge also admits application class targets and
  [Raven lambdas/captures](raven-delegates-lambdas.md) through Func. New application
  delegate declarations, explicit/default interfaces and generic application hierarchies,
  general pointer arithmetic/layouts, unboxing and constrained calls need further
  compiler/importer work. Raw native pointer reads/writes currently admit Int32;
  other supported primitive buffers use Get/Set/indexers.
- Inherited Object methods visible from declaration metadata are not executable
  library APIs unless separately admitted. Reflection remains introspection-only.
  Cleanup on terminal faults and nullability metadata remain separate designs.

These are explicit POC boundaries. In particular, Clonable/Closable metadata alone
must not be presented as a demonstrated custom implementation. The API review is
complete for this bounded projection; full CLR feature compatibility is not.

## Comparison and maintenance

.NET compilers normally consume their class library metadata directly. This
experiment still uses declaration metadata and a bounded importer, with real runtime
library algorithms behind the calls. The duplicated metadata has a maintenance cost;
keep the signature checks, source audit and executable examples current. The eventual
runtime-derived binary catalog remains important groundwork.

Reuse [API design](api-design.md), [target contracts](raven-target-contracts.md), and
[design research](design-research.md) for subsequent changes. Review both the benefit
and the migration cost before calling a divergence an improvement. Result/Option
handling and terminal runtime faults remain distinct. Use the
[preview criteria](raven-preview-acceptance.md) and
[experimental release procedure](experiments/raven-target/RELEASING.md) for distribution.
