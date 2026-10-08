# Modules as namespaces: initial assessment

**Follow-up, 2026-10-09:** the author now directs implementation of the foundation,
explicitly retaining assemblies as packaging/identity and allowing many modules per
assembly. See the [implemented contract](../declaration-modules.md). The original
assessment below is retained as the historical proposal stage.

**Exploration recorded 2026-10-08; no implementation or format change.**
The [original Version 0.2 proposal](../proposals/first-class-module-system.md) is
preserved verbatim. The author noted that its artifact names do not reflect current
direction, then clarified: “The module becomes the namespace, technically.”
The author subsequently directed that RuntimeContext must adapt around modules,
since Assembly is no longer prominent. The assessment below is the assistant's
proposed interpretation, not an adopted
specification. The [platform roadmap](../platform-roadmap.md) retains work priority.

## Meaning of the proposed change

A module would be the namespace itself: a named declaration container with metadata
identity and visibility rules. Dots would still qualify names; there need not be a
second, independent namespace container above or alongside it.

For example, within assembly `System.Runtime`, module `System.Math` could own the
function `Sign` and constant `Pi`. Module `System` could own type `String`, which
would continue to own its methods. These are conceptual ownership examples using
existing names, not compiled module syntax or a proposed library repartition.

This would evolve the [current assembly-level member model](extended-cli-metadata.md#assembly-level-members-and-qualified-names-2026-10-08).
Today namespaces qualify names; assembly-level types, functions and constants have
an assembly member view. The supplied proposal retains the assembly as ultimate owner and deployment
identity, with the module becoming the immediate declaration container. The author's
RuntimeContext follow-up puts that assembly prominence up for reassessment: retaining
assembly packaging internally is an option, not a required public discovery model. An assembly-wide enumeration could remain an aggregate view. Existing
`ModuleDefinition` and `ModuleInfo` contracts describe metadata modules; they cannot
silently acquire these new semantics. Nested types and type members retain type owners.

The concrete benefit to investigate is whether a library can keep helpers private
to a functional area while exposing types, functions and constants through one
inspectable container, consistently for every frontend and execution mode.

## RuntimeContext centered on modules

**Author direction:** “RuntimeContext will have to be adapted around modules too.
Since Assembly is not prominent anymore.” This extends the discussion beyond naming
and declaration ownership into the runtime-facing API. It does not yet select exact
signatures or remove the underlying assembly identity machinery.

The assistant recommends evaluating direct module discovery and resolution through
RuntimeContext, without requiring callers to enumerate assemblies first. A module
descriptor would lead to its declarations; runtime binding would associate those
descriptions with executable entities in the relevant context. Metadata-only readers
would continue to work without implicitly loading or executing anything.

An executing-module query is a possible counterpart to today's ExecutingAssembly,
not a selected property name or implemented API. Its identity must follow the logical
owner of the calling code, independent of native inlining, runtime service wrappers
or packaging. A type method belongs to the module through its declaring type. Test
calls from different modules and dependencies in both execution modes. Loading a
module must not imply that it has its own file, GC heap, scheduler or unload lifetime.

Unresolved choices include discovery versus loading, duplicate module paths,
context-specific realization, dependency resolution, hot-reload identity, and whether
assembly descriptors remain an advanced packaging view. If modules become independently
versioned or loadable, that would go beyond the supplied proposal's subdivision model
and require a separate decision. Existing assembly-based APIs need an explicit migration;
this document neither renames nor removes them.

## Comparison and alternatives

Primary sources consulted 2026-10-08:

- [ECMA-335, sixth edition](https://ecma-international.org/wp-content/uploads/ECMA-335_6th_edition_june_2012.pdf),
  Partition I §8.5.1 and Partition II §22 (Module, TypeDef and MethodDef tables): CLI
  namespaces are encoded names rather than independent declaration containers.
  Physical modules contain metadata; globals use the special `<Module>` type.
  This is the CLI baseline, not evidence that every modern .NET host supports all
  multi-module deployment options. A neoCLR logical module needs its own explicit
  representation rather than assuming the existing Module row means a namespace.
- [Rust Reference: visibility and privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html)
  documents modules as scopes with access checks, restricted visibility and re-exports.
  It demonstrates the utility of real module boundaries, but its descendant modules
  can access parent-private items. The supplied neoCLR proposal explicitly denies that
  privilege. Rust's source-language rules do not establish neoCLR runtime enforcement.

| Alternative | Benefit | Cost / limitation |
| --- | --- | --- |
| Keep assembly ownership and qualified namespace names | Preserves the current reader/writer and source contracts | No module-private boundary or first-class namespace descriptor |
| Add compiler-only modules, lowering to current metadata | Smaller runtime change; useful source organization | Other frontends and hand-authored metadata can bypass compiler-only boundaries |
| Make modules the namespaces in native metadata | Explicit ownership, privacy and discovery across frontends | Changes resolution, metadata APIs, compatibility and verification |

The third alternative best matches the author's question. This is a provisional
assessment of fit, not evidence that its migration cost is justified for the release.
No performance advantage is claimed. Independent .NET project experience and concrete
API-review discussions have not been evaluated in this initial assessment; they remain
research inputs before settling the cross-cutting contract.

## Boundaries to decide

1. **Logical versus physical modules.** Multiple namespace modules should not require
   multiple files, independent token scopes or separate deployment. Keep the existing
   physical metadata module concept distinct in the codec, even if public terminology
   eventually changes. The original's artifact extensions and Raven-like `fn`/`use`
   syntax are illustrative, unvalidated source material, not selected spellings.
2. **Identity across assemblies.** The original proposes assembly identity plus module
   name. Two assemblies defining `System.Math` would therefore own distinct modules.
   An import could expose both through a combined lookup view, or require qualification
   or aliases; it must not merge their private scopes. Duplicate names and overloads
   need deterministic ambiguity rules. Dotted parents need not exist or confer access
   unless explicitly specified. Decide whether moving a declaration between modules
   changes binary identity and what forwarding or re-exports preserve.
3. **Accessibility.** Distinguish module-private declarations from private members of
   types. Define whether modules themselves have visibility, whether public members of
   hidden modules can be re-exported, and whether public signatures may expose private
   types. Enforce the same rules on source and direct metadata inputs. Decide which
   checks happen at load/verification versus execution; do not assume current checks
   already cover the proposed ownership model.
4. **Metadata compatibility.** Reusing MethodDef and ordinary calls is attractive;
   explicit ownership still requires a versioned encoding and resolution changes.
   A physical CLI projection may need carriers even when native semantic ownership
   does not. Preserve imported type-owned methods and token provenance. Projecting
   CLI namespace names as modules should be an explicit adapter view, not a silent
   identity rewrite. Ordinary .NET targeting remains the default in Raven.
5. **State and execution.** Start the evaluation with types, functions and constants.
   Mutable fields, properties and events add initialization, fault, concurrency and
   runtime-context lifetime questions. Those must agree in interpreted and native
   execution, including future scheduling and hot reload. Modules alone do not resolve
   these questions or define an unload boundary.
6. **Native metadata and ABI.** Module descriptors could organize exports in embedded
   or sidecar metadata. Stable calling conventions, symbol mapping, layout, version
   matching and metadata retention remain separate contracts. Namespace ownership
   alone neither supplies reflection nor makes trimming safe; retained declarations
   and reachable executable code need explicit policy. No artifact suffix is selected.

## Suggested bounded experiment, not performed

Use two logical modules within one assembly and a separately compiled consumer.
Include an existing-style type, overloaded function, constant and private helper.
Round-trip their owners through the writer, reader and introspection model; compare
source and direct-metadata references. Then introduce an identically named module in
another assembly and a child dotted path. Require explicit ambiguity diagnostics,
private-access rejection and unchanged type-owned method identity.

Run admitted calls in the interpreter and AOT with identical output and faults.
Test a CLI import projection and unsupported-reader rejection rather than assuming
binary compatibility. Avoid mutable module state in this first experiment.
Success would demonstrate useful ownership and access semantics without requiring
new deployment files or changing ordinary call behavior. Failure would be evidence
to retain current ownership or narrow the design. Neither implementation nor a
release commitment follows from recording this proposal.
