//! Exact constructor invocation uses explicit metadata and executable retention.
use neoclr::metadata::{FunctionRef, Instruction as Op, MemberId, Type, TypeDefId, Visibility};
use serde_json::Value;
type Error = Box<dyn std::error::Error>;

fn reject(body: &mut Vec<Op>, check: bool, status: i32) {
    if check {
        body.extend([Op::Int(status), Op::Return]);
    } else {
        body.push(Op::Fault(format!(
            "reflection constructor invocation rejected: {status}"
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
    methods: &[MemberId],
    retention: &Value,
) -> Result<Vec<usize>, Error> {
    let object = Type::from_name("System.Object");
    let services: Vec<_> = input
        .functions
        .iter()
        .enumerate()
        .filter(|(_, f)| {
            matches!(
                f.name.as_str(),
                "neoCLR.Runtime.ReflectionConstructorInvoke"
                    | "neoCLR.Runtime.ReflectionConstructorInvokeCheck"
            )
        })
        .map(|(i, _)| i)
        .collect();
    let mut roots = vec![];
    for index in services {
        let f = &input.functions[index];
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
            return Err(
                "constructor invocation requires exact reserved InternalCall contract".into(),
            );
        }
        let mut body = vec![];
        for row in retention["types"].as_array().into_iter().flatten() {
            if row["constructors"] != true {
                continue;
            }
            let definition: TypeDefId = serde_json::from_value(row["definition"].clone())?;
            let ti = definitions
                .iter()
                .position(|d| *d == definition)
                .ok_or("missing constructor root")?;
            let d = &source.types[ti];
            let owners: Vec<Type> = if row.get("constructorOwners").is_some() {
                serde_json::from_value(row["constructorOwners"].clone())?
            } else {
                vec![Type::from_name(&d.name)]
            };
            for owner in owners {
                let owner_branch = body.len() + 3;
                body.extend([
                    Op::Arg(0),
                    Op::LoadTypeToken(owner.clone()),
                    Op::Equal,
                    Op::BranchFalse(0),
                ]);
                for (fi, constructor) in source.functions.iter().enumerate().filter(|(_, f)| {
                    f.owner.as_ref().and_then(Type::definition_name) == Some(d.name.as_str())
                        && f.instance
                        && f.name.ends_with("..ctor")
                }) {
                    let mut constructor = constructor.clone();
                    if let Type::Constructed { arguments, .. } = &owner {
                        constructor.parameters = constructor
                            .parameters
                            .iter()
                            .map(|t| {
                                t.substitute_type_parameters(arguments)
                                    .map_err(|e| e.to_string())
                            })
                            .collect::<Result<_, _>>()?;
                        constructor.owner = Some(owner.clone());
                    }
                    let next = body.len() + 3;
                    body.extend([
                        Op::Arg(1),
                        Op::Int(i32::try_from(methods[fi].index)?),
                        Op::Equal,
                        Op::BranchFalse(0),
                    ]);
                    let public = d.visibility == Visibility::Public
                        && d.origin
                            .as_ref()
                            .is_none_or(|o| o.publicly_visible != Some(false))
                        && constructor.visibility == Visibility::Public
                        && !constructor.origin.as_ref().is_some_and(|o| {
                            o.member_access != Some(neoclr::metadata_origin::SourceAccess::Public)
                        });
                    let supported = !d.is_abstract
                        && constructor.generic_parameters.is_empty()
                        && !constructor.is_internal_call()
                        && constructor.pinvoke.is_none()
                        && constructor.no_result
                        && constructor.returns == Type::Void
                        && constructor.parameters.iter().all(|ty| {
                            source.is_object_reference_type(ty)
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
                                )
                        });
                    if !supported {
                        reject(&mut body, check, 2);
                    } else if !public || row["construct"] != true {
                        reject(&mut body, check, 3);
                    } else {
                        body.extend([Op::Arg(2), Op::ReferenceIsNull]);
                        require(&mut body, check, 6);
                        body.extend([Op::Arg(3), Op::ReferenceIsNull, Op::Bool(false), Op::Equal]);
                        require(&mut body, check, 8);
                        body.extend([
                            Op::Arg(3),
                            Op::ArrayLength,
                            Op::ConvertInt32,
                            Op::Int(i32::try_from(constructor.parameters.len())?),
                            Op::Equal,
                        ]);
                        require(&mut body, check, 8);
                        for (pi, ty) in constructor.parameters.iter().enumerate() {
                            let reference = source.is_object_reference_type(ty);
                            let skip_null = if reference {
                                let branch = body.len() + 4;
                                body.extend([
                                    Op::Arg(3),
                                    Op::Int(i32::try_from(pi)?),
                                    Op::ArrayElement(object.clone()),
                                    Op::ReferenceIsNull,
                                    Op::BranchTrue(0),
                                ]);
                                Some(branch)
                            } else {
                                None
                            };
                            body.extend([
                                Op::Arg(3),
                                Op::Int(i32::try_from(pi)?),
                                Op::ArrayElement(object.clone()),
                                Op::IsInstance(ty.clone()),
                                Op::ReferenceIsNull,
                                Op::Bool(false),
                                Op::Equal,
                            ]);
                            require(&mut body, check, 8);
                            if let Some(branch) = skip_null {
                                body[branch] = Op::BranchTrue(body.len());
                            }
                        }
                        if check {
                            body.push(Op::Int(0));
                        } else {
                            for (pi, ty) in constructor.parameters.iter().enumerate() {
                                body.extend([
                                    Op::Arg(3),
                                    Op::Int(i32::try_from(pi)?),
                                    Op::ArrayElement(object.clone()),
                                    if source.is_object_reference_type(ty) {
                                        Op::CastClass(ty.clone())
                                    } else {
                                        Op::UnboxAny(ty.clone())
                                    },
                                ]);
                            }
                            let target = &input.functions[fi];
                            body.push(Op::Construct(FunctionRef {
                                definition: target.definition.clone(),
                                name: target.name.clone(),
                                owner: Some(owner.clone()),
                                instance: true,
                                generic_arguments: vec![],
                                parameters: constructor.parameters.clone(),
                            }));
                            body.push(if d.is_reference_type {
                                Op::CastClass(object.clone())
                            } else {
                                Op::BoxValue(owner.clone())
                            });
                            if d.generic_parameters.is_empty() && !roots.contains(&fi) {
                                roots.push(fi);
                            }
                        }
                        body.push(Op::Return);
                    }
                    body[next] = Op::BranchFalse(body.len());
                }
                reject(&mut body, check, 4);
                body[owner_branch] = Op::BranchFalse(body.len());
            }
        }
        reject(&mut body, check, 1);
        input.functions[index].impl_flags = 0;
        input.functions[index].body = body;
    }
    Ok(roots)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn execute(
        check: bool,
        retained: bool,
        allowed: bool,
        argument_count: i32,
        public: bool,
    ) -> (neoclr::Value, usize) {
        let name = if check {
            "ReflectionConstructorInvokeCheck"
        } else {
            "ReflectionConstructorInvoke"
        };
        let result = if check { "Int32" } else { "System.Object" };
        let mut source = neoclr::assemble(&format!(".module ConstructorTest\n.type class System.Object\n.method instance .ctor() -> void\nret\n.end\n.end\n.type class Item\n.extends System.Object\n.field Value Int32\n.method instance .ctor(Int32 value) -> void\nldarg 0\ncall instance System.Object::.ctor()\nldarg 0\nldarg value\nstfld 0\nret\n.end\n.end\n.function neoCLR.Runtime.{name}(RuntimeTypeHandle,Int32,System.Object,arrayref<System.Object>) -> {result}\n.methodimpl InternalCall\n.end\n.entry Main\n.function Main() -> Int32\nldc.i4 0\nret\n.end")).unwrap();
        let object = Type::from_name("System.Object");
        let constructor = source
            .functions
            .iter()
            .position(|f| f.name == "Item..ctor")
            .unwrap();
        source.functions[constructor].visibility = if public {
            Visibility::Public
        } else {
            Visibility::Private
        };
        let service = source
            .functions
            .iter()
            .position(|f| f.is_internal_call())
            .unwrap();
        let main = source
            .functions
            .iter()
            .position(|f| f.name == "Main")
            .unwrap();
        let target = &source.functions[service];
        let call = FunctionRef {
            definition: target.definition.clone(),
            name: target.name.clone(),
            owner: None,
            instance: false,
            parameters: target.parameters.clone(),
            generic_arguments: vec![],
        };
        let mut body = vec![
            Op::LocalAddress(0),
            Op::InitializeObject(object.clone()),
            Op::Int(argument_count),
            Op::NewArray(object.clone()),
            Op::Store(1),
        ];
        if argument_count == 1 {
            body.extend([
                Op::Load(1),
                Op::Int(0),
                Op::Int(42),
                Op::BoxValue(Type::Int32),
                Op::StoreArrayElement(object.clone()),
            ]);
        }
        body.extend([
            Op::LoadTypeToken(Type::from_name("Item")),
            Op::Int(
                source.functions[constructor]
                    .definition
                    .as_ref()
                    .unwrap()
                    .index as i32,
            ),
            Op::Load(0),
            Op::Load(1),
            Op::Call(call),
        ]);
        if !check {
            body.extend([Op::CastClass(Type::from_name("Item")), Op::Field(0)]);
        }
        body.push(Op::Return);
        source.functions[main].locals = vec![object.clone(), Type::ArrayRef(Box::new(object))];
        source.functions[main].body = body;
        let definitions: Vec<_> = source
            .types
            .iter()
            .map(|t| t.definition.clone().unwrap())
            .collect();
        let methods: Vec<_> = source
            .functions
            .iter()
            .map(|f| f.definition.clone().unwrap())
            .collect();
        let roots = if retained {
            json!([{"definition":definitions[1],"constructors":true,"construct":allowed}])
        } else {
            json!([])
        };
        let mut projected = source.clone();
        let selected = bind(
            &mut projected,
            &source,
            &definitions,
            &methods,
            &json!({"types":roots}),
        )
        .unwrap();
        let program = neoclr::LoadedProgram::new(&projected).unwrap();
        program.verify().unwrap();
        (
            program.run(neoclr::Limits::default()).unwrap().value,
            selected.len(),
        )
    }

    #[test]
    fn exact_constructor_invocation_preserves_arguments_and_result() {
        assert_eq!(
            execute(false, true, true, 1, true),
            (neoclr::Value::Int32(42), 1)
        );
        assert_eq!(
            execute(true, true, true, 1, true),
            (neoclr::Value::Int32(0), 0)
        );
    }

    #[test]
    fn checks_reject_unretained_private_and_wrong_arity_without_execution_roots() {
        for (retained, allowed, count, public, status) in [
            (false, false, 1, true, 1),
            (true, false, 1, true, 3),
            (true, true, 1, false, 3),
            (true, true, 0, true, 8),
        ] {
            assert_eq!(
                execute(true, retained, allowed, count, public),
                (neoclr::Value::Int32(status), 0)
            );
        }
    }
}
