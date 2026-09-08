//! Neo concept-language front end for exercising NeoCLR end to end.
//! This is a separate experimental language subset, not a Raven compiler.
mod library;

use crate::{Fault, Module};
use std::collections::HashMap;

#[derive(Clone, Debug)]
struct Token {
    text: String,
    line: usize,
    column: usize,
}
impl Token {
    fn error(&self, message: impl AsRef<str>) -> Fault {
        Fault::new(format!(
            "source {}:{}: {}",
            self.line,
            self.column,
            message.as_ref()
        ))
    }
}

fn lex(source: &str) -> Result<Vec<Token>, Fault> {
    if source.len() > 1_048_576 {
        return Err(Fault::new("source exceeds 1 MiB limit"));
    }
    let mut tokens = Vec::new();
    let (mut offset, mut line, mut column) = (0, 1, 1);
    while offset < source.len() {
        let rest = &source[offset..];
        let first = rest.chars().next().unwrap();
        let start = Token {
            text: String::new(),
            line,
            column,
        };
        if rest.starts_with("//") {
            let length = rest.find('\n').unwrap_or(rest.len());
            column += rest[..length].chars().count();
            offset += length;
            continue;
        }
        if first == '\n' {
            tokens.push(Token {
                text: "\n".into(),
                ..start
            });
            offset += 1;
            line += 1;
            column = 1;
            continue;
        }
        if first.is_whitespace() {
            offset += first.len_utf8();
            column += 1;
            continue;
        }
        let length = if first.is_ascii_alphabetic() || first == '_' {
            rest.bytes()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == b'_')
                .count()
        } else if first.is_ascii_digit() {
            rest.bytes().take_while(u8::is_ascii_digit).count()
        } else if first == '"' {
            let mut escaped = false;
            let mut end = None;
            for (index, ch) in rest.char_indices().skip(1) {
                if ch == '\n' || ch == '\r' {
                    return Err(start.error("string literal must end on the same line"));
                }
                if !escaped && ch == '"' {
                    end = Some(index + 1);
                    break;
                }
                escaped = !escaped && ch == '\\';
            }
            end.ok_or_else(|| start.error("unterminated string literal"))?
        } else if rest.starts_with("..<") {
            3
        } else if ["->", "=>", "..", "==", "!=", "<=", ">=", "&&", "||"]
            .iter()
            .any(|op| rest.starts_with(op))
        {
            2
        } else if "(){}[]:,.;&*+-=/!<>".contains(first) {
            first.len_utf8()
        } else {
            return Err(start.error(format!("unsupported character {first:?}")));
        };
        let text = &rest[..length];
        tokens.push(Token {
            text: text.into(),
            ..start
        });
        column += text.chars().count();
        offset += length;
    }
    tokens.push(Token {
        text: String::new(),
        line,
        column,
    });
    Ok(tokens)
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Ty {
    Int,
    String,
    Bool,
    Void,
    Record(String),
    Ref(Box<Ty>),
    Array(Box<Ty>),
}
impl Ty {
    fn from_metadata(ty: &crate::metadata::Type) -> Result<Self, Fault> {
        use crate::metadata::Type;
        Ok(match ty {
            Type::Int32 => Self::Int,
            Type::Boolean => Self::Bool,
            Type::String => Self::String,
            Type::Void => Self::Void,
            Type::Array(target) => Self::Array(Box::new(Self::from_metadata(target)?)),
            Type::ByRef(target) => Self::Ref(Box::new(Self::from_metadata(target)?)),
            Type::Constructed {
                definition,
                arguments,
            } => Self::Record(format!(
                "{}<{}>",
                definition,
                arguments
                    .iter()
                    .map(|a| Self::from_metadata(a).map(|t| t.il()))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(",")
            )),
            _ => Self::Record(
                ty.definition_name()
                    .ok_or_else(|| Fault::new("unsupported source type"))?
                    .into(),
            ),
        })
    }
    fn il(&self) -> String {
        match self {
            Self::Int => "Int32".into(),
            Self::String => "String".into(),
            Self::Bool => "Boolean".into(),
            Self::Void => "Void".into(),
            Self::Record(name) => name.clone(),
            Self::Ref(ty) => format!("{}&", ty.il()),
            Self::Array(ty) => format!("{}[]", ty.il()),
        }
    }
}
#[derive(Clone)]
struct Field {
    name: Token,
    ty: Ty,
    output: bool,
    readonly: bool,
}
struct Record {
    name: Token,
    fields: Vec<Field>,
    implements: Vec<Ty>,
    methods: Vec<Function>,
}
struct Interface {
    name: Token,
    methods: Vec<Function>,
}
struct Function {
    name: Token,
    receiver_readonly: bool,
    parameters: Vec<Field>,
    returns: Ty,
    body: Vec<Stmt>,
}
struct Source {
    interfaces: Vec<Interface>,
    records: Vec<Record>,
    functions: Vec<Function>,
    console_import: bool,
}
struct Expr {
    at: Token,
    kind: ExprKind,
    depth: usize,
}
enum ExprKind {
    Int(i32),
    String(String),
    Bool(bool),
    Name(String),
    Field(Box<Expr>, Token),
    Call(Box<Expr>, Vec<Expr>),
    Unary(String, Box<Expr>),
    Binary(String, Box<Expr>, Box<Expr>),
    Match(Box<Expr>, Vec<Arm>),
    TypeOf(Ty),
    Out(Box<Expr>),
    InterfaceCast(Box<Expr>, Ty),
    ArrayLiteral(Vec<Expr>),
    NewArray(Ty, Box<Expr>, Vec<Expr>),
    Index(Box<Expr>, Box<Expr>),
}
enum Pattern {
    Wildcard(Token),
    Case(Token, Option<Option<Token>>), // None: no payload; Some(None): discarded payload.
}
struct Arm {
    pattern: Pattern,
    body: ArmBody,
}
enum ArmBody {
    Expression(Expr),
    Block(Vec<Stmt>),
}
enum Stmt {
    Bind {
        name: Token,
        mutable: bool,
        annotation: Option<Ty>,
        extent: Option<i32>,
        value: Option<Expr>,
    },
    Assign(Expr, Expr),
    Return(Token, Option<Expr>),
    Expression(Expr),
    If(Expr, Vec<Stmt>, Vec<Stmt>),
    While(Expr, Vec<Stmt>),
    Loop(Token, Vec<Stmt>),
    For(Token, Expr, Expr, bool, Vec<Stmt>),
    Break(Token),
    Continue(Token),
}
struct Parser {
    tokens: Vec<Token>,
    position: usize,
    depth: usize,
}
impl Parser {
    fn current(&self) -> &Token {
        &self.tokens[self.position]
    }
    fn at(&self, text: &str) -> bool {
        self.current().text == text
    }
    fn take(&mut self) -> Token {
        let token = self.current().clone();
        if !token.text.is_empty() {
            self.position += 1;
        }
        token
    }
    fn eat(&mut self, text: &str) -> bool {
        if self.at(text) {
            self.take();
            true
        } else {
            false
        }
    }
    fn expect(&mut self, text: &str) -> Result<Token, Fault> {
        if self.at(text) {
            Ok(self.take())
        } else {
            Err(self.current().error(format!(
                "expected {text:?}, found {:?}",
                self.current().text
            )))
        }
    }
    fn newlines(&mut self) {
        while self.eat("\n") {}
    }
    fn lines(&mut self) {
        while self.eat("\n") || self.eat(";") {}
    }
    fn name(&mut self) -> Result<Token, Fault> {
        let token = self.take();
        if !crate::metadata::valid_slot_name(&token.text) {
            return Err(token.error("expected a name"));
        }
        if [
            "func",
            "record",
            "let",
            "var",
            "return",
            "new",
            "true",
            "false",
            "import",
            "if",
            "else",
            "while",
            "for",
            "in",
            "loop",
            "break",
            "continue",
            "match",
            "typeof",
            "interface",
            "readonly",
            "as",
            "this",
        ]
        .contains(&token.text.as_str())
        {
            return Err(token.error("keyword cannot be used as a name"));
        }
        Ok(token)
    }
    fn ty(&mut self) -> Result<Ty, Fault> {
        if self.depth >= 32 {
            return Err(self.current().error("type nesting limit exceeded"));
        }
        self.depth += 1;
        let result = self.ty_inner();
        self.depth -= 1;
        result
    }
    fn ty_inner(&mut self) -> Result<Ty, Fault> {
        let mut ty = if self.eat("(") {
            self.expect(")")?;
            Ty::Void
        } else {
            let mut name = self.name()?.text;
            while self.eat(".") {
                name.push('.');
                name.push_str(&self.name()?.text);
            }
            let name = match name.as_str() {
                "Option" => "System.Option",
                "Result" => "System.Result",
                "bool" => "Boolean",
                "unit" => "Void",
                "byte" => "Byte",
                _ => &name,
            }
            .to_owned();
            if self.eat("<") {
                let mut arguments = vec![self.ty()?];
                while self.eat(",") {
                    arguments.push(self.ty()?);
                }
                self.expect(">")?;
                Ty::Record(format!(
                    "{name}<{}>",
                    arguments.iter().map(Ty::il).collect::<Vec<_>>().join(",")
                ))
            } else {
                Ty::from_metadata(&crate::metadata::Type::from_name(&name))?
            }
        };
        let mut array_depth = 0;
        while self.at("[")
            && self
                .tokens
                .get(self.position + 1)
                .is_some_and(|t| t.text == "]")
        {
            self.take();
            self.take();
            array_depth += 1;
            if self.depth + array_depth > 32 {
                return Err(self.current().error("type nesting limit exceeded"));
            }
            ty = Ty::Array(Box::new(ty));
        }
        if self.eat("&") {
            ty = Ty::Ref(Box::new(ty));
        }
        Ok(ty)
    }
    fn fields(&mut self, parameters: bool) -> Result<Vec<Field>, Fault> {
        self.expect("(")?;
        self.newlines();
        let mut fields: Vec<Field> = Vec::new();
        if !self.at(")") {
            loop {
                let readonly = parameters && self.eat("readonly");
                let output = parameters && self.eat("out");
                let name = self.name()?;
                if fields.iter().any(|field| field.name.text == name.text) {
                    return Err(name.error("duplicate parameter or field"));
                }
                self.expect(":")?;
                let ty = self.ty()?;
                if fields.len() >= 1024 {
                    return Err(name.error("parameter/field limit exceeded"));
                }
                if output && !matches!(ty, Ty::Ref(_)) {
                    return Err(name.error("out parameter requires a managed reference type"));
                }
                if readonly && (output || !matches!(ty, Ty::Ref(_))) {
                    return Err(name.error("readonly parameter requires a managed input reference"));
                }
                fields.push(Field {
                    name,
                    ty,
                    output,
                    readonly,
                });
                self.newlines();
                if !self.eat(",") {
                    break;
                }
                self.newlines();
            }
        }
        self.expect(")")?;
        Ok(fields)
    }
    fn function(&mut self, abstract_member: bool) -> Result<Function, Fault> {
        self.expect("func")?;
        let name = self.name()?;
        let parameters = self.fields(true)?;
        self.expect("->")?;
        let returns = self.ty()?;
        let body = if abstract_member {
            self.end_statement()?;
            Vec::new()
        } else {
            self.block()?
        };
        Ok(Function {
            name,
            receiver_readonly: false,
            parameters,
            returns,
            body,
        })
    }
    fn methods(&mut self, abstract_members: bool) -> Result<Vec<Function>, Fault> {
        self.expect("{")?;
        self.lines();
        let mut methods: Vec<Function> = Vec::new();
        while !self.at("}") {
            if methods.len() >= 1024 {
                return Err(self.current().error("method limit exceeded"));
            }
            let receiver_readonly = self.eat("readonly");
            let mut method = self.function(abstract_members)?;
            method.receiver_readonly = receiver_readonly;
            if methods.iter().any(|m| m.name.text == method.name.text) {
                return Err(method
                    .name
                    .error("duplicate method; source overload declarations are not supported"));
            }
            methods.push(method);
            self.lines();
        }
        self.expect("}")?;
        Ok(methods)
    }
    fn source(&mut self) -> Result<Source, Fault> {
        let mut source = Source {
            interfaces: Vec::new(),
            records: Vec::new(),
            functions: Vec::new(),
            console_import: false,
        };
        self.lines();
        while !self.at("") {
            if self.eat("import") {
                for word in ["System", ".", "Console", ".", "*"] {
                    self.expect(word)?;
                }
                source.console_import = true;
                self.end_statement()?;
            } else if self.eat("interface") {
                let name = self.name()?;
                self.newlines();
                let methods = self.methods(true)?;
                source.interfaces.push(Interface { name, methods });
            } else if self.eat("record") {
                let name = self.name()?;
                let fields = self.fields(false)?;
                let mut implements = Vec::new();
                if self.eat(":") {
                    loop {
                        if implements.len() >= 1024 {
                            return Err(name.error("interface count limit exceeded"));
                        }
                        implements.push(self.ty()?);
                        if !self.eat(",") {
                            break;
                        }
                    }
                }
                let saved = self.position;
                self.newlines();
                let methods = if self.at("{") {
                    self.methods(false)?
                } else {
                    self.position = saved;
                    self.end_statement()?;
                    Vec::new()
                };
                source.records.push(Record {
                    name,
                    fields,
                    implements,
                    methods,
                });
            } else if self.at("func") {
                source.functions.push(self.function(false)?);
            } else {
                return Err(self
                    .current()
                    .error("expected interface, record, func, or import System.Console.*"));
            }
            if source.records.len() + source.functions.len() + source.interfaces.len() > 1024 {
                return Err(self.current().error("declaration limit exceeded"));
            }
            self.lines();
        }
        Ok(source)
    }
    fn end_statement(&mut self) -> Result<(), Fault> {
        if self.at("}") || self.at("") {
            return Ok(());
        }
        if !self.at("\n") && !self.at(";") {
            return Err(self.current().error("expected newline or semicolon"));
        }
        self.lines();
        Ok(())
    }
    fn block(&mut self) -> Result<Vec<Stmt>, Fault> {
        self.newlines();
        self.expect("{")?;
        self.lines();
        let mut body = Vec::new();
        while !self.at("}") && !self.at("") {
            body.push(self.statement()?);
            self.lines();
        }
        self.expect("}")?;
        Ok(body)
    }
    fn statement(&mut self) -> Result<Stmt, Fault> {
        if self.depth >= 32 {
            return Err(self.current().error("statement nesting limit exceeded"));
        }
        self.depth += 1;
        let result = self.statement_inner();
        self.depth -= 1;
        result
    }
    fn statement_inner(&mut self) -> Result<Stmt, Fault> {
        if self.eat("if") {
            let condition = self.expression(0)?;
            let yes = self.block()?;
            let saved = self.position;
            self.newlines();
            let no = if self.eat("else") {
                if self.at("if") {
                    vec![self.statement()?]
                } else {
                    self.block()?
                }
            } else {
                self.position = saved;
                Vec::new()
            };
            return Ok(Stmt::If(condition, yes, no));
        }
        if self.eat("while") {
            let condition = self.expression(0)?;
            return Ok(Stmt::While(condition, self.block()?));
        }
        if self.eat("loop") {
            return Ok(Stmt::Loop(
                self.tokens[self.position - 1].clone(),
                self.block()?,
            ));
        }
        if self.eat("for") {
            let name = self.name()?;
            self.expect("in")?;
            let start = self.expression(0)?;
            let inclusive = if self.eat("..") {
                true
            } else {
                self.expect("..<")?;
                false
            };
            let end = self.expression(0)?;
            return Ok(Stmt::For(name, start, end, inclusive, self.block()?));
        }
        if self.at("break") || self.at("continue") {
            let at = self.take();
            let is_break = at.text == "break";
            self.end_statement()?;
            return Ok(if is_break {
                Stmt::Break(at)
            } else {
                Stmt::Continue(at)
            });
        }
        let statement =
            if self.at("let") || self.at("var") {
                let mutable = self.take().text == "var";
                let name = self.name()?;
                let annotation = if self.eat(":") {
                    Some(self.ty()?)
                } else {
                    None
                };
                let extent = if annotation.is_some() && self.eat("[") {
                    let at = self.take();
                    let length = at.text.parse::<i32>().map_err(|_| {
                        at.error("array extent must be a nonnegative Int32 literal")
                    })?;
                    self.expect("]")?;
                    Some(length)
                } else {
                    None
                };
                let annotation = if extent.is_some() {
                    Some(Ty::Array(Box::new(annotation.unwrap())))
                } else {
                    annotation
                };
                let value = if self.eat("=") {
                    Some(self.expression(0)?)
                } else {
                    if !mutable || annotation.is_none() {
                        return Err(name
                            .error("uninitialized declaration requires var and an explicit type"));
                    }
                    None
                };
                if extent.is_some() && value.is_none() {
                    return Err(name.error("fixed-extent local requires an initializer"));
                }
                Stmt::Bind {
                    name,
                    mutable,
                    annotation,
                    extent,
                    value,
                }
            } else if self.at("return") {
                let at = self.take();
                let value = if ["\n", ";", "}", ""].contains(&self.current().text.as_str()) {
                    None
                } else {
                    Some(self.expression(0)?)
                };
                Stmt::Return(at, value)
            } else {
                let left = self.expression(0)?;
                if self.eat("=") {
                    Stmt::Assign(left, self.expression(0)?)
                } else {
                    Stmt::Expression(left)
                }
            };
        if !matches!(&statement, Stmt::Expression(Expr { kind: ExprKind::Match(_, arms), .. })
            if arms.iter().any(|a| matches!(a.body, ArmBody::Block(_))))
        {
            self.end_statement()?;
        }
        Ok(statement)
    }
    fn node(&self, at: Token, kind: ExprKind, depth: usize) -> Result<Expr, Fault> {
        if depth > 128 {
            return Err(at.error("expression nesting limit exceeded"));
        }
        Ok(Expr { at, kind, depth })
    }
    fn expression(&mut self, minimum: u8) -> Result<Expr, Fault> {
        if self.depth >= 32 {
            return Err(self.current().error("expression nesting limit exceeded"));
        }
        self.depth += 1;
        let result = self.expression_inner(minimum);
        self.depth -= 1;
        result
    }
    fn expression_inner(&mut self, minimum: u8) -> Result<Expr, Fault> {
        let at = self.take();
        let mut left = if at.text == "[" {
            self.newlines();
            let mut elements = vec![self.expression(0)?];
            self.newlines();
            while self.eat(",") {
                self.newlines();
                elements.push(self.expression(0)?);
                self.newlines();
            }
            self.expect("]")?;
            let depth = elements.iter().map(|e| e.depth).max().unwrap_or(0) + 1;
            self.node(at, ExprKind::ArrayLiteral(elements), depth)?
        } else if at.text == "new" {
            let saved = self.position;
            let ty = self.ty()?;
            if self.eat("[") {
                let length = self.expression(0)?;
                self.expect("]")?;
                let mut elements = Vec::new();
                if self.eat("{") {
                    self.newlines();
                    while !self.at("}") {
                        elements.push(self.expression(0)?);
                        self.newlines();
                        if !self.eat(",") {
                            break;
                        }
                        self.newlines();
                    }
                    self.expect("}")?;
                }
                let depth = elements
                    .iter()
                    .map(|e| e.depth)
                    .max()
                    .unwrap_or(0)
                    .max(length.depth)
                    + 1;
                self.node(
                    at,
                    ExprKind::NewArray(ty, Box::new(length), elements),
                    depth,
                )?
            } else {
                self.position = saved;
                let operand = self.expression(30)?;
                let depth = operand.depth + 1;
                self.node(at, ExprKind::Unary("new".into(), Box::new(operand)), depth)?
            }
        } else if at.text == "out" {
            let value = self.expression(30)?;
            let depth = value.depth + 1;
            self.node(at, ExprKind::Out(Box::new(value)), depth)?
        } else if at.text == "typeof" {
            self.expect("(")?;
            self.newlines();
            let ty = self.ty()?;
            self.newlines();
            self.expect(")")?;
            self.node(at, ExprKind::TypeOf(ty), 1)?
        } else if at.text == "-"
            && self
                .current()
                .text
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_digit)
        {
            let literal = self.take();
            let value: i64 = literal
                .text
                .parse()
                .map_err(|_| literal.error("integer literal is outside Int32 range"))?;
            let value = i32::try_from(-value)
                .map_err(|_| literal.error("integer literal is outside Int32 range"))?;
            self.node(at, ExprKind::Int(value), 1)?
        } else if ["&", "-", "!", "new"].contains(&at.text.as_str()) {
            let operand = self.expression(30)?;
            let depth = operand.depth + 1;
            self.node(
                at.clone(),
                ExprKind::Unary(at.text.clone(), Box::new(operand)),
                depth,
            )?
        } else if at.text == "(" {
            self.newlines();
            let value = self.expression(0)?;
            self.newlines();
            self.expect(")")?;
            value
        } else if at.text.starts_with('"') {
            let value: String =
                serde_json::from_str(&at.text).map_err(|_| at.error("invalid string escape"))?;
            self.node(at, ExprKind::String(value), 1)?
        } else if at.text.as_bytes().first().is_some_and(u8::is_ascii_digit) {
            let value = at
                .text
                .parse()
                .map_err(|_| at.error("integer literal is outside Int32 range"))?;
            self.node(at, ExprKind::Int(value), 1)?
        } else if at.text == "true" || at.text == "false" {
            let value = at.text == "true";
            self.node(at, ExprKind::Bool(value), 1)?
        } else if crate::metadata::valid_slot_name(&at.text) {
            // A closed generic owner followed by a member is a static call path.
            // Speculation must leave ordinary comparison expressions untouched.
            let saved = self.position;
            self.position -= 1;
            let candidate = self.ty();
            let owner = candidate
                .ok()
                .filter(|ty| ty.il().contains('<') && self.at("."));
            if let Some(owner) = owner {
                self.node(at.clone(), ExprKind::Name(owner.il()), 1)?
            } else {
                self.position = saved;
                self.node(at.clone(), ExprKind::Name(at.text.clone()), 1)?
            }
        } else {
            return Err(at.error("expected expression"));
        };
        loop {
            if self.at("as") && minimum <= 7 {
                self.take();
                let target = self.ty()?;
                let depth = left.depth + 1;
                left = self.node(
                    left.at.clone(),
                    ExprKind::InterfaceCast(Box::new(left), target),
                    depth,
                )?;
            } else if self.at("match") && minimum == 0 {
                self.take();
                self.newlines();
                self.expect("{")?;
                self.lines();
                let mut arms = Vec::new();
                let mut depth = left.depth + 1;
                while !self.at("}") && !self.at("") {
                    let pattern = if self.at("_") {
                        Pattern::Wildcard(self.take())
                    } else {
                        let name = self.name()?;
                        let payload = if self.eat("(") {
                            let binding = if self.eat("_") {
                                None
                            } else {
                                self.expect("let")?;
                                Some(self.name()?)
                            };
                            self.expect(")")?;
                            Some(binding)
                        } else {
                            None
                        };
                        Pattern::Case(name, payload)
                    };
                    self.expect("=>")?;
                    self.newlines();
                    let body = if self.at("{") {
                        ArmBody::Block(self.block()?)
                    } else {
                        let value = self.expression(0)?;
                        depth = depth.max(value.depth + 1);
                        ArmBody::Expression(value)
                    };
                    arms.push(Arm { pattern, body });
                    if !self.at("}") && !self.eat(",") {
                        self.expect("\n")?;
                    }
                    self.lines();
                }
                self.expect("}")?;
                left = self.node(
                    left.at.clone(),
                    ExprKind::Match(Box::new(left), arms),
                    depth,
                )?;
            } else if self.eat("[") {
                let index = self.expression(0)?;
                self.expect("]")?;
                let depth = left.depth.max(index.depth) + 1;
                left = self.node(
                    left.at.clone(),
                    ExprKind::Index(Box::new(left), Box::new(index)),
                    depth,
                )?;
            } else if self.at(".") {
                self.take();
                let field = self.name()?;
                let depth = left.depth + 1;
                left = self.node(
                    left.at.clone(),
                    ExprKind::Field(Box::new(left), field),
                    depth,
                )?;
            } else if self.at("(") {
                self.take();
                self.newlines();
                let mut arguments = Vec::new();
                if !self.at(")") {
                    loop {
                        arguments.push(self.expression(0)?);
                        self.newlines();
                        if !self.eat(",") {
                            break;
                        }
                        self.newlines();
                    }
                }
                self.expect(")")?;
                let depth = arguments
                    .iter()
                    .map(|arg| arg.depth)
                    .max()
                    .unwrap_or(0)
                    .max(left.depth)
                    + 1;
                left = self.node(
                    left.at.clone(),
                    ExprKind::Call(Box::new(left), arguments),
                    depth,
                )?;
            } else {
                let binding = match self.current().text.as_str() {
                    "||" => 1,
                    "&&" => 2,
                    "==" | "!=" => 3,
                    "<" | ">" | "<=" | ">=" => 4,
                    "+" | "-" => 10,
                    "*" | "/" => 20,
                    _ => break,
                };
                if binding < minimum {
                    break;
                }
                let operation = self.take();
                let right = self.expression(binding + 1)?;
                let depth = left.depth.max(right.depth) + 1;
                left = self.node(
                    operation.clone(),
                    ExprKind::Binary(operation.text, Box::new(left), Box::new(right)),
                    depth,
                )?;
            }
        }
        Ok(left)
    }
}

