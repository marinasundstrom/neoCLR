# Constructors, invocation and fields

Author-selected development slice, 2026-09-27. This extends the existing
[construction/property execution](reflection-execution.md) checkpoint. It is not
part of Preview 10. Introspection describes metadata; importing
`System.Runtime.Reflection` adds execution extensions. A descriptor alone does not
promise an executable target in the current RuntimeContext.

## Public contract

| Area | API |
| --- | --- |
| Introspection | `TypeInfo.GetConstructors()` and `GetConstructors(BindingFlags)` return `Sequence<ConstructorInfo>` |
| Reflection | `MethodInfo.Invoke(receiver: Object?, params arguments: Object?[]) -> Result<Object?, ReflectionError>` |
| Reflection | `FieldInfo.GetValue(receiver: Object?) -> Result<Object?, ReflectionError>` |
| Reflection | `FieldInfo.SetValue(receiver: Object?, value: Object?) -> Result<(), ReflectionError>` |
| Reflection | `TypeInfo.CreateInstance(params arguments: Object?[]) -> Result<Object, ReflectionError>` |
| Reflection | `TypeInfo.CreateInstance<T>(params arguments: Object?[]) -> Result<T, ReflectionError>` |

The existing parameterless CreateInstance overload remains. The author explicitly
selected Result returns after clarifying the initially sketched direct Object/T
results. Expected validation failures return errors; faults thrown by invoked code
remain terminal, matching the existing property/constructor policy.

ConstructorInfo is a separate MemberInfo contract, with IsPublic, IsPrivate,
IsAssembly, IsStatic, DefinitionIndex and GetParameters. It has no ReturnType and
is not a MethodInfo. Default enumeration returns declared public instance
constructors; flags can inspect nonpublic constructors. Constructors are not
inherited and remain excluded from GetMethods. Metadata tokens and parameter
identity follow the existing module-scoped contracts. Enumeration does not run code.
The target currently has no static initializer import/execution model.

## Bounded execution policy

Execution supports runtime-owned descriptors for nongeneric reference classes.
Construction requires a concrete publicly accessible class. Instance methods and
fields require a compatible non-null receiver; static methods require null.
Public IL methods are supported, including ordinary virtual dispatch. Native,
open-generic, byref/out and custom-value signatures remain outside this slice.
Abstract/interface method descriptors are not independently invocable in this slice.
Fields are instance fields; static storage is not currently part of this model.

Values support references/null and the same exact boxed built-in scalar types as
property execution. No numeric widening, string conversion, optional-argument
completion or recursive params expansion is performed by reflection. The facade's
params syntax packages arguments at the call site; a reflected target that itself
has a params array does not acquire a binder expansion rule.

Constructor selection considers public supported constructors of the correct arity,
requiring exact scalar types or assignable references for every argument. Zero
matches returns MissingConstructor; more than one returns AmbiguousConstructor.
There is no best-match ranking between assignable reference overloads. An explicit
null element can therefore make overload selection ambiguous. A null argument-array
object is invalid, distinct from an empty array or an array containing null.

Typed CreateInstance validates assignability of the described type to T **before**
constructor execution, then delegates to the same selection policy. T describes the
result view; it does not override the receiver TypeInfo or select its constructor.
Void-returning method invocation succeeds with a null object result. Field assignment
succeeds with unit. Invocation/assignment side effects are not transactional.

Source field access and read-only flags are retained separately from widened
executable helper visibility. Reflection denies nonpublic fields and writes to
read-only fields. Imported artifacts lacking the new source flags are not admitted
for field execution. Direct private-service calls revalidate the same conditions.
Existing descriptive field queries use source accessibility when available.

## Comparison and tradeoffs

Primary contracts reviewed 2026-09-27:

