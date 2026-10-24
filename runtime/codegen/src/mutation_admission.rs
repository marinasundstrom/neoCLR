//! Call-closure gate for the experimental live-GC pipeline. Store lowering and
//! host capability validation remain separate obligations; this is not admission
//! of arbitrary providers merely because their name and ABI match.
use cranelift_codegen::ir::{self, ExternalName, InstructionData, Opcode, Signature};
use cranelift_module::{FuncId, Linkage, Module};
use std::collections::HashMap;
type Error = Box<dyn std::error::Error>;

pub(crate) fn check_call_closure(
    module: &impl Module,
    bodies: &[(FuncId, &ir::Function)],
    providers: &[(&str, Signature)],
) -> Result<(), Error> {
    let declarations: HashMap<_, _> = module.declarations().get_functions().collect();
    let mut generated = HashMap::new();
    for &(id, body) in bodies {
        let declaration = declarations
            .get(&id)
            .ok_or("missing generated declaration")?;
        if declaration.linkage == Linkage::Import || declaration.signature != body.signature {
            return Err("generated body does not match its definition contract".into());
        }
        if generated.insert(id, body).is_some() {
            return Err("duplicate generated body".into());
        }
    }
    let mut admitted = HashMap::new();
    for (name, signature) in providers {
        if admitted.insert(*name, signature).is_some() {
            return Err("duplicate provider contract".into());
        }
    }
    for &(_, body) in bodies {
        for block in body.layout.blocks() {
            for inst in body.layout.block_insts(block) {
                let instruction = &body.dfg.insts[inst];
                if !instruction.opcode().is_call() {
                    continue;
                }
                let InstructionData::Call {
                    opcode: Opcode::Call,
                    func_ref,
                    ..
                } = instruction
                else {
                    return Err(
                        "live-GC closure rejects indirect, tail and exceptional calls".into(),
                    );
                };
                let external = &body.dfg.ext_funcs[*func_ref];
                let ExternalName::User(reference) = external.name else {
                    return Err("live-GC closure requires an explicit module target".into());
                };
                let name = &body.params.user_named_funcs()[reference];
                if name.namespace != 0 {
                    return Err("invalid module function namespace".into());
                }
                let target = FuncId::from_u32(name.index);
                let declaration = declarations.get(&target).ok_or("undeclared call target")?;
                if body.dfg.signatures[external.signature] != declaration.signature {
                    return Err("call-site signature does not match target".into());
                }
                if generated.contains_key(&target) {
                    continue;
                }
                if declaration.linkage != Linkage::Import {
                    return Err("generated call target has no supplied body".into());
                }
                let signature = declaration
                    .name
                    .as_deref()
                    .and_then(|name| admitted.get(name))
                    .ok_or("native call target has no admitted provider contract")?;
                if **signature != declaration.signature {
                    return Err("native provider signature mismatch".into());
                }
            }
        }
    }
    Ok(())
}

// Exact private experimental query ABI. These bits describe required semantics,
// not optional performance policies or a blanket provider admission.
const HOST_VERSION: u32 = 1;
const REQUIRED_HOST_FEATURES: u64 = 1 | 2 | 4;