#[derive(Clone)]
struct Binding {
    ty: Ty,
    mutable: bool,
    load: String,
    address: String,
    scoped: bool,
}
struct Lowerer<'a> {
    document: &'a str,
    receiver: Option<&'a Token>,
    source: &'a Source,
    function: &'a Function,
    bindings: HashMap<String, Binding>,
    locals: Vec<String>,
    body: Vec<String>,
    labels: usize,
    scope: usize,
    loops: Vec<(String, String)>,
}
impl Lowerer<'_> {
    fn sequence(&mut self, at: &Token) {
        self.body.push(format!(
            ".sequence {}",
            serde_json::json!({
                "instruction": 0, "document": self.document, "line": at.line, "column": at.column
            })
        ));
    }

    fn require(&self, actual: &Ty, expected: &Ty, at: &Token) -> Result<(), Fault> {
        if actual != expected {
            return Err(at.error(format!("expected {}, got {}", expected.il(), actual.il())));
        }
        Ok(())
    }
    fn binding(&self, name: &str, at: &Token) -> Result<Binding, Fault> {
        self.bindings
            .get(name)
            .cloned()
            .ok_or_else(|| at.error(format!("unknown binding {name}")))
    }
    fn field(&self, owner: &Ty, name: &Token) -> Result<Ty, Fault> {
        let Ty::Record(record) = owner else {
            return Err(name.error("field access requires a record"));
        };
        self.source
            .records
            .iter()
            .find(|definition| &definition.name.text == record)
            .and_then(|definition| {
                definition
                    .fields
                    .iter()
                    .find(|field| field.name.text == name.text)
            })
            .map(|field| field.ty.clone())
            .ok_or_else(|| name.error(format!("unknown field {}.{}", record, name.text)))
    }
    fn read(&mut self, ty: Ty) -> Ty {
        if let Ty::Ref(target) = ty {
            self.body.push(format!("ldobj {}", target.il()));
            *target
        } else {
            ty
        }
    }
    fn value_expression(&mut self, expression: &Expr) -> Result<Ty, Fault> {
        if let ExprKind::Match(value, arms) = &expression.kind {
            return self
                .match_arms(value, arms, false, None, true)
                .map(|(ty, _)| ty);
        }
        let ty = self.expression(expression)?;
        Ok(self.read(ty))
    }
    fn expression_for(&mut self, expression: &Expr, expected: &Ty) -> Result<Ty, Fault> {
        let actual = if let ExprKind::Match(value, arms) = &expression.kind {
            self.match_arms(value, arms, false, Some(expected), false)?
                .0
        } else if matches!(expected, Ty::Ref(_)) {
            self.expression(expression)?
        } else {
            self.value_expression(expression)?
        };
        self.convert_reference(actual, expected, &expression.at)
    }
    fn convert_reference(&mut self, actual: Ty, expected: &Ty, at: &Token) -> Result<Ty, Fault> {
        if actual != *expected {
            if let (Ty::Ref(concrete), Ty::Ref(interface)) = (&actual, expected) {
                if library::implements(concrete, interface)?
                    || self
                        .source
                        .interfaces
                        .iter()
                        .any(|i| i.name.text == interface.il())
                        && self.source.records.iter().any(|r| {
                            r.name.text == concrete.il() && r.implements.contains(interface)
                        })
                {
                    self.body
                        .push(format!("interface.borrow {}", interface.il()));
                    return Ok(expected.clone());
                }
            }
        }
        self.require(&actual, expected, at)?;
        Ok(actual)
    }
    fn parameter_argument(&mut self, argument: &Expr, parameter: &Field) -> Result<(), Fault> {
        if parameter.readonly {
            if let ExprKind::Unary(operation, value) = &argument.kind {
                if operation == "&" {
                    let actual = Ty::Ref(Box::new(self.place_with_access(value, true, true)?));
                    return self
                        .convert_reference(actual, &parameter.ty, &argument.at)
                        .map(|_| ());
                }
            }
        }
        if parameter.output {
            let ExprKind::Out(value) = &argument.kind else {
                return Err(argument
                    .at
                    .error("output parameter requires an out argument"));
            };
            let actual = Ty::Ref(Box::new(self.place(value, true)?));
            self.require(&actual, &parameter.ty, &argument.at)
        } else {
            self.expression_for(argument, &parameter.ty).map(|_| ())
        }
    }
    fn output_arguments(
        &self,
        arguments: &[Expr],
        function: &crate::metadata::Function,
    ) -> Result<(), Fault> {
        for (index, argument) in arguments.iter().enumerate() {
            let output =
                function.out_parameters.contains(&index) || function.out_when_true.contains(&index);
            if matches!(argument.kind, ExprKind::Out(_)) != output {
                return Err(argument
                    .at
                    .error("out argument must match an output parameter"));
            }
        }
        Ok(())
    }
    fn library_argument(&mut self, expression: &Expr) -> Result<Ty, Fault> {
        if let ExprKind::Out(value) = &expression.kind {
            return self.place(value, true).map(|ty| Ty::Ref(Box::new(ty)));
        }
        if matches!(&expression.kind, ExprKind::Unary(op, _) if op == "&") {
            self.expression(expression)
        } else {
            self.value_expression(expression)
        }
    }
    fn library_receiver(
        &mut self,
        expression: &Expr,
        ty: &Ty,
        start: usize,
        byref: bool,
        readonly: bool,
    ) -> Result<(), Fault> {
        if byref {
            if !matches!(ty, Ty::Ref(_)) {
                self.body.truncate(start);
                self.place_with_access(expression, true, readonly)?;
            }
        } else if let Ty::Ref(target) = ty {
            self.body.push(format!("ldobj {}", target.il()));
        }
        Ok(())
    }
    fn check_array_length(&mut self, count: usize, array: bool) {
        let valid = self.label();
        self.body.push("dup".into());
        if array {
            self.body.extend(["ldlen".into(), "conv.i4".into()]);
        }
        self.body.extend([
            format!("ldc.i4 {count}"),
            "ceq".into(),
            format!("brtrue {valid}"),
            "fault \"array length does not match initializer or extent\"".into(),
            format!("{valid}:"),
        ]);
    }
    fn indexer(
        &mut self,
        owner: &Expr,
        index: &Expr,
        value: Option<&Expr>,
    ) -> Result<Option<Ty>, Fault> {
        let saved = self.body.len();
        let ty = self.expression(owner)?;
        let target = if let Ty::Ref(target) = &ty {
            target.as_ref()
        } else {
            &ty
        };
        if matches!(target, Ty::Array(_)) {
            self.body.truncate(saved);
            return Ok(None);
        }
        let (signature, function) =
            library::indexer(target, value.is_some()).map_err(|e| owner.at.error(e.message))?;
        let interface = library::is_interface(target)?;
        self.library_receiver(
            owner,
            &ty,
            saved,
            function.receiver_byref || interface,
            function.receiver_readonly,
        )?;
        self.expression_for(index, &Ty::from_metadata(&function.parameters[0])?)?;
        if let Some(value) = value {
            self.expression_for(value, &Ty::from_metadata(&function.parameters[1])?)?;
        }
        self.body.push(format!(
            "{} {signature}",
            if interface { "callvirt" } else { "call" }
        ));
        Ok(Some(Ty::from_metadata(&function.returns)?))
    }
    fn array_owner(&mut self, expression: &Expr) -> Result<Ty, Fault> {
        // Reading a local array element need not copy its whole value.
        if let ExprKind::Name(name) = &expression.kind {
            let binding = self.binding(name, &expression.at)?;
            if let Ty::Array(element) = binding.ty {
                self.body.push(binding.address);
                return Ok(Ty::Ref(Box::new(Ty::Array(element))));
            }
        }
        self.expression(expression)
    }
    fn array_element(ty: Ty, at: &Token) -> Result<Ty, Fault> {
        match ty {
            Ty::Array(element) => Ok(*element),
            Ty::Ref(target) => Self::array_element(*target, at),
            _ => Err(at.error("indexing requires an array")),
        }
    }
    fn expression(&mut self, expression: &Expr) -> Result<Ty, Fault> {
        self.sequence(&expression.at);
        match &expression.kind {
            ExprKind::InterfaceCast(value, target) => {
                let Ty::Ref(interface) = target else {
                    return Err(expression.at.error(
                        "interface projection requires an interface reference target (Contract&)",
                    ));
                };
                if !self
                    .source
                    .interfaces
                    .iter()
                    .any(|i| i.name.text == interface.il())
                {
                    return Err(expression
                        .at
                        .error("projection target must be a declared source interface"));
                }
                let actual = self.expression(value)?;
                if !matches!(actual, Ty::Ref(_)) {
                    return Err(value
                        .at
                        .error("interface projection requires a managed reference; use &value"));
                }
                self.convert_reference(actual, target, &value.at)
            }
            ExprKind::NewArray(element, length, elements) => {
                if !elements.is_empty()
                    && matches!(length.kind, ExprKind::Int(count) if count as usize != elements.len())
                {
                    return Err(length
                        .at
                        .error("array initializer count does not match length"));
                }
                self.expression_for(length, &Ty::Int)?;
                let ty = Ty::Ref(Box::new(Ty::Array(Box::new(element.clone()))));
                if elements.is_empty() {
                    self.body.push(format!("newarr {}", element.il()));
                } else {
                    self.check_array_length(elements.len(), false);
                    self.body.push(format!("array.alloc {}", element.il()));
                    let local = self.temp(&ty);
                    self.body
                        .extend([format!("local.reset {local}"), format!("stloc {local}")]);
                    for (index, value) in elements.iter().enumerate() {
                        self.body
                            .extend([format!("ldloc {local}"), format!("ldc.i4 {index}")]);
                        self.expression_for(value, element)?;
                        self.body.push(format!("stelem {}", element.il()));
                    }
                    self.body.push(format!("ldloc {local}"));
                }
                Ok(ty)
            }
            ExprKind::ArrayLiteral(elements) => {
                self.body.push(format!("ldc.i4 {}", elements.len()));
                let element = self.value_expression(&elements[0])?;
                self.body.push(format!("array.create {}", element.il()));
                let ty = Ty::Array(Box::new(element.clone()));
                let local = self.temp(&ty);
                self.body.push(format!("local.reset {local}"));
                self.body.push(format!("stloc {local}"));
                for (index, value) in elements.iter().enumerate().skip(1) {
                    self.body
                        .extend([format!("ldloca {local}"), format!("ldc.i4 {index}")]);
                    self.expression_for(value, &element)?;
                    self.body.push(format!("stelem {}", element.il()));
                }
                self.body.push(format!("ldloc {local}"));
                Ok(ty)
            }
            ExprKind::Index(owner, index) => {
                if let Some(ty) = self.indexer(owner, index, None)? {
                    return Ok(ty);
                }
                let owner_ty = self.array_owner(owner)?;
                let element = Self::array_element(owner_ty, &owner.at)?;
                self.expression_for(index, &Ty::Int)?;
                self.body.push(format!("ldelem {}", element.il()));
                Ok(element)
            }
            ExprKind::TypeOf(ty) => {
                self.body.push(format!("ldtoken {}", ty.il()));
                self.body
                    .push("call System.Type::GetTypeFromHandle(System.RuntimeTypeHandle)".into());
                Ok(Ty::Record("System.Type".into()))
            }
            ExprKind::Int(value) => {
                self.body.push(format!("ldc.i4 {value}"));
                Ok(Ty::Int)
            }
            ExprKind::String(value) => {
                self.body
                    .push(format!("ldstr {}", serde_json::to_string(value).unwrap()));
                Ok(Ty::String)
            }
            ExprKind::Bool(value) => {
                self.body.push(format!("ldc.bool {value}"));
                Ok(Ty::Bool)
            }
            ExprKind::Name(name) => {
                let binding = self.binding(name, &expression.at)?;
                self.body.push(binding.load);
                Ok(binding.ty)
            }
            ExprKind::Field(value, field) => {
                let saved = self.body.len();
                let mut owner = self.array_owner(value)?;
                if field.text == "Length"
                    && matches!(&owner, Ty::Array(_) | Ty::Ref(_))
                    && Self::array_element(owner.clone(), field).is_ok()
                {
                    self.body.extend(["ldlen".into(), "conv.ovf.i4".into()]);
                    return Ok(Ty::Int);
                }
                let target = if let Ty::Ref(target) = &owner {
                    target.as_ref()
                } else {
                    &owner
                };
                if let Some((ty, getter, byref, readonly)) =
                    library::property(target, &field.text).map_err(|e| field.error(e.message))?
                {
                    let interface = library::is_interface(target)?;
                    self.library_receiver(value, &owner, saved, byref || interface, readonly)?;
                    let opcode = if interface { "callvirt" } else { "call" };
                    self.body.push(format!("{opcode} {getter}"));
                    return Ok(ty);
                }
                if let Ty::Ref(target) = owner {
                    self.body.push(format!("ldobj {}", target.il()));
                    owner = *target;
                }
                let ty = self.field(&owner, field)?;
                self.body
                    .push(format!("ldfld {}::{}", owner.il(), field.text));
                Ok(ty)
            }
            ExprKind::Unary(operation, value) if operation == "&" => {
                Ok(Ty::Ref(Box::new(self.place(value, true)?)))
            }
            ExprKind::Unary(operation, value) if operation == "new" => {
                let ExprKind::Call(callee, _) = &value.kind else {
                    return Err(expression.at.error("new requires record construction"));
                };
                let ExprKind::Name(name) = &callee.kind else {
                    return Err(expression.at.error("new requires record construction"));
                };
                if !self
                    .source
                    .records
                    .iter()
                    .any(|record| &record.name.text == name)
                    && name != "array"
                {
                    return Err(expression.at.error("new requires record construction"));
                }
                let ty = self.expression(value)?;
                self.body.push("heap.new".into());
                Ok(Ty::Ref(Box::new(ty)))
            }
            ExprKind::Unary(operation, value) => {
                let ty = self.value_expression(value)?;
                if operation == "!" {
                    self.require(&ty, &Ty::Bool, &expression.at)?;
                    self.body.extend(["ldc.bool false".into(), "ceq".into()]);
                    Ok(Ty::Bool)
                } else {
                    self.require(&ty, &Ty::Int, &expression.at)?;
                    self.body.push("neg".into());
                    Ok(Ty::Int)
                }
            }
            ExprKind::Binary(operation, left, right) => {
                let ty = self.value_expression(left)?;
                if operation == "&&" || operation == "||" {
                    self.require(&ty, &Ty::Bool, &left.at)?;
                    let skip = self.label();
                    self.body.push("dup".into());
                    self.body.push(format!(
                        "{} {skip}",
                        if operation == "&&" {
                            "brfalse"
                        } else {
                            "brtrue"
                        }
                    ));
                    self.body.push("pop".into());
                    let rhs = self.value_expression(right)?;
                    self.require(&rhs, &Ty::Bool, &right.at)?;
                    self.body.push(format!("{skip}:"));
                    return Ok(Ty::Bool);
                }
                let equality = operation == "==" || operation == "!=";
                if !equality || !matches!(ty, Ty::Int | Ty::Bool) {
                    self.require(&ty, &Ty::Int, &left.at)?;
                }
                let rhs = self.value_expression(right)?;
                self.require(&rhs, &ty, &right.at)?;
                let op = match operation.as_str() {
                    "+" => "add",
                    "-" => "sub",
                    "*" => "mul",
                    "/" => "div",
                    "==" | "!=" => "ceq",
                    "<" | ">=" => "clt",
                    _ => "cgt",
                };
                self.body.push(op.into());
                if ["!=", "<=", ">="].contains(&operation.as_str()) {
                    self.body.extend(["ldc.bool false".into(), "ceq".into()]);
                }
                Ok(if ["add", "sub", "mul", "div"].contains(&op) {
                    Ty::Int
                } else {
                    Ty::Bool
                })
            }
            ExprKind::Out(_) => Err(expression
                .at
                .error("out is only valid on an output argument")),
            ExprKind::Call(callee, arguments) => self.call(callee, arguments),
            ExprKind::Match(value, arms) => self
                .match_arms(value, arms, false, None, false)
                .map(|(ty, _)| ty),
        }
    }
    fn call(&mut self, callee: &Expr, arguments: &[Expr]) -> Result<Ty, Fault> {
        let path = Self::qualified_name(callee);
        let binding = path.as_ref().and_then(|p| p.split('.').next());
        let bound_receiver = binding.is_some_and(|name| self.bindings.contains_key(name));
        if let ExprKind::Field(owner, member) = &callee.kind {
            if bound_receiver || path.is_none() {
                let saved = self.body.len();
                let ty = self.expression(owner)?;
                let target = if let Ty::Ref(t) = &ty {
                    t.as_ref()
                } else {
                    &ty
                };
                let contract = self
                    .source
                    .interfaces
                    .iter()
                    .find(|i| i.name.text == target.il());
                let record = self
                    .source
                    .records
                    .iter()
                    .find(|r| r.name.text == target.il());
                let declared = contract
                    .map(|i| &i.methods)
                    .or_else(|| record.map(|r| &r.methods))
                    .is_some_and(|methods| methods.iter().any(|m| m.name.text == member.text));
                if member.text == "GetType"
                    && arguments.is_empty()
                    && matches!(ty, Ty::Ref(_))
                    && !declared
                    && library::parameters(target, "GetType", 0)?.is_none()
                {
                    self.body.push("ref.type".into());
                    self.body.push(
                        "call System.Type::GetTypeFromHandle(System.RuntimeTypeHandle)".into(),
                    );
                    return Ok(Ty::Record("System.Type".into()));
                }
                if contract.is_some() || record.is_some() {
                    let method = contract
                        .map(|i| &i.methods)
                        .or_else(|| record.map(|r| &r.methods))
                        .unwrap()
                        .iter()
                        .find(|m| m.name.text == member.text)
                        .ok_or_else(|| member.error("unknown source instance method"))?;
                    let receiver_readonly = method.receiver_readonly;
                    let parameters = method.parameters.clone();
                    let returns = method.returns.clone();
                    let signature = format!(
                        "instance {}::{}({})",
                        target.il(),
                        member.text,
                        parameters
                            .iter()
                            .map(|p| p.ty.il())
                            .collect::<Vec<_>>()
                            .join(",")
                    );
                    if !matches!(ty, Ty::Ref(_)) {
                        self.body.truncate(saved);
                        self.place_with_access(owner, true, receiver_readonly)?;
                    }
                    if parameters.len() != arguments.len() {
                        return Err(member.error("argument count mismatch"));
                    }
                    for (argument, parameter) in arguments.iter().zip(&parameters) {
                        self.parameter_argument(argument, parameter)?;
                    }
                    self.body.push(format!(
                        "{} {signature}",
                        if contract.is_some() {
                            "callvirt"
                        } else {
                            "call"
                        }
                    ));
                    return Ok(returns);
                }
                let argument_start = self.body.len();
                let types = if let Some(parameters) =
                    library::parameters(target, &member.text, arguments.len())?
                {
                    for (argument, parameter) in arguments.iter().zip(&parameters) {
                        if matches!(argument.kind, ExprKind::Out(_)) {
                            let actual = self.library_argument(argument)?;
                            self.require(&actual, parameter, &argument.at)?;
                        } else {
                            self.expression_for(argument, parameter)?;
                        }
                    }
                    parameters
                } else {
                    arguments
                        .iter()
                        .map(|a| self.library_argument(a))
                        .collect::<Result<Vec<_>, _>>()?
                };
                let signature = format!(
                    "instance {}::{}({})",
                    target.il(),
                    member.text,
                    types.iter().map(Ty::il).collect::<Vec<_>>().join(",")
                );
                let function =
                    library::resolve(&signature).map_err(|e| callee.at.error(e.message))?;
                self.output_arguments(arguments, &function)?;
                let argument_body = self.body.split_off(argument_start);
                let interface = library::is_interface(target)?;
                // Interface calls consume a view even when dispatch subsequently
                // copies the concrete receiver for a value-receiver implementation.
                self.library_receiver(
                    owner,
                    &ty,
                    saved,
                    function.receiver_byref || interface,
                    function.receiver_readonly,
                )?;
                self.body.extend(argument_body);
                let opcode = if interface { "callvirt" } else { "call" };
                self.body.push(format!("{opcode} {signature}"));
                return Ty::from_metadata(&function.returns);
            }
        }
        if bound_receiver {
            return Err(callee
                .at
                .error("calling through a binding is not supported"));
        }
        if matches!(&callee.kind, ExprKind::Name(name) if name == "ReferenceEquals")
            && !self
                .source
                .functions
                .iter()
                .any(|f| f.name.text == "ReferenceEquals")
            && !self
                .source
                .records
                .iter()
                .any(|r| r.name.text == "ReferenceEquals")
        {
            if arguments.len() != 2 {
                return Err(callee
                    .at
                    .error("ReferenceEquals requires two managed references"));
            }
            for argument in arguments {
                if !matches!(self.expression(argument)?, Ty::Ref(_)) {
                    return Err(argument
                        .at
                        .error("ReferenceEquals requires explicit managed references"));
                }
            }
            self.body.push("ref.eq".into());
            return Ok(Ty::Bool);
        }
        if matches!(&callee.kind, ExprKind::Name(name) if name == "array") {
            if arguments.len() != 2 {
                return Err(callee
                    .at
                    .error("array(length, initialValue) requires two arguments"));
            }
            self.expression_for(&arguments[0], &Ty::Int)?;
            let element = self.value_expression(&arguments[1])?;
            self.body.push(format!("array.create {}", element.il()));
            return Ok(Ty::Array(Box::new(element)));
        }
        if matches!(&callee.kind, ExprKind::Name(name) if name == "int") {
            if arguments.len() != 1 {
                return Err(callee.at.error("int conversion requires one argument"));
            }
            let ty = self.value_expression(&arguments[0])?;
            if ty != Ty::Int && ty != Ty::Record("System.Byte".into()) {
                return Err(callee
                    .at
                    .error("int conversion supports byte and int in this subset"));
            }
            self.body.push("conv.ovf.i4".into());
            return Ok(Ty::Int);
        }
        if matches!(&callee.kind, ExprKind::Field(owner, name) if matches!(&owner.kind, ExprKind::Name(owner) if owner == "Console") && name.text == "WriteLine")
            || matches!(&callee.kind, ExprKind::Name(name) if name == "WriteLine" && self.source.console_import)
        {
            if arguments.len() != 1 {
                return Err(callee.at.error("WriteLine requires one argument"));
            }
            let ty = self.value_expression(&arguments[0])?;
            if !matches!(ty, Ty::Int | Ty::String) {
                return Err(callee
                    .at
                    .error("WriteLine supports int and string in this subset"));
            }
            self.body
                .push(format!("call System.Console::WriteLine({})", ty.il()));
            return Ok(Ty::Void);
        }
        if let Some(path) = Self::qualified_name(callee) {
            if path.contains('.') {
                let (owner, member) = path.rsplit_once('.').unwrap();
                let owner = if owner.starts_with("System.") {
                    owner.to_owned()
                } else {
                    format!("System.{owner}")
                };
                let types = arguments
                    .iter()
                    .map(|a| self.library_argument(a))
                    .collect::<Result<Vec<_>, _>>()?;
                let signature = format!(
                    "{owner}::{member}({})",
                    types.iter().map(Ty::il).collect::<Vec<_>>().join(",")
                );
                let function =
                    library::resolve(&signature).map_err(|e| callee.at.error(e.message))?;
                self.output_arguments(arguments, &function)?;
                self.body.push(format!("call {signature}"));
                return Ty::from_metadata(&function.returns);
            }
        }
        let ExprKind::Name(name) = &callee.kind else {
            return Err(callee
                .at
                .error("only free functions and record constructors are supported"));
        };
        let (parameters, returns, opcode) = if let Some(record) = self
            .source
            .records
            .iter()
            .find(|record| &record.name.text == name)
        {
            (
                record.fields.clone(),
                Ty::Record(name.clone()),
                format!("newobj {name}"),
            )
        } else if let Some(function) = self
            .source
            .functions
            .iter()
            .find(|function| &function.name.text == name)
        {
            (
                function.parameters.clone(),
                function.returns.clone(),
                format!(
                    "call {name}({})",
                    function
                        .parameters
                        .iter()
                        .map(|field| field.ty.il())
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            )
        } else {
            return Err(callee
                .at
                .error(format!("unknown function or record {name}")));
        };
        if parameters.len() != arguments.len() {
            return Err(callee.at.error("argument count mismatch"));
        }
        for (argument, parameter) in arguments.iter().zip(&parameters) {
            self.parameter_argument(argument, parameter)?;
        }
        self.body.push(opcode);
        Ok(returns)
    }
    // Emit a managed address of an assignable source location.
    fn place(&mut self, expression: &Expr, borrowing: bool) -> Result<Ty, Fault> {
        self.place_with_access(expression, borrowing, false)
    }
    fn place_with_access(
        &mut self,
        expression: &Expr,
        borrowing: bool,
        readonly: bool,
    ) -> Result<Ty, Fault> {
        match &expression.kind {
            ExprKind::Index(owner, index) => {
                if let Some(ty) = self.indexer(owner, index, None)? {
                    return match ty {
                        Ty::Ref(target) => Ok(*target),
                        _ => Err(expression
                            .at
                            .error("value-returning indexer is not an addressable location")),
                    };
                }
                let owner_ty = self.place_with_access(owner, borrowing, readonly)?;
                let element = Self::array_element(owner_ty, &owner.at)?;
                self.expression_for(index, &Ty::Int)?;
                if let Ty::Ref(target) = &element {
                    self.body.push(format!("ldelem {}", element.il()));
                    return Ok(*target.clone());
                }
                self.body.push(format!("ldelema {}", element.il()));
                Ok(element)
            }

            ExprKind::Name(name) => {
                let binding = self.binding(name, &expression.at)?;
                if let Ty::Ref(target) = binding.ty {
                    self.body.push(binding.load);
                    return Ok(*target);
                }
                if !binding.mutable && !readonly {
                    return Err(expression.at.error(
                        "cannot assign or take a writable address of an immutable binding",
                    ));
                }
                if borrowing && binding.scoped {
                    return Err(expression.at.error("taking addresses of block-local values is not supported; use an outer local or managed heap storage"));
                }
                self.body.push(binding.address);
                Ok(binding.ty)
            }
            ExprKind::Field(owner, field) => {
                let saved = self.body.len();
                let ty = self.expression(owner)?;
                let (target, referenced_owner) = if let Ty::Ref(target) = ty {
                    (*target, true)
                } else {
                    (ty, false)
                };
                let field_type = self.field(&target, field)?;
                if let Ty::Ref(referent) = field_type {
                    if referenced_owner {
                        self.body.push(format!("ldobj {}", target.il()));
                    }
                    self.body
                        .push(format!("ldfld {}::{}", target.il(), field.text));
                    return Ok(*referent);
                }
                if !referenced_owner {
                    self.body.truncate(saved);
                    self.place_with_access(owner, borrowing, readonly)?;
                }
                self.body
                    .push(format!("ldflda {}::{}", target.il(), field.text));
                Ok(field_type)
            }
            _ => {
                let ty = self.expression(expression)?;
                if let Ty::Ref(target) = ty {
                    Ok(*target)
                } else {
                    Err(expression.at.error("expected an assignable location"))
                }
            }
        }
    }
    fn qualified_name(expression: &Expr) -> Option<String> {
        match &expression.kind {
            ExprKind::Name(name) => Some(name.clone()),
            ExprKind::Field(owner, field) => {
                Some(format!("{}.{}", Self::qualified_name(owner)?, field.text))
            }
            _ => None,
        }
    }
    fn match_arms(
        &mut self,
        value: &Expr,
        arms: &[Arm],
        statement: bool,
        expected: Option<&Ty>,
        values: bool,
    ) -> Result<(Ty, bool), Fault> {
        let mut ty = self.expression(value)?;
        if let Ty::Ref(target) = ty {
            self.body.push(format!("ldobj {}", target.il()));
            ty = *target;
        }
        let cases = library::cases(&ty).map_err(|e| value.at.error(e.message))?;
        let scrutinee = self.temp(&ty);
        self.body.push(format!("local.reset {scrutinee}"));
        self.body.push(format!("stloc {scrutinee}"));
        let mut seen = std::collections::HashSet::new();
        let mut wildcard = false;
        let end = self.label();
        let mut result_type = None;
        let mut result_local = None;
        let mut all_exit = true;
        for arm in arms {
            let at = match &arm.pattern {
                Pattern::Wildcard(at) | Pattern::Case(at, _) => at,
            };
            if wildcard || seen.len() == cases.len() {
                return Err(at.error("unreachable match arm"));
            }
            let next = self.label();
            let saved = self.bindings.clone();
            self.scope += 1;
            match &arm.pattern {
                Pattern::Wildcard(_) => {
                    wildcard = true;
                }
                Pattern::Case(name, binding) => {
                    let case = cases
                        .iter()
                        .find(|c| c.name == name.text)
                        .ok_or_else(|| name.error("unknown union case"))?;
                    if !seen.insert(name.text.clone()) {
                        return Err(name.error("duplicate match case"));
                    }
                    if binding.is_some() != case.payload.is_some() {
                        return Err(name.error("case payload pattern does not match its shape"));
                    }
                    self.body.extend([
                        format!("ldloc {scrutinee}"),
                        format!("call {}", case.test),
                        format!("brfalse {next}"),
                    ]);
                    if let Some(Some(name)) = binding {
                        if self.bindings.contains_key(&name.text) {
                            return Err(name.error("duplicate binding"));
                        }
                        let (payload, accessor) = case.payload.as_ref().unwrap();
                        self.body.extend([
                            format!("ldloc {scrutinee}"),
                            format!("call {}", case.extract),
                            format!("call {accessor}"),
                        ]);
                        let index = self.temp(payload);
                        self.body.push(format!("local.reset {index}"));
                        self.body.push(format!("stloc {index}"));
                        self.bindings.insert(
                            name.text.clone(),
                            Binding {
                                ty: payload.clone(),
                                mutable: false,
                                load: format!("ldloc {index}"),
                                address: format!("ldloca {index}"),
                                scoped: true,
                            },
                        );
                    }
                }
            }
            let exits = match &arm.body {
                ArmBody::Expression(expression) => {
                    let actual = if let Some(expected) = expected {
                        self.expression_for(expression, expected)?
                    } else if values {
                        self.value_expression(expression)?
                    } else {
                        self.expression(expression)?
                    };
                    if statement {
                        self.body.push("pop".into());
                    } else {
                        if let Some(expected) = &result_type {
                            self.require(&actual, expected, &expression.at)?;
                        } else {
                            result_type = Some(actual.clone());
                            result_local = Some(self.temp(&actual));
                        }
                        self.body
                            .push(format!("local.reset {}", result_local.unwrap()));
                        self.body.push(format!("stloc {}", result_local.unwrap()));
                    }
                    false
                }
                ArmBody::Block(body) => {
                    if !statement {
                        return Err(at.error("match expressions require expression arms"));
                    }
                    self.statements(body)?
                }
            };
            self.scope -= 1;
            self.bindings = saved;
            all_exit &= exits;
            if !exits {
                self.body.push(format!("br {end}"));
            }
            self.body.push(format!("{next}:"));
        }
        if !wildcard && seen.len() != cases.len() {
            return Err(value.at.error("non-exhaustive match"));
        }
        // Even exhaustive source matches must fault on a malformed carrier.
        self.body
            .push("fault \"invalid union case in match\"".into());
        if !all_exit {
            self.body.push(format!("{end}:"));
            if let Some(index) = result_local {
                self.body.push(format!("ldloc {index}"));
            }
        }
        Ok((result_type.unwrap_or(Ty::Void), all_exit))
    }
    fn label(&mut self) -> String {
        let label = format!("NeoLabel{}", self.labels);
        self.labels += 1;
        label
    }
    fn temp(&mut self, ty: &Ty) -> usize {
        let index = self.locals.len();
        self.locals.push(format!(".local {}", ty.il()));
        index
    }
    fn condition(&mut self, expression: &Expr) -> Result<(), Fault> {
        let ty = self.value_expression(expression)?;
        self.require(&ty, &Ty::Bool, &expression.at)
    }
    fn block(&mut self, statements: &[Stmt]) -> Result<bool, Fault> {
        let saved = self.bindings.clone();
        self.scope += 1;
        let result = self.statements(statements);
        self.scope -= 1;
        self.bindings = saved;
        result
    }
    fn statements(&mut self, statements: &[Stmt]) -> Result<bool, Fault> {
        let mut returned = false;
        for statement in statements {
            if returned {
                return Err(self
                    .function
                    .name
                    .error("statements after return, break or continue are not supported"));
            }
            let at = match statement {
                Stmt::Bind { name, .. } | Stmt::For(name, ..) => name,
                Stmt::Assign(e, _) | Stmt::Expression(e) | Stmt::If(e, ..) | Stmt::While(e, ..) => {
                    &e.at
                }
                Stmt::Return(at, _) | Stmt::Break(at) | Stmt::Continue(at) | Stmt::Loop(at, _) => {
                    at
                }
            };
            self.sequence(at);
            match statement {
                Stmt::If(condition, yes, no) => {
                    self.condition(condition)?;
                    let otherwise = self.label();
                    let end = self.label();
                    self.body.push(format!("brfalse {otherwise}"));
                    let yes_exits = self.block(yes)?;
                    if !yes_exits {
                        self.body.push(format!("br {end}"));
                    }
                    self.body.push(format!("{otherwise}:"));
                    let no_exits = self.block(no)?;
                    returned = yes_exits && no_exits;
                    if !returned {
                        self.body.push(format!("{end}:"));
                    }
                }
                Stmt::While(condition, body) => {
                    let start = self.label();
                    let end = self.label();
                    self.body.push(format!("{start}:"));
                    self.condition(condition)?;
                    self.body.push(format!("brfalse {end}"));
                    self.loops.push((end.clone(), start.clone()));
                    let exits = self.block(body)?;
                    self.loops.pop();
                    if !exits {
                        self.body.push(format!("br {start}"));
                    }
                    self.body.push(format!("{end}:"));
                }
                Stmt::Loop(_, body) => {
                    let start = self.label();
                    let end = self.label();
                    self.body.push(format!("{start}:"));
                    self.loops.push((end.clone(), start.clone()));
                    let exits = self.block(body)?;
                    self.loops.pop();
                    if !exits {
                        self.body.push(format!("br {start}"));
                    }
                    self.body.push(format!("{end}:"));
                }
                Stmt::For(name, start, limit, inclusive, body) => {
                    if self.bindings.contains_key(&name.text) {
                        return Err(name.error("duplicate binding"));
                    }
                    let start_ty = self.value_expression(start)?;
                    self.require(&start_ty, &Ty::Int, &start.at)?;
                    let index = self.temp(&Ty::Int);
                    self.body.push(format!("local.reset {index}"));
                    self.body.push(format!("stloc {index}"));
                    let limit_ty = self.value_expression(limit)?;
                    self.require(&limit_ty, &Ty::Int, &limit.at)?;
                    let bound = self.temp(&Ty::Int);
                    self.body.push(format!("stloc {bound}"));
                    let head = self.label();
                    let step = self.label();
                    let end = self.label();
                    self.body.extend([
                        format!("{head}:"),
                        format!("ldloc {index}"),
                        format!("ldloc {bound}"),
                        if *inclusive {
                            "cgt".into()
                        } else {
                            "clt".into()
                        },
                        format!("{} {end}", if *inclusive { "brtrue" } else { "brfalse" }),
                    ]);
                    self.bindings.insert(
                        name.text.clone(),
                        Binding {
                            ty: Ty::Int,
                            mutable: false,
                            load: format!("ldloc {index}"),
                            address: format!("ldloca {index}"),
                            scoped: true,
                        },
                    );
                    self.loops.push((end.clone(), step.clone()));
                    self.block(body)?;
                    self.loops.pop();
                    self.bindings.remove(&name.text);
                    // Test the inclusive endpoint before incrementing to avoid Int32 wraparound.
                    self.body.extend([
                        format!("{step}:"),
                        format!("ldloc {index}"),
                        format!("ldloc {bound}"),
                        "ceq".into(),
                        format!("brtrue {end}"),
                        format!("ldloc {index}"),
                        "ldc.i4 1".into(),
                        "add".into(),
                        format!("stloc {index}"),
                        format!("br {head}"),
                        format!("{end}:"),
                    ]);
                }
                Stmt::Break(at) | Stmt::Continue(at) => {
                    let Some((end, step)) = self.loops.last() else {
                        return Err(at.error("break/continue requires a loop"));
                    };
                    self.body.push(format!(
                        "br {}",
                        if matches!(statement, Stmt::Break(_)) {
                            end
                        } else {
                            step
                        }
                    ));
                    returned = true;
                }
                Stmt::Bind {
                    name,
                    mutable,
                    annotation,
                    extent,
                    value,
                } => {
                    if self.bindings.contains_key(&name.text) {
                        return Err(name.error("duplicate binding"));
                    }
                    if matches!((extent, value),
                        (Some(count), Some(Expr { kind: ExprKind::ArrayLiteral(elements), .. }))
                        if *count as usize != elements.len())
                    {
                        return Err(name.error("array initializer count does not match extent"));
                    }
                    let ty = match (value, annotation) {
                        (Some(value), Some(annotation)) => {
                            self.expression_for(value, annotation)?
                        }
                        (Some(value), None) => self.expression(value)?,
                        (None, Some(annotation)) => annotation.clone(),
                        (None, None) => unreachable!(),
                    };
                    if let Some(count) = extent {
                        self.check_array_length(*count as usize, true);
                    }
                    let index = self.locals.len();
                    self.locals
                        .push(format!(".local {} {}_{index}", ty.il(), name.text));
                    self.body.push(format!("local.reset {index}"));
                    if value.is_some() {
                        self.body.push(format!("stloc {index}"));
                    }
                    self.bindings.insert(
                        name.text.clone(),
                        Binding {
                            ty,
                            mutable: *mutable,
                            load: format!("ldloc {index}"),
                            address: format!("ldloca {index}"),
                            scoped: self.scope != 0,
                        },
                    );
                }
                Stmt::Assign(left, right) => {
                    if let ExprKind::Index(owner, index) = &left.kind {
                        if self.indexer(owner, index, Some(right))?.is_some() {
                            self.body.push("pop".into());
                            continue;
                        }
                    }
                    if let ExprKind::Name(name) = &left.kind {
                        let binding = self.binding(name, &left.at)?;
                        let rebind = matches!(binding.ty, Ty::Ref(_))
                            && matches!(&right.kind, ExprKind::Unary(op, _) if op == "&");
                        if let Ty::Ref(target) = &binding.ty {
                            if !rebind {
                                self.body.push(binding.load);
                                self.expression_for(right, target)?;
                                self.body.push(format!("stobj {}", target.il()));
                                continue;
                            }
                        }
                        if !binding.mutable {
                            return Err(left.at.error("cannot assign an immutable binding"));
                        }
                        self.expression_for(right, &binding.ty)?;
                        self.body.push(binding.load.replacen("ldloc", "stloc", 1));
                    } else {
                        let expected = self.place(left, false)?;
                        self.expression_for(right, &expected)?;
                        self.body.push(format!("stobj {}", expected.il()));
                    }
                }
                Stmt::Return(at, value) => {
                    let ty = if let Some(value) = value {
                        self.expression_for(value, &self.function.returns)?
                    } else {
                        self.body.push("ldvoid".into());
                        Ty::Void
                    };
                    self.require(&ty, &self.function.returns, at)?;
                    self.body.push("ret".into());
                    returned = true;
                }
                Stmt::Expression(expression) => {
                    if let ExprKind::Match(value, arms) = &expression.kind {
                        returned = self.match_arms(value, arms, true, None, false)?.1;
                    } else {
                        self.expression(expression)?;
                        self.body.push("pop".into());
                    }
                }
            }
        }
        Ok(returned)
    }
    fn lower(mut self) -> Result<String, Fault> {
        self.sequence(&self.function.name);
        if let Some(receiver) = self.receiver {
            self.bindings.insert(
                "this".into(),
                Binding {
                    ty: Ty::Ref(Box::new(Ty::Record(receiver.text.clone()))),
                    mutable: false,
                    load: "ldarg 0".into(),
                    address: "ldarga 0".into(),
                    scoped: false,
                },
            );
        }
        for (index, parameter) in self.function.parameters.iter().enumerate() {
            let index = index + usize::from(self.receiver.is_some());
            self.bindings.insert(
                parameter.name.text.clone(),
                Binding {
                    ty: parameter.ty.clone(),
                    mutable: false,
                    load: format!("ldarg {index}"),
                    address: format!("ldarga {index}"),
                    scoped: false,
                },
            );
        }
        let returned = self.statements(&self.function.body)?;
        if !returned {
            if self.function.returns != Ty::Void {
                return Err(self
                    .function
                    .name
                    .error("function requires an explicit return"));
            }
            self.body.extend(["ldvoid".into(), "ret".into()]);
        }
        Ok(format!(
            "{} {}({}) -> {}\n{}\n{}\n.end\n",
            if self.receiver.is_some() {
                if self.function.receiver_readonly {
                    ".method instance readonly byref"
                } else {
                    ".method instance byref"
                }
            } else {
                ".function"
            },
            self.function.name.text,
            self.function
                .parameters
                .iter()
                .map(|field| format!(
                    "{}{} {}",
                    if field.output {
                        "out "
                    } else if field.readonly {
                        "readonly "
                    } else {
                        ""
                    },
                    field.ty.il(),
                    field.name.text
                ))
                .collect::<Vec<_>>()
                .join(","),
            self.function.returns.il(),
            self.locals.join("\n"),
            self.body.join("\n")
        ))
    }
}

