//! Lexical capture analysis; binding locations distinguish separate block declarations.
use super::*;
use std::collections::HashSet;
pub(super) type Key = (usize, usize);
pub(super) fn key(token: &Token) -> Key {
    (token.line, token.column)
}

#[derive(Default)]
struct Analysis {
    captured: HashSet<Key>,
    free: HashSet<String>,
}
type Scope = HashMap<String, (Key, usize)>;
impl Analysis {
    fn expression(&mut self, e: &Expr, scope: &Scope, depth: usize) {
        match &e.kind {
            ExprKind::Name(name) => match scope.get(name) {
                Some((key, declared)) if *declared < depth => {
                    self.captured.insert(*key);
                }
                None => {
                    self.free.insert(name.clone());
                }
                _ => (),
            },
            ExprKind::Lambda(parameters, body) => {
                let mut scope = scope.clone();
                for (name, _) in parameters {
                    scope.insert(name.text.clone(), (key(name), depth + 1));
                }
                self.arm(body, &scope, depth + 1);
            }
            ExprKind::Field(e, _)
            | ExprKind::Unary(_, e)
            | ExprKind::Generic(e, _)
            | ExprKind::Out(e)
            | ExprKind::InterfaceCast(e, _) => self.expression(e, scope, depth),
            ExprKind::Call(e, args) => {
                self.expression(e, scope, depth);
                for a in args {
                    self.expression(a, scope, depth);
                }
            }
            ExprKind::Binary(_, a, b) | ExprKind::Index(a, b) => {
                self.expression(a, scope, depth);
                self.expression(b, scope, depth);
            }
            ExprKind::ArrayLiteral(items) => {
                for e in items {
                    self.expression(e, scope, depth);
                }
            }
            ExprKind::NewArray(_, n, items) => {
                self.expression(n, scope, depth);
                for e in items {
                    self.expression(e, scope, depth);
                }
            }
            ExprKind::Match(value, arms) => {
                self.expression(value, scope, depth);
                for arm in arms {
                    let mut scope = scope.clone();
                    if let Pattern::Case(_, Some(Some(name))) = &arm.pattern {
                        scope.insert(name.text.clone(), (key(name), depth));
                    }
                    self.arm(&arm.body, &scope, depth);
                }
            }
            ExprKind::Int(_)
            | ExprKind::String(_)
            | ExprKind::Bool(_)
            | ExprKind::TypeOf(_)
            | ExprKind::Default(_) => (),
        }
    }
    fn arm(&mut self, body: &ArmBody, scope: &Scope, depth: usize) {
        match body {
            ArmBody::Expression(e) => self.expression(e, scope, depth),
            ArmBody::Block(b) => self.block(b, scope, depth),
        }
    }
    fn block(&mut self, statements: &[Stmt], scope: &Scope, depth: usize) {
        let mut scope = scope.clone();
        for s in statements {
            match s {
                Stmt::Bind { name, value, .. } => {
                    if let Some(e) = value {
                        self.expression(e, &scope, depth);
                    }
                    scope.insert(name.text.clone(), (key(name), depth));
                }
                Stmt::Assign(a, b) => {
                    self.expression(a, &scope, depth);
                    self.expression(b, &scope, depth);
                }
                Stmt::Return(_, value) => {
                    if let Some(e) = value {
                        self.expression(e, &scope, depth);
                    }
                }
                Stmt::Expression(e) => self.expression(e, &scope, depth),
                Stmt::If(e, a, b) => {
                    self.expression(e, &scope, depth);
                    self.block(a, &scope, depth);
                    self.block(b, &scope, depth);
                }
                Stmt::While(e, b) => {
                    self.expression(e, &scope, depth);
                    self.block(b, &scope, depth);
                }
                Stmt::Loop(_, b) => self.block(b, &scope, depth),
                Stmt::For(name, a, b, _, body) => {
                    self.expression(a, &scope, depth);
                    self.expression(b, &scope, depth);
                    let mut nested = scope.clone();
                    nested.insert(name.text.clone(), (key(name), depth));
                    self.block(body, &nested, depth);
                }
                Stmt::Break(_) | Stmt::Continue(_) => (),
            }
        }
    }
}
pub(super) fn captured(function: &Function) -> HashSet<Key> {
    let mut scope = Scope::new();
    scope.insert("this".into(), (key(&function.name), 0));
    for p in &function.parameters {
        scope.insert(p.name.text.clone(), (key(&p.name), 0));
    }
    let mut a = Analysis::default();
    a.block(&function.body, &scope, 0);
    a.captured
}
pub(super) fn free(parameters: &[(Token, Option<Ty>)], body: &ArmBody) -> HashSet<String> {
    let scope = parameters
        .iter()
        .map(|(t, _)| (t.text.clone(), (key(t), 0)))
        .collect();
    let mut a = Analysis::default();
    a.arm(body, &scope, 0);
    a.free
}
