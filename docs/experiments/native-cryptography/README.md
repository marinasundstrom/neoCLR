# Native cryptography service qualification

Development, 2026-10-12. The explicit `--bind-cryptography` compiler option lowers
four exact byte-array InternalCall contracts through the same host adapter in JIT
and AOT. It requires `--compile-system`, `--reference-arena` and a host exporting
`neoclr_cryptography_slices_v1`. Ordinary managed method bodies are never replaced. Missing
binding permission fails compilation; missing JIT host exports fail before execution.

Production contracts are in `runtime/System.Runtime/src/System.Runtime/neoCLR/Runtime/RuntimeCryptographyCalls.rvn`;
low-level regression declarations live in `tests/fixtures/cryptography.neoil`: SHA-256,
HMAC-SHA-256, verification and secure random fill. The adapter does not collect,
call guest code, retain buffers or run a separate scheduler. It passes byte payloads
to the existing ring-backed provider after checking allocator ownership, slice bounds/capabilities and complete window
initialization, and publishes reserved output slots only after provider completion.
A short digest destination returns -1 unchanged; uninitialized input is a terminal
RuntimeError, not an operation error. Successful hash/MAC writes affect exactly 32
bytes and can overlap input. Random-provider failure clears output and returns -2.
Verification returns a Boolean. The helper return status remains reserved for
terminal faults; operation results use a separate output slot.

This follows the synchronous computation layer behind .NET's HashData and
RandomNumberGenerator APIs, while using neoCLR's explicit runtime-service and Fault
contracts. The byte-slice/provider ABI is private and experimental; it is not a public
Span/Memory substitute. See [comparison and provider limits](../../shared-security-provider.md).

## Reproduce

Build the runtime and native compiler, then run:

```sh
cargo build --locked --bin neoclr
cargo build --locked --manifest-path tools/aot-poc/Cargo.toml
python3 docs/experiments/native-cryptography/validate.py \
  --runtime target/debug/neoclr \
  --compiler tools/aot-poc/target/debug/neoclr-aot-poc \
  --output target/native-cryptography
```

On Windows x64, run from an MSVC x64 developer environment and use `.exe` for the
two tool paths. The same script is wired into the Windows/macOS security workflow.
The script verifies that the standalone `tools/native-crypto` dependency graph
contains no TLS provider, builds its archive and a focused GC/stack host, assembles the
same IL for every mode, and checks real process results. Detailed commands and
output go to the target directory's `validation.json`. The checked-in
[checkpoint](validation.json) records the local case outcomes and input hashes.

Twelve cases pass locally in interpreter, JIT and AOT (36 executions): empty/short/
multiblock SHA input, aliased SHA input/output, RFC HMAC output and verification,
tampered MAC rejection, zero/nonzero random fill, unchanged short destination,
reserved output, and uninitialized input at one byte and above 64 KiB. The script
also checks that compilation without explicit binding fails without publishing an
object. A focused compiler unit test rejects changed signatures, generic contracts
and ordinary managed method bodies.

## Remaining gates

Windows execution is pending; adding CI is not platform qualification. This is a
private bytecode checkpoint, not the public Raven cryptography API. The qualification host now links only the standalone
cryptography archive for its raw C exports. Packaged native build kits,
allocating and streaming managed overloads remain next. The bounded
[public facade](../../shared-security-provider.md#managed-cryptography-facade-development-2026-10-12)
and both API snapshots are now available. Managed SslStream and certificate/trust APIs are
also still open.

## Installed JIT host

`scripts/prepare-jit-bundle.py` now links the standalone provider into the normal
shared JIT host. Windows builds select static CRT consistently with the C host.
`neoclr run` admits the verified private bindings by default; no per-program host
or compiler override is needed. The preparation manifest records source/archive
hashes and `jit.cryptography: true`. Rebuild the matching runtime/compiler/host
bundle when adopting this change.

```sh
python3 docs/experiments/native-cryptography/validate-bundle.py \
  --bundle path/to/prepared/bundle --output target/crypto-installed-validation
```

The test verifies bundle hashes, declares the private InternalCalls in each test
assembly, and runs all twelve cases against the installed seed and libraries.
These test-owned declarations do not alter the installed seed. Public managed
facade validation is tracked separately in the runtime cryptography test suite. Terminal faults use the normal host's exit code 1 and RuntimeError
diagnostic, rather than the isolated test host's exit code 103.
[Installed evidence](installed-validation.json) records macOS outcomes and a retained
seven-case managed MemoryStream regression. The Windows JIT workflow now runs the
same cryptography consumer after bundle preparation; Windows execution is pending.

## Raven source project integration

The source-checkout `build-native-project.py` path now admits the verified
cryptography contracts and links the standalone provider only when a selected
function uses them. It records the adapter, provider source and archive hashes in
`build.json`, together with `cryptography: true`. Merely declaring an unused
private service does not link the provider. Windows console and HTTP profiles use
the same adapter and static-CRT archive; macOS uses the same provider implementation.
Packaged build kits are not upgraded by this source-checkout change.

[The project](project/Main.rvn) uses existing Raven `MethodImpl(InternalCall)`
declarations owned by the test assembly. No new Raven compiler mapping is required.
This remains a private runtime contract rather than a public cryptography facade.
The compiler's existing native CLI bridge retains those signatures and implementation
flags; the normal native binder validates them before selecting host code.

```sh
python3 docs/experiments/native-cryptography/validate-project.py \
  --bundle path/to/prepared/bundle --aot path/to/neoclr-aot-poc \
  --output target/crypto-project-validation
```

The [source-project checkpoint](project-validation.json) covers SHA-256 and HMAC
known answers, verification and tamper rejection, preserved destination tails,
unchanged short destinations and zero/nonzero random fill in interpreter, default
JIT and AOT. AOT runs from a directory containing only the executable. A second
project build proves unused declarations do not pull in the provider. Windows runs
this consumer in its JIT workflow; execution of this checkpoint remains pending CI.

The retained earlier validation JSON records the old whole-array boundary. Current
validation constructs explicit borrowed windows using test-only factories. The
Raven suite uses implicit array conversions and range slicing, including overlapping
subwindows and sentinels. The old bootstrap neoIL mirror remains historical.

As of the slice migration, `validate-bundle.py` builds the current Raven cryptography
suite against the installed library and checks both configured JIT execution and
bare `neoclr run` default-JIT dispatch. It no longer injects private array/view type
fixtures into an installed library load set. `validate.py` separately retains all
12 lower-level interpreter/JIT/AOT cases, with explicitly synthetic source origins
for its canonical view types. Those origins preserve identity through native
load-set relocation; unannotated interpreter-only System fixtures do not currently
survive that relocation. This is a fixture/backend integration limitation, not a
new public Slice semantic restriction. Packaged-library primitive and nominal-array
contracts must come from the matched native producer.

## Current public project fixture — 2026-10-30

The standalone project now calls public `SHA256`, `HMACSHA256` and
`RandomNumberGenerator` APIs. Removed test-owned array InternalCalls that predated
the slice-based public facade. Known answers, trailing-byte preservation, short
destination errors, altered-tag rejection and random fills remain checked.
Interpreter, default JIT and isolated native execution pass on the Preview 19 macOS
candidate; the plain-project control still verifies unused provider omission.
This updates an existing Raven consumer, without starting the deferred Python
behavior-fixture migration. See the [release receipt](../../validation/release-preparation-20261030/macos-final-public-crypto-project.json.gz).
