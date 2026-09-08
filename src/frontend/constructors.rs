//! Public library constructor calls and argument-based owner inference.
use super::*;

impl Lowerer<'_> {
    pub(super) fn library_constructor(
        &mut self,
        callee: &Expr,
        type_arguments: &[Ty],
        arguments: &[Expr],
    ) -> Result<Option<Ty>, Fault> {
        let Some(path) = Self::qualified_name(callee) else {
            return Ok(None);
        };
        if self.bindings.contains_key(path.split('.').next().unwrap())
            || self.source.records.iter().any(|r| r.name.text == path)
        {
            return Ok(None);
        }
        let path = if matches!(callee.kind, ExprKind::Name(_)) {
            imports::lookup(&self.source.case_aliases, &path, &callee.at)?.unwrap_or(path)
        } else {
            path
        };
        let path = match path.as_str() {
            "Result" => "System.Result".to_owned(),
            "Option" => "System.Option".to_owned(),
            _ => path,
        };
        let inferred;
        let type_arguments = if type_arguments.is_empty() {
            inferred = self.infer_constructor_owner(callee, &path, arguments)?;
            inferred.as_slice()
        } else {
            type_arguments
        };
        let owner = if type_arguments.is_empty() {
            path
        } else {
            format!(
                "{path}<{}>",
                type_arguments
                    .iter()
                    .map(Ty::il)
                    .collect::<Vec<_>>()
                    .join(",")
            )
        };
        let ty = Ty::Record(owner.clone());
        let Some(candidates) =
            library::constructors(&ty, arguments.len()).map_err(|e| callee.at.error(e.message))?
        else {
            return Ok(None);
        };
        self.lower_library_constructor(callee, &owner, arguments, candidates)
    }

    fn lower_library_constructor(
        &mut self,
        callee: &Expr,
        owner: &str,
        arguments: &[Expr],
        candidates: Vec<crate::metadata::Function>,
    ) -> Result<Option<Ty>, Fault> {
        let mut selected = None;
        let mut failure = None;
        for candidate in candidates {
            let mut probe = self.clone();
            let lowered = (|| {
                for (index, (argument, parameter)) in
                    arguments.iter().zip(&candidate.parameters).enumerate()
                {
                    probe.parameter_argument(
                        argument,
                        &Field {
                            name: callee.at.clone(),
                            ty: Ty::from_metadata(parameter)?,
                            output: candidate.out_parameters.contains(&index),
                            readonly: candidate.readonly_parameters.contains(&index),
                        },
                    )?;
                }
                let parameters = candidate
                    .parameters
                    .iter()
                    .map(|p| Ty::from_metadata(p).map(|p| p.il()))
                    .collect::<Result<Vec<_>, _>>()?;
                probe.body.push(format!(
                    "newobj instance {owner}::.ctor({})",
                    parameters.join(",")
                ));
                Ok::<_, Fault>(())
            })();
            match lowered {
                Ok(()) => {
                    if selected.is_some() {
                        return Err(callee.at.error("ambiguous library constructor"));
                    }
                    selected = Some(probe);
                }
                Err(error) => failure = Some(error),
            }
        }
        if let Some(probe) = selected {
            *self = probe;
            Ok(Some(Ty::Record(owner.to_owned())))
        } else {
            Err(failure.unwrap_or_else(|| {
                callee
                    .at
                    .error("no accessible library constructor matches the arguments")
            }))
        }
    }

    fn infer_constructor_owner(
        &mut self,
        callee: &Expr,
        path: &str,
        arguments: &[Expr],
    ) -> Result<Vec<Ty>, Fault> {
        use crate::metadata::Type;
        let module = crate::library::system()?;
        // An exact nongeneric owner keeps ordinary constructor resolution.
        if module
            .types
            .iter()
            .any(|d| d.name == path && d.generic_parameters.is_empty())
        {
            return Ok(Vec::new());
        }
        let definitions = module
            .types
            .iter()
            .filter(|d| d.name == path && !d.generic_parameters.is_empty())
            .collect::<Vec<_>>();
        if definitions.is_empty() {
            return Ok(Vec::new());
        }
        let key = callee as *const Expr as usize;
        if let Some(types) = self.inferred_calls.borrow().get(&key).cloned() {
            return Ok(types);
        }
        let mut selected = None;
        for definition in definitions {
            if definition.visibility != crate::metadata::Visibility::Public
                || definition.is_abstract
            {
                continue;
            }
            let placeholders = (0..definition.generic_parameters.len())
                .map(|i| Type::MethodTypeParameter(i as u16))
                .collect::<Vec<_>>();
            for constructor in module.functions.iter().filter(|f| {
                f.instance
                    && f.owner.as_ref() == Some(&definition.open_type())
                    && f.name.ends_with("..ctor")
                    && f.visibility == crate::metadata::Visibility::Public
                    && f.parameters.len() == arguments.len()
                    && f.generic_parameters.is_empty()
            }) {
                let mut found = vec![None; placeholders.len()];
                let mut probe = self.clone();
                let attempt = (|| {
                    for (argument, parameter) in arguments.iter().zip(&constructor.parameters) {
                        let actual = if matches!(
                            parameter,
                            Type::TypeParameter(_) | Type::ByRef(_) | Type::ReadOnlyByRef(_)
                        ) {
                            probe.expression(argument)?
                        } else {
                            probe.library_argument(argument)?
                        };
                        let formal = parameter.substitute_type_parameters(&placeholders)?;
                        infer_parameters(
                            &formal,
                            &crate::assembler::parse_type(&actual.il())?,
                            &mut found,
                        )?;
                    }
                    found
                        .into_iter()
                        .map(|ty| {
                            ty.ok_or_else(|| {
                                callee.at.error(
                                    "cannot infer all constructor type arguments; supply explicit type arguments",
                                )
                            })
                            .and_then(|ty| Ty::from_metadata(&ty))
                        })
                        .collect::<Result<Vec<_>, Fault>>()
                })();
                if let Ok(types) = attempt {
                    if selected.as_ref().is_some_and(|previous| previous != &types) {
                        return Err(callee.at.error(
                            "ambiguous inferred constructor owner; supply explicit type arguments",
                        ));
                    }
                    selected = Some(types);
                }
            }
        }
        let types = selected.ok_or_else(|| {
            callee.at.error(
                "cannot infer all constructor type arguments; supply explicit type arguments",
            )
        })?;
        self.inferred_calls.borrow_mut().insert(key, types.clone());
        Ok(types)
    }
}
