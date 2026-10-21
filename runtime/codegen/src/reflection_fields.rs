//! Explicitly retained field access lowered to ordinary verified field operations.
use neoclr::metadata::{Instruction as Op, Representation, Type, TypeDefId, Visibility};
use serde_json::Value;
type Error = Box<dyn std::error::Error>;
fn reject(body: &mut Vec<Op>, check: bool, status: i32) {
    if check {
        body.extend([Op::Int(status), Op::Return]);
    } else {
        body.push(Op::Fault(format!(
            "reflection field access rejected: {status}"
        )));
    }
}
fn require(body: &mut Vec<Op>, check: bool, status: i32) {
    let branch = body.len();
    body.push(Op::BranchTrue(0));
    reject(body, check, status);
    body[branch] = Op::BranchTrue(body.len());
}
pub fn bind(
    input: &mut neoclr::Module,
    source: &neoclr::Module,
    definitions: &[TypeDefId],
    retention: &Value,
) -> Result<(), Error> {
    let object = Type::from_name("System.Object");
    let services: Vec<_> = input
        .functions
        .iter()
        .enumerate()
        .filter(|(_, f)| {
            matches!(
                f.name.as_str(),
                "neoCLR.Runtime.ReflectionFieldGet"
                    | "neoCLR.Runtime.ReflectionFieldGetCheck"
                    | "neoCLR.Runtime.ReflectionFieldSet"
                    | "neoCLR.Runtime.ReflectionFieldSetCheck"
            )
        })
        .map(|(i, _)| i)
        .collect();
    for index in services {
        let f = &input.functions[index];
        let setter = f.name.contains("FieldSet");
        let check = f.name.ends_with("Check");
        if f.owner.is_some()
            || f.instance
            || f.parameters
                != [
                    Type::RuntimeTypeHandle,
                    Type::Int32,
                    object.clone(),
                    Type::ArrayRef(Box::new(object.clone())),
                ]
            || f.returns != if check { Type::Int32 } else { object.clone() }
            || f.no_result
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
            return Err("field reflection requires exact reserved InternalCall contract".into());
        }
        let mut body = vec![Op::LocalAddress(0), Op::InitializeObject(object.clone())];
        for row in retention["types"].as_array().into_iter().flatten() {
            if row["fields"] != true {
                continue;
            }
            let definition: TypeDefId = serde_json::from_value(row["definition"].clone())?;
            let ti = definitions
                .iter()
                .position(|d| *d == definition)
                .ok_or("missing field root")?;
            let d = &source.types[ti];
            let owner = Type::Named(d.name.clone());
            let next_owner = body.len() + 3;
            body.extend([
                Op::Arg(0),
                Op::LoadTypeToken(owner.clone()),
                Op::Equal,
                Op::BranchFalse(0),
            ]);
            if d.representation != Representation::Record || !d.generic_parameters.is_empty() {
                reject(&mut body, check, 2);
            } else {
                let mut offset = 0;
                let mut parent = d.base.as_ref();
                while let Some(ty) = parent {
                    let base = source.type_definition(ty).ok_or("missing field base")?;
                    offset += base.fields.len();
                    parent = base.base.as_ref();
                }
                let fields =
                    d.fields
                        .iter()
                        .enumerate()
                        .map(|(i, f)| {
                            (
                                f.ty.clone(),
                                f.visibility,
                                false,
                                d.origin
                                    .as_ref()
                                    .is_some_and(|o| o.field_readonly.get(i) != Some(&false)),
                                None,
                                i + offset,
                            )
                        })
                        .chain(
                            d.static_fields.iter().enumerate().map(|(i, f)| {
                                (f.ty.clone(), f.visibility, true, f.readonly, None, i)
                            }),
                        )
                        .chain(
                            d.literal_fields.iter().map(|f| {
                                (f.value.ty(), f.visibility, true, true, Some(&f.value), 0)
                            }),
                        );
                for (ordinal, (ty, access, is_static, readonly, constant, slot)) in
                    fields.enumerate()
                {
                    let next = body.len() + 3;
                    body.extend([
                        Op::Arg(1),
                        Op::Int(ordinal as i32),
                        Op::Equal,
                        Op::BranchFalse(0),
                    ]);
                    let reference = source.is_object_reference_type(&ty);
                    let allowed_type = reference
                        || matches!(
                            ty,
                            Type::Boolean
                                | Type::Char
                                | Type::SByte
                                | Type::Byte
                                | Type::Int16
                                | Type::UInt16
                                | Type::Int32
                                | Type::UInt32
                                | Type::Int64
                                | Type::UInt64
                                | Type::IntPtr
                                | Type::UIntPtr
                                | Type::Single
                                | Type::Double
                        );
                    let allowed = row[if setter {
                        "fieldSetters"
                    } else {
                        "fieldGetters"
                    }] == true;
                    if !allowed
                        || (!is_static
                            && d.origin.as_ref().is_some_and(|o| {
                                o.field_access.get(ordinal)
                                    != Some(&neoclr::metadata_origin::SourceAccess::Public)
                            }))
                        || access != Visibility::Public
                        || d.visibility != Visibility::Public
                        || d.origin
                            .as_ref()
                            .is_some_and(|o| o.publicly_visible != Some(true))
                        || setter && readonly
                    {
                        reject(&mut body, check, 3);
                    } else if !allowed_type || !is_static && !d.is_reference_type {
                        reject(&mut body, check, 2);
                    } else {
                        if !is_static {
                            body.extend([
                                Op::Arg(2),
                                Op::IsInstance(owner.clone()),
                                Op::ReferenceIsNull,
                                Op::Bool(false),
                                Op::Equal,
                            ]);
                            require(&mut body, check, 6);
                        }
                        body.extend([Op::Arg(3), Op::ReferenceIsNull, Op::Bool(false), Op::Equal]);
                        require(&mut body, check, 8);
                        body.extend([
                            Op::Arg(3),
                            Op::ArrayLength,
                            Op::ConvertInt32,
                            Op::Int(if setter { 1 } else { 0 }),
                            Op::Equal,
                        ]);
                        require(&mut body, check, 8);
                        if setter {
                            let null_skip = if reference {
                                let at = body.len() + 4;
                                body.extend([
                                    Op::Arg(3),
                                    Op::Int(0),
                                    Op::ArrayElement(object.clone()),
                                    Op::ReferenceIsNull,
                                    Op::BranchTrue(0),
                                ]);
                                Some(at)
                            } else {
                                None
                            };
                            body.extend([
                                Op::Arg(3),
                                Op::Int(0),
                                Op::ArrayElement(object.clone()),
                                Op::IsInstance(ty.clone()),
                                Op::ReferenceIsNull,
                                Op::Bool(false),
                                Op::Equal,
                            ]);
                            require(&mut body, check, 7);
                            if let Some(at) = null_skip {
                                body[at] = Op::BranchTrue(body.len());
                            }
                        }
                        if check {
                            body.push(Op::Int(0));
                        } else {
                            if !is_static {
                                body.extend([Op::Arg(2), Op::CastClass(owner.clone())]);
                            }
                            if setter {
                                body.extend([
                                    Op::Arg(3),
                                    Op::Int(0),
                                    Op::ArrayElement(object.clone()),
                                    if reference {
                                        Op::CastClass(ty.clone())
                                    } else {
                                        Op::UnboxAny(ty.clone())
                                    },
                                ]);
                                body.push(if is_static {
                                    Op::SetStaticField {
                                        owner: owner.clone(),
                                        index: slot,
                                    }
                                } else {
                                    Op::SetField(slot)
                                });
                                body.push(Op::Load(0));
                            } else {
                                body.push(if let Some(value) = constant {
                                    value.instruction().map_err(|e| e.to_string())?
                                } else if is_static {
                                    Op::StaticField {
                                        owner: owner.clone(),
                                        index: slot,
                                    }
                                } else {
                                    Op::Field(slot)
                                });
                                body.push(if reference {
                                    Op::CastClass(object.clone())
                                } else {
                                    Op::BoxValue(ty.clone())
                                });
                            }
                        }
                        body.push(Op::Return);
                    }
                    body[next] = Op::BranchFalse(body.len());
                }
                reject(&mut body, check, 4);
            }
            body[next_owner] = Op::BranchFalse(body.len());
        }
        reject(&mut body, check, 1);
        let f = &mut input.functions[index];
        f.impl_flags = 0;
        f.locals = vec![object.clone()];
        f.body = body;
    }
    Ok(())
}
