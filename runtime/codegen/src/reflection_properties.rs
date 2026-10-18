//! Checked property adapters; metadata presence never implies retained user code.
use neoclr::metadata::{
    FunctionRef, Instruction as Op, MemberId, Representation, Type, TypeDefId, Visibility,
};
use serde_json::{Value, json};
type Error = Box<dyn std::error::Error>;

fn public_type(source: &neoclr::Module, ty: &Type) -> bool {
    match ty {
        Type::ArrayRef(t)
        | Type::Array(t)
        | Type::ByRef(t)
        | Type::ReadOnlyByRef(t)
        | Type::Ptr(t) => public_type(source, t),
        _ => {
            if let Type::Constructed { arguments, .. } = ty {
                if !arguments.iter().all(|t| public_type(source, t)) {
                    return false;
                }
            }
            let Some(mut def) = source.type_definition(ty) else {
                return ty
                    .definition_name()
                    .is_some_and(|n| n.starts_with("System."));
            };
            loop {
                if def.visibility != Visibility::Public
                    || def
                        .origin
                        .as_ref()
                        .is_some_and(|o| o.publicly_visible != Some(true))
                {
                    return false;
                }
                let Some(parent) = &def.declaring_type else {
                    return true;
                };
                let Some(found) = source
                    .types
                    .iter()
                    .find(|d| d.definition.as_ref() == Some(parent))
                else {
                    return false;
                };
                def = found;
            }
        }
    }
}

