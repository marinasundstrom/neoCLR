//! Bounded C-ABI import thunks. Managed private status/result ABI stays separate.
use crate::scalar_abi::scalar_type;
use cranelift_codegen::ir::{AbiParam, InstBuilder, MemFlags, StackSlotData, StackSlotKind, types};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{DataDescription, FuncId, Linkage, Module};
use neoclr::metadata::{Function, Type};
type Error = Box<dyn std::error::Error>;

pub(crate) fn validate(f: &Function) -> Result<(), Error> {
    let import = f.pinvoke.as_ref().ok_or("missing native import")?;
    if f.owner.is_some()
        || f.instance
        || if f.no_result {
            f.returns != Type::Void
        } else {
            scalar_type(&f.returns).is_none()
        }
        || f.parameters.len() > 16
        || f.parameters.iter().any(|t| scalar_type(t).is_none())
        || !f.locals.is_empty()
        || !f.body.is_empty()
        || f.impl_flags != 0
        || f.is_abstract
        || f.is_virtual
        || f.is_override
        || f.receiver_byref
        || f.receiver_readonly
        || !f.generic_parameters.is_empty()
        || !f.generic_arguments.is_empty()
        || !f.generic_constraints.is_empty()
        || !f.out_parameters.is_empty()
        || !f.out_when_true.is_empty()
        || !f.readonly_parameters.is_empty()
        || import.string_marshalling.is_some()
        || !import.parameter_marshalling.is_empty()
        || import.return_marshalling.is_some()
        || !import.callback_parameters.is_empty()
        || import.function_return.is_some()
    {
        return Err(format!("{}: native import profile requires free scalar functions with at most 16 Int32, Int64/UInt64, Single/Double, native-sized integer or supported pointer parameters and no marshalling", f.name).into());
    }
    // The ordinary metadata verifier validates names and calling conventions.
    Ok(())
}

pub(crate) fn emit<M: Module>(
    module: &mut M,
    f: &Function,
    id: FuncId,
    index: usize,
) -> Result<(), Error> {
    let import = f.pinvoke.as_ref().expect("validated import");
    let mut strings = Vec::new();
    for (part, text) in [&import.library, &import.entry_point].iter().enumerate() {
        let id = module.declare_data(
            &format!("neoclr_import_{index}_{part}"),
            Linkage::Local,
            false,
            false,
        )?;
        let mut data = DataDescription::new();
        let mut bytes = text.as_bytes().to_vec();
        bytes.push(0);
        data.define(bytes.into_boxed_slice());
        module.define_data(id, &data)?;
        strings.push(id);
    }
    let mut resolve_sig = module.make_signature();
    resolve_sig.params = vec![AbiParam::new(types::I64); 3];
    resolve_sig.returns.push(AbiParam::new(types::I32));
    let resolver = module.declare_function(
        "neoclr_native_import_resolve_v1",
        Linkage::Import,
        &resolve_sig,
    )?;
    let mut native_sig = module.make_signature();
    native_sig.params = f
        .parameters
        .iter()
        .map(|t| AbiParam::new(scalar_type(t).unwrap()))
        .collect();
    if !f.no_result {
        native_sig
            .returns
            .push(AbiParam::new(scalar_type(&f.returns).unwrap()));
    }
    let mut ctx = module.make_context();
    ctx.func.signature = module
        .declarations()
        .get_function_decl(id)
        .signature
        .clone();
    let resolver = module.declare_func_in_func(resolver, &mut ctx.func);
    let data: Vec<_> = strings
        .into_iter()
        .map(|id| module.declare_data_in_func(id, &mut ctx.func))
        .collect();
    let mut frontend = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut ctx.func, &mut frontend);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let args = b.block_params(entry).to_vec();
        let slot = b.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 8, 3));
        let output = b.ins().stack_addr(types::I64, slot, 0);
        let library = b.ins().global_value(types::I64, data[0]);
        let symbol = b.ins().global_value(types::I64, data[1]);
        let call = b.ins().call(resolver, &[library, symbol, output]);
        let status = b.inst_results(call)[0];
        let failed = b
            .ins()
            .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::NotEqual, status, 0);
        crate::compiler::return_if_detailed(&mut b, failed, status, None);
        let address = b.ins().stack_load(types::I64, slot, 0);
        let signature = b.import_signature(native_sig);
        let call = b
            .ins()
            .call_indirect(signature, address, &args[..f.parameters.len()]);
        if !f.no_result {
            let result = b.inst_results(call)[0];
            b.ins()
                .store(MemFlags::new(), result, args[f.parameters.len()], 0);
        }
        let success = b.ins().iconst(types::I32, 0);
        b.ins().return_(&[success]);
        b.seal_all_blocks();
        b.finalize();
    }
    module.define_function(id, &mut ctx)?;
    Ok(())
}
