# Neo grammar

This is the grammar of the implemented Neo subset, not a grammar for Raven.
The standalone [neo.ebnf](neo.ebnf) is the grammar source. The [Neo guide](neo.md)
contains runnable examples and commands. Parser and semantic regressions live in
[neo.rs](../tests/neo.rs).

The EBNF uses commas for sequence, `|` for alternatives, brackets for optional forms,
braces for repetition, and `? … ?` for lexical/context conditions. Character ranges
use `..`. `terminator` may be empty only immediately before `}` or end of input.
There are no empty statements apart from separators.

## Declarations and statements

```ebnf
program          = separators, { declaration, separators }, end_of_input ;
declaration      = union_decl | delegate_decl | import_decl | class_decl | record_decl | interface_decl | function_decl ;
union_decl       = "union", identifier, newlines,
                   ("(", newlines, type, { newlines, "|", newlines, type }, newlines, ")", terminator
                   | "{", separators, union_case, { separators, union_case }, separators, "}") ;
union_case       = "case", identifier, [ field_list ], terminator ;
delegate_decl    = "delegate", identifier, parameter_list, "->", type, terminator ;
import_decl      = "import", qualified_name, ".", "*", terminator ;
class_decl       = [ "abstract" ], "class", identifier, [ ":", type, { ",", type } ],
                   newlines, "{", separators, { class_member, separators }, "}" ;
class_member     = class_field | record_member ;
class_field      = "var", identifier, ":", type, [ "=", expression ], terminator ;
record_decl      = [ "abstract" ], "record", identifier, field_list, [ ":", type, { ",", type } ],
                   (terminator | newlines, "{", separators, { record_member, separators }, "}") ;
record_member    = init_decl | explicit_method | "static", generic_function_member | [ "readonly" ], [ "abstract" ], [ "virtual" | "override" ], method_decl ;
init_decl        = "init", parameter_list, [ ":", "base", "(", newlines, [ expression, { ",", newlines, expression } ], newlines, ")" ],
                   newlines, "{", separators, { statement, separators }, "}" ;
interface_decl   = "interface", identifier, [ ":", type, { ",", type } ], newlines, "{", separators,
                   { interface_member, separators }, "}" ;
interface_member = [ "readonly" ], [ "abstract" ], "func", identifier,
                   [ ".", identifier ], parameter_list, "->", type,
                   (terminator | block) ;
method_decl      = "func", identifier, parameter_list, "->", type,
                   (terminator | newlines, "{", separators, { statement, separators }, "}") ;
explicit_method  = [ "readonly" ], "func", identifier, ".", identifier,
                   parameter_list, "->", type, newlines, block ;
generic_parameters = "<", identifier, { ",", identifier }, ">" ;
generic_function_member = "func", identifier, [ generic_parameters ], parameter_list, "->", type, newlines, block ;
function_decl    = "func", qualified_name, [ generic_parameters ], parameter_list, "->", type, newlines,
                   "{", separators, { statement, separators }, "}" ;
field_list       = "(", newlines,
                   [ field, newlines, { ",", newlines, field, newlines } ], ")" ;
field            = identifier, ":", type ;
parameter_list   = "(", newlines, [ parameter, newlines,
                   { ",", newlines, parameter, newlines } ], ")" ;
parameter        = [ "out" | "readonly" ], identifier, ":", type ;
type             = "readonly", type | (qualified_name, [ "<", type, { ",", type }, ">" ] | "(", ")"), { "[", "]" }, [ "&" ] ;
qualified_name   = identifier, { ".", identifier } ;

block            = newlines, "{", separators, { statement, separators }, "}" ;
statement        = (binding | return_statement | expression_statement | "break" | "continue"), terminator
                 | if_statement | let_else_statement | "while", expression, block | "loop", block
                 | "for", identifier, "in", expression, (".." | "..<"), expression, block ;
if_statement     = "if", [ "let", conditional_pattern, "=" ], expression, block,
                   [ newlines, "else", (if_statement | block) ] ;
let_else_statement = "let", conditional_pattern, "=", expression, newlines, "else", block ;
conditional_pattern = identifier, [ "(", (identifier | "_"), ")" ] ;
binding          = ("let" | "var"), identifier, [ ":", local_type ], "=", expression
                 | "var", identifier, ":", type ;
local_type       = type, [ "[", integer, "]" ] ;
return_statement = "return", [ expression ] ;
expression_statement = expression, [ "=", expression ] ;
```

