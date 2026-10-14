# Native structural types: closed-reference POC

This is the acceptance entry point for the paired feature branches
`codex/structural-union-intersection` (neoCLR) and
`codex/neoclr-union-intersection` (Raven). It covers the bounded interpreter POC;
full open-generic support is the next stage, as directed by the author. It does
not mark the full structural type feature or the platform roadmap complete.

Compared with CLR metadata, unions and intersections are native signature types
with runtime identity and structural descriptors, rather than nominal ad hoc
carriers. Declared Raven unions retain their nominal declarations. This preserves
any-of/all-of contracts through compilation, imports and execution, at the cost
of native-only interoperability and additional runtime relationship checks.

Interfaces remain nominal: types explicitly implement them, directly or through
declared inheritance. “Structural” describes the composition signature, not
member-shape matching or implicit interface conformance. This is not a Rust-style
trait system. A future compiler-level simulation of trait characteristics is a
separate exploratory possibility, not part of this runtime contract.

Acceptance on 2026-10-14: all 20 checks passed, including 72 experimental native
tests, the default-profile rejection test, host metadata checks, both source examples
and eight native consumer executions. See [retained results](results.json) for
commands, revisions and artifact hashes. This completes the bounded POC described
below; the remaining implementation work is listed separately. Follow-up host and
Raven checks also reject a matching-member class that lacks the required nominal
interface declaration.

## Acceptance contract

| Capability | Executable evidence |
| --- | --- |
| Source union parameters, results and locals | Source structural union probe; no metadata patching |
| `Task<Response \| HttpError \| JsonError>` | Three completed and three queued deliveries; each payload type and value checked after `await` |
| Intersection parameters, results, locals and trait calls | `Animal & Quacks`; source intersection probe |
| Structural introspection | `UnionTypeInfo` / `IntersectionTypeInfo`, constituents, membership and closed wrapper argument identity |
| Separate library consumption | Source union/intersection producers and consumers; native constraint library and consumer |
| Closed structural method bounds | Any-of/all-of checking, closed `Response<A> \| B`, forwarding and invalid-argument rejection |
| Generic-body traits | Local, imported nongeneric and imported constructed interface targets through locally declared closed bounds |
| Compatibility boundaries | Default runtime/catalog/backend rejection, CLI projection rejection, nominal/Self regressions |
| Intersection admissibility | Native tests reject CLR-final/value operands; source semantic evidence preserves Raven sealed hierarchies |

The generic-body proof requires every alternative of a union to guarantee the
requested trait. A closed nominal class can satisfy an interface intersection;
it cannot be written as an intersection operand. Invariant constructed arguments
remain exact: `Reads<int>` does not imply `Reads<string>`.

## Reproduce

Prerequisites: the two matching feature checkouts, .NET 10, Rust, and a native
class-library directory built with the structural introspection APIs. The runner
checks the library inputs, rebuilds the compiler and interpreter, and records exact
revisions and hashes. A published library without these APIs is not interchangeable.
On macOS set `SDKROOT` to the installed macOS SDK if the Rust linker needs it.

From the neoCLR repository:

```sh
python3 docs/experiments/structural-poc/verify.py \
  --raven /path/to/raven-union-intersection \
  --library target/composition-info-libraries-v2
```

The runner writes applications, per-step logs and a machine-readable report under
`target/structural-poc`. It fails on an unexpected exit code and retains a failure
report. It enables native compositions explicitly; the source projects disable the
temporary generic async bridge and use native async state machines. It does not
rewrite signatures, edit the supplied library or publish artifacts. Retained
acceptance evidence belongs in `results.json` alongside this document.

## Remaining implementation work

- Open operands and complete recursive substitution, including `T | HttpError |
  JsonError`, `Request<T> | HttpError | JsonError`, and
  `Task<Response<T> | HttpError | JsonError>` in open generic definitions.
- Generic owner/type-parameter compiler
  emission, broader constrained member imports and mixed source formulas.
- Guest generic-constraint reflection, structural member/hierarchy/custom-attribute
  enumeration and unrestricted generic method specialization with structural arguments.
- JIT/AOT qualification and broader storage/value/function-type composition profiles.

Those remain explicit gaps, not permanent semantic restrictions. Host metadata
constraint introspection is implemented; it must not be confused with guest
constraint reflection. Native runtime support for generic type constraints is
broader than the current Raven method-constraint authoring profile.

Earlier implementation checkpoints are retained under
[structural compositions](../structural-compositions/README.md) and
[structural constraints](../composition-constraints/README.md). Their pending-work
statements describe their individual checkpoints; this document is the current
bounded acceptance contract.

Follow-up: imported closed union alternatives are now admitted and the acceptance
runner also executes a separate consumer of such a constraint library. See the
[external union checkpoint](../composition-constraints/README.md#external-union-constraint-alternatives--2026-10-14-development).
The retained 20-check report above predates this additional consumer.

Resolved closed external intersection bounds are also supported. Their native reads
require an explicit dependency resolver; the Raven catalog supplies selected snapshots.
The follow-up consumer checks nominal conformance and reverse dependency ordering.
See the [resolved intersection checkpoint](../composition-constraints/README.md#resolved-external-intersection-bounds--2026-10-14-development).
