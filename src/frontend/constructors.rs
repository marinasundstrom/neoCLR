//! Ordinary public library constructor calls, with explicit closed owner types.
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
        let path = match path.as_str() {
            "Result" => "System.Result".to_owned(),
            "Option" => "System.Option".to_owned(),
            _ => path,
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
            Ok(Some(ty))
        } else {
            Err(failure.unwrap_or_else(|| {
                callee
                    .at
                    .error("no accessible library constructor matches the arguments")
            }))
        }
    }
}
