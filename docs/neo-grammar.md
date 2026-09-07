# Neo grammar

This is the grammar of the implemented first Neo subset, not a grammar for Raven.
The standalone [neo.ebnf](neo.ebnf) is the grammar source. The [Neo guide](neo.md)
contains runnable examples and commands. Parser and semantic regressions live in
[neo.rs](../tests/neo.rs).

The EBNF uses commas for sequence, `|` for alternatives, brackets for optional forms,
braces for repetition, and `? … ?` for lexical/context conditions. Character ranges
use `..`. `terminator` may be empty only immediately before `}` or end of input.
There are no empty statements apart from separators.

## Declarations and statements

```ebnf
program       = separators, { declaration, separators }, end_of_input ;
declaration   = import_decl | record_decl | function_decl ;
import_decl   = "import", "System", ".", "Console", ".", "*", terminator ;
record_decl   = "record", identifier, field_list, terminator ;
function_decl = "func", identifier, field_list, "->", type, newlines,
                "{", separators, { statement, separators }, "}" ;
field_list    = "(", newlines,
                [ field, newlines, { ",", newlines, field, newlines } ], ")" ;
field         = identifier, ":", type ;
type          = (identifier | "(", ")"), [ "&" ] ;

statement = (binding | return_statement | expression_statement), terminator ;
binding = ("let" | "var"), identifier, [ ":", type ], "=", expression ;
return_statement = "return", [ expression ] ;
expression_statement = expression, [ "=", expression ] ;
```

A program must define one parameterless `Main`. Other declarations may appear before
or after it. There are no top-level executable statements in this slice. Fields and
parameters share name-before-type syntax. Trailing commas, overload declarations,
function expression bodies and nested statement blocks are unsupported.

Types resolve to `int`/`Int32`, `string`/`String`, `bool`/`Boolean`, `unit`/`Void`/`()`,
or a declared record. One `&` suffix forms a managed reference. Qualified type names,
generic types and raw pointer syntax are not part of this grammar.

## Expressions

```ebnf
expression     = additive ;
additive       = multiplicative, { ("+" | "-"), multiplicative } ;
multiplicative = unary, { ("*" | "/"), unary } ;
unary          = ("&" | "*" | "-" | "new"), unary | postfix ;
postfix        = primary, { ".", identifier | arguments } ;
arguments      = "(", newlines,
                 [ expression, newlines,
                   { ",", newlines, expression, newlines } ], ")" ;
primary        = integer | string | "true" | "false" | identifier
               | "(", newlines, expression, newlines, ")" ;
```

Member access and calls bind most tightly, then unary operations, multiplication and
division, then addition and subtraction. Binary arithmetic associates to the left.
`&counter.Age` therefore addresses the field; `*reference + 1` adds to the dereferenced
value. All arithmetic in this subset uses Int32.

The grammar permits general postfix shapes, but semantic checks restrict calls to
free functions, positional record construction and the supported Console API.
`new` requires a record-construction call, such as `new Counter(0)`. It does not
accept arbitrary factory calls or copy expressions in this slice. Assignment and
`&` require appropriate addressable locations. There is no assignment expression,
comparison operator, Boolean operator or implicit conversion.

## Lexical and layout rules

Identifiers use ASCII letters, digits and underscore and cannot start with a digit.
Reserved words are `func`, `record`, `let`, `var`, `return`, `new`, `true`, `false` and
`import`. Type aliases and `Console`/`WriteLine` are additionally reserved declaration
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
parentheses of a grouped expression, and before a function's opening brace. They
do not implicitly continue a binary expression or split the `name: type` pair.
Semicolons inside lists/grouped expressions are not layout whitespace.

The implementation bounds source input at 1 MiB, expression nesting at 128,
record/function declarations at 1024, and fields/parameters per declaration at 1024.
Backend metadata and execution limits still apply. Source syntax and bounds remain
preview contracts and can change as end-to-end scenarios require.
