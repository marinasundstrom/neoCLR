# Development JIT execution and readiness

2026-10-12 development behavior: installed JIT bundles default to JIT on macOS
ARM64 and Windows x64. Other hosts retain interpreter execution.

```sh
neoclr run app.dll -- --filter collection
neoclr run app.dll --execution interpreter --system lib/System.runtime.neox \
  --module lib/System.Runtime.dll --object-root lib/System.Runtime.dll
```

The runtime discovers `neoclr-aot-poc` (`.exe` on Windows) and
`neoclr-jit-host.dylib` (`.dll` on Windows) beside its own executable. Standard
libraries come from `../lib/bundle.json`; extra `--module` dependencies are merged
with those libraries. Explicit `--jit-compiler`, `--jit-host`, `--system` and
`--object-root` overrides remain available. Helpers are trusted native code and
are never discovered relative to the guest or current directory. Missing tools
are errors, with no silent interpreter fallback.

Development JIT runs can supply an explicit metadata-retention policy:

```sh
neoclr run app.dll --reflection-roots roots.json -- --filter collection
```

`--reflection-roots` is a single file path relative to the calling directory (quote
paths containing spaces). The CLI requires an existing file and forwards it unchanged
to the selected backend, which validates its schema and source identities before
execution. No ambient policy is discovered. Missing, duplicate or malformed options
fail; an invalid policy does not fall back to interpretation. The option is rejected
with `--execution interpreter`, whose loaded metadata is not trimmed.
See [schema-5 module/function/attribute retention](experiments/guest-functions/native-retention/README.md).
[The CLI consumer](experiments/guest-functions/native-retention/cli-validation.json)
passes on macOS ARM64, including missing-retention faults. Windows retention execution
is not qualified by this check.

Prepare a development installation from a verified compiler/library bundle and a
passing native host recipe:

```sh
python3 scripts/prepare-jit-bundle.py \
  --bundle target/development-bundle \
  --runtime target/release/neoclr \
  --jit-compiler tools/aot-poc/target/release/neoclr-aot-poc \
  --host-recipe target/native-async-validation/validation.json \
  --output target/jit-install
```

On Windows use a passing native HTTP `build.json` recipe and `.exe` binaries.
The preparer rebuilds the helper, stages matching tools and records hashes. Use
`--libraries` with its matching `--compiler-directory` to replace an old compiler
and library pair coherently. Default development limits are a 32 MiB managed heap
and 4 MiB byte arrays, configurable at preparation with `--heap-bytes` and
`--byte-array-limit`. These are bounded experimental hosts, not general production
memory limits. Standalone interpreter packages do not contain JIT dependencies;
select `--execution interpreter` explicitly on supported JIT hosts.

The runtime launches the optional compiler backend with inherited standard streams.
The backend selects the executing host, compiles the verified closed-world assembly
in memory and runs its entry. Guest output is streamed normally and its exit code is
returned, without the private probe's JSON footer. Arguments after `--` reach the
guest, including empty strings, Unicode and option-like text. JIT resources are
released before backend exit. Compilation/host errors remain explicit; there is no
silent fallback to interpretation. No guest AOT object is emitted by this run command.

`--execution interpreter` selects existing interpretation. Omitting the option selects JIT on the supported hosts. Debugging remains interpreted. Interpreter-only instruction
budgets, array budgets, GC diagnostics and `--show-result` are rejected with JIT rather
than ignored. Native execution retains the current host's bounded heap/operation
limits and has no instruction quota. AOT project builds remain separate.

## Additional library qualification

The macOS development CLI passes all 16 JSON DOM and 12 JSON stream tests in JIT
and AOT ([evidence](experiments/jit-cli/json-validation.json)), plus eight
encoding/stream tests in JIT ([evidence](experiments/jit-cli/encoding-validation.json)).
The JSON run exposed and fixed shared native selection of unselected generic
collection overloads after specialization. The first private shape reuses its
source name; selection must distinguish its closed methods from remaining open
templates. Original metadata verification remains required. This is an internal
lowering correction, with no Raven API or compiler bridge change.

## Target status

