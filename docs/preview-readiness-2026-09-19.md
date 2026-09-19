# Preview readiness review — 2026-09-19

This reviews the current development state against the author's goal: a small,
working preview that shows the platform's direction and invites feedback. It does
not select a version or release date, certify a release candidate, or promise the
entire proposal catalog. The String/website work is locally committed; nothing in
this review publishes a runtime, extension or website.

## What evaluators can try now

| Area | Working development surface | Evidence and limits |
| --- | --- | --- |
| Runtime foundation | Raven-authored System.Runtime and programs executing on neoCLR | 77 generated implementation slices reproduce cleanly; selected-profile ownership check reports 862 declarations and 74 explicit native services |
| Introspection | TypeInfo in the sealed MemberInfo family; Object.GetType/typeof; RuntimeContext.ExecutingAssembly; assembly references, module-scoped tokens and Sequence collections | Runnable discovery/matching samples and earlier targeted runtime/editor checks; retained loaded metadata only, without dynamic loading, invocation or emit |
| Strings | Grapheme Char, String.Length/iteration, explicit scalar traversal, canonical UTF-8 storage, strict Utf8 Encode/Decode and ordinal ordering | UTF-8 round trips, invalid sequences, BOM/NUL preservation, snapshot independence, allocation bounds, signature/editor checks and saved-project programs pass |
| Outcomes | Option/Result, patterns and propagation | Executable examples expose recoverable outcomes without requiring carrier-specific extraction methods |
| Collections and queries | Capability interfaces, array/list views, lazy queries and typed terminal outcomes | Existing sample-backed implementation; read-only views are not immutable snapshots |
| Dates and clocks | Date/Time, Instant/Duration and injectable Clock/SystemClock | Fixed-clock and system-clock sample; calendar/timezone/globalization proposals are broader than the implementation |
| Files | Bounded synchronous UTF-8 whole-file reads/writes with typed errors | Existing file verification passed earlier in this work; filesystem capabilities and streams remain proposals |
| Tooling and explanation | Fresh local VS Code workspace, six feature pages, proposal overview and website maintenance rules | Local saved-source grapheme execution and completion pass; ten pages build with checked links and source excerpts |

The fresh local workspace is
`~/.neoclr/experiments/grapheme-text-20260919/demo`. Its manifest records runtime,
core, bridge, compiler and server hashes. It preserves the older workspace instead
of overwriting evaluator edits. This is a local development snapshot, not a public
SDK release or evidence for untested hosts.

## Grapheme text foundation

The approved development model now uses owned grapheme Char values, String.Length
and Iterable<char>, with explicit Sequence<uint> scalar traversal and UnicodeScalar
classification. UTF-8 remains canonical storage. The working Raven sample includes
combining marks, family emoji, literal patterns, arrays and interface iteration.
See [the contract and migration](design/text-abstraction.md).

The current profile contains 77 Raven implementation slices, with 862 declarations
and 74 explicit native service declarations in the ownership check. The earlier
scalar-Char snapshot is historical. Evaluators must refresh the matching compiler,
reference metadata, library and runtime together.

Remaining text design questions are normalization/collation, a dedicated scalar
value type, efficient iteration and cursor-based slicing. Integer string indexing
is deferred. Literal diagnostics currently use host Unicode segmentation, while
runtime construction enforces pinned Unicode 16 rules; this tooling difference and
CLI constant metadata need resolution before promising a stable contract. These
limits should be stated in the preview rather than suggesting all text processing
or all Char language features are complete.

## Keep the demonstration small

Use the current feature pages as the evaluator entry points. Each should say what
works, how to run its example, known limits and where it may go next. Prefer a few
verified examples over more declarations that imply unavailable runtime behavior.
The pages for Introspection, Strings, outcomes, collections, time and files cover
the main library story. The proposal overview carries the broader direction.

No additional networking, Task runtime, stream family, culture API, dynamic assembly
loading, reflection invocation or emit implementation is required merely to make
this preview communicate the direction. These have useful proposals but depend on
contracts and runtime work beyond the current small demonstration.

## Release gates still outstanding

1. Review the documented grapheme String/Char limits above before
   presenting the native text model as complete. Re-run affected runtime and Raven
   tests; keep neoCLR policies on Raven's neoclr branch and extract general fixes
   independently if any are discovered.
2. Select exact neoCLR/Raven candidate revisions and run the full source/archive
   validation matrix and matching SDK/VSIX package gates. Focused local checks in
   this work do not replace Linux/macOS/Windows release evidence.
3. Build and verify a fresh extracted evaluator bundle: completion, saved-source
   build/run, matching reference metadata, sample output and no stale-artifact
   fallback. The local snapshot proves a path, not the final distribution.
4. Review migration notes: System.Type → TypeInfo, sealed MemberInfo matches,
   Sequence returns, IsEmpty property and changed ordinal ordering. Include the
   grapheme-Char migration. Preserve published release notes.
5. Select version/date and scope, then align downloads and website status with the
   actual release. Push/PR site validation is automatic; publication is manually
   dispatched on main after review. No deployment has been run here.

The existing [validation procedure](next-preview-validation.md) remains useful;
its older candidate selection must not be mistaken for this review's candidate.
The [website workflow](design/feature-pages.md) is now a durable part of feature and
release maintenance.

## Local evidence from this work

- 77-slice clean bootstrap regeneration and input/output hashes pass.
- Full runtime all-targets testing completed: 1280 tests pass after correcting
  two outdated native-service inventory counts and rerunning their suites.
- 3 UTF-8 runtime tests, 8 String tests and 5 disposal tests pass.
- 5 ordinal tests and the Raven boundary sample pass after the native-order change.
- UTF-8 plus three existing String sample programs and 23 edit/rejection checks pass.
- Four feature-page sample checks (propagation, collection capabilities, query
  terminals and clocks), plus their 23 edit/rejection checks, pass.
- Exact-signature admission and String/Utf8 language-server completion pass.
- The fresh grapheme snapshot builds/runs its expanded sample (including fields,
  defaults and copying) and passes 20 completion sections.
- The grapheme compiler contract passes 94 focused compiler/lexer tests. The
  saved-project regression passes 31 outcomes; scalar checks pass 125 Boolean/numeric
  outcomes plus four invalid classifications, and 137 signature checks pass.
- Tokenizer test, 3 website-builder tests, all ten page builds and repository-link
  target checks pass. Workflow YAML parses; manual-only deployment guard reviewed.

Introspection, file and broader library checks are earlier evidence in this task,
not a newly run full release gate. No CI workflow or remote publication result is
inferred from local success.
