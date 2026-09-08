//! Immutable resolved metadata shared by execution and analysis.
use crate::{
    Execution, ExecutionOptions, Fault, Module, TypeIdentity, Verification, metadata::Type,
};

/// A validated, linked metadata snapshot with calls bound before specialization.
///
/// Loading does not perform typed verification or execute code. Each execution owns
/// fresh frames, allocations, output, and native-library state. This Rust API is not
/// a stable native hosting ABI or a serialized artifact format. An explicitly supplied
/// host console may share external I/O state across executions.
#[derive(Debug)]
pub struct LoadedProgram {
    module: Module,
}

impl LoadedProgram {
    /// Prepare an application with bundled System, or prepare System alone for analysis.
    pub fn new(module: &Module) -> Result<Self, Fault> {
        if module.name == "System" {
            crate::vm::validate(module)?;
            let mut module = module.clone();
            module.normalize_definition_ids()?;
            let mut module = crate::scope::normalize_module(&module, &module)?;
            crate::library::bind_member_references(&mut module)?;
            Ok(Self { module })
        } else {
            Self::with_library(module, crate::library::system()?)
        }
    }

    /// Prepare an application with an explicitly supplied System artifact.
    pub fn with_library(module: &Module, library: &Module) -> Result<Self, Fault> {
        Ok(Self {
            module: crate::library::link(module, library)?,
        })
    }

    /// Prepare an explicit load set. Additional modules must have unique names and
    /// no entry points. Symbol lookup currently uses one shared namespace.
    pub fn with_modules(
        module: &Module,
        library: &Module,
        dependencies: &[Module],
    ) -> Result<Self, Fault> {
        Ok(Self {
            module: crate::library::link_modules(module, library, dependencies)?,
        })
    }

    /// Analyze the already-bound metadata without executing code.
    pub fn verify(&self) -> Result<Verification, Fault> {
        crate::verifier::analyze(&self.module)
    }

    /// Resolve a closed signature in this snapshot, without relinking.
    pub fn resolve_type_identity(&self, ty: &Type) -> Result<TypeIdentity, Fault> {
        crate::type_identity::resolve(&self.module, ty)
    }

    /// Describe a type without granting construction, invocation, or mutation rights.
    pub fn describe_type(&self, ty: &Type) -> Result<crate::type_identity::TypeDescriptor, Fault> {
        crate::type_identity::describe(&self.module, ty)
    }

    /// Resolve a closed type in this snapshot and calculate its explicit target layout.
    pub fn layout_of(
        &self,
        ty: &Type,
        target: crate::memory::TargetLayout,
    ) -> Result<crate::memory::Layout, Fault> {
        let ty = crate::scope::normalize_type(&self.module, ty)?;
        crate::vm::check_type(&ty, &self.module)?;
        crate::references::check_type(&self.module, &self.module, &ty)?;
        crate::memory::layout_for(&self.module, &ty, target)
    }

    /// Build a conservative closed call graph from explicit roots without executing code.
    /// The limit counts distinct definition/closed-owner instantiations, including imports.
    pub fn analyze_reachability(
        &self,
        roots: &[crate::metadata::FunctionRef],
        max_functions: usize,
    ) -> Result<crate::Reachability, Fault> {
        crate::reachability::analyze(&self.module, roots, max_functions)
    }

    /// Resolve a closed IL function with validated owned inputs.
    /// The returned handle borrows this immutable program and needs no entry point.
    pub fn resolve_function(
        &self,
        target: &crate::metadata::FunctionRef,
    ) -> Result<LoadedFunction<'_>, Fault> {
        let mut target = target.clone();
        if let Some(owner) = &mut target.owner {
            *owner = crate::scope::normalize_type(&self.module, owner)?;
            crate::vm::check_type(owner, &self.module)?;
        }
        for parameter in target
            .parameters
            .iter_mut()
            .chain(&mut target.generic_arguments)
        {
            *parameter = crate::scope::normalize_type(&self.module, parameter)?;
            crate::vm::check_type(parameter, &self.module)?;
        }
        crate::references::check_call(&self.module, &self.module, &target)?;
        let function = crate::vm::resolve(&self.module, &target)?;
        crate::access::check_call(&self.module, None, &function)?;
        crate::access::check_signature(&self.module, None, &function)?;
        if function.is_abstract
            || function.is_internal_call()
            || function.pinvoke.is_some()
            || crate::interfaces::is_contract(&self.module, &function)
        {
            return Err(Fault::new(
                "host invocation currently requires an IL function; use an IL wrapper for native declarations",
            ));
        }
        let inputs = crate::input::resolve(&self.module, &function.argument_types())?;
        let definition = function
            .definition
            .clone()
            .ok_or_else(|| Fault::new("missing resolved function identity"))?;
        Ok(LoadedFunction {
            program: self,
            function,
            definition,
            inputs,
        })
    }

    /// Execute the entry point with fresh state and native imports disabled.
    pub fn run(&self, options: impl Into<ExecutionOptions>) -> Result<Execution, Fault> {
        self.check_entry()?;
        crate::vm::interpret(&self.module, options.into(), None)
    }

    /// Execute the entry point with fresh state and native imports enabled.
    ///
    /// # Safety
    /// Every executed native declaration must match its exported C ABI signature.
    /// Native code and library initializers/destructors must uphold pointer validity,
    /// allocation lifetimes, and Rust's memory safety requirements. Guest metadata
    /// alone cannot establish these guarantees. The caller must trust the code.
    pub unsafe fn run_with_native(
        &self,
        options: impl Into<ExecutionOptions>,
    ) -> Result<Execution, Fault> {
        self.check_entry()?;
        crate::vm::interpret(
            &self.module,
            options.into(),
            Some(crate::interop::NativeLibraries::default()),
        )
    }

    fn check_entry(&self) -> Result<(), Fault> {
        if self.module.entry.is_empty() {
            return Err(Fault::new(
                "cannot execute a library without an entry point",
            ));
        }
        Ok(())
    }
}

