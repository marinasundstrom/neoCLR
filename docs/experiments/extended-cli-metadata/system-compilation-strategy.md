# Strategy: compile the entire Raven-authored System library

Author-directed reprioritization, 2026-10-04. This supersedes the API-by-API
time-zone continuation in earlier roadmap entries. Individual APIs are acceptance
consumers of shared capabilities, not the unit of prioritization.

## Evidence and limits

The fresh [audit](system-compilation-audit-2026-10-04.json) covers all 166 Raven source
files under System, twelve source-family compilation attempts, and six minimal
capability probes. It uses Raven bc30fb0f2 and the explicit Clock core/seed.
The existing native execution gate covers 48 source files. This is not a claim
that 29% of the implementation work is complete: large, connected areas remain outside it.

- Full-source compilation stops in binding: 563 error diagnostics, including 367
  missing-member errors. Diagnostic counts are not independent defect counts.
- Excluding primitive declarations still stops in binding. The bootstrap gap is
  not confined to replacing core types.
- Lexical inventory finds 178 distinct RuntimeServices member names across 48 files.
  IsValue and UnpackValue each appear in 23 source files. These are potential reach
  counts, not promises that implementing two operations compiles 23 files.
- A host-service-free MemoryStream source group binds, then rejects the
  Result<unit, StreamError> Flush signature. A separate minimal Result<unit, string>
  probe reproduces it. Selecting the existing RuntimeUnitContract for the configured
  System.Void did not fix the rejection; do not assume this is only configuration.
- Minimal ordinary-driver probes reject enum declarations, double arithmetic,
  class inheritance, mutable captured locals and Self. These tests isolate failures
  hidden by larger binding cascades.
- OptionOperators/ResultOperators compile in the audit. This is emission evidence
  only, not a new execution gate.
- No async/await keyword use was found in the audited sources after comment removal.
  Tasks are implemented with explicit queues, callbacks and state. A general compiler
  async rewrite is not justified as the next prerequisite.

Family sets are diagnostic groupings over the accepted source baseline, not complete
dependency-closed production assemblies. Some include absent dependencies intentionally
to identify the frontier. Failures in binding mask later emission/metadata/runtime
failures; no rejected compilation was executed. The full source input is unchanged.
The audit does not make unsupported bootstrap references into implementations.

## Prioritized work

| Order | Shared work | Why it has leverage | Completion gate |
|---|---|---|---|
| 1 | Complete inhabited-unit and erased-value contracts | Unit occurs in 15 source files, including streams, encoders, JSON and HTTP. IsValue/UnpackValue span 23 files and underpin native outcome decoding. | Independently compiled MemoryStream/OutputStream with Result<unit, E>; typed native value outcomes round-trip success/error for multiple primitive kinds. |
| 2 | Replace incremental service wiring with a checked service ABI catalog | Missing runtime members dominate the full binding frontier and recur across 48 files. The existing bridge has useful signature and adapter knowledge that should be migrated, not rediscovered. | One catalog drives matching declarations/bindings/validation; at least text, I/O and task-service families compile through the ordinary driver and execute real host calls. Unsupported signatures reject explicitly. |
| 3 | Close reusable metadata/emission shapes | Enums block TaskState, EntryKind, StringComparison, BindingFlags and HttpStatusCode. Inheritance is required by JSON value classes and HTTP handlers. Wider numeric support blocks core/numeric and JSON work. | Separate library/consumer tests for enum storage/import, derived construction and virtual calls, and signed/unsigned/floating signatures and operations, with .NET controls. |
| 4 | Finish source-owned core identity and Self/static numeric contracts | Compiling applications against intrinsic primitives is not the same as compiling their member implementations. Number uses Self and static members; the full audit shows cascaded failures across ten numeric implementations. | A staged source-built core subset retains canonical primitive identity/storage, imports native Self relationships and implements Number; no seed/source duplicates or recursive ordinary fields for intrinsic storage. |
| 5 | Expand callback, storage and generic lowering only from cross-family failures | Task queues, cancellation, HTTP and JSON depend on callbacks, shared storage and generic contracts. Mutable captures and array-element receiver addresses have known gaps. | Observable retained callbacks with mutation/lifetime/identity; ref/out/array-address behavior; imported generic constraints/substitution needed by these source families. |
| 6 | Complete typeof/introspection/opaque-handle integration | Object/Introspection and JSON object mapping form a dependency cluster. A new runtime reflection facade is not needed: use the existing metadata/context model and explicit target contracts. | Source-built introspection contracts plus separate JSON consumer preserve identity and metadata resolution; no translated application/rebuilt-library references. |