A program must define one parameterless `Main`. Other declarations may appear before
or after it. There are no top-level executable statements in this slice. Fields and
parameters share name-before-type syntax. Except for heap array initializers, trailing commas, overload declarations,
function expression bodies and standalone blocks are unsupported. Structured statements
introduce nested scopes; active names cannot be shadowed, but sibling scopes may reuse names.

Types resolve to `int`/`Int32`, `string`/`String`, `bool`/`Boolean`, `unit`/`Void`/`()`,
or a declared record, interface or bundled System type. Qualified names and closed generic
arguments are supported; Option/Result abbreviate System.Option/System.Result. One
`&` suffix forms a managed reference. Raw pointer syntax and generic type declarations
are unsupported. Free functions and static methods accept generic parameters. Runtime restrictions on ByRef generic arguments still apply.

## Expressions

```ebnf
expression       = lambda | logical_or, { "match", newlines, "{", separators,
                   [ match_arm, { arm_separator, match_arm }, [ arm_separator ] ], "}" } ;
lambda           = (identifier | "(", newlines, [ lambda_parameter,
                   { ",", newlines, lambda_parameter } ], newlines, ")"),
                   "=>", newlines, (expression | block) ;
lambda_parameter = identifier, [ ":", type ], newlines ;
match_arm        = pattern, "=>", newlines, (expression | block) ;
pattern          = "_" | identifier, [ "(", ("let", identifier | "_"), ")" ] ;
arm_separator    = ("," | newline), separators ;
logical_or       = logical_and, { "||", logical_and } ;
logical_and      = equality, { "&&", equality } ;
equality         = comparison, { ("==" | "!="), comparison } ;
comparison       = additive, { ("<" | ">" | "<=" | ">="), additive } ;
additive         = multiplicative, { ("+" | "-"), multiplicative } ;
multiplicative   = projection, { ("*" | "/"), projection } ;
projection       = unary, { "as", type } ;
unary            = ("&" | "-" | "!" | "new"), unary
                 | "new", type, "[", expression, "]", [ array_initializer ] | postfix ;
array_initializer = "{", newlines, [ expression, newlines,
                    { ",", newlines, expression, newlines }, [ ",", newlines ] ], "}" ;
postfix          = primary, { ".", identifier | arguments | "<", type, { ",", type }, ">" | "[", expression, "]" } ;
arguments        = "(", newlines,
                   [ argument, newlines,
                     { ",", newlines, argument, newlines } ], ")" ;
argument         = [ "out" ], expression ;
generic_member   = qualified_name, "<", type, { ",", type }, ">", ".", identifier ;
primary          = generic_member | integer | floating | string | character | "true" | "false" | "this" | identifier
                 | "[", newlines, expression, newlines, { ",", newlines, expression, newlines }, "]"
                 | ("typeof" | "default"), "(", newlines, type, newlines, ")"
                 | "(", newlines, expression, newlines, ")" ;
```

Member access and calls bind most tightly, then unary operations, interface projection with `as`, multiplication and
division, addition and subtraction, ordering comparisons, equality, `&&`, then `||`.
Binary operators associate to the left. Conditions require Boolean; `&&` and `||`
short-circuit. Ordering supports Int32 and Double; equality also supports Boolean.
Double ordering is false for NaN, while inequality is true.
`&counter.Age` therefore addresses the field; `reference + 1` automatically reads a
managed integer reference. Arithmetic uses matching Int32 or Double operands; mixed operands require a future
explicit conversion facility. Unary `*` is not
a managed-reference operator; pointer syntax is still outside this grammar.