/// Lower a Neo source program to inspectable neoIL.
pub fn lower_to_il(source: &str) -> Result<String, Fault> {
    lower_to_il_named(source, "<source>")
}

/// Lower with a diagnostic source document name, retained in JSON artifacts.
pub fn lower_to_il_named(source: &str, document: &str) -> Result<String, Fault> {
    let tokens = lex(source)?;
    let source = Parser {
        tokens,
        position: 0,
        depth: 0,
    }
    .source()?;
    let mut names = HashMap::new();
    for name in source
        .records
        .iter()
        .map(|record| &record.name)
        .chain(source.functions.iter().map(|function| &function.name))
        .chain(source.interfaces.iter().map(|interface| &interface.name))
    {
        if names.insert(name.text.clone(), ()).is_some()
            || [
                "int",
                "Int32",
                "String",
                "string",
                "bool",
                "Boolean",
                "Void",
                "unit",
                "Console",
                "WriteLine",
                "System",
                "Option",
                "Result",
                "Byte",
                "byte",
                "array",
            ]
            .contains(&name.text.as_str())
        {
            return Err(name.error("duplicate or reserved declaration name"));
        }
    }
    let main = source
        .functions
        .iter()
        .find(|function| function.name.text == "Main")
        .ok_or_else(|| Fault::new("source requires func Main()"))?;
    if !main.parameters.is_empty() {
        return Err(main.name.error("Main must be parameterless"));
    }
    let mut il = String::from(".module SourceProgram\n.entry Main\n");
    for interface in &source.interfaces {
        il.push_str(&format!(".interface {}\n", interface.name.text));
        for method in &interface.methods {
            il.push_str(&format!(
                ".method instance {}byref {}({}) -> {}\n.end\n",
                if method.receiver_readonly {
                    "readonly "
                } else {
                    ""
                },
                method.name.text,
                method
                    .parameters
                    .iter()
                    .map(|p| format!(
                        "{}{}",
                        if p.output {
                            "out "
                        } else if p.readonly {
                            "readonly "
                        } else {
                            ""
                        },
                        p.ty.il()
                    ))
                    .collect::<Vec<_>>()
                    .join(","),
                method.returns.il()
            ));
        }
        il.push_str(".end\n");
    }
    for record in &source.records {
        il.push_str(&format!(".type {}\n", record.name.text));
        for interface in &record.implements {
            if !source
                .interfaces
                .iter()
                .any(|i| i.name.text == interface.il())
            {
                return Err(record
                    .name
                    .error("conformance requires a declared source interface"));
            }
            il.push_str(&format!(".implements {}\n", interface.il()));
        }
        for field in &record.fields {
            il.push_str(&format!(".field {} {}\n", field.name.text, field.ty.il()));
        }
        for function in &record.methods {
            il.push_str(
                &Lowerer {
                    document,
                    receiver: Some(&record.name),
                    source: &source,
                    function,
                    bindings: HashMap::new(),
                    locals: Vec::new(),
                    body: Vec::new(),
                    labels: 0,
                    scope: 0,
                    loops: Vec::new(),
                }
                .lower()?,
            );
        }
        il.push_str(".end\n");
    }
    for function in &source.functions {
        il.push_str(
            &Lowerer {
                document,
                receiver: None,
                source: &source,
                function,
                bindings: HashMap::new(),
                locals: Vec::new(),
                body: Vec::new(),
                labels: 0,
                scope: 0,
                loops: Vec::new(),
            }
            .lower()?,
        );
    }
    Ok(il)
}

/// Compile through the existing assembler and verifier; execution still enforces
/// reference provenance and runtime limits independently.
pub fn compile(source: &str) -> Result<Module, Fault> {
    compile_named(source, "<source>")
}

/// Compile with source mapping to a named document.
pub fn compile_named(source: &str, document: &str) -> Result<Module, Fault> {
    let module = crate::assemble(&lower_to_il_named(source, document)?)?;
    crate::LoadedProgram::new(&module)?.verify()?;
    Ok(module)
}