Orders 1–2 form the first implementation batch. Within order 3, do enums first,
then rank numeric and inheritance work by the newly exposed blocked families.
Design the ownership model for order 4 during order 2; do not postpone discovering
core identity conflicts until every host service is wired. Do not treat this table
as permission for a wholesale backend rewrite.

### Batch 1: concrete next work

1. Reduce and fix the existing unit signature/representation mismatch through
   signatures, generic arguments, values, imports and no-result calls. Keep the
   language's inhabited value separate from CLI void returns. The MemoryStream
   source group is the first gate because it needs no new host service.
2. Specify System.Value as the existing native erased carrier at the compiler/metadata
   boundary; migrate the already implemented IsValue/UnpackValue intrinsic behavior.
   Test correct kinds, wrong-kind failures and identity without silently substituting
   CLR object boxing. Keep target policy explicit.
3. Inventory and classify existing service signatures: scalar, no-result/unit,
   erased outcome, vector, callback, opaque handle and source-owned nominal return.
   Reuse RuntimeServiceBindings and native runtime declarations as evidence. Separate
   real host services from bridge aliases that merely call source-owned factories.
4. Generate or validate matching primitive declarations and executable bindings
   from that catalog, with exact identities and failure-before-publication checks.
   Start with signature categories used by the most source families. Do not first
   hand-author 178 individual wrappers, and do not count declaration-only stubs as success.
5. Re-run the audit, group new first failures by common cause, and pick the next
   category using the rule below. Continue ordinary .NET regressions after shared changes.

## Ownership and architectural constraints

Maintain a single full-library declaration ownership plan. Each declaration has one
owner: intrinsic bootstrap, retained runtime implementation, or source-built library.
A migration must update signatures, binding, emitted references and linking together.
The current System.Runtime.rvnproj is a legacy per-slice bootstrap project; its default
source selection is not a full native System build. A production native build manifest
must represent the entire intended source set and its explicit prerequisites.

Readers -> introspection -> symbols and symbols -> emitter -> builders -> definitions
-> metadata -> PE remain separate boundaries. Do not reopen importer objects at emission.
Keep Reflection/Emit for ordinary .NET. General fixes should be isolated and validated
on main; fixes to portable contracts absent from main remain recorded integration work.
Existing Self infrastructure should be connected and completed, not replaced with a
new semantic design. Structural Function experimentation remains out of this strategy.

Use CLI-compatible signature/encoding and behavior where applicable: enums, inheritance,
numeric instructions and calling conventions. NeoCLR unit, erased Value, primitive
storage and Self differences need explicit mappings and cost/compatibility notes.
Re-use existing platform/bridge research for these contracts. No format fork, new
reflection replacement, new API design or performance claim is required for this batch.

## Selection rule and scorecard

Prioritize a fix by the number of independently blocked source families it advances,
its dependency reach and the confidence/cost of the change. File counts are supporting
evidence; do not invent exact unlock estimates from lexical counts or cascaded errors.
A small isolated API is worth doing when it is the shortest representative test of a
shared capability, not simply because it is easy to add.

For each family record separately:

1. Binding with explicit dependency identities.
2. Emission and metadata verification.
3. Reimport into a consumer with library sources absent.
4. Runtime execution and semantic checks.
5. Ordinary .NET regression/control status.

Report changes in those stages after each capability batch. Do not report a reduced
diagnostic count, successful declaration projection, larger instruction budget, or
increased accepted file count as equivalent to completion.

Keep one cumulative native gate, plus focused capability tests. Use the broad
application for integration and MemoryStream, text/encoding, task queue and JSON
consumers for complementary coverage. Re-run unchanged suites only when impacted.
The seven native consumers and metadata test baseline remain controls.

## Deferred work and maintenance

- Individual time-zone service wiring is no longer the next priority. It should
  fall out of the common scalar/vector service categories.
- Full async/exception features are not prerequisites without a source requirement.
- Full .NET execution of the rebuilt calendar/core subset remains unproved; preserve
  .NET controls rather than assuming native success establishes parity.
- The legacy translated snapshot refresh fails in SourceUnionReferences.Project
  with 'None' not in scope; the Instant digest is stale. Keep this visible as a
  separate reproduction/maintenance task. Do not substitute that bridge for native
  imports or make fixing all legacy tooling a prerequisite for native progress.
- No compiler/runtime feature was implemented in this assessment.

## Reproduce

Run audit_system_compilation.py with --compiler, --core, --seed and a fresh --output
directory. Use the matching Clock comparer core/seed documented in bootstrap/README.md.
The JSON records commands, source hashes, revisions, errors and output publication.
Minimal probes are embedded in the tool and materialized in the audit directory.
The checked-in evidence also records the additional explicit-unit-contract experiment.
