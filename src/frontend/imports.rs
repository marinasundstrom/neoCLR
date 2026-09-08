//! File-wide imports of independently named union cases.
use super::*;

pub(super) fn resolve(source: &Source) -> Result<HashMap<String, Vec<String>>, Fault> {
    let mut aliases: HashMap<String, Vec<String>> = HashMap::new();
    for import in &source.case_imports {
        let variants = if let Some(union) = source
            .unions
            .iter()
            .find(|union| union.name.text == import.text)
        {
            union.variants.iter().map(Ty::il).collect()
        } else {
            library::imported_cases(&import.text)
                .map_err(|error| import.error(error.message))?
                .ok_or_else(|| {
                    import.error(
                        "case import requires a declared source union or marked bundled union",
                    )
                })?
        };
        for qualified in variants {
            let name = qualified.rsplit('.').next().unwrap().to_owned();
            if qualified == name || PREDEFINED_NAMES.contains(&name.as_str()) {
                continue;
            }
            // File-level type declarations take precedence over imported names.
            if source.records.iter().any(|record| record.name.text == name)
                || source.unions.iter().any(|union| union.name.text == name)
                || source
                    .interfaces
                    .iter()
                    .any(|interface| interface.name.text == name)
                || source
                    .delegates
                    .iter()
                    .any(|delegate| delegate.name.text == name)
            {
                continue;
            }
            let targets = aliases.entry(name).or_default();
            if !targets.contains(&qualified) {
                targets.push(qualified);
            }
        }
    }
    Ok(aliases)
}

pub(super) fn lookup(
    aliases: &HashMap<String, Vec<String>>,
    name: &str,
    at: &Token,
) -> Result<Option<String>, Fault> {
    match aliases.get(name).map(Vec::as_slice) {
        None => Ok(None),
        Some([target]) => Ok(Some(target.clone())),
        Some(_) => Err(at.error(format!(
            "ambiguous imported case {name}; use its qualified type name"
        ))),
    }
}

impl Lowerer<'_> {
    pub(super) fn allocate_record(&mut self, expression: &Expr, value: &Expr) -> Result<Ty, Fault> {
        let ExprKind::Call(callee, _) = &value.kind else {
            return Err(expression.at.error("new requires record construction"));
        };
        if let ExprKind::Generic(target, _) = &callee.kind {
            let name = Self::qualified_name(target)
                .ok_or_else(|| expression.at.error("new requires record construction"))?;
            if !self
                .source
                .records
                .iter()
                .any(|record| record.name.text == name)
            {
                return Err(expression.at.error("new requires record construction"));
            }
            let ty = self.expression(value)?;
            self.body.push("heap.new".into());
            return Ok(Ty::Ref(Box::new(ty)));
        }
        let name = Self::qualified_name(callee)
            .ok_or_else(|| expression.at.error("new requires record construction"))?;
        let resolved = imports::lookup(&self.source.case_aliases, &name, &callee.at)?
            .unwrap_or_else(|| name.clone());
        if !self
            .source
            .records
            .iter()
            .any(|record| record.name.text == resolved)
            && name != "array"
        {
            return Err(expression.at.error("new requires record construction"));
        }
        let ty = if resolved != name || !matches!(callee.kind, ExprKind::Name(_)) {
            let ExprKind::Call(_, arguments) = &value.kind else {
                unreachable!()
            };
            self.call(
                &Expr {
                    kind: ExprKind::Name(resolved),
                    ..callee.as_ref().clone()
                },
                arguments,
            )?
        } else {
            self.expression(value)?
        };
        self.body.push("heap.new".into());
        Ok(Ty::Ref(Box::new(ty)))
    }
    pub(super) fn imported_call(
        &mut self,
        callee: &Expr,
        arguments: &[Expr],
    ) -> Result<Option<Ty>, Fault> {
        if let ExprKind::Name(name) = &callee.kind {
            if !self.bindings.contains_key(name)
                && !self
                    .source
                    .functions
                    .iter()
                    .any(|function| function.name.text == *name)
            {
                if let Some(qualified) =
                    imports::lookup(&self.source.case_aliases, name, &callee.at)?
                {
                    // Bundled constructors resolve the alias using the original AST node.
                    // Inference caches must never use a temporary rewritten callee address.
                    if !self
                        .source
                        .records
                        .iter()
                        .any(|record| record.name.text == qualified)
                    {
                        return Ok(None);
                    }
                    return self
                        .call(
                            &Expr {
                                kind: ExprKind::Name(qualified),
                                ..callee.clone()
                            },
                            arguments,
                        )
                        .map(Some);
                }
            }
        }
        Ok(None)
    }
}
