# Preview 19 candidate release notes

Runtime candidate version: `0.1.0-preview.19`. Preparation started 2026-10-30. Preview 19 is the candidate designation; no release
is published and its publication date is unset. [Preview 18](preview-18-release-notes.md)
remains the current release. Target platforms remain macOS ARM64 and Windows x64.
See the [candidate qualification record](validation/release-preparation-20261030/README.md)
for exact inputs, failures and remaining gates.

## Application services and configuration

DI and localization move into separately referenced `Neo.*` extension assemblies.
Logging adds typed loggers, state/formatter contracts, scopes and console/null
implementations. Separate `.Abstractions` assemblies support contract-only consumers.
Configuration adds hierarchical sections, ordered providers, explicit null masking,
in-memory and optional JSON sources, DI registration and opt-in debounced reload.
JSON configuration accepts comments, trailing commas and a UTF-8 BOM; strict DOM
syntax remains the default. These are bounded counterparts of .NET application
services, not full Microsoft.Extensions compatibility.

Standalone `System.IO.FileSystemWatcher` delivers `FileSystemNotification` unions
through disposable `Subscribe` registrations. Windows execution and packaged reload
qualification remain release gates. See [configuration](configuration.md),
[application services](application-services.md) and [watching](file-system-watcher.md).

## API and compiler migration

- Replace older Windows bundles to obtain native bounded UTF-8 file reads and
  matching default-JIT file-input/watcher bindings. This target extension preserves
  the existing FileText error/byte-limit contract; file output remains separate.
- Rebuild libraries, applications and interface implementations with the matching
  compiler/runtime bundle. Moved service types have new assembly identities and no
  type forwarders; update project references and imports.
- Replace `System.Fail` with `System.Panic`. The rename does not implement panic
  unwinding. Synchronous panic cleanup remains planned.
- Stream, networking and HTTP APIs consolidate forwarding overloads through scalar,
  reference-null and nominal value defaults, including `CancellationToken = default`.
  Defaults are inserted at call sites; removed overloads do not remain as delegate
  adapters. Use lambdas where a method-group binding requires an old arity.
- Replace watcher event arguments/registrations with `FileSystemNotification` and
  `Subscribe`. Preserve both rename paths and handle rescan requests.
- Backtick test names replace description attributes in ordinary suites. Name-based
  filters, IDs and discovery order change; description-feature fixtures remain.

Compact interpreter fault diagnostics now show decoded logical function names, matching
native presentation; captured identities remain unchanged.

Direct synchronous/asynchronous Result entry points are qualified in source:
unit/integer success determines exit status; failure displays to stderr and exits
one. Bounded dynamic-member retention and native reflection improvements accompany
this work. See [default arguments](parameter-defaults.md) and the
[changelog](../CHANGELOG.md) for the complete migration inventory.

## Runtime and experimental work

Shared metadata/selection preparation, lazy JIT investigation, GC and scheduler
correctness work retain measured tradeoffs and failed experiments. Eager JIT and
synchronous GC remain the ordinary defaults; shared ready queues stay opt-in.
JIT is the development execution path and native AOT the distribution direction.
No universal throughput gain, complete scheduler unification or new concurrency
capacity guarantee is claimed. Existing experiment measurements do not qualify
fresh release packages.

Native format 6 adds experimental whole-function signatures and return projections;
format 5 remains readable. Function reflection and the separate `Neo.Signals`
experiment retain explicit limitations: interpreter signal consumers pass, while
native signal execution is blocked by unit boxing. Signals is not part of the
standard packaged class-library selection. Broader structural work stays parked.

## Qualification and distribution

Candidate matching SDK/editor version: `0.1.19-neoclr.20261030.1`, Raven
`6ceeb1be8790e105a1394cb99fded41fafad5b4c`. Source validation, extracted packages,
editor, HTTPS, Windows watcher/reload behavior and release measurements must pass
before publication. Preserve optional PostgreSQL on macOS only if the matching
package passes its existing acceptance. No artifact download or deployment is
announced by these notes.