The grammar permits general postfix shapes, but semantic checks restrict calls to
free functions, positional record construction, explicit `int(byteOrInt)` conversion,
public static/ordinary instance bundled System calls, and declared record/interface
instance methods. Library overloads are selected by exact types after reading bare
reference arguments when the declared contract expects a value. Library receiver and
output contracts are projected as described in [output parameters](neo-outputs.md).
Source record/interface methods use managed reference receivers. Explicit generic free/static calls are supported; generic instance methods remain unsupported. `int(value)` supports Byte/Int32
only and lowers to checked Int32 conversion. These are static restrictions on the
existing call grammar; no new expression production is needed.
`new` accepts record construction, such as `new SimpleCounter(0)`, or managed array
construction (`new int[3]`, `new int[3] { 1, 2, 3 }`, and the earlier
`new array(3, 0)` form). It does not accept arbitrary
factory calls or copy expressions in this slice. Assignment and
`&` require appropriate addressable locations or existing managed references. There is
no assignment expression or implicit numeric conversion. T& is read automatically
when a value is needed. Source signatures retain references for T& parameters/returns;
System calls use explicit `&argument` for reference arguments.

## Lexical and layout rules

Identifiers use ASCII letters, digits and underscore and cannot start with a digit.
Reserved words are `func`, `record`, `let`, `var`, `return`, `new`, `true`, `false` and
`import`, `if`, `else`, `while`, `for`, `in`, `loop`, `break`, `continue`, `match`, and `typeof`. Type aliases and `Console`/`WriteLine` are additionally reserved declaration
names. Fields, parameters and bindings must be unique in their applicable scope.

Integer tokens contain decimal digits only. Positive literals must fit Int32; a
literal immediately prefixed by minus may represent -2147483648. No suffixes, digit
separators are implemented. Double literals have a fractional part or exponent:
`digits, [ ".", digits ], [ ("e" | "E"), [ "+" | "-" ], digits ]`, with at least
a fraction or exponent. A decimal point requires digits on both sides; exponents
require digits. Literals must be finite and lower to ldc.r8. Strings use double quotes and
JSON-style escapes, including Unicode escapes; raw line breaks are rejected.
Character literals use single quotes around one UTF-16 unit or an escape; see
[character classification](character-classification.md) for the bounded escape grammar.
Supplementary scalars need two units and cannot form a single Char literal.

Whitespace other than LF is ignored outside strings. LF is a statement separator;
CRLF works because CR is ignored. `//` comments end before LF. Semicolons also separate
statements. A final statement may omit its separator before `}` or end of source.
Blank lines and repeated separators are accepted between declarations/statements.

Newlines are accepted at field/argument-list boundaries, inside the opening/closing
parentheses of a grouped expression, and before an opening block brace or `else`. They
do not implicitly continue a binary expression or split the `name: type` pair.
Semicolons inside lists/grouped expressions are not layout whitespace.

The implementation bounds source input at 1 MiB, recursive parser nesting at 32 (shared by types, statements and expressions),
expression tree depth at 128,
record/function declarations at 1024, and fields/parameters per declaration at 1024.
Backend metadata and execution limits still apply. Source syntax and bounds remain
preview contracts and can change as end-to-end scenarios require.

## Match semantics

`match` binds below Boolean operators. Its scrutinee is evaluated once and copied
into temporary value storage (a T& scrutinee is read through its reference). Case
patterns bind copied payloads with `Case(let name)`, discard them with `Case(_)`, or
use a bare name for a payload-free case. Nested unions use nested matches. `_` covers
all remaining cases. Arms are separated by commas or newlines, with a trailing
separator permitted. Block arms are available only for a standalone match statement;
that form needs no trailing statement terminator.

Both forms require exhaustive coverage and reject duplicate or unreachable cases.
Expression arms have one exact result type after contextual reference reads. Statement arms may perform actions,
return, break or continue; their expression results are discarded. Payload names
are immutable, arm-local and cannot shadow active names. References follow the
ordinary lifetime rules; no address to copied arm payload storage can escape.

Bundled System union coverage uses the UnionAttribute marker,
constructor-declared cases, and typed public test/extraction/payload accessors.
The compiler validates that contract; arbitrary source records are not unions merely
because their names resemble Option or Result. Unsupported contracts are diagnosed.
Non-generic source declarations generate the corresponding constructors and accessors;
their case bindings capture whole variants. Guards, nested destructuring patterns,
generic source unions and subtype patterns remain future features.

## Type operands and metadata properties

