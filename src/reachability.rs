//! Conservative closed call graphs for backend planning, without executing code.
use std::collections::HashMap;

use crate::{
    Fault, Module,
    metadata::{Function, FunctionRef, Instruction, MemberId, NativeImport, Type},
};

/// A report-local graph. Function indices are deterministic for a fixed root order.
#[derive(Debug, Clone)]
pub struct Reachability {
    /// One index per requested root, including repeated roots.
    pub roots: Vec<usize>,
    pub functions: Vec<ReachableFunction>,
}

#[derive(Debug, Clone)]
pub struct ReachableFunction {
    /// Canonical closed signature, including the selected definition row.
    pub target: FunctionRef,
    pub returns: Type,
    pub receiver_byref: bool,
    pub receiver_readonly: bool,
    pub out_parameters: Vec<usize>,
    pub out_when_true: Vec<usize>,
    pub readonly_parameters: Vec<usize>,
    pub implementation: FunctionImplementation,
    /// Every syntactic call in the specialized IL body, including unreachable code.
    pub calls: Vec<ReachableCall>,
    /// Logical runtime services used directly by this body or import declaration.
    pub services: Vec<crate::ServiceUse>,
}

#[derive(Debug, Clone)]
pub enum FunctionImplementation {
    Il,
    InternalCall,
    NativeImport(NativeImport),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReachableCall {
    pub instruction: usize,
    /// Index into Reachability::functions, not a metadata token.
    pub target: usize,
}

pub(crate) fn analyze(
    module: &Module,
    roots: &[FunctionRef],
    max_functions: usize,
) -> Result<Reachability, Fault> {
    let mut functions = Vec::<Function>::new();
    let mut seen = HashMap::<(MemberId, Option<Type>), usize>::new();
    let mut intern = |function: Function, functions: &mut Vec<Function>| -> Result<usize, Fault> {
        let definition = function
            .definition
            .clone()
            .ok_or_else(|| Fault::new("missing reachable function identity"))?;
        let key = (definition, function.owner.clone());
        if let Some(index) = seen.get(&key) {
            return Ok(*index);
        }
        if functions.len() >= max_functions {
            return Err(Fault::new("closed call graph exceeds function limit"));
        }
        let index = functions.len();
        seen.insert(key, index);
        functions.push(function);
        Ok(index)
    };
    let mut root_indices = Vec::with_capacity(roots.len());
    for root in roots {
        let mut root = root.clone();
        if let Some(owner) = &mut root.owner {
            *owner = crate::scope::normalize_type(module, owner)?;
            crate::vm::check_type(owner, module)?;
        }
        for parameter in &mut root.parameters {
            *parameter = crate::scope::normalize_type(module, parameter)?;
            crate::vm::check_type(parameter, module)?;
        }
        crate::references::check_call(module, module, &root)?;
        root_indices.push(intern(crate::vm::resolve(module, &root)?, &mut functions)?);
    }
    let mut nodes = Vec::new();
    while nodes.len() < functions.len() {
        let function = functions[nodes.len()].clone();
        if function.is_abstract || crate::interfaces::is_contract(module, &function) {
            return Err(crate::Fault::new(
                "an abstract declaration is not an executable graph root",
            ));
        }
        let implementation = if let Some(import) = &function.pinvoke {
            FunctionImplementation::NativeImport(import.clone())
        } else if function.is_internal_call() {
            FunctionImplementation::InternalCall
        } else {
            FunctionImplementation::Il
        };
        let mut calls = Vec::new();
        for (instruction, op) in function.body.iter().enumerate() {
            let callees = match op {
                Instruction::CallVirtual(target) => {
                    crate::vm::resolve(module, target).and_then(|contract| {
                        if crate::interfaces::is_contract(module, &contract) {
                            crate::interfaces::dispatch_targets(module, &contract)
                        } else {
                            crate::inheritance::dispatch_targets(module, &contract)
                        }
                    })
                }
                Instruction::Call(target) | Instruction::Construct(target) => {
                    // The loader already checked the declaring module's references.
                    crate::vm::resolve(module, target).map(|callee| vec![callee])
                }
                _ => continue,
            }
            .map_err(|mut fault| {
                fault.function = Some(function.name.clone());
                fault.instruction = Some(instruction);
                fault
            })?;
            for callee in callees {
                let target = intern(callee, &mut functions).map_err(|mut fault| {
                    fault.function = Some(function.name.clone());
                    fault.instruction = Some(instruction);
                    fault
                })?;
                calls.push(ReachableCall {
                    instruction,
                    target,
                });
            }
        }
        let services = crate::services::uses(&function)?;
        nodes.push(ReachableFunction {
            target: FunctionRef {
                definition: function.definition,
                name: function.name,
                owner: function.owner,
                instance: function.instance,
                parameters: function.parameters,
            },
            returns: function.returns,
            receiver_byref: function.receiver_byref,
            receiver_readonly: function.receiver_readonly,
            out_parameters: function.out_parameters,
            out_when_true: function.out_when_true,
            readonly_parameters: function.readonly_parameters,
            implementation,
            calls,
            services,
        });
    }
    Ok(Reachability {
        roots: root_indices,
        functions: nodes,
    })
}

impl Reachability {
    /// Distinct service requirements, in RuntimeService enum order.
    pub fn required_services(&self) -> Vec<crate::RuntimeService> {
        self.functions
            .iter()
            .flat_map(|function| function.services.iter().map(|usage| usage.service))
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// Every use absent from the supplied service set, in function/instruction order.
    /// An empty result is not a guarantee of backend opcode, layout, or ABI support.
    pub fn missing_services(
        &self,
        available: &[crate::RuntimeService],
    ) -> Vec<crate::MissingService> {
        self.functions
            .iter()
            .enumerate()
            .flat_map(|(function, node)| {
                node.services
                    .iter()
                    .filter(|usage| !available.contains(&usage.service))
                    .map(move |usage| crate::MissingService {
                        function,
                        instruction: usage.instruction,
                        service: usage.service,
                    })
            })
            .collect()
    }
}