fn reject(body: &mut Vec<Op>, check: bool, status: i32) {
    if check {
        body.extend([Op::Int(status), Op::Return]);
    } else {
        let reason = match status {
            2 => "UnsupportedShape",
            3 => "AccessDenied",
            4 => "InvalidProperty",
            5 => "MissingAccessor",
            6 => "InvalidReceiver",
            7 => "InvalidValue",
            _ => "UnboundType",
        };
        body.push(Op::Fault(format!(
            "reflection property access rejected: {reason}"
        )));
    }
}
fn guard(body: &mut Vec<Op>, check: bool, status: i32) {
    let branch = body.len();
    body.push(Op::BranchFalse(0));
    reject(body, check, status);
    body[branch] = Op::BranchFalse(body.len());
}
pub fn bind(
    input: &mut neoclr::Module,
    source: &neoclr::Module,
    definitions: &[TypeDefId],
    methods: &[MemberId],
    retention: &mut Value,
) -> Result<Vec<usize>, Error> {
    let object = Type::from_name("System.Object");
    let mut roots = vec![];
    let mut bindings = vec![];
    let services: Vec<_> = input
        .functions
        .iter()
        .enumerate()
        .filter(|(_, f)| {
            matches!(
                f.name.as_str(),
                "neoCLR.Runtime.ReflectionPropertyGet"
                    | "neoCLR.Runtime.ReflectionPropertyGetCheck"
                    | "neoCLR.Runtime.ReflectionPropertySet"
                    | "neoCLR.Runtime.ReflectionPropertySetCheck"
            )
        })
        .map(|(i, _)| i)
        .collect();
    for index in services {
        let f = &input.functions[index];
        let setter = f.name.contains("PropertySet");
        let check = f.name.ends_with("Check");
        let mut parameters = vec![Type::RuntimeTypeHandle, Type::Int32, object.clone()];
        if setter {
            parameters.push(object.clone());
        }
        if f.owner.is_some()
            || f.instance
            || f.parameters != parameters
            || f.returns
                != if check {
                    Type::Int32
                } else if setter {
                    Type::Void
                } else {
                    object.clone()
                }
            || (!setter || check) && f.no_result
            || f.impl_flags != 0x1000
            || !f.body.is_empty()
            || !f.locals.is_empty()
            || f.pinvoke.is_some()
            || f.receiver_byref
            || f.receiver_readonly
            || f.is_virtual
            || f.is_abstract
            || f.is_override
            || !f.generic_parameters.is_empty()
            || !f.generic_arguments.is_empty()
            || !f.generic_constraints.is_empty()
            || !f.interface_implementations.is_empty()
            || !f.out_parameters.is_empty()
            || !f.out_when_true.is_empty()
            || !f.readonly_parameters.is_empty()
        {
            return Err(
                "property reflection requires the exact reserved InternalCall contract".into(),
            );
        }
        let no_result = f.no_result;
        let mut body = vec![];
        let mut map = vec![];
        let mut locals = vec![];
        let mut boxed_receivers = vec![];
        for row in retention["types"].as_array().unwrap() {
            if row["properties"] != true {
                continue;
            }
            let definition: TypeDefId = serde_json::from_value(row["definition"].clone())?;
            let owner_index = definitions
                .iter()
                .position(|d| *d == definition)
                .ok_or("missing property root")?;
            let ty = &source.types[owner_index];
            let owner = Type::Named(ty.name.clone());
            let owner_branch = body.len() + 3;
            body.extend([
                Op::Arg(0),
                Op::LoadTypeToken(owner.clone()),
                Op::Equal,
                Op::BranchFalse(0),
            ]);
            if (setter && !ty.is_reference_type)
                || ty.enum_info.is_some()
                || ty.representation != Representation::Record
                || !ty.generic_parameters.is_empty()
            {
                reject(&mut body, check, 2);
            } else {
                for (property_index, p) in ty.properties.iter().enumerate() {
                    let next = body.len() + 3;
                    body.extend([
                        Op::Arg(1),
                        Op::Int(property_index as i32),
                        Op::Equal,
                        Op::BranchFalse(0),
                    ]);
                    let target = if setter { &p.setter } else { &p.getter };
                    let target_index = target
                        .as_ref()
                        .map(|r| {
                            if let Some(id) = &r.definition {
                                methods.iter().position(|m| m == id)
                            } else {
                                source.functions.iter().position(|f| {
                                    f.name == r.name
                                        && f.owner == r.owner
                                        && f.parameters == r.parameters
                                        && f.instance == r.instance
                                })
                            }
                            .ok_or("missing original property accessor")
                        })
                        .transpose()?;
                    let reference = source.is_object_reference_type(&p.ty);
                    let status = if !p.instance || !p.parameters.is_empty() {
                        2
                    } else if let Some(i) = target_index {
                        let f = &source.functions[i];
                        if f.receiver_byref == ty.is_reference_type
                            || !f.instance
                            || f.is_internal_call()
                            || f.pinvoke.is_some()
                        {
                            2
                        } else if !public_type(source, &owner)
                            || !public_type(source, &p.ty)
                            || f.visibility != Visibility::Public
                            || ty
                                .origin
                                .as_ref()
                                .is_some_and(|o| o.publicly_visible != Some(true))
                            || f.origin.as_ref().is_some_and(|o| {
                                o.member_access
                                    != Some(neoclr::metadata_origin::SourceAccess::Public)
                            })
                        {
                            3
                        } else {
                            0
                        }
                    } else {
                        5
                    };
                    if status != 0 {
                        reject(&mut body, check, status);
                    } else {
                        if !reference
                            && !matches!(p.ty, Type::Int32 | Type::Int64 | Type::Boolean)
                            && !source.type_definition(&p.ty).is_some_and(|d| {
                                !d.is_reference_type
                                    && d.representation == Representation::Record
                                    && d.enum_info.is_none()
                                    && d.generic_parameters.is_empty()
                            })
                        {
                            return Err("native property adapters currently require Int32, Int64, Boolean or reference values".into());
                        }
                        body.extend([
                            Op::Arg(2),
                            Op::IsInstance(owner.clone()),
                            Op::ReferenceIsNull,
                        ]);
                        guard(&mut body, check, 6);
                        if setter {
                            // Reference null is valid; scalar null is not a box.
                            let null_branch = if reference {
                                let at = body.len() + 2;
                                body.extend([Op::Arg(3), Op::ReferenceIsNull, Op::BranchTrue(0)]);
                                Some(at)
                            } else {
                                None
                            };
                            body.extend([
                                Op::Arg(3),
                                Op::IsInstance(p.ty.clone()),
                                Op::ReferenceIsNull,
                            ]);
                            guard(&mut body, check, 7);
                            if let Some(at) = null_branch {
                                body[at] = Op::BranchTrue(body.len());
                            }
                        }
                        let i = target_index.unwrap();
                        let retained = row[if setter { "setters" } else { "getters" }] == true;
                        if retained && !roots.contains(&i) {
                            roots.push(i);
                        }
                        if check {
                            body.extend([Op::Int(0), Op::Return]);
                        } else if !retained {
                            body.push(Op::Fault(
                                "native reflection accessor invocation was not retained".into(),
                            ));
                        } else {
                            let target = &input.functions[i];
                            let call = FunctionRef {
                                definition: target.definition.clone(),
                                name: target.name.clone(),
                                owner: target.owner.clone(),
                                instance: true,
                                parameters: target.parameters.clone(),
                                generic_arguments: vec![],
                            };
                            let start = body.len();
                            body.push(Op::Arg(2));
                            if ty.is_reference_type {
                                body.push(Op::CastClass(owner.clone()));
                            } else {
                                // Verifiable value-call placeholder. Native lowering borrows
                                // the rooted box at the recorded address site, preserving
                                // mutations from user getter code instead of copying this.
                                let local = locals.len();
                                locals.push(owner.clone());
                                body.extend([Op::UnboxAny(owner.clone()), Op::Store(local)]);
                                boxed_receivers.push(body.len());
                                body.push(Op::LocalAddress(local));
                            }
                            if setter {
                                body.extend([
                                    Op::Arg(3),
                                    if reference {
                                        Op::CastClass(p.ty.clone())
                                    } else {
                                        Op::UnboxAny(p.ty.clone())
                                    },
                                ]);
                            }
                            body.push(if ty.is_reference_type {
                                Op::CallVirtual(call)
                            } else {
                                Op::Call(call)
                            });
                            if setter {
                                if target.no_result && !no_result {
                                    body.push(Op::Void);
                                } else if !target.no_result && no_result {
                                    body.push(Op::Pop);
                                }
                            } else {
                                body.push(if reference {
                                    Op::CastClass(object.clone())
                                } else {
                                    Op::BoxValue(p.ty.clone())
                                });
                            }
                            body.push(Op::Return);
                            map.extend(
                                (start..body.len()).map(
                                    |pc| json!({"instruction":pc,"adapterInstruction":pc-start}),
                                ),
                            );
                        }
                    }
                    body[next] = Op::BranchFalse(body.len());
                }
                reject(&mut body, check, 4);
            }
            body[owner_branch] = Op::BranchFalse(body.len());
        }
        body.push(Op::Fault(
            "native reflection property metadata was not retained; configure --reflection-roots"
                .into(),
        ));
        let f = &mut input.functions[index];
        f.impl_flags = 0;
        f.body = body;
        f.locals = locals;
        bindings.push(json!({"definition":methods[index],"name":f.name,"instructionMap":map,"boxedReceivers":boxed_receivers}));
    }
    retention["propertyBindings"] = json!(bindings);
    retention["accessorRoots"] = json!(roots.iter().map(|i| &methods[*i]).collect::<Vec<_>>());
    Ok(roots)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_internal_retention_faults_instead_of_granting_property_access() {
        for (name, setter, check) in [
            ("ReflectionPropertyGet", false, false),
            ("ReflectionPropertyGetCheck", false, true),
            ("ReflectionPropertySet", true, false),
            ("ReflectionPropertySetCheck", true, true),
        ] {
            let parameters = if setter {
                "RuntimeTypeHandle,Int32,System.Object,System.Object"
            } else {
                "RuntimeTypeHandle,Int32,System.Object"
            };
            let result = if check {
                "Int32"
            } else if setter {
                "Void"
            } else {
                "System.Object"
            };
            let extra = if setter { "ldloc receiver\n" } else { "" };
            let text = format!(
                ".module DeniedProperties\n.type class System.Object\n.end\n.function neoCLR.Runtime.{name}({parameters}) -> {result}\n.methodimpl InternalCall\n.end\n.entry Main\n.function Main() -> Int32\n.local System.Object receiver\nldloca receiver\ninitobj System.Object\nldtoken System.Object\nldc.i4 0\nldloc receiver\n{extra}call neoCLR.Runtime.{name}({parameters})\npop\nldc.i4 0\nret\n.end"
            );
            let source = neoclr::assemble(&text).unwrap();
            let mut projected = source.clone();
            let methods = source
                .functions
                .iter()
                .map(|f| f.definition.clone().unwrap())
                .collect::<Vec<_>>();
            let mut retention = json!({"types":[]});
            assert!(
                bind(&mut projected, &source, &[], &methods, &mut retention)
                    .unwrap()
                    .is_empty()
            );
            let program = neoclr::LoadedProgram::new(&projected).unwrap();
            let fault = program.run(neoclr::Limits::default()).unwrap_err();
            assert!(
                fault.to_string().contains("metadata was not retained"),
                "{name}: {fault}"
            );
            assert_eq!(retention["accessorRoots"], json!([]));
        }
    }
}