/// The library is trusted native code and must outlive all bound generated code.
/// Call before binding providers or defining/invoking the experimental image.
/// The query is the only host function invoked by this compatibility check.
pub(crate) unsafe fn require_host(library: &libloading::Library) -> Result<(), Error> {
    type Query = unsafe extern "C" fn(u32) -> u64;
    let query: libloading::Symbol<Query> =
        unsafe { library.get(b"experiment_gc_live_capabilities_v1") }
            .map_err(|_| "host lacks the experimental live-GC capability query")?;
    let features = unsafe { query(HOST_VERSION) };
    check_host_features(features)
}
pub(crate) unsafe fn require_managed_host(library: &libloading::Library) -> Result<(), Error> {
    unsafe { require_host(library) }?;
    type Query = unsafe extern "C" fn(u32) -> u64;
    let query: libloading::Symbol<Query> =
        unsafe { library.get(b"experiment_gc_live_capabilities_v1") }?;
    if unsafe { query(HOST_VERSION) } & 8 == 0 {
        return Err("host lacks published-boundary epoch completion".into());
    }
    Ok(())
}
fn check_host_features(features: u64) -> Result<(), Error> {
    if features & REQUIRED_HOST_FEATURES != REQUIRED_HOST_FEATURES {
        return Err("host lacks required version-1 live-GC semantics".into());
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn requires_all_semantics_and_allows_additive_features() {
        for features in 0..7 {
            assert!(check_host_features(features).is_err());
        }
        assert!(check_host_features(7).is_ok());
        assert!(check_host_features(7 | (1 << 63)).is_ok());
    }
}

/// Initial internal provider set for generated bodies; application/native service
/// providers must be qualified separately before extending this list.
pub(crate) fn root_providers(module: &impl Module) -> Vec<(&'static str, Signature)> {
    use ir::{types::*, AbiParam};
    let make = |parameters: &[ir::Type], returns: &[ir::Type]| {
        let mut signature = module.make_signature();
        signature
            .params
            .extend(parameters.iter().copied().map(AbiParam::new));
        signature
            .returns
            .extend(returns.iter().copied().map(AbiParam::new));
        signature
    };
    vec![
        ("neoclr_task_queue_host_read_v1", make(&[I64, I64], &[I32])),
        ("experiment_gc_prepare_entry_v1", make(&[I64], &[I32])),
        ("neoclr_gc_complete_invocation_v1", make(&[I64], &[I32])),
        ("experiment_gc_store", make(&[I64, I64, I64, I32], &[I32])),
        ("neoclr_gc_callback_read_v1", make(&[I64, I64, I64], &[I32])),
        ("neoclr_socket_listen_v1", make(&[I64, I32, I32, I64, I64], &[I32])),
        ("neoclr_socket_local_port_v1", make(&[I64, I64, I64], &[I32])),
        ("neoclr_socket_close_v1", make(&[I64, I64, I64], &[I32])),
        ("neoclr_socket_accept_v1", make(&[I64, I64, I64, I64], &[I32])),
        ("neoclr_socket_connect_result_v1", make(&[I64, I64, I64], &[I32])),
        ("neoclr_socket_cancel_v1", make(&[I64, I64, I64], &[I32])),
        ("neoclr_socket_deadline_after_v1", make(&[I32, I64, I64], &[I32])),
        ("neoclr_socket_deadline_expired_v1", make(&[I64, I64, I64], &[I32])),
        ("neoclr_socket_receive_v1", make(&[I64, I64, I32, I32, I64, I64, I64], &[I32])),
        ("neoclr_socket_send_v1", make(&[I64, I64, I32, I32, I64, I64, I64], &[I32])),
        ("neoclr_socket_transfer_result_v1", make(&[I64, I64, I64], &[I32])),
        ("neoclr_socket_receive_until_v1", make(&[I64, I64, I32, I32, I64, I64, I64, I64], &[I32])),
        ("neoclr_socket_send_until_v1", make(&[I64, I64, I32, I32, I64, I64, I64, I64], &[I32])),
        ("neoclr_socket_receive_memory_v1", make(&[I64, I64, I32, I32, I64, I64, I64], &[I32])),
        ("neoclr_socket_send_memory_v1", make(&[I64, I64, I32, I32, I64, I64, I64], &[I32])),
        ("neoclr_socket_receive_until_memory_v1", make(&[I64, I64, I32, I32, I64, I64, I64, I64], &[I32])),
        ("neoclr_socket_send_until_memory_v1", make(&[I64, I64, I32, I32, I64, I64, I64, I64], &[I32])),
        ("neoclr_runtime_async_create_v1", make(&[I64, I64], &[I32])),
        ("neoclr_runtime_async_start_v1", make(&[I64, I64, I64, I64], &[I32])),
        ("neoclr_runtime_async_createowned_v1", make(&[I64, I64, I64], &[I32])),
        ("neoclr_runtime_async_currentowner_v1", make(&[I64, I64], &[I32])),
        ("neoclr_runtime_async_startmanaged_v1", make(&[I64, I64, I64, I64, I64], &[I32])),
        ("neoclr_runtime_async_completeunit_v1", make(&[I64, I64, I64], &[I32])),
        ("neoclr_runtime_async_awaitunit_v1", make(&[I64, I64, I64], &[I32])),
        ("neoclr_runtime_async_release_v1", make(&[I64, I64, I64], &[I32])),
        ("neoclr_runtime_async_iscompleted_v1", make(&[I64, I64, I64], &[I32])),
        ("neoclr_runtime_async_waitcompletion_v1", make(&[I64, I64, I64], &[I32])),
        ("experiment_gc_string_slice_utf8_rooted_v1", make(&[I64, I32, I32, I64, I64], &[I32])),
        ("neoclr_byte_view_create_all_v1", make(&[I64, I64, I64, I64], &[I32])),
        ("neoclr_byte_view_create_v1", make(&[I64, I64, I32, I32, I64, I64], &[I32])),
        ("neoclr_byte_view_slice_v1", make(&[I64, I64, I32, I32, I64], &[I32])),
        ("neoclr_byte_view_get_v1", make(&[I64, I64, I32, I64], &[I32])),
        ("neoclr_byte_view_set_v1", make(&[I64, I64, I32, ir::types::I8], &[I32])),
        ("neoclr_byte_view_utf8_v1", make(&[I64, I64, I64], &[I32])),
        ("experiment_gc_static_storage_rooted_v1", make(&[I64, I32, I32, I32, I64], &[I32])),
        ("neoclr_string_concat_rooted_v1", make(&[I64, I64, I64, I64], &[I32])),
        ("neoclr_string_byte_count_v1", make(&[I64, I64], &[I32])),
        ("neoclr_string_compare_canonical_v1", make(&[I64, I64, I64], &[I32])),
        ("neoclr_string_compare_ignore_case_v1", make(&[I64, I64, I64], &[I32])),
        ("neoclr_string_starts_with_ordinal_v1", make(&[I64, I64, I64], &[I32])),
        ("experiment_gc_store_text", make(&[I64, I64, I32, I64], &[I32])),
        ("neoclr_allocate_bytes_rooted_v1", make(&[I64, I32, I64], &[I32])),
        ("neoclr_reserve_bytes_rooted_v1", make(&[I64, I32, I64], &[I32])),
        ("neoclr_allocate_scalars_rooted_v1", make(&[I64, I32, I32, I64], &[I32])),
        ("neoclr_allocate_strings_rooted_v1", make(&[I64, I32, I32, I64], &[I32])),
        ("experiment_gc_store_record", make(&[I64, I64, I32, I64, I32], &[I32])),
        ("neoclr_allocate_references_rooted_v1", make(&[I64, I32, I64], &[I32])),
        // Generated calls publish live values first and pass a stack result slot.
        // The provider initializes only freshly allocated storage before returning.
        ("neoclr_allocate_object_rooted_v1", make(&[I64, I32, I32, I64], &[I32])),
        (
            "neoclr_gc_state_enter_v1",
            make(&[I64, I64, I32, I64, I32, I64, I32], &[I64]),
        ),
        ("neoclr_gc_state_leave_v1", make(&[I64, I64], &[])),
        (
            "neoclr_gc_state_stack_v1",
            make(&[I64, I64, I32, I64, I32, I64, I32], &[]),
        ),
        (
            "neoclr_gc_state_transient_v1",
            make(&[I64, I64, I32, I64, I32, I64, I32], &[]),
        ),
        ("neoclr_native_stack_check_v1", make(&[], &[I32])),
    ]
}
