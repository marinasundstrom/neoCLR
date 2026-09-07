# Exact module revision labels

A module may declare an optional artifact revision:

```text
.module Models
.revision build-1
```

Labels are case-sensitive, nonempty ASCII letters, digits, underscores, dots, or
hyphens. They are opaque identifiers: there is no version ordering, compatibility
range, or automatic generation. The source module metadata stores `revision`; its
type and function definition identities include the same optional field. Supplied
definition rows must match the source name, revision, and row. Missing legacy
definition identities are derived from the containing module, including its revision.

## Dependency pins and definition references

```text
.references (Models#build-1, Utilities)
call [Models]Point::Answer() @ Models#build-1:0
```

The first dependency requires exactly build-1. A supplied Models artifact with another
label, or no label, fails even when its names and signatures match. The requirement
is checked whether or not any instruction uses Models. Utilities is name-only and
accepts any supplied revision. System remains implicit but can be pinned explicitly.

Name-only references preserve their old JSON string encoding. A pinned reference is
an object: `{"name":"Models","revision":"build-1"}`. A definition identity becomes
`{"module":"Models","revision":"build-1","index":0}`. Revision fields are omitted
for unversioned modules and definitions, retaining their existing encoding.

Explicit row identities always match exactly. `@ Models:0` identifies an unversioned
definition; it is not a wildcard for any revision. Use `@ Models#build-1:0` for that
revision, or omit the entire row selector and let symbolic call binding choose a
definition from the validated load set. The same selector syntax works for attribute
constructor references. Closed type keys and verifier function identities retain
revisions, so definitions from different declared revisions compare differently.

Scoped type operands remain `[Models]Point`: the load set and optional dependency
pin select the one Models artifact, and the scope checks its name. No revision syntax
is added inside type qualifiers in this slice.

## Limits and producer responsibility

A label is a producer assertion, not a content digest, signature, or compatibility
proof. Reusing a label for changed metadata can defeat the intended distinction;
producers must assign a new label when creating a distinct artifact revision and
regenerate definition rows. Editing only the revision header of an existing artifact
with old explicit rows is rejected as inconsistent metadata.

Rows can still change when definitions are reordered. Revisions distinguish declared
artifacts; they do not provide stable member identities across revisions. Unversioned
artifacts and name-only dependency references remain supported for compatibility.
Duplicate module names are rejected even if their revisions differ. Side-by-side
versions, automatic dependency discovery, content verification, and binary format
mapping remain future work.

`cargo run --example revisions` loads a pinned Answers dependency and prints 42.
The equivalent CLI invocation is `cargo run -- run examples/revisions/app.neoil
--module examples/revisions/answers.neoil`.
