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
declaration      = import_decl | record_decl | function_decl ;
import_decl      = "import", "System", ".", "Console", ".", "*", terminator ;
record_decl      = "record", identifier, field_list, terminator ;
function_decl    = "func", identifier, field_list, "->", type, newlines,
                   "{", separators, { statement, separators }, "}" ;
field_list       = "(", newlines,
                   [ field, newlines, { ",", newlines, field, newlines } ], ")" ;
field            = identifier, ":", type ;
type             = (qualified_name, [ "<", type, { ",", type }, ">" ] | "(", ")"), [ "&" ] ;
qualified_name   = identifier, { ".", identifier } ;

block            = newlines, "{", separators, { statement, separators }, "}" ;
statement        = (binding | return_statement | expression_statement | "break" | "continue"), terminator
                 | if_statement | "while", expression, block | "loop", block
                 | "for", identifier, "in", expression, (".." | "..<"), expression, block ;
if_statement     = "if", expression, block, [ newlines, "else", (if_statement | block) ] ;
binding          = ("let" | "var"), identifier, [ ":", type ], "=", expression ;
return_statement = "return", [ expression ] ;
expression_statement = expression, [ "=", expression ] ;
```

A program must define one parameterless `Main`. Other declarations may appear before
or after it. There are no top-level executable statements in this slice. Fields and
parameters share name-before-type syntax. Trailing commas, overload declarations,
function expression bodies and standalone blocks are unsupported. Structured statements
introduce nested scopes; active names cannot be shadowed, but sibling scopes may reuse names.

Types resolve to `int`/`Int32`, `string`/`String`, `bool`/`Boolean`, `unit`/`Void`/`()`,
or a declared record or bundled System type. Qualified names and closed generic
arguments are supported; Option/Result abbreviate System.Option/System.Result. One
`&` suffix forms a managed reference. Raw pointer syntax and generic declarations
are unsupported. Runtime restrictions on ByRef generic arguments still apply.

## Expressions

```ebnf
expression       = logical_or, { "match", newlines, "{", separators,
                   [ match_arm, { arm_separator, match_arm }, [ arm_separator ] ], "}" } ;
match_arm        = pattern, "=>", newlines, (expression | block) ;
pattern          = "_" | identifier, [ "(", ("let", identifier | "_"), ")" ] ;
arm_separator    = ("," | newline), separators ;
logical_or       = logical_and, { "||", logical_and } ;
logical_and      = equality, { "&&", equality } ;
equality         = comparison, { ("==" | "!="), comparison } ;
comparison       = additive, { ("<" | ">" | "<=" | ">="), additive } ;
additive         = multiplicative, { ("+" | "-"), multiplicative } ;
multiplicative   = unary, { ("*" | "/"), unary } ;
unary            = ("&" | "*" | "-" | "!" | "new"), unary | postfix ;
postfix          = primary, { ".", identifier | arguments } ;
arguments        = "(", newlines,
                   [ expression, newlines,
                     { ",", newlines, expression, newlines } ], ")" ;
primary          = integer | string | "true" | "false" | identifier
                 | "(", newlines, expression, newlines, ")" ;
```

Member access and calls bind most tightly, then unary operations, multiplication and
division, addition and subtraction, ordering comparisons, equality, `&&`, then `||`.
Binary operators associate to the left. Conditions require Boolean; `&&` and `||`
short-circuit. Ordering uses Int32; equality supports Int32 and Boolean.
`&counter.Age` therefore addresses the field; `*reference + 1` adds to the dereferenced
value. All arithmetic in this subset uses Int32.

The grammar permits general postfix shapes, but semantic checks restrict calls to
free functions, positional record construction, explicit `int(byteOrInt)` conversion,
and public static/ordinary instance bundled System calls.
Library overloads are selected by exact argument types; out/byref-receiver contracts
are not exposed. Instance receivers are values (T& is read with ldobj). Generic method
calls and user-declared methods remain unsupported. `int(value)` supports Byte/Int32
only and lowers to checked Int32 conversion. These are static restrictions on the
existing call grammar; no new expression production is needed.
`new` requires a record-construction call, such as `new Counter(0)`. It does not
accept arbitrary factory calls or copy expressions in this slice. Assignment and
`&` require appropriate addressable locations. There is no assignment expression or implicit conversion.

## Lexical and layout rules

Identifiers use ASCII letters, digits and underscore and cannot start with a digit.
Reserved words are `func`, `record`, `let`, `var`, `return`, `new`, `true`, `false` and
`import`, `if`, `else`, `while`, `for`, `in`, `loop`, `break`, `continue`, and `match`. Type aliases and `Console`/`WriteLine` are additionally reserved declaration
names. Fields, parameters and bindings must be unique in their applicable scope.

Integer tokens contain decimal digits only. Positive literals must fit Int32; a
literal immediately prefixed by minus may represent -2147483648. No suffixes, digit
separators or floating literals are implemented. Strings use double quotes and
JSON-style escapes, including Unicode escapes; raw line breaks are rejected.

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
Expression arms have one exact result type. Statement arms may perform actions,
return, break or continue; their expression results are discarded. Payload names
are immutable, arm-local and cannot shadow active names. References follow the
ordinary lifetime rules; no address to copied arm payload storage can escape.

Coverage is limited to bundled System unions with the UnionAttribute marker,
constructor-declared cases, and typed public test/extraction/payload accessors.
The compiler validates that contract; arbitrary source records are not unions merely
because their names resemble Option or Result. Unsupported contracts are diagnosed.
Guards, nested destructuring patterns, user-declared unions and subtype patterns are
future features.
