//! A bounded enum projection using ordinary typed helper methods and integer IL.
use super::*;

pub(super) struct Enum {
    pub name: Token,
    pub info: crate::metadata::EnumInfo,
}

pub(super) fn parse(parser: &mut Parser, source: &mut Source, flags: bool) -> Result<(), Fault> {
    let name = parser.name()?;
    if parser.eat(":") && parser.ty()? != Ty::Int {
        return Err(name.error("enum underlying type must currently be int"));
    }
    parser.newlines();
    parser.expect("{")?;
    parser.lines();
    let mut members = Vec::new();
    let mut next = Some(0i32);
    while !parser.at("}") && !parser.at("") {
        let member = parser.name()?;
        if [
            "FromValue",
            "Value",
            "get_Value",
            "Or",
            "And",
            "Xor",
            "Not",
            "HasFlag",
            "Equals",
        ]
        .contains(&member.text.as_str())
            || members
                .iter()
                .any(|m: &crate::metadata::EnumMember| m.name == member.text)
        {
            return Err(member.error("duplicate or reserved enum member name"));
        }
        let value = if parser.eat("=") {
            let negative = parser.eat("-");
            let token = parser.take();
            let number = token
                .text
                .parse::<i64>()
                .map_err(|_| token.error("enum value requires an integer literal"))?;
            i32::try_from(if negative { -number } else { number })
                .map_err(|_| token.error("enum literal is outside Int32"))?
        } else {
            next.ok_or_else(|| member.error("implicit enum value overflows Int32"))?
        };
        next = value.checked_add(1);
        members.push(crate::metadata::EnumMember {
            name: member.text,
            value,
        });
        if members.len() > 1024 {
            return Err(name.error("enum member limit exceeded"));
        }
        if !parser.eat(",") {
            parser.end_statement()?;
        }
        parser.lines();
    }
    parser.expect("}")?;
    source.enums.push(Enum {
        name,
        info: crate::metadata::EnumInfo {
            underlying: crate::metadata::Type::Int32,
            flags,
            members,
        },
    });
    Ok(())
}

impl Lowerer<'_> {
    pub(super) fn enum_info(&self, ty: &Ty) -> Result<Option<crate::metadata::EnumInfo>, Fault> {
        if let Some(def) = self.source.enums.iter().find(|e| e.name.text == ty.il()) {
            return Ok(Some(def.info.clone()));
        }
        let module = crate::library::system()?;
        let metadata = crate::assembler::parse_type(&ty.il())?;
        Ok(module
            .type_definition(&metadata)
            .and_then(|def| def.enum_info.clone()))
    }
    pub(super) fn enum_field(&mut self, owner: &Expr, member: &Token) -> Result<Option<Ty>, Fault> {
        if let Some(path) = Self::qualified_name(owner)
            .filter(|p| !self.bindings.contains_key(p.split('.').next().unwrap()))
        {
            let ty = Ty::Record(path);
            if let Some(info) = self.enum_info(&ty)? {
                if let Some(constant) = info.members.iter().find(|m| m.name == member.text) {
                    self.body.extend([
                        format!("ldc.i4 {}", constant.value),
                        format!("newobj {}", ty.il()),
                    ]);
                    return Ok(Some(ty));
                }
                return Err(member.error("unknown enum constant"));
            }
        }
        Ok(None)
    }
    pub(super) fn enum_call(
        &mut self,
        callee: &Expr,
        arguments: &[Expr],
    ) -> Result<Option<Ty>, Fault> {
        let ExprKind::Field(owner, member) = &callee.kind else {
            return Ok(None);
        };
        if let Some(path) = Self::qualified_name(owner)
            .filter(|p| !self.bindings.contains_key(p.split('.').next().unwrap()))
        {
            let ty = Ty::Record(path);
            if let Some(info) = self.enum_info(&ty)? {
                let params = if member.text == "FromValue" {
                    vec![Ty::Int]
                } else if info.members.iter().any(|m| m.name == member.text) {
                    vec![]
                } else {
                    return Err(member.error("unknown enum static member"));
                };
                if params.len() != arguments.len() {
                    return Err(member.error("argument count mismatch"));
                }
                for (arg, ty) in arguments.iter().zip(&params) {
                    self.expression_for(arg, ty)?;
                }
                self.body.push(format!(
                    "call {}::{}({})",
                    ty.il(),
                    member.text,
                    params.iter().map(Ty::il).collect::<Vec<_>>().join(",")
                ));
                return Ok(Some(ty));
            }
        }
        if !["Or", "And", "Xor", "Not", "HasFlag", "Equals", "get_Value"]
            .contains(&member.text.as_str())
        {
            return Ok(None);
        }
        // Keep successful receiver lowering; no receiver expression executes twice.
        let mut probe = self.clone();
        let Ok(ty) = probe.value_expression(owner) else {
            return Ok(None);
        };
        if self.enum_info(&ty)?.is_none() {
            return Ok(None);
        }
        let (count, result) = match member.text.as_str() {
            "Or" | "And" | "Xor" => (1, ty.clone()),
            "Not" => (0, ty.clone()),
            "HasFlag" | "Equals" => (1, Ty::Bool),
            "get_Value" => (0, Ty::Int),
            _ => return Ok(None),
        };
        if arguments.len() != count {
            return Err(member.error("argument count mismatch"));
        }
        *self = probe;
        for arg in arguments {
            self.expression_for(arg, &ty)?;
        }
        self.body.push(format!(
            "call instance {}::{}({})",
            ty.il(),
            member.text,
            if count == 1 { ty.il() } else { String::new() }
        ));
        Ok(Some(result))
    }
    pub(super) fn enum_binary(
        &mut self,
        ty: &Ty,
        operation: &str,
        right: &Expr,
    ) -> Result<Option<Ty>, Fault> {
        if self.enum_info(ty)?.is_none() {
            return Ok(None);
        }
        let method = match operation {
            "|" => "Or",
            "&" => "And",
            "^" => "Xor",
            "==" | "!=" => "Equals",
            _ => return Err(right.at.error("unsupported enum operator")),
        };
        self.expression_for(right, ty)?;
        self.body
            .push(format!("call instance {}::{method}({})", ty.il(), ty.il()));
        if operation == "!=" {
            self.body.extend(["ldc.bool false".into(), "ceq".into()]);
        }
        Ok(Some(if matches!(operation, "==" | "!=") {
            Ty::Bool
        } else {
            ty.clone()
        }))
    }
    pub(super) fn bit_not(&mut self, value: &Expr) -> Result<Ty, Fault> {
        let ty = self.value_expression(value)?;
        if self.enum_info(&ty)?.is_some() {
            self.body.push(format!("call instance {}::Not()", ty.il()));
        } else {
            self.require(&ty, &Ty::Int, &value.at)?;
            self.body.push("not".into());
        }
        Ok(ty)
    }
}
