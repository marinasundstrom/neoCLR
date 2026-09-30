//! Native implementing-type contracts. Self is substituted before argument checking,
//! and is never an erased interface value or a generic parameter supplied by callers.
use crate::{
    Fault, Module,
    metadata::{Function, FunctionRef, Type},
};

pub(crate) fn signature_has_self(function: &Function) -> bool {
    function.parameters.iter().any(Type::contains_self) || function.returns.contains_self()
}

pub(crate) fn signature(
    module: &Module,
    caller: &Function,
    self_type: &Type,
    target: &FunctionRef,
) -> Result<Function, Fault> {
    if self_type.contains_self() {
        return Err(Fault::new(
            "callself requires an implementing type, not Self",
        ));
    }
    let contract = crate::vm::resolve(module, target)?;
    if !crate::interfaces::is_contract(module, &contract)
        || crate::interfaces::is_helper(&contract)
        || !contract.interface_implementations.is_empty()
    {
        return Err(Fault::new("callself requires an interface declaration"));
    }
    let interface = contract.owner.as_ref().unwrap();
    if matches!(
        self_type,
        Type::TypeParameter(_) | Type::MethodTypeParameter(_)
    ) {
        if !crate::constraints::permits_view(module, caller, self_type, interface, true) {
            return Err(Fault::new(
                "callself requires an implementing-type interface bound",
            ));
        }
        if contract.instance {
            return Err(Fault::new(
                "open callself instance receivers require a concrete value/reference mode",
            ));
        }
    } else {
        crate::interfaces::ensure_implementation(module, self_type, interface)?;
        return implementation(module, self_type, target);
    }
    contract.map_types(|ty| ty.substitute_self(self_type))
}

pub(crate) fn implementation(
    module: &Module,
    self_type: &Type,
    target: &FunctionRef,
) -> Result<Function, Fault> {
    let contract = crate::vm::resolve(module, target)?;
    if !crate::interfaces::is_contract(module, &contract) {
        return Err(Fault::new("callself requires an interface declaration"));
    }
    crate::interfaces::implementation(
        module,
        self_type,
        contract.owner.as_ref().unwrap(),
        &contract,
    )
}
