# Endpoint binding descriptors — 2026-10-31

Development follow-up to `8ba1d1655`, using Raven `813dad20c` on macOS ARM64.
Neo.Web is an experimental web framework built on top of System.Web.Http.HttpServer.

The endpoint descriptor owns the parsed route, precedence, method and cached
execution callback. Generic registration shares that route with HandlerPlan instead
of parsing twice. Function signatures, parameter attributes and source/conversion
choices are resolved before publication. Request arrays and service instances remain
request-owned. Routing still scans patterns and dynamic invocation remains checked;
this is not a throughput or allocation improvement claim.

FromRoute selects a capture with an optional nullable Name override. FromServices
selects DI ahead of inference, including an explicitly registered string service
whose parameter name matches a route capture. Ordinary sample services remain
inferred. Invalid aliases, conflicting sources and unsupported service types reject
registration atomically. Invalid route integers return 400 without invocation.

## Runtime and tooling gaps fixed

The initial interpreter check accepted a missing alias because module-function
parameter attributes were invisible. RuntimeParameterInfo now falls back to its
module descriptor when no nominal owner exists. Interpreter and native module
attribute queries distinguish method tokens from parameter tokens. The focused
runtime regression checks exact selection and invalid-token rejection; it passes.
Ten native reflection snapshot tests pass, including unretained-query failures.

Native execution initially failed because parameter annotations were not retained.
Project retention supports customAttributes on types and version-2 exact module
selectors, including referenced assemblies. Nine tooling tests pass, covering
capability separation, combined roots, ambiguity, malformed policies and exact
scope matching. Constructor/property invocation rights are not granted by attribute
selection. Missing retention remains explicit; the planner never guesses that an
unretained annotation is absent.

Native/JIT generic handler consumers must retain attributes for their handler
classes/modules even when unannotated. This is a temporary development tooling
requirement. WebDemo's policy selects Examples.Web from WebDemo.Logic; framework
tests select their handler type and declared module. No compiler or native metadata
encoding change was needed. System.Runtime is rebuilt for the parameter projection
fix; other dependency libraries preserve the prior qualified bundle provenance.

## Execution qualification

Framework HTTP tests pass 5/5 in interpreter, native and JIT modes. They exercise
inferred and explicit binding, named module functions and bound instance methods,
concurrent argument values, invalid input, scopes, middleware and shutdown. A separate
annotated-lambda interpreter probe observes one parameter attribute and exits 42;
this probe is not a separate native/JIT qualification claim for all lambda forms.

Direct WebDemo tests pass 4/4 and the independent consumer passes 12 HTTP checks
in each of interpreter, native and JIT modes, including the imported annotated
Greeting function. API fingerprints and all four snapshot tests pass. Exact commands,
source/artifact hashes and component provenance are in [results.json](results.json). The initial
attribute failures are retained rather than converted into expected passes. Build,
transport and interpreter deadlines are unchanged. Website validation checks 6,913
pages, retaining 131 legacy pages and existing API-description warnings. The first
website check caught repository-only retention links; corrected URLs pass. No website
publication occurred.

## Limits and next decisions

Query/header/body families, optional/default scalar policies, a nonactivating DI
availability capability, prepared invocation adapters and indexed route lookup
remain open. ASP.NET Core is a researched reference, not a required architecture;
see the [design comparison and author decisions](../../design/neo-web.md). Performance
work must measure the resulting request path against direct HttpServer before claiming
low-overhead parity. No general result serializer or request-abort token is added.
