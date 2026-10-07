# Windows native metadata release qualification

Windows x64 is required for the next native metadata release. The existing Windows
runtime tests and neoIL package do not establish support for Raven native metadata,
source-built libraries, or VS Code. No Windows native SDK release is claimed yet.

## Build and execution gate

Workflow changes trigger branch qualification using the pinned Raven revision. Once
registered on the default branch, run **Native toolchain qualification** (`.github/workflows/native-toolchain.yml`) on the
candidate neoCLR revision with an exact Raven commit. Its default pins the compiler used
for the independent-checkout macOS evidence; changes require requalification.

The job builds the runtime and matching compiler/server/VSIX, regenerates primitive
bootstrap inputs, builds System.Runtime, System.Data, System.Networking and System.Web,
and packages them together. It extracts into a path containing spaces and runs the
packaged verifier: collections, Tasks/await, JSON mapping and live HTTP server/client.
Source and artifact identities are recorded by the existing build/package tools. The
workflow uploads the candidate and reports even after a failure where available.

The initial host is Windows x64 with Rust stable, .NET 11 SDK
`11.0.100-rc.1.26425.128`, .NET 10 SDK, Node 22 and Python 3.12. Building the SDK uses
Git Bash and zip. These are build prerequisites, not all end-user runtime prerequisites.
The primitive CLI bootstrap and retained runtime seed remain explicit dependencies.

## Remaining qualification

- Execute the workflow on Windows and fix any build, linking or execution failures.
- Install its VSIX with its SDK in an isolated Windows VS Code profile. Run the native
  editor acceptance against the split bundle, checking hover/help, completion, navigation,
  reference refresh and build/run tasks. Audit shell commands for PowerShell portability.
- Test the archive on a clean Windows host and determine the exact runtime prerequisites,
  including whether the Visual C++ redistributable is needed by the packaged executable.
- Publish a matched Windows download only after passing these gates; then add that one
  installation path to the website. Keep qualification logs out of the setup instructions.

The portable HTTP readiness regression runs locally and in the Windows workflow. It
covers a port line, early process exit and a partial-line timeout. Actual HTTP execution
is still required: a unit test does not establish Windows socket/runtime behavior.

## Local validation (2026-10-07)

On macOS arm64, the three pipe-readiness tests pass, and the revised verifier runs
both the Python-to-native HTTP requests and native client/server pair successfully
against the previously qualified extracted split bundle. This reuses the assemblies
from [clean bootstrap qualification](experiments/extended-cli-metadata/clean-bootstrap-reproduction-2026-10-07.md).
It verifies the tool change locally; it does not substitute for a Windows run.
