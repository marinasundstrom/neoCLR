use super::*;

/// Reenter the current invocation without creating a second heap or draining its
/// enclosing scheduler. The outer host-call roots remain published throughout.
#[allow(clippy::too_many_arguments)]
pub(super) fn invoke(
    module: &Module,
    callback: &Value,
    mut arguments: Vec<Value>,
    parent: &mut InstructionState,
    participant: &mut crate::shared_heap::Participant,
    options: &ExecutionOptions,
    frame_depth: usize,
    output: &mut Vec<String>,
    bytes: &mut [Vec<u8>; 2],
) -> Result<Value, Fault> {
    if parent.foreign_depth >= 32 || options.limits.frames <= frame_depth + 1 {
        return Err(Fault::coded(
            crate::FaultCode::StackOverflow,
            "native callback frame limit exceeded",
        ));
    }
    let Value::Function(binding) = callback else {
        return Err(Fault::new("native callback requires Function"));
    };
    let mut wrapper = crate::function_objects::contract(module, &binding.ty)?;
    let target = crate::metadata::FunctionRef {
        definition: None,
        name: wrapper.name.clone(),
        owner: Some(binding.ty.clone()),
        instance: true,
        generic_arguments: vec![],
        parameters: wrapper.parameters.clone(),
    };
    wrapper.name = "$NativeCallback".into();
    wrapper.instance = false;
    wrapper.owner = None;
    wrapper.parameters.insert(0, binding.ty.clone());
    wrapper.body = (0..wrapper.parameters.len())
        .map(crate::metadata::Instruction::Arg)
        .chain([
            crate::metadata::Instruction::Call(target),
            crate::metadata::Instruction::Return,
        ])
        .collect();
    arguments.insert(0, callback.clone());
    let mut callback_participant = participant
        .enter()
        .fork_values(&arguments, &options.limits)?;
    let mut frames = vec![Frame::new(wrapper, arguments)?];
    let mut state = InstructionState::with_invocation(parent.invocation.clone());
    state.foreign_depth = parent.foreign_depth + 1;
    state.dispatch_identity = parent.dispatch_identity.clone();
    state.work_control = parent.work_control.clone();
    state.work_output_bytes = parent.work_output_bytes;
    let mut options = options.clone();
    options.limits.frames -= frame_depth;
    let mut memory = parent.invocation.memory.clone();
    let result = drive_instructions(
        module,
        &mut frames,
        &options,
        &mut None,
        &mut callback_participant,
        &mut memory,
        output,
        bytes,
        &mut state,
    );
    parent.work_output_bytes = state.work_output_bytes;
    result.map_err(|fault| {
        fault.with_stack_trace(crate::StackTrace::capture(
            frames
                .iter()
                .rev()
                .map(|f| (f.function.as_ref(), f.trace_pc)),
        ))
    })
}
