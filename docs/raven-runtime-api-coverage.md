# Existing runtime API coverage for the Raven POC

The release scope is the existing runtime library developed with Neo, accessible
through Raven. It is not the entire .NET class library or full CLR/compiler feature
parity. Keep Neo outside this migration. The latest direction is recorded in the
[conversation timeline](development-timeline.md#2026-09-12--existing-runtime-apis-define-the-raven-poc-scope).

## Source inventory

[The generated inventory](experiments/raven-target/runtime-api-inventory.json) scans
the System manifest and includes: 614 visible declaration candidates in 81 source
files at the time of this audit. This includes types, properties and their accessor
methods, fields, enum literals and service functions; it is **not** a count of 614
independent APIs or working Raven calls. Private declarations and their contents are
excluded. Runtime service functions remain explicitly marked for review rather than
silently disappearing from the checklist.

Regenerate after runtime changes and review the diff:

```sh
python3 docs/experiments/raven-target/inventory_runtime_api.py
python3 docs/experiments/raven-target/inventory_runtime_api.py --check
```

For every application API/overload, record target metadata visibility, imported call
support, type/category adaptation, execution evidence and editor evidence where
relevant. Closed generic demonstrations do not prove arbitrary generic support.
Decide explicitly which service helpers are implementation details, and show coverage
through their public library callers. The source scanner is a checklist, not an
assembly metadata reader or verifier.

## Current audit and projected slices

| Area | Current Raven evidence | Work to close the existing-library gap |
| --- | --- | --- |
| Numeric primitives, Boolean, Char, String, Value/Error and error unions | [Primitive storage and character APIs](raven-primitive-api.md), Int32/String/Boolean storage, [nine String methods](raven-string-api.md), [Int32.Parse](raven-parsing-api.md), [Int32.Divide](raven-division-api.md), [Int32 instance methods](raven-integer-api.md) and selected static calls; limited error carriers | [Canonical Boolean API boundaries](raven-boolean-api.md) are projected; general interface/generic paths remain; [error-value APIs](raven-error-api.md) are projected |
| Math and Console | All 20 existing Math methods ([Int32 Clamp](raven-clamp-api.md), [Double methods](raven-floating-math-api.md)) and [Console input/output](raven-process-api.md) | Broader numeric/compiler support remains separate |
| Option, Result, Void and Propagatable | [Generic value-payload bindings](raven-union-api.md), case/carrier APIs, completion/error propagation and typed matching | Reference/application payload shapes and general generic paths |
| File and Path | Bounded UTF-8 File calls, propagation, error predicates and [both Path methods](raven-path-api.md) | File-error APIs are projected; retain existing I/O contracts |
| Date, Time, LocalDateTime and Clock | [All current calendar/clock methods](raven-calendar-api.md), validation Results and live-clock check | Error-value APIs are projected; interface dispatch remains separate |
| Environment | [All three existing process APIs](raven-process-api.md), argument arrays, current directory and variable Results/Options | Retain live host semantics; environment mutation is not an existing API |
| Array, ArrayList, List, Iterable, Iterator | [Closed collection elements and Copy](raven-generic-collections.md), Int32/String vectors, class aliasing and foreach | Predicate/delegate APIs, native Array<T> and remaining element shapes; preserve reference categories and document cleanup limits |
| Equatable, Comparable, Clonable, Disposable, Closable | Selected interface dispatch through collections | General existing interface contracts and representative runtime-library callers |
| Func delegate families | Runtime declarations and invocation support exist | Raven target metadata and existing delegate API invocation; new closure/lambda features are not implied |
| Type, RuntimeTypeHandle, Reflection and BindingFlags | Existing runtime introspection library | Raven metadata/type-handle projection, descriptor hierarchy, flags and all existing public introspection members |
| Runtime service functions | Bundled implementation calls run behind admitted APIs | Review public-versus-implementation status and verify each application-facing service path |

The [shared signature mechanics](raven-signature-projection.md) now cover file and
collection catalogs. Continue consolidating the bridge's type mapping and metadata catalog so broader
APIs do not require another independent whitelist for each closed Result. Then cover
primitive/string/Console/Math families, date/time and environment/path families,
collections/delegates/interfaces, and reflection. Continue exposing useful examples
as each group passes. File read/write is the current completed executable slice;
this table is a plan, not a declaration that the remaining groups are implemented.

## Comparison and tradeoffs

.NET compilers normally consume the class library's metadata directly. The current
experiment uses manually authored declaration metadata plus a bounded importer. It
proves the runtime calls are real, but duplicating every signature and closed generic
shape is a maintenance cost and a risk of contract drift. Moving toward a shared
runtime-derived catalog and general signature handling is the next groundwork to
evaluate; generated metadata must still preserve CLR type categories, visibility,
inheritance, generic constraints and neoCLR's named Void semantics.

Reuse the existing research in [runtime API design](api-design.md),
[file input](file-input.md), [file output](file-output.md), and
[the target-contract comparison](raven-target-contracts.md) as the contracts are
projected. Research substantive divergences separately; a different interface name
or metadata representation is not by itself an improvement. Keep Result/Option
behavior and runtime faults distinct, and record any migration cost rather than
hiding it in a compiler workaround.

The [preview criteria](raven-preview-acceptance.md) and
[experimental release procedure](experiments/raven-target/RELEASING.md) govern
completion and packaging. A lightweight Raven distribution process still requires
honest API coverage and successful package-level demonstrations.
