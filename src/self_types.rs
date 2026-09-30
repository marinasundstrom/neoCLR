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
    borrowed: bool,
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
    if borrowed && !contract.instance {
        return Err(Fault::new(
            "borrowed callself requires an instance contract",
        ));
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
        if contract.instance && !borrowed {
            return Err(Fault::new(
                "open callself instance receivers require a concrete value/reference mode",
            ));
        }
    } else {
        crate::interfaces::ensure_implementation(module, self_type, interface)?;
        if !borrowed {
            return implementation(module, self_type, target);
        }
        let body = implementation(module, self_type, target)?;
        if !module.is_reference_type(self_type) && !body.receiver_byref {
            return Err(Fault::new(
                "borrowed Self dispatch requires a byref value receiver",
            ));
        }
        if contract.receiver_readonly && body.receiver_byref && !body.receiver_readonly {
            return Err(Fault::new(
                "readonly Self contract requires a readonly value receiver",
            ));
        }
    }
    let mut signature = contract.map_types(|ty| ty.substitute_self(self_type))?;
    if borrowed {
        signature.owner = Some(self_type.clone());
        signature.receiver_byref = true;
    }
    Ok(signature)
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
    if !crate::interfaces::satisfies_self_bound(
        module,
        self_type,
        contract.owner.as_ref().unwrap(),
    )? {
        return Err(Fault::new(
            "callself requires Self conformance declared on the implementing type",
        ));
    }
    crate::interfaces::implementation(
        module,
        self_type,
        contract.owner.as_ref().unwrap(),
        &contract,
    )
}
