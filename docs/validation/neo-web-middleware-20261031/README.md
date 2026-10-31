# Typed Neo.Web middleware — 2026-10-31

Development follow-up to `02659917f`, using unified Raven `813dad20c` on main on macOS ARM64.
Neo.Web is an experimental web framework built on top of
System.Web.Http.HttpServer.

`UseMiddleware<T: Middleware>()` composes with `Use(handler)` in registration
order. The adapter resolves T from the request scope only when reached and calls
its interface directly. Short circuits avoid activation; the scope lasts through
the entire awaited pipeline. DI registration controls lifetime and disposal.
Missing registration follows the sanitized 500 response path. Reflectively
activated middleware requires a public type/constructor and native/JIT constructor
retention; the framework test and WebDemo carry explicit retention policies.

Both middleware registration methods return `Result<WebApplication, WebError>`.
Map methods retain their existing unit result. WebDemo uses separate registration
statements with postfix `?` propagation. Raven conditional member access remains
mapping for properties and methods, including nested Results; no compiler change
to that language contract is included.

## Compiler and runtime qualification

The matching Raven compiler borrows constrained receivers held in locals and
allows declaring-type parameters in native constrained calls. Native metadata
validates nominal owner/method bounds and preserves imported bounds on nonvirtual
generic reference-class methods. Invalid scopes and unproven bounds still reject.
Native specialization uses original conformance metadata and adapts borrowed
reference receivers while preserving value mutation and virtual dispatch.

The reduced compiler consumer returns 42 in interpreter and JIT execution. It
exercises owner and method bounds, local value mutation, reference state and
virtual overrides. Focused metadata checks pass. The focused .NET conditional-access, Result,
propagation and Native Self tests pass 60/60. Codegen checks include the
original-conformance proof, borrowed calls with arguments and branch remapping.
Runtime generic-bound and Self suites pass (12 and 17 tests). The broader legacy
open-constrained metadata suite still encounters a primitive Double static-bound
fixture failure; this slice does not claim that suite is qualified.

Format-6 disassembly is accepted alongside format 5, matching the runtime's
supported inputs. This fixes constructor-retention discovery across the sample's
referenced libraries. Seven disassembler tests pass, including JSON/NEOX parity
and rejection of unsupported format 7.

## Execution evidence

Final execution: framework tests pass 5/5, direct WebDemo tests pass 4/4,
and the independent Python consumer passes 12 HTTP checks in each of interpreter,
JIT and native modes. All sample processes shut down cleanly. [results.json](results.json)
records the exact source hashes, commands and artifacts.
The previous framework run passed 5/5 tests in native, interpreter and JIT modes;
its sample build exposed the disassembler gap above. Earlier internal fixture
types also produced activation failures and were made public. An AOT build hit
its existing deadline during competing builds, then completed in isolation.
Transport, interpreter and build deadlines were not increased.

API source fingerprints and all four snapshot tests pass. The native reference
includes the Middleware interface and the shorter T generic parameter. Website
validation checks 6,903 pages, retaining 131 legacy pages and existing missing
API-description warnings. No publication occurred.

Core libraries retain the prior qualified bundle provenance. Neo.Web is rebuilt
with the matching compiler and tested runtime/codegen. This is development
qualification, not a release, deployment or throughput claim.

The [.NET comparison and API decisions](../../design/neo-web.md) follow ASP.NET
Core's per-request IMiddleware activation and separate endpoint registration
contracts. General parameter attributes, request-body binding, cached invocation
adapters and endpoint lookup performance remain subsequent work.

Focused reproduction (from neoCLR, with the matching Raven build):

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- --nominal-instance-calls
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- --structural-method-constraints
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- --authored-structural-constraints target/middleware-constraints
cargo test -p neoclr-codegen nominal_
cargo test -p neoclr-codegen borrowed_nominal_calls
cargo test --test generic_bounds --test self_types
cargo test --test disassemble
python3 scripts/validate-neo-web.py --bundle target/neo-web-middleware/input-bundle \
  --output target/neo-web-middleware/validation8 --codegen target/release/neoclr-codegen \
  --mode native --mode interpreter --mode jit
```

The validator requires a fresh output directory. Exact final build/run commands
and source hashes are retained in results.json and the component build records.