/// A resolved IL function belonging to one immutable program snapshot.
#[derive(Debug)]
pub struct LoadedFunction<'program> {
    program: &'program LoadedProgram,
    function: crate::metadata::Function,
    definition: crate::metadata::MemberId,
    inputs: Vec<crate::input::Input>,
}

impl LoadedFunction<'_> {
    pub fn definition(&self) -> &crate::metadata::MemberId {
        &self.definition
    }
    /// The explicit copied receiver type, or None for static functions.
    pub fn receiver_type(&self) -> Option<&Type> {
        if self.function.instance {
            self.function.owner.as_ref()
        } else {
            None
        }
    }

    /// Declared parameters, excluding the instance receiver.
    pub fn parameters(&self) -> &[Type] {
        &self.function.parameters
    }
    pub fn returns(&self) -> &Type {
        &self.function.returns
    }

    /// Invoke with validated owned storage values and fresh guest state.
    pub fn invoke(
        &self,
        arguments: Vec<crate::Value>,
        options: impl Into<ExecutionOptions>,
    ) -> Result<Execution, Fault> {
        let arguments = self.import_arguments(None, arguments)?;
        crate::vm::interpret_function(
            &self.program.module,
            self.function.clone(),
            arguments,
            options.into(),
            None,
        )
    }

    /// Invoke with fresh state and native imports enabled.
    ///
    /// # Safety
    /// Native declarations must match their exported C ABI signatures. Native code
    /// and library initializers/destructors must uphold pointer validity, allocation
    /// lifetimes, and Rust's memory safety requirements. The caller must trust the code.
    pub unsafe fn invoke_with_native(
        &self,
        arguments: Vec<crate::Value>,
        options: impl Into<ExecutionOptions>,
    ) -> Result<Execution, Fault> {
        let arguments = self.import_arguments(None, arguments)?;
        crate::vm::interpret_function(
            &self.program.module,
            self.function.clone(),
            arguments,
            options.into(),
            Some(crate::interop::NativeLibraries::default()),
        )
    }

    /// Invoke an instance method with an owned receiver snapshot and fresh state.
    /// Changes to `this` do not update a value retained by the caller.
    pub fn invoke_instance(
        &self,
        receiver: crate::Value,
        arguments: Vec<crate::Value>,
        options: impl Into<ExecutionOptions>,
    ) -> Result<Execution, Fault> {
        let arguments = self.import_arguments(Some(receiver), arguments)?;
        crate::vm::interpret_function(
            &self.program.module,
            self.function.clone(),
            arguments,
            options.into(),
            None,
        )
    }

    /// Invoke an instance method with an owned receiver and native imports enabled.
    ///
    /// # Safety
    /// Native declarations must match their exported C ABI signatures. Native code
    /// and library initializers/destructors must uphold pointer validity, allocation
    /// lifetimes, and Rust's memory safety requirements. The caller must trust the code.
    pub unsafe fn invoke_instance_with_native(
        &self,
        receiver: crate::Value,
        arguments: Vec<crate::Value>,
        options: impl Into<ExecutionOptions>,
    ) -> Result<Execution, Fault> {
        let arguments = self.import_arguments(Some(receiver), arguments)?;
        crate::vm::interpret_function(
            &self.program.module,
            self.function.clone(),
            arguments,
            options.into(),
            Some(crate::interop::NativeLibraries::default()),
        )
    }

    fn import_arguments(
        &self,
        receiver: Option<crate::Value>,
        arguments: Vec<crate::Value>,
    ) -> Result<Vec<crate::Value>, Fault> {
        let fault = |message| Fault {
            message,
            function: Some(self.function.name.clone()),
            instruction: None,
            stack_trace: None,
        };
        if receiver.is_some() != self.function.instance {
            return Err(fault(if self.function.instance {
                "instance invocation requires an explicit receiver".into()
            } else {
                "static invocation does not accept a receiver".into()
            }));
        }
        if arguments.len() != self.function.parameters.len() {
            return Err(fault(format!(
                "invocation expected {} arguments, got {}",
                self.function.parameters.len(),
                arguments.len()
            )));
        }
        let mut schemas = self.inputs.iter();
        let mut imported = Vec::with_capacity(self.inputs.len());
        if let Some(receiver) = receiver {
            let schema = schemas
                .next()
                .ok_or_else(|| fault("missing receiver schema".into()))?;
            imported.push(
                schema
                    .import(&self.program.module, receiver)
                    .map_err(|error| fault(format!("invocation receiver: {}", error.message)))?,
            );
        }
        for (index, (value, schema)) in arguments.into_iter().zip(schemas).enumerate() {
            imported.push(
                schema
                    .import(&self.program.module, value)
                    .map_err(|error| {
                        fault(format!("invocation argument {index}: {}", error.message))
                    })?,
            );
        }
        Ok(imported)
    }
}
