# Generic type-state foundation

Development increment, 2026-10-21. Implements the author's requested generic-state
reflection properties for TypeInfo and replaces DI's private generic-parameter
walk with ContainsGenericParameters. See the
[design and .NET comparison](../../introspection-design.md#generic-type-state-development-2026-10-21).

Public properties: IsGenericType, IsGenericTypeDefinition,
IsConstructedGenericType, IsGenericParameter, ContainsGenericParameters.
Inheritance IsOpen is unchanged. Identity checks do not materialize intermediate guest
descriptor arrays. No throughput or end-to-end allocation benchmark is claimed.

The runtime identity test covers owner-sensitive definition recognition and
recursive wrappers/compositions. Three native query tests cover retained open
parameters/definitions, mixed open/closed catalogs and invalid selectors, including
native machine-code execution. Built-in type-relations assertions exercise the
public properties for closed generic types, ordinary inheritance-open classes,
arrays and structural functions. The DI suite checks the migrated registration path.

MethodInfo generic-state flags were pending at this slice. The subsequent
[method-state groundwork](../generic-method-state/README.md) adds them and interpreter
definition discovery; native type-method retention remains pending. This is a support gap, not a
policy that open methods are nongeneric. MakeGenericType/MakeGenericMethod and open
generic DI registrations are outside this increment. Native generic implementation
constructor retention remains separate.

Use matching runtime/backend and Raven class-library builds. The existing private
TypeShape contract adds selectors 21–24; parameter selector 17 is reused. There is
no native metadata format or compiler-emission change; compiler 793b71408 supplies
the class-library build used here.

Validation: all 24 built-in tests (six type-relation and 18 DI) pass on interpreter,
native and JIT, for 72 executions. See [contract evidence](contracts.json),
[runtime identity regression](runtime-tests.log) and
[three native query regressions](native-query-tests.log). API snapshot validation
passes and the website build checks 5,574 pages.
