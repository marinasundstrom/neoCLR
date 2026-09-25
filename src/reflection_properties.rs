//! Checked private property execution over loaded metadata, using ordinary calls.
use crate::{
    Fault, Module, Value,
    metadata::{Function, FunctionRef, Instruction as Op, Representation, Type},
};

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
enum Error {
    UnboundType = 1,
    UnsupportedShape = 2,
    AccessDenied = 3,
    InvalidProperty = 4,
    MissingAccessor = 5,
    InvalidReceiver = 6,
    InvalidValue = 7,
}

struct Accessor {
    owner: Type,
    value_type: Type,
    target: FunctionRef,
    no_result: bool,
}

fn supported_value(module: &Module, ty: &Type) -> bool {
    module.is_object_reference_type(ty)
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
}

fn value_matches(module: &Module, value: &Value, target: &Type) -> bool {
    match value {
        Value::NullObjectReference(_) => module.is_object_reference_type(target),
        Value::ObjectReference(object) => {
            let actual = object.concrete_type();
            actual == *target
                || (module.is_object_reference_type(target)
                    && (*target == Type::from_name("System.Object")
                        || module.reference_assignable(&actual, target)
                        || (crate::interfaces::interface_definition(module, target).is_ok()
                            && crate::interfaces::ensure_implementation(module, &actual, target)
                                .is_ok())))
        }
        _ => false,
    }
}

fn resolve(module: &Module, args: &[Value], setter: bool) -> Result<Accessor, Error> {
    if args.len() != if setter { 4 } else { 3 } {
        return Err(Error::InvalidProperty);
    }
    let owner = crate::reflection_execution::bound_type(module, &args[0])
        .map_err(|_| Error::UnboundType)?;
    let definition = module
        .type_definition(&owner)
        .ok_or(Error::UnsupportedShape)?;
    if !definition.is_reference_type
        || definition.representation != Representation::Record
        || !definition.generic_parameters.is_empty()
    {
        return Err(Error::UnsupportedShape);
    }
    let Value::Int32(index) = args[1] else {
        return Err(Error::InvalidProperty);
    };
    let property = usize::try_from(index)
        .ok()
        .and_then(|i| definition.properties.get(i))
        .ok_or(Error::InvalidProperty)?;
    if !property.instance
        || !property.parameters.is_empty()
        || !supported_value(module, &property.ty)
    {
        return Err(Error::UnsupportedShape);
    }
    let mut target = if setter {
        &property.setter
    } else {
        &property.getter
    }
    .clone()
    .ok_or(Error::MissingAccessor)?;
    let method = crate::vm::resolve(module, &target).map_err(|_| Error::InvalidProperty)?;
    if method.receiver_byref
        || !method.instance
        || method.is_internal_call()
        || method.pinvoke.is_some()
    {
        return Err(Error::UnsupportedShape);
    }
    if !crate::metadata_origin::reflection_public(module, &owner, &method) {
        return Err(Error::AccessDenied);
    }
    crate::access::check_call(module, None, &method).map_err(|_| Error::AccessDenied)?;
    crate::access::check_signature(module, None, &method).map_err(|_| Error::AccessDenied)?;
    let Value::ObjectReference(receiver) = &args[2] else {
        return Err(Error::InvalidReceiver);
    };
    if !module.reference_assignable(&receiver.concrete_type(), &owner) {
        return Err(Error::InvalidReceiver);
    }
    if setter && !value_matches(module, &args[3], &property.ty) {
        return Err(Error::InvalidValue);
    }
    target.definition = method.definition;
    Ok(Accessor {
        owner,
        value_type: property.ty.clone(),
        target,
        no_result: method.no_result,
    })
}

pub(crate) fn check(module: &Module, args: &[Value], setter: bool) -> i32 {
    resolve(module, args, setter).map_or_else(|error| error as i32, |_| 0)
}

pub(crate) fn adapter(
    module: &Module,
    service: &Function,
    args: &[Value],
    setter: bool,
) -> Result<Function, Fault> {
    let accessor = resolve(module, args, setter)
        .map_err(|error| Fault::new(format!("reflection property access rejected: {error:?}")))?;
    let reference_value = module.is_object_reference_type(&accessor.value_type);
    let mut body = vec![Op::Arg(2), Op::CastClass(accessor.owner)];
    if setter {
        body.push(Op::Arg(3));
        body.push(if reference_value {
            Op::CastClass(accessor.value_type.clone())
        } else {
            Op::UnboxAny(accessor.value_type.clone())
        });
    }
    body.push(Op::CallVirtual(accessor.target));
    if setter {
        if accessor.no_result {
            body.push(Op::Void);
        }
    } else {
        body.push(if reference_value {
            Op::CastClass(Type::from_name("System.Object"))
        } else {
            Op::BoxValue(accessor.value_type)
        });
    }
    body.push(Op::Return);
    let mut adapter = service.clone();
    adapter.impl_flags = 0;
    adapter.body = body;
    Ok(adapter)
}