`typeof(T)` accepts a type signature, including aliases, qualified names, source
records, closed generic types, `()` and T&. It produces an ordinary System.Type value
through `ldtoken T` and `System.Type.GetTypeFromHandle`; it does not evaluate an
expression or create an instance of T. A binding name or function call is not a type
operand. Unknown/invalid signatures are rejected by the existing metadata checks.

Public non-indexed instance properties of bundled System types can be read with
member syntax, including `typeof(int).Name` and `descriptor.GenericArgumentCount`.
The compiler resolves the declared getter and checks its public, typed value-receiver
contract. It does not expose private backing fields, property assignment or addresses
of properties. Source record field access keeps its existing rules.

A reference RHS retargets a mutable reference binding (`r = otherReference`).
A value RHS writes the target, including through immutable reference bindings.
Reference creation from a value remains explicit (`r = &otherValue`). `&r` forwards r's reference without constructing T&&.
Inferred bindings preserve references; explicitly value-typed bindings copy their
referents. These are source access rules over existing managed references, not changes
to native pointer semantics. See the [managed-access guide](neo.md#managed-references-are-transparent-pointers-are-explicit).

## Interface declarations and reference projections

See [Neo interfaces](neo-interfaces.md) for the implemented semantics and example.
Record instance methods have an implicit managed `this` receiver. Interface methods
are signatures without bodies. Conformance and `as Contract&` targets must name
source-declared interfaces. Concrete managed references implicitly project to
implemented interface references in typed argument, binding and return contexts;
bare values are not implicitly addressed. Interface and record generic declarations remain unsupported. `interface`, `as` and `this` are reserved names. Interface names follow
ordinary Neo naming, without an `I` prefix.

## Closed generic static member calls

`System.Collections.ArrayList<Counter&>.Allocate(0)` selects a static member on a
closed generic type. Type arguments may include managed references and nested closed
types. Generic free/static functions are specified separately below; argument inference is available for source free/static functions. Ordinary
comparisons remain expressions. A reference to a bundled type can implicitly convert
to an interface declared by that closed type, without addressing or boxing a value.
See the [complete collection example](../examples/source/collections.neo).

## Output parameters and uninitialized locals

`func Initialize(out value: Foo&)` declares an unconditional output contract;
`Initialize(out destination)` forms or forwards its destination reference.
`var destination: Foo` declares uninitialized storage. The existing verifier checks
caller initialization; runtime guards enforce callee assignment obligations. Calls
to library conditional outputs are supported on direct success branches. See
[output references](neo-outputs.md) for syntax, examples and limitations.

## Local array extents and heap initializers

`let a: int[3] = [1, 2, 3]` checks a local array extent. The extent must be a
nonnegative Int32 literal and the declaration must have an initializer. Literal
count mismatches are compilation errors; dynamically produced arrays are checked
at runtime before binding. Subsequent writes retain the runtime's fixed-shape rules.
This is a local constraint on T[], not a distinct runtime type: signatures, fields,
generic arguments and typeof continue to use T[]. Use `var` to take a writable
reference to owned storage; `int[]&` can address either frame or managed heap storage.

Use `new T[length]` for default initialization. Empty braces are accepted but
omitted from samples.
Defaults remain limited to the runtime-supported element types. A nonempty initializer
must supply exactly length elements, including for strings and records that have no
default. The length is evaluated once and checked before elements run; elements run
once each, left to right. A dynamic count mismatch faults before element evaluation.
Newlines and a trailing comma are allowed inside braces. Earlier array construction
forms remain accepted.

Bracket expressions also project bundled instance Item properties with one index
parameter. Reads select the getter; direct indexed assignments select the setter.
Interface views dispatch virtually. Only reference-returning getters can supply
managed addresses; value-returning getters yield copies. See [library indexers](neo.md#library-indexers).

`readonly name: T&` declares a managed input-reference restriction. It excludes
output parameters and leaves call syntax unchanged. An explicit & argument may
address an immutable owned binding in this context. See [readonly parameters](readonly-parameters.md)
for enforcement, the partial verifier projection and current limitations.

Inside record/interface bodies, `readonly func` declares a readonly managed receiver.
The modifier is not valid on free functions. See [receiver semantics](readonly-parameters.md#readonly-instance-receivers).

See [readonly storage and return signatures](readonly-storage.md) for implemented
readonly T& type positions, checked boundaries and migration. Binding immutability
remains a language feature. [Explicit nullability](nullability.md) is a planned
signature characteristic and special state, not implemented syntax or zeroing.

A record may name one source record base first in its colon list, before interfaces.
Aggregate arguments include inherited fields first. See [inherited value layout](inherited-layout.md)
for current restrictions. [Base-reference conversions](base-views.md) are implemented;
[Virtual/abstract methods](class-dispatch.md) are implemented; abstract methods
require a terminator instead of a body.

Explicit init declarations and base(...) initializers are implemented; see
[constructor chaining](constructor-chaining.md) for the one-initializer source limit,
field initialization rules and unchanged aggregate construction without init.

[Explicit interface implementations](explicit-interfaces.md) use `func Interface.Member`
in records, optionally readonly. They do not introduce ordinary callable class members.
Record explicit bodies cannot be virtual/override/abstract. Interfaces now support
[default bodies and qualified replacements/reabstraction](default-interface-implementations.md).
An abstract declaration has no body; virtual/override modifiers remain unsupported
on interface declarations.

## Generic free functions and static methods

A free function may have a qualified name (`func Utility.Identity<T>`) without a
namespace block. Static record members use `static func Identity<T>`. Both support
argument-based type inference; `<type, ...>` is a postfix type-argument
list immediately followed by ordinary call arguments. For example,
`Utility.Identity<int>(42)` and `Helpers.Identity<int&>(&value)`. Speculative parsing
leaves ordinary `<` comparisons unchanged. `static` is reserved. Static methods have
no `this`; instance/interface generic methods and source overloads are not supported.
See [generic functions](function-generics.md) for the runtime contract and sample.

Inference is semantic, not extra grammar: an ordinary call can infer method arguments
by matching argument types. Explicit `<type, ...>` overrides inference. Missing
evidence or conflicting inferred types requires explicit arguments. Reference intent
follows the [value/reference inference rules](function-generics.md#neo-projection).

## Ordinary classes and typed defaults

`class` and `default` are reserved. Class bodies declare typed mutable fields;
records retain positional declarations. A class without init receives a parameterless
constructor only when every field has an initializer. An omitted class base initializer
requests the base's parameterless init. Field initializers run after base completion,
before the body, and outside constructor parameter scope. See [classes and defaults](classes-and-defaults.md).

`default(T)` is an expression using the runtime's existing initobj contract. No
contextual bare default literal is added. It does not run constructors. Types without
valid defaults, including non-nullable managed references, are rejected. Nullable
signatures and null literals remain unimplemented.


Delegate declarations are nominal. Explicit construction accepts a source method group,
for example `Transform(counter.Add)` or `System.Func<int, void>(Print)`.
Generic target methods require explicit type arguments. A generic method group without
an immediate invocation is accepted in a delegate construction argument or where an
expected delegate type supplies context (parameters, typed bindings and returns).
Delegate-valued expressions support `callback(args)`, including fields and returned
callables; `value.Invoke(args)` also works through managed references. Custom source declarations are nongeneric in this slice; bundled
generic Func and generic IL declarations are supported. Lambdas require an expected delegate type; captures use shared managed storage. See [delegates](delegates.md).

Records and classes may implement bundled System interfaces as well as source-declared
interfaces, including `System.Comparable<Score>`. The linked runtime verifies the
method contracts. Source interface inheritance still requires source-declared bases.

`Result<T,E>.Ok(payload)` and `Result<T,E>.Error(payload)` use ordinary static-call
syntax. Explicit [library case constructors](neo-library-constructors.md), such as
`System.Result.Ok<int>(42)` and `Result<int,string>(ok)`, reuse call syntax. Source case
imports are supported; external imports, generic constructor inference and implicit
external case-to-carrier conversion remain separate work.

Non-generic source unions support the two forms above. Cases must be distinct
concrete declared source value types; inline cases generate nested record types.
A source-union case binding captures the complete variant, unlike the existing
bundled Option/Result payload projection. See [union declarations](neo-unions.md).

See [conditional union bindings](conditional-patterns.md) for `if let` and
`let … else`, including scope, failure-path rules and the order-workflow example.
