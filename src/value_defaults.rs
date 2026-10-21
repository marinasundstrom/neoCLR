//! Runtime-owned fieldwise defaults. These bodies are execution helpers, not
//! declared source members or synthesized EquatableTo/operator contracts.
use crate::{
    Fault, Module,
    metadata::{Function, FunctionRef, Instruction as Op, Representation, Type},
};

fn integral_width(ty: &Type) -> Option<bool> {
    match ty {
        Type::SByte | Type::Byte | Type::Int16 | Type::UInt16 | Type::Int32 | Type::UInt32 => {
            Some(false)
        }
        Type::Int64 | Type::UInt64 | Type::IntPtr | Type::UIntPtr => Some(true),
        _ => None,
    }
}

pub fn body(module: &Module, owner: &Type, contract: &Function) -> Result<Option<Function>, Fault> {
    let Some(definition) = module.type_definition(owner) else {
        return Ok(None);
    };
    let equality = contract.name.rsplit('.').next() == Some("Equals")
        && contract.parameters == [Type::Named("System.Object".into())]
        && contract.returns == Type::Boolean;
    let hashing = contract.name.rsplit('.').next() == Some("GetHashCode")
        && contract.parameters.is_empty()
        && contract.returns == Type::Int32;
    if definition.is_reference_type
        || definition.memory_view.is_some()
        || definition.primitive_reference
        || crate::memory_view_contract::contains_borrowed(module, owner)
        || Type::from_name(&definition.name).is_primitive()
        || definition.enum_info.is_some()
        || definition.representation != Representation::Record
        || !(equality || hashing)
    {
        return Ok(None);
    }
    let object = Type::Named("System.Object".into());
    let slot = if equality { "Equals" } else { "GetHashCode" };
    let call = FunctionRef {
        definition: contract.definition.clone(),
        name: contract.name.clone(),
        owner: contract.owner.clone(),
        instance: true,
        generic_arguments: vec![],
        parameters: contract.parameters.clone(),
    };
    let fields = &definition.fields;
    let mut method = contract.clone();
    method.definition = None;
    method.origin = None;
    method.sequence_points.clear();
    method.custom_attributes.clear();
    let mut discriminator = 0;
    loop {
        method.name = format!("{}.$ValueDefault{discriminator}_{slot}", definition.name);
        if !module.functions.iter().any(|f| f.name == method.name) {
            break;
        }
        discriminator += 1;
    }
    method.owner = Some(owner.clone());
    method.receiver_byref = true;
    method.receiver_readonly = false; // Private body never writes through its receiver.
    method.is_override = false;
    method.is_virtual = false;
    method.value_default_slot = Some(slot.into());
    method.impl_flags = 0;
    method.local_names.clear();
    let mut code = vec![];
    if equality {
        method.locals = vec![owner.clone(), object.clone(), object.clone()];
        code.extend([
            Op::Arg(1),
            Op::IsInstance(owner.clone()),
            Op::ReferenceIsNull,
        ]);
        let mut failure = vec![code.len()];
        code.push(Op::BranchTrue(0));
        code.extend([Op::Arg(1), Op::UnboxAny(owner.clone()), Op::Store(0)]);
        for (index, field) in fields.iter().enumerate() {
            let ty = field
                .ty
                .substitute_type_parameters(owner.generic_arguments())?;
            if integral_width(&ty).is_some() || ty == Type::Boolean || matches!(ty, Type::Ptr(_)) {
                code.extend([
                    Op::Arg(0),
                    Op::Field(index),
                    Op::Load(0),
                    Op::Field(index),
                    Op::Equal,
                ]);
                failure.push(code.len());
                code.push(Op::BranchFalse(0));
                continue;
            }
            code.extend([
                Op::Arg(0),
                Op::Field(index),
                Op::BoxValue(ty.clone()),
                Op::Store(1),
                Op::Load(0),
                Op::Field(index),
                Op::BoxValue(ty),
                Op::Store(2),
                Op::Load(1),
                Op::ReferenceIsNull,
            ]);
            let nonnull = code.len();
            code.push(Op::BranchFalse(0));
            code.extend([Op::Load(2), Op::ReferenceIsNull]);
            failure.push(code.len());
            code.push(Op::BranchFalse(0));
            let next = code.len();
            code.push(Op::Branch(0));
            code[nonnull] = Op::BranchFalse(code.len());
            code.extend([Op::Load(1), Op::Load(2), Op::CallVirtual(call.clone())]);
            failure.push(code.len());
            code.push(Op::BranchFalse(0));
            code[next] = Op::Branch(code.len());
        }
        code.extend([Op::Bool(true), Op::Return]);
        let end = code.len();
        for index in failure {
            code[index] = match code[index] {
                Op::BranchTrue(_) => Op::BranchTrue(end),
                _ => Op::BranchFalse(end),
            };
        }
        code.extend([Op::Bool(false), Op::Return]);
    } else {
        method.locals = vec![object, Type::Int32];
        code.extend([Op::Int(17), Op::Store(1)]);
        for (index, field) in fields.iter().enumerate() {
            let ty = field
                .ty
                .substitute_type_parameters(owner.generic_arguments())?;
            if matches!(ty, Type::Ptr(_)) {
                // Hash address bits without boxing a pointer or reading its target.
                code.extend([
                    Op::Load(1),
                    Op::Int(31),
                    Op::Mul,
                    Op::Arg(0),
                    Op::Field(index),
                    Op::ConvertNativeUInt,
                    Op::ConvertUInt64,
                    Op::Arg(0),
                    Op::Field(index),
                    Op::ConvertNativeUInt,
                    Op::ConvertUInt64,
                    Op::Int(32),
                    Op::ShiftRight,
                    Op::BitXor,
                    Op::ConvertInt32,
                    Op::BitXor,
                    Op::Store(1),
                ]);
                continue;
            }
            if let Some(wide) = integral_width(&ty) {
                code.extend([
                    Op::Load(1),
                    Op::Int(31),
                    Op::Mul,
                    Op::Arg(0),
                    Op::Field(index),
                ]);
                if wide {
                    code.push(Op::ConvertInt64);
                    code.extend([
                        Op::Arg(0),
                        Op::Field(index),
                        Op::ConvertInt64,
                        Op::Int(32),
                        Op::ShiftRight,
                        Op::BitXor,
                        Op::ConvertInt32,
                    ]);
                }
                if !wide {
                    code.push(Op::ConvertInt32);
                }
                code.extend([Op::BitXor, Op::Store(1)]);
                continue;
            }
            if ty == Type::Boolean {
                code.extend([
                    Op::Load(1),
                    Op::Int(31),
                    Op::Mul,
                    Op::Store(1),
                    Op::Arg(0),
                    Op::Field(index),
                ]);
                let next = code.len();
                code.push(Op::BranchFalse(0));
                code.extend([Op::Load(1), Op::Int(1), Op::BitXor, Op::Store(1)]);
                code[next] = Op::BranchFalse(code.len());
                continue;
            }
            code.extend([
                Op::Arg(0),
                Op::Field(index),
                Op::BoxValue(ty),
                Op::Store(0),
                Op::Load(1),
                Op::Int(31),
                Op::Mul,
                Op::Store(1),
                Op::Load(0),
                Op::ReferenceIsNull,
            ]);
            let next = code.len();
            code.push(Op::BranchTrue(0));
            code.extend([
                Op::Load(1),
                Op::Load(0),
                Op::CallVirtual(call.clone()),
                Op::BitXor,
                Op::Store(1),
            ]);
            code[next] = Op::BranchTrue(code.len());
        }
        code.extend([Op::Load(1), Op::Return]);
    }
    method.body = code;
    Ok(Some(method))
}

