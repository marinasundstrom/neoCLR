//! Bounded dynamic calls and fields, executed through ordinary interpreter frames.
use crate::reflection_properties::{supported_value, value_matches};
use crate::{
    Fault, Module, Value,
    metadata::{Function, FunctionRef, Instruction as Op, Representation, Type, Visibility},
};

#[derive(Clone, Copy, Debug)]
#[repr(i32)]
enum Error {
    Unbound = 1,
    Unsupported = 2,
    Access = 3,
    Missing = 4,
    Receiver = 6,
    Value = 7,
    Arguments = 8,
    Ambiguous = 9,
}
struct Plan {
    body: Vec<Op>,
}
fn object() -> Type {
    Type::from_name("System.Object")
}
fn values(value: &Value) -> Result<Vec<Value>, Error> {
    let Value::ObjectReference(array) = value else {
        return Err(Error::Arguments);
    };
    let Value::Array { element, elements } =
        array.reference.read().map_err(|_| Error::Arguments)?
    else {
        return Err(Error::Arguments);
    };
    if element != object() {
        return Err(Error::Arguments);
    }
    Ok(elements)
}
fn convert(module: &Module, ty: &Type) -> Op {
    if module.is_object_reference_type(ty) {
        Op::CastClass(ty.clone())
    } else {
        Op::UnboxAny(ty.clone())
    }
}
fn target(f: &Function) -> FunctionRef {
    FunctionRef {
        definition: f.definition.clone(),
        name: f.name.clone(),
        owner: f.owner.clone(),
        instance: f.instance,
        generic_arguments: vec![],
        parameters: f.parameters.clone(),
    }
}
fn shape(module: &Module, f: &Function, constructor: bool) -> bool {
    f.generic_parameters.is_empty()
        && !f.receiver_byref
        && !f.is_internal_call()
        && f.pinvoke.is_none()
        && !crate::interfaces::is_bodyless(module, f)
        && f.parameters.iter().all(|t| supported_value(module, t))
        && (constructor
            || f.no_result
            || f.returns == Type::Void
            || supported_value(module, &f.returns))
}
fn matches(module: &Module, f: &Function, args: &[Value]) -> bool {
    f.parameters.len() == args.len()
        && f.parameters
            .iter()
            .zip(args)
            .all(|(t, v)| value_matches(module, v, t))
}
// kind: 0 selects a constructor, 1 invokes a method, 2 reads a field, 3 writes a field.
fn plan(module: &Module, args: &[Value], kind: u8) -> Result<Plan, Error> {
    let owner =
        crate::reflection_execution::bound_type(module, &args[0]).map_err(|_| Error::Unbound)?;
    let d = module.type_definition(&owner).ok_or(Error::Unsupported)?;
    if !d.is_reference_type
        || d.representation != Representation::Record
        || !d.generic_parameters.is_empty()
    {
        return Err(Error::Unsupported);
    }
    if d.origin
        .as_ref()
        .is_some_and(|o| o.publicly_visible != Some(true))
    {
        return Err(Error::Access);
    }
    let Value::Int32(index) = args[1] else {
        return Err(Error::Missing);
    };
    let arguments = values(&args[3])?;
    let mut body = vec![Op::LocalAddress(0), Op::InitializeObject(object())];
    if kind >= 2 {
        let index = usize::try_from(index).map_err(|_| Error::Missing)?;
        let field = d.fields.get(index).ok_or(Error::Missing)?;
        if !supported_value(module, &field.ty) {
            return Err(Error::Unsupported);
        }
        if field.visibility != Visibility::Public
            || d.origin.as_ref().is_some_and(|o| {
                o.field_access.get(index) != Some(&crate::metadata_origin::SourceAccess::Public)
            })
        {
            return Err(Error::Access);
        }
        if kind == 3
            && (d
                .origin
                .as_ref()
                .is_some_and(|o| o.field_readonly.get(index) != Some(&false)))
        {
            return Err(Error::Access);
        }
        let Value::ObjectReference(receiver) = &args[2] else {
            return Err(Error::Receiver);
        };
        if !module.reference_assignable(&receiver.concrete_type(), &owner) {
            return Err(Error::Receiver);
        }
        if arguments.len() != if kind == 3 { 1 } else { 0 } {
            return Err(Error::Arguments);
        }
        if kind == 3 && !value_matches(module, &arguments[0], &field.ty) {
            return Err(Error::Value);
        }
        let offset = crate::inheritance::fields(module, &owner)
            .map_err(|_| Error::Unsupported)?
            .len()
            - d.fields.len()
            + index;
        if d.visibility != Visibility::Public {
            return Err(Error::Access);
        }
        body.extend([Op::Arg(2), Op::CastClass(owner)]);
        if kind == 3 {
            body.extend([
                Op::Arg(3),
                Op::Int(0),
                Op::ArrayElement(object()),
                convert(module, &field.ty),
                Op::SetField(offset),
                Op::Load(0),
            ]);
        } else {
            body.push(Op::Field(offset));
            body.push(if module.is_object_reference_type(&field.ty) {
                Op::CastClass(object())
            } else {
                Op::BoxValue(field.ty.clone())
            });
        }
    } else {
        let constructor = kind == 0;
        let f = if constructor {
            if d.is_abstract || !module.reference_assignable(&owner, &object()) {
                return Err(Error::Unsupported);
            }
            let candidates: Vec<_> = module
                .functions
                .iter()
                .filter(|f| {
                    f.owner.as_ref() == Some(&owner)
                        && f.name.ends_with("..ctor")
                        && f.instance
                        && shape(module, f, true)
                        && matches(module, f, &arguments)
                        && crate::metadata_origin::reflection_public(module, &owner, f)
                        && crate::access::check_call(module, None, f).is_ok()
                })
                .collect();
            if candidates.len() > 1 {
                return Err(Error::Ambiguous);
            }
            *candidates.first().ok_or(Error::Missing)?
        } else {
            module
                .functions
                .iter()
                .find(|f| {
                    f.owner.as_ref() == Some(&owner)
                        && f.definition
                            .as_ref()
                            .is_some_and(|id| id.index == index as u32)
                        && !f.name.ends_with("..ctor")
                })
                .ok_or(Error::Missing)?
        };
        if !shape(module, f, constructor) {
            return Err(Error::Unsupported);
        }
        if !crate::metadata_origin::reflection_public(module, &owner, f) {
            return Err(Error::Access);
        }
        crate::access::check_call(module, None, f).map_err(|_| Error::Access)?;
        crate::access::check_signature(module, None, f).map_err(|_| Error::Access)?;
        if !matches(module, f, &arguments) {
            return Err(Error::Arguments);
        }
        if !constructor && f.instance {
            let Value::ObjectReference(receiver) = &args[2] else {
                return Err(Error::Receiver);
            };
            if !module.reference_assignable(&receiver.concrete_type(), &owner) {
                return Err(Error::Receiver);
            }
            body.extend([Op::Arg(2), Op::CastClass(owner.clone())]);
        } else if !matches!(args[2], Value::NullObjectReference(_)) {
            return Err(Error::Receiver);
        }
        for (i, ty) in f.parameters.iter().enumerate() {
            body.extend([
                Op::Arg(3),
                Op::Int(i as i32),
                Op::ArrayElement(object()),
                convert(module, ty),
            ]);
        }
        body.push(if constructor {
            Op::Construct(target(f))
        } else if f.instance {
            Op::CallVirtual(target(f))
        } else {
            Op::Call(target(f))
        });
        if constructor {
            body.push(Op::CastClass(object()));
        } else if f.no_result {
            body.push(Op::Load(0));
        } else if f.returns == Type::Void {
            body.extend([Op::Pop, Op::Load(0)]);
        } else {
            body.push(if module.is_object_reference_type(&f.returns) {
                Op::CastClass(object())
            } else {
                Op::BoxValue(f.returns.clone())
            });
        }
    }
    body.push(Op::Return);
    Ok(Plan { body })
}
pub(crate) fn check(module: &Module, args: &[Value], kind: u8) -> i32 {
    plan(module, args, kind).map_or_else(|e| e as i32, |_| 0)
}
pub(crate) fn adapter(
    module: &Module,
    service: &Function,
    args: &[Value],
    kind: u8,
) -> Result<Function, Fault> {
    let plan = plan(module, args, kind)
        .map_err(|e| Fault::new(format!("reflection member execution rejected: {e:?}")))?;
    let mut f = service.clone();
    f.impl_flags = 0;
    f.locals = vec![object()];
    f.body = plan.body;
    Ok(f)
}
pub(crate) fn assignable(module: &Module, args: &[Value]) -> bool {
    let Ok(source) = crate::reflection_execution::bound_type(module, &args[0]) else {
        return false;
    };
    let Ok(target) = crate::reflection_execution::bound_type(module, &args[1]) else {
        return false;
    };
    module.reference_assignable(&source, &target)
        || crate::interfaces::ensure_implementation(module, &source, &target).is_ok()
}
