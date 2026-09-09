//! Conditional case bindings reuse the same carrier tests and extraction as match.
use super::*;

impl Parser {
    pub(super) fn conditional_pattern(&mut self) -> Result<Pattern, Fault> {
        let case = self.name()?;
        let binding = if self.eat("(") {
            let binding = self.name()?;
            self.expect(")")?;
            Some(if binding.text == "_" {
                None
            } else {
                Some(binding)
            })
        } else {
            None
        };
        Ok(Pattern::Case(case, binding))
    }
}

impl Lowerer<'_> {
    pub(super) fn conditional_case(
        &mut self,
        pattern: &Pattern,
        value: &Expr,
        missing: &str,
    ) -> Result<(), Fault> {
        let ty = self.value_expression(value)?;
        let source_union = self.source.source_union(&ty);
        let cases = if let Some(union) = source_union {
            union.cases(&ty)?
        } else {
            library::cases(&ty).map_err(|e| value.at.error(e.message))?
        };
        let Pattern::Case(name, binding) = pattern else {
            return Err(value
                .at
                .error("conditional binding requires a case pattern"));
        };
        let case = cases
            .iter()
            .find(|c| c.name == name.text)
            .ok_or_else(|| name.error("unknown union case"))?;
        if source_union.is_none() && binding.is_some() != case.payload.is_some() {
            return Err(name.error("case payload pattern does not match its shape"));
        }
        let scrutinee = self.temp(&ty);
        self.body.extend([
            format!("local.reset {scrutinee}"),
            format!("stloc {scrutinee}"),
            format!("ldloc {scrutinee}"),
            format!("call {}", case.test),
            format!("brfalse {missing}"),
        ]);
        if let Some(Some(name)) = binding {
            if self.bindings.contains_key(&name.text) {
                return Err(name.error("duplicate binding"));
            }
            let (payload, accessor) = case.payload.as_ref().unwrap();
            self.body.extend([
                format!("ldloc {scrutinee}"),
                format!("call {}", case.extract),
            ]);
            if !accessor.is_empty() {
                self.body.push(format!("call {accessor}"));
            }
            let index = self.temp(payload);
            self.body
                .extend([format!("local.reset {index}"), format!("stloc {index}")]);
            self.bindings.insert(
                name.text.clone(),
                Binding {
                    cell: None,
                    ty: payload.clone(),
                    mutable: false,
                    load: format!("ldloc {index}"),
                    address: format!("ldloca {index}"),
                    scoped: true,
                },
            );
            if self.captures.contains(&closures::key(name)) {
                self.lift_binding(name, Some(index))?;
            }
        }
        Ok(())
    }
}