/// Add helpers only to a verified private execution projection. Callers retain
/// the original module for reflection and source/member identity.
pub fn materialize(module: &mut Module) -> Result<usize, Fault> {
    let contracts: Vec<_> = module
        .functions
        .iter()
        .filter(|f| {
            f.owner == Some(Type::Named("System.Object".into()))
                && f.instance
                && f.is_virtual
                && !f.is_abstract
                && matches!(f.name.rsplit('.').next(), Some("Equals" | "GetHashCode"))
        })
        .cloned()
        .collect();
    let mut added = vec![];
    for ty in &module.types {
        let owner = if ty.generic_parameters.is_empty() {
            Type::Named(ty.name.clone())
        } else {
            Type::Constructed {
                definition: ty.name.clone(),
                arguments: (0..ty.generic_parameters.len())
                    .map(|i| Type::TypeParameter(i as u16))
                    .collect(),
            }
        };
        for contract in &contracts {
            if module.functions.iter().any(|f| {
                f.owner.as_ref().and_then(Type::definition_name) == owner.definition_name()
                    && f.instance
                    && (implements_contract(f, contract)
                        || (f.is_override
                            && f.name.rsplit('.').next() == contract.name.rsplit('.').next()))
                    && f.parameters == contract.parameters
            }) {
                continue;
            }
            if let Some(method) = body(module, &owner, contract)? {
                added.push(method);
            }
        }
    }
    let count = added.len();
    module.functions.extend(added);
    Ok(count)
}

/// Recognize only runtime-created helpers, never similarly named source members.
pub fn implements_contract(method: &Function, contract: &Function) -> bool {
    method.value_default_slot.as_deref().is_some_and(|slot| {
        Some(slot) == contract.name.rsplit('.').next()
            && method.parameters == contract.parameters
            && method.returns == contract.returns
    })
}