- [.NET 10 Type.GetConstructors](https://learn.microsoft.com/en-us/dotnet/api/system.type.getconstructors?view=net-10.0)
  uses ConstructorInfo and defaults to public instance constructors. neoCLR keeps
  that distinction and uses its existing Sequence contract instead of an array API.
- [.NET 10 MethodBase.Invoke](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.methodbase.invoke?view=net-10.0)
  and [Activator.CreateInstance](https://learn.microsoft.com/en-us/dotnet/api/system.activator.createinstance?view=net-10.0)
  offer substantially broader binding and invocation. neoCLR retains object-shaped
  arguments/results but makes validation errors Result values. Exact scalar matching
  reduces implicit conversion policy, at the cost of compatibility and convenience.
- [.NET 10 FieldInfo.SetValue](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.fieldinfo.setvalue?view=net-10.0)
  has binder/access restrictions and behavior beyond this checkpoint. neoCLR denies
  every source read-only field write; no visibility bypass or privileged public API
  is introduced. Broader metadata discovery does not grant execution permission.
- [Java 21 Constructor](https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/lang/reflect/Constructor.html)
  combines description and newInstance and permits widening argument conversions.
  neoCLR keeps execution in extensions and exact scalar matching. That separation
  leaves the introspection model usable without an execution provider, but requires
  an import and does not by itself solve independent runtime-context identity.

The alternative is to follow .NET's broader binder and exception policy, or make
construction a mandatory TypeInfo member. Neither is necessary for the current
consumer. A separate Activator object would also work; TypeInfo extensions preserve
familiar receiver syntax without adding execution requirements to the interface.
No performance improvement over .NET is claimed. Adapter frames, validation, boxing
and argument-array allocation have costs. This reuses existing research rather than
claiming a novel reflection mechanism or an alternative-.NET performance result.

## Implementation and validation boundary

Private member services build ordinary newobj/call/callvirt/field adapters. They use
the current interpreter frames, roots, cancellation and instruction/frame budgets.
Source field admission vectors must match declared field counts. Canonical type and
member identities are checked; display names do not bind executable targets.

The compiler bridge admits exact new signatures and a bounded generic activation
wrapper. It retains public nongeneric static IL methods when importing a described
application class, and admits public read-only fields while retaining its existing
source-store restrictions. Full pruning still needs an explicit dynamic target
policy. No new Raven Runtime Contract configuration is selected.

The generic wrapper uses a private type-token intrinsic because compiling the
TypeInfo contract and typeof-based wrapper in the same source slice otherwise falls
back to System.Type.GetTypeFromHandle. It emits the existing ldtoken instruction;
this is a target bridge workaround, not a language change or a new native service.

The executable source consumer is in `docs/experiments/reflection-members`.
Native adapter tests are in `tests/reflection_members.rs`; existing construction
and property tests remain relevant regressions. API documentation and library
artifacts must be rebuilt together. Generic class execution, value-type receivers,
static fields, ConstructorInfo.Invoke, asynchronous adaptation, optional arguments
and binder conversions remain future work.

## Validation recorded 2026-09-27

The matching Raven compiler/target bridge executed both the new member consumer and
the existing construction/property consumer successfully. Native construction,
property and member suites passed 19 checks; 26 exact reference signature checks
passed. Library source/artifact hashes, API reference fingerprints, XML member
uniqueness and extracted feature-page examples were checked. The subsequent combined
GC integration also built and checked the complete website; no platform matrix was
run. Raven integration documentation is commit `4dc15a17a`
on the `neoclr` branch; no compiler source changes were required.


## Retained constructors and union values (2026-09-27)

The next bounded startup-mapping prerequisite adds `ConstructorInfo.Invoke` as an
extension in System.Runtime.Reflection. It selects the descriptor's exact method
identity and executes through ordinary newobj/boxing frames. No overload search
occurs on invocation. Argument/access checks still run each time; this is not a
compiled delegate or globally cached native plan. Nongeneric value records, including
admitted union cases and carriers, now work through this descriptor path. Existing
TypeInfo.CreateInstance and method/property/field contracts remain unchanged.

The [.NET ConstructorInfo.Invoke contract](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.constructorinfo.invoke?view=net-10.0)
(reviewed 2026-09-27) is the ergonomic baseline: select a constructor and invoke it
with an argument array. neoCLR uses explicit Result validation errors and exact
boxed types instead of the richer CLR binder/coercion and exception model. This
keeps target UTF-8 strings on their normal boxed/reference path; it adds boxing,
argument-array and adapter costs. No performance advantage is claimed.

Application value constructors were previously imported only as CLI lowering
helpers. The bridge now also emits checked instance constructor wrappers retaining
the actual source identity and parameter metadata; the helper stays an implementation
detail. Wrappers call the original body and do not synthesize union tags or bypass
initialization. This is neoCLR bridge policy, not a Raven compiler change. Runtime
Contract configuration is unchanged; matching bridge/library/reference artifacts
are required. The VM now returns the initialized value from no-result by-reference
newobj constructors, as it already does for reference constructors.

[The executable case](experiments/union-construction/README.md) retains the case and
carrier constructors discovered via route attributes, constructs independent values
and recovers an ordinary Raven union. General cached route-schema parsing remains
next; JSON enum/Uuid/Option work stays pending.
