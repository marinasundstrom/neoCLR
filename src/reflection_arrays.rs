//! Private array access for the JSON mapper, executed as ordinary checked IL.
use crate::{
    Fault, Module, Value,
    metadata::{Function, Instruction as Op, Type},
};

pub(crate) fn adapter(
    module: &Module,
    service: &Function,
    args: &[Value],
    kind: u8,
) -> Result<Function, Fault> {
    let object = Type::from_name("System.Object");
    let array = if kind == 2 {
        let Value::ObjectReference(info) = &args[0] else {
            return Err(Fault::new("array construction requires runtime TypeInfo"));
        };
        if info.concrete_type() != Type::from_name("System.Introspection.RuntimeTypeInfo") {
            return Err(Fault::new("array construction requires runtime TypeInfo"));
        }
        let Value::Object { fields, .. } = info.reference.read()? else {
            return Err(Fault::new("invalid runtime TypeInfo"));
        };
        let handle = fields
            .first()
            .ok_or_else(|| Fault::new("missing runtime type handle"))?;
        crate::reflection_execution::bound_type(module, handle)
            .map_err(|_| Fault::new("array construction requires a bound type"))?
    } else {
        let Value::ObjectReference(value) = &args[0] else {
            return Err(Fault::new("array access requires a non-null array"));
        };
        value.concrete_type()
    };
    let Type::ArrayRef(element) = &array else {
        return Err(Fault::new("array service requires a vector type"));
    };
    if !crate::reflection_properties::supported_value(module, element) {
        return Err(Fault::new("unsupported array element type"));
    }
    let mut adapter = service.clone();
    adapter.impl_flags = 0;
    adapter.body = match kind {
        0 => vec![
            Op::Arg(0),
            Op::CastClass(array.clone()),
            Op::ArrayLength,
            Op::ConvertInt32,
            Op::Return,
        ],
        1 => vec![
            Op::Arg(0),
            Op::CastClass(array.clone()),
            Op::Arg(1),
            Op::ArrayElement((**element).clone()),
            if module.is_object_reference_type(element) {
                Op::CastClass(object)
            } else {
                Op::BoxValue((**element).clone())
            },
            Op::Return,
        ],
        2 => {
            // A fixed-size loop keeps planning independent of the requested length.
            // Allocation, casts, stores and GC roots use the ordinary VM contracts.
            adapter.locals = vec![array.clone(), Type::Int32];
            adapter.local_names = vec![None, None];
            vec![
                Op::Arg(1),
                Op::ArrayLength,
                Op::ConvertInt32,
                Op::ReserveArray((**element).clone()),
                Op::Store(0),
                Op::Int(0),
                Op::Store(1),
                Op::Load(1),
                Op::Arg(1),
                Op::ArrayLength,
                Op::ConvertInt32,
                Op::Less,
                Op::BranchFalse(25),
                Op::Load(0),
                Op::Load(1),
                Op::Arg(1),
                Op::Load(1),
                Op::ArrayElement(object.clone()),
                if module.is_object_reference_type(element) {
                    Op::CastClass((**element).clone())
                } else {
                    Op::UnboxAny((**element).clone())
                },
                Op::StoreArrayElement((**element).clone()),
                Op::Load(1),
                Op::Int(1),
                Op::Add,
                Op::Store(1),
                Op::Branch(7),
                Op::Load(0),
                Op::CastClass(object),
                Op::Return,
            ]
        }
        _ => return Err(Fault::new("unknown array operation")),
    };
    Ok(adapter)
}
