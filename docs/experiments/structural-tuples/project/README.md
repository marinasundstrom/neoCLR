# Native structural project sample

Open this directory in VS Code and run the supplied build/run tasks. Select the
matching feature-branch `rvnc.dll`. The projects import the local
`target/tuple-names-libraries-v2/NeoCLR.ClassLibrary.props` bundle; build the runtime
with `cargo build` from the repository root first. This is a local development
sample, not a self-contained distribution.

No structural feature properties are required. The library exports union and
intersection functions and a named tuple. The app imports all three, checks the
nominal payloads, mutates the tuple and verifies copy preservation. Success is
process exit **42** (VS Code may display it as a nonzero task result).

Run `python3 verify.py --compiler /absolute/path/to/rvnc.dll` here to verify the
positive case and disabled/malformed tuple-profile rejection in disposable copies.
The runtime is the repository's `target/debug/neoclr`. Language-server diagnostics
require a matching native Raven server; the tasks select only the compiler.

The bundle was built with compiler 793219d855b17f01fb805e215968b78871eb3804.
The intersection parser and library writer blockers are fixed. Fresh Runtime,
Data and Networking builds pass; Web still fails structural-union conversions
in Result error propagation/case construction. A complete new bundle is pending.
The default native project runner uses the interpreter; JIT/AOT are unsupported.