macOS ARM64 has executable [CLI HTTP evidence](experiments/jit-cli/http-validation.json)
and [JIT-only test-framework evidence](experiments/jit-cli/framework-validation.json). The Windows x64
implementation uses the Windows calling convention, the same inline stack probes as
AOT, a helper DLL with explicit native exports, and UTF-16 entry arguments. The shared
host preserves the UTF-8 byte-stream contract by disabling CRT translation during the
call and restoring the prior modes afterward. The
[Windows execution gate](experiments/jit-windows/README.md) passes at `f993e94b`:
22 HTTP and 23 framework cases, plus scalar JIT and CLI checks. Windows ARM64
remains a separate target gap.

Like .NET, code generation and runtime services are separate concerns; see the
[existing execution investigation](native-execution-investigation.md). Reusing the
AOT libraries and rooted operation scope avoids a second HTTP implementation. The
current optional compiler process keeps Cranelift out of interpreter-only builds,
at the cost of process startup and an explicit host/tool installation. In-process
integration, lazy compilation and tiering remain future work.

## Test policy and the eventual default

The author directs JIT to become the default when judged ready, assessed by running
our tests routinely with JIT. Interpreter and AOT remain supported and tested
selectively when relevant to the behavior or target being changed. This is not a
requirement to run every test in every mode.

The runtime-library harness now accepts repeatable `--mode jit`, `--mode interpreter`
and `--mode native` selectors. JIT discovers the helper from prepared bundles, with an optional `--jit-host` override; the existing `--aot` option
selects the backend compiler for both native and JIT work. For example:

```sh
python3 scripts/test-runtime-library.py \
  --bundle target/extension-indexer-final-bundle \
  --aot tools/aot-poc/target/release/neoclr-aot-poc \
  --mode jit --jit-host target/jit-cli-http-validation/host.dylib \
  --suite index-range --suite runner-contract \
  --output target/jit-cli-framework-validation
```

Selectors, intentional failures, bounds guards and entry arguments all use the
selected modes. JIT-only and interpreter-only runs compile the Raven project directly to metadata,
without emitting a native object or linking an executable. Interpreter-only runs do
not require the native compiler ([focused interpreter check](experiments/jit-cli/interpreter-validation.json)). The omitted-mode default is now JIT, and its run commands omit `--execution` to exercise the CLI default.

Qualification covers representative library suites and HTTP/lifetime behavior,
installed helper discovery, and routine JIT test coverage. Keep explicit
mode selection and visible unsupported-program diagnostics after the switch.
Use focused interpreter/AOT parity checks for changes to shared lowering, GC, async
lifetime, platform bindings or other mode-sensitive behavior. AOT remains the preferred
deployment target. These criteria are implementation work, not a new approval step.


Runtime-managed suspension is enabled by the default CLI on macOS ARM64. Windows
x64 currently uses compiler-generated async state machines; native runtime
suspension and Windows ARM64 JIT remain gaps. Unsupported programs report compiler
errors. Eager closed-world compilation, the external compiler process, bounded
native heaps and interpreted debugging remain development limitations.

## Installed default qualification — 2026-10-12

The [completed default-mode gate](experiments/jit-cli/default/README.md) validates
29 macOS library suites from a relocated installation, eight HTTP cases on each
supported host, and the current Windows scalar/CLI/framework checks. Preparation
includes the complete text/time-zone helper. These results qualify the development
default; the remaining platform and execution limits above still apply.

### Shared runtime services in separately compiled libraries (2026-10-12)

The SQL provider and its consumer can each include the existing opt-in async adapter.
Original-scope verification already accepts their independently declared, exactly
matched private InternalCalls. Native lowering now also gives explicitly admitted
free service placeholders unique private names before re-verifying its flattened
module. This extends the existing seed-service renaming to services absent from the
bundled System seed; it neither merges source identities nor admits arbitrary host
calls. Reports preserve each original definition/name and its private compiled name.
The focused `sql_linking` native-tool test covers admitted and unadmitted variants;
[SQL qualification](experiments/sql-postgresql/README.md) supplies the real consumer.
The change is shared by JIT/AOT selection; it introduces no host ABI or compiler setting.

Explicit managed `GC.Collect` uses the shared native collector when `--native-gc`
is enabled, independently of runtime suspension. This applies to generated async
state machines on Windows x64 as well as macOS ARM64. The collector uses the
published guest and host roots; enabling it does not enable runtime async on Windows.
