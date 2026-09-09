//! Bound member lookup uses existing borrowed views; no boxing or implicit argument borrowing.
use super::*;

struct Member {
    owner: String,
    parameters: Vec<Field>,
    returns: Ty,
    interface: bool,
    virtual_call: bool,
    readonly: bool,
}

impl Lowerer<'_> {
    pub(super) fn constrained_call(
        &mut self,
        receiver: &Ty,
        member: &Token,
        arguments: &[Expr],
    ) -> Result<Option<Ty>, Fault> {
        let (target, reference) = match receiver {
            Ty::Ref(t) | Ty::ReadOnlyRef(t) => (t.as_ref(), true),
            t => (t, false),
        };
        let Some(index) = self
            .function
            .generic_parameters
            .iter()
            .position(|p| p == &target.il())
        else {
            return Ok(None);
        };
        if !reference {
            return Err(member.error("constrained member access currently requires a T& receiver"));
        }
        let mut found: Option<Member> = None;
        for constraint in &self.function.generic_constraints {
            if constraint.parameter as usize != index {
                continue;
            }
            let crate::metadata::ConstraintKind::TypeBound(bound) = &constraint.kind else {
                continue;
            };
            let bound = Ty::from_metadata(bound)?;
            let interface = self
                .source
                .interfaces
                .iter()
                .any(|i| i.name.text == bound.il())
                || library::is_interface(&bound)?;
            let source = if interface {
                self.source.interface_method(&bound, &member.text)?
            } else {
                self.source.record_method(&bound, &member.text)
            };
            let candidate = if let Some((owner, method)) = source {
                Member {
                    owner,
                    parameters: method.parameters.clone(),
                    returns: method.returns.clone(),
                    interface,
                    virtual_call: interface || method.is_virtual,
                    readonly: method.receiver_readonly,
                }
            } else if let Some(parameters) =
                library::parameters(&bound, &member.text, arguments.len())?
            {
                let signature = format!(
                    "instance {}::{}({})",
                    bound.il(),
                    member.text,
                    parameters
                        .iter()
                        .map(Ty::parameter_il)
                        .collect::<Vec<_>>()
                        .join(",")
                );
                let function = library::resolve(&signature)?;
                if !interface && !function.receiver_byref {
                    return Err(member.error(
                        "constrained record calls currently require a byref method receiver",
                    ));
                }
                Member {
                    owner: Ty::from_metadata(function.owner.as_ref().unwrap())?.il(),
                    parameters: function
                        .parameters
                        .iter()
                        .enumerate()
                        .map(|(i, ty)| {
                            Ok(Field {
                                name: member.clone(),
                                ty: Ty::from_metadata(ty)?,
                                output: function.out_parameters.contains(&i),
                                readonly: function.readonly_parameters.contains(&i),
                            })
                        })
                        .collect::<Result<_, Fault>>()?,
                    returns: Ty::from_metadata(&function.returns)?,
                    interface,
                    virtual_call: interface || function.is_virtual,
                    readonly: function.receiver_readonly,
                }
            } else {
                continue;
            };
            if let Some(previous) = &found {
                if previous.owner == candidate.owner {
                    continue;
                }
                return Err(member
                    .error("ambiguous constrained member; use a function with a narrower bound"));
            }
            found = Some(candidate);
        }
        let method =
            found.ok_or_else(|| member.error("member is not provided by generic constraints"))?;
        if matches!(receiver, Ty::ReadOnlyRef(_)) && !method.readonly {
            return Err(member.error("readonly reference cannot invoke a writable receiver"));
        }
        if method.parameters.len() != arguments.len() {
            return Err(member.error("argument count mismatch"));
        }
        self.body.push(format!(
            "{} {}",
            if method.interface {
                "interface.borrow"
            } else {
                "castclass"
            },
            method.owner
        ));
        for (argument, parameter) in arguments.iter().zip(&method.parameters) {
            self.parameter_argument(argument, parameter)?;
        }
        self.body.push(format!(
            "{} instance {}::{}({})",
            if method.virtual_call {
                "callvirt"
            } else {
                "call"
            },
            method.owner,
            member.text,
            method
                .parameters
                .iter()
                .map(|p| p.ty.parameter_il())
                .collect::<Vec<_>>()
                .join(",")
        ));
        Ok(Some(method.returns))
    }
}
