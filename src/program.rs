//! Immutable resolved metadata shared by execution and analysis.
use crate::{Execution, Fault, Limits, Module, TypeIdentity, Verification, metadata::Type};

/// A validated, linked metadata snapshot with calls bound before specialization.
///
/// Loading does not perform typed verification or execute code. Each execution owns
/// fresh frames, allocations, output, and native-library state. This Rust API is not
/// a stable native hosting ABI or a serialized artifact format.
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

    /// Resolve a closed static IL function with primitive input parameters.
    /// The returned handle borrows this immutable program and needs no entry point.
    pub fn resolve_function(
        &self,
        target: &crate::metadata::FunctionRef,
    ) -> Result<LoadedFunction<'_>, Fault> {
        if target.instance {
            return Err(Fault::new(
                "host invocation does not yet support instance receivers",
            ));
        }
        let mut target = target.clone();
        if let Some(owner) = &mut target.owner {
            *owner = crate::scope::normalize_type(&self.module, owner)?;
            crate::vm::check_type(owner, &self.module)?;
        }
        for parameter in &mut target.parameters {
            *parameter = crate::scope::normalize_type(&self.module, parameter)?;
            crate::vm::check_type(parameter, &self.module)?;
        }
        crate::references::check_call(&self.module, &self.module, &target)?;
        let function = crate::vm::resolve(&self.module, &target)?;
        if function.is_internal_call() || function.pinvoke.is_some() {
            return Err(Fault::new(
                "host invocation currently requires an IL function; use an IL wrapper for native declarations",
            ));
        }
        if !function.parameters.iter().all(Type::is_primitive) {
            return Err(Fault::new(
                "host invocation currently requires primitive input parameters",
            ));
        }
        let definition = function
            .definition
            .clone()
            .ok_or_else(|| Fault::new("missing resolved function identity"))?;
        Ok(LoadedFunction {
            program: self,
            function,
            definition,
        })
    }

    /// Execute the entry point with fresh state and native imports disabled.
    pub fn run(&self, limits: Limits) -> Result<Execution, Fault> {
        self.check_entry()?;
        crate::vm::interpret(&self.module, limits, None)
    }

    /// Execute the entry point with fresh state and native imports enabled.
    ///
    /// # Safety
    /// Every executed native declaration must match its exported C ABI signature.
    /// Native code and library initializers/destructors must uphold pointer validity,
    /// allocation lifetimes, and Rust's memory safety requirements. Guest metadata
    /// alone cannot establish these guarantees. The caller must trust the code.
    pub unsafe fn run_with_native(&self, limits: Limits) -> Result<Execution, Fault> {
        self.check_entry()?;
        crate::vm::interpret(
            &self.module,
            limits,
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

/// A resolved static IL function belonging to one immutable program snapshot.
#[derive(Debug)]
pub struct LoadedFunction<'program> {
    program: &'program LoadedProgram,
    function: crate::metadata::Function,
    definition: crate::metadata::MemberId,
}

impl LoadedFunction<'_> {
    pub fn definition(&self) -> &crate::metadata::MemberId {
        &self.definition
    }
    pub fn parameters(&self) -> &[Type] {
        &self.function.parameters
    }
    pub fn returns(&self) -> &Type {
        &self.function.returns
    }

    /// Invoke with exact primitive storage values and fresh guest state.
    pub fn invoke(&self, arguments: Vec<crate::Value>, limits: Limits) -> Result<Execution, Fault> {
        self.check_arguments(&arguments)?;
        crate::vm::interpret_function(
            &self.program.module,
            self.function.clone(),
            arguments,
            limits,
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
        limits: Limits,
    ) -> Result<Execution, Fault> {
        self.check_arguments(&arguments)?;
        crate::vm::interpret_function(
            &self.program.module,
            self.function.clone(),
            arguments,
            limits,
            Some(crate::interop::NativeLibraries::default()),
        )
    }

    fn check_arguments(&self, arguments: &[crate::Value]) -> Result<(), Fault> {
        let fault = |message| Fault {
            message,
            function: Some(self.function.name.clone()),
            instruction: None,
        };
        if arguments.len() != self.function.parameters.len() {
            return Err(fault(format!(
                "invocation expected {} arguments, got {}",
                self.function.parameters.len(),
                arguments.len()
            )));
        }
        for (index, (argument, expected)) in
            arguments.iter().zip(&self.function.parameters).enumerate()
        {
            if matches!(
                argument,
                crate::Value::Object { .. }
                    | crate::Value::Union { .. }
                    | crate::Value::Pointer(_)
                    | crate::Value::Reference { .. }
            ) || argument.ty() != *expected
            {
                return Err(fault(format!(
                    "invocation argument {index}: expected {expected:?}, got {:?}",
                    argument.ty()
                )));
            }
        }
        Ok(())
    }
}
