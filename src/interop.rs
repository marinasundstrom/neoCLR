//! Explicit trusted native imports with call-scoped input string marshalling.
use crate::{
    Fault, Value,
    memory::SharedMemory,
    metadata::{
        CallingConvention, Function, NativeAbi, NativeMarshalling, NativeStringMarshalling, Type,
    },
};
use libffi::middle::{Arg, Cif, CodePtr, Type as FfiType, arg};
use std::{collections::HashMap, ffi::c_void, path::Path};

fn ffi_type(ty: &Type) -> Result<FfiType, Fault> {
    Ok(match ty {
        Type::Void => FfiType::void(),
        Type::SByte => FfiType::i8(),
        Type::Byte => FfiType::u8(),
        Type::Int16 => FfiType::i16(),
        Type::UInt16 => FfiType::u16(),
        Type::Int32 => FfiType::i32(),
        Type::UInt32 => FfiType::u32(),
        Type::Int64 => FfiType::i64(),
        Type::UInt64 => FfiType::u64(),
        Type::IntPtr => FfiType::isize(),
        Type::UIntPtr => FfiType::usize(),
        Type::Single => FfiType::f32(),
        Type::Double => FfiType::f64(),
        Type::Ptr(_) => FfiType::pointer(),
        _ => return Err(Fault::new(format!("unsupported native ABI type {ty:?}"))),
    })
}

pub(crate) fn validate(function: &Function) -> Result<(), Fault> {
    let import = function
        .pinvoke
        .as_ref()
        .ok_or_else(|| Fault::new("missing native import"))?;
    if function.instance
        || function.impl_flags != 0
        || !function.locals.is_empty()
        || !function.body.is_empty()
    {
        return Err(Fault::new(
            "native import must be static with no IL body, locals, or implementation flags",
        ));
    }
    for name in [&import.library, &import.entry_point] {
        if name.is_empty() || name.contains('\0') {
            return Err(Fault::new("invalid native library or entry point"));
        }
    }
    validate_abi(function, &NativeAbi::from(import))
}

fn validate_abi(function: &Function, import: &NativeAbi) -> Result<(), Fault> {
    validate_abi_depth(function, import, 0)
}

fn validate_abi_depth(function: &Function, import: &NativeAbi, depth: usize) -> Result<(), Fault> {
    if depth > 8 {
        return Err(Fault::new("native callable ABI nesting limit exceeded"));
    }
    for (index, abi) in &import.callback_parameters {
        let index = index
            .parse::<usize>()
            .ok()
            .filter(|i| i.to_string() == *index)
            .filter(|i| *i < function.parameters.len())
            .ok_or_else(|| Fault::new("native callback index outside signature"))?;
        let ty = &function.parameters[index];
        let nested = callable_contract(ty)?;
        // Reverse marshalling initially admits scalar values only. In particular,
        // a callback cannot hand native code a managed address or a temporary stub.
        if abi.string_marshalling.is_some()
            || !abi.callback_parameters.is_empty()
            || abi.function_return.is_some()
            || nested
                .parameters
                .iter()
                .chain([&nested.returns])
                .any(|ty| matches!(ty, Type::String | Type::Ptr(_) | Type::Function(_)))
        {
            return Err(Fault::new(
                "native callbacks currently require scalar signatures",
            ));
        }
        validate_abi_depth(&nested, abi, depth + 1)?;
        if import
            .parameter_marshalling
            .contains_key(&index.to_string())
        {
            return Err(Fault::new("callback parameter has conflicting marshalling"));
        }
    }
    if let Some(abi) = &import.function_return {
        if import.return_marshalling.is_some() {
            return Err(Fault::new(
                "native Function result has conflicting marshalling",
            ));
        }
        let nested = callable_contract(&function.returns)?;
        validate_abi_depth(&nested, abi, depth + 1)?;
    }

    if import.parameter_marshalling.keys().any(|key| {
        key.parse::<usize>().map_or(true, |index| {
            index >= function.parameters.len() || index.to_string() != *key
        })
    }) {
        return Err(Fault::new(
            "native parameter marshalling index outside signature",
        ));
    }
    for (index, ty) in function.parameters.iter().enumerate() {
        if *ty == Type::Void {
            return Err(Fault::new("Void is not a native argument type"));
        }
        argument_type(import, index, ty)?;
    }
    // String returns need an explicit allocator/length/lifetime contract first.
    if function.returns == Type::String {
        return Err(Fault::new(
            "native String returns require an ownership contract",
        ));
    }
    result_type(&function.returns, import)?;
    Ok(())
}

pub(crate) fn indirect_contract(
    module: &crate::Module,
    ty: &Type,
    abi: &NativeAbi,
) -> Result<Function, Fault> {
    let Type::Function(shape) = ty else {
        return Err(Fault::new("native binding requires a Function signature"));
    };
    shape.validate()?;
    if !shape.out_parameters.is_empty() || !shape.out_when_true.is_empty() {
        return Err(Fault::new(
            "native Function output contracts are not supported",
        ));
    }
    let mut function = crate::function_objects::contract(module, ty)?;
    function.instance = false;
    function.owner = None;
    validate_abi(&function, abi)?;
    Ok(function)
}

fn callable_contract(ty: &Type) -> Result<Function, Fault> {
    let Type::Function(shape) = ty else {
        return Err(Fault::new(
            "native callable mapping requires a Function signature",
        ));
    };
    shape.validate()?;
    if !shape.out_parameters.is_empty() || !shape.out_when_true.is_empty() {
        return Err(Fault::new(
            "native Function output contracts are not supported",
        ));
    }
    let mut function = crate::function_objects::structural_contract(ty)?;
    function.name = "$NativeFunction.Invoke".into();
    function.owner = None;
    function.instance = false;
    function.pinvoke = None;
    function.parameters = shape.parameters.clone();
    function.returns = shape.returns.clone();
    function.no_result = shape.no_result;
    Ok(function)
}

fn argument_type(abi: &NativeAbi, index: usize, ty: &Type) -> Result<FfiType, Fault> {
    if abi.callback_parameters.contains_key(&index.to_string()) {
        Ok(FfiType::pointer())
    } else {
        marshalled_type(ty, parameter_marshalling(abi, index, ty))
    }
}
fn result_type(ty: &Type, abi: &NativeAbi) -> Result<FfiType, Fault> {
    if abi.function_return.is_some() {
        Ok(FfiType::pointer())
    } else {
        marshalled_type(ty, abi.return_marshalling)
    }
}

#[path = "interop_callbacks.rs"]
mod callbacks;
pub(crate) type CallbackRunner<'a> = dyn FnMut(&Value, Vec<Value>) -> Result<Value, Fault> + 'a;

fn parameter_marshalling(import: &NativeAbi, index: usize, ty: &Type) -> Option<NativeMarshalling> {
    import
        .parameter_marshalling
        .get(&index.to_string())
        .copied()
        .or_else(|| {
            if *ty != Type::String {
                return None;
            }
            import.string_marshalling.map(|encoding| match encoding {
                NativeStringMarshalling::Utf8 => NativeMarshalling::Utf8,
                NativeStringMarshalling::Utf16 => NativeMarshalling::Utf16,
            })
        })
}

fn marshalled_type(ty: &Type, marshalling: Option<NativeMarshalling>) -> Result<FfiType, Fault> {
    match (ty, marshalling) {
        (Type::String, Some(NativeMarshalling::Utf8 | NativeMarshalling::Utf16)) => {
            Ok(FfiType::pointer())
        }
        (Type::Boolean, Some(NativeMarshalling::Bool8)) => Ok(FfiType::u8()),
        (Type::Boolean, Some(NativeMarshalling::Bool32)) => Ok(FfiType::i32()),
        (_, None) => ffi_type(ty),
        _ => Err(Fault::new("native marshalling does not match managed type")),
    }
}

fn native_abi(convention: CallingConvention) -> libffi::middle::FfiAbi {
    // System follows extern "system": stdcall only on 32-bit Windows, C elsewhere.
    #[cfg(all(windows, target_arch = "x86"))]
    if matches!(convention, CallingConvention::System) {
        return libffi::raw::ffi_abi_FFI_STDCALL;
    }
    let _ = convention;
    libffi::middle::ffi_abi_FFI_DEFAULT_ABI
}

// Separate owned buffers avoid exposing managed string storage to native code.
// Their allocations and address slots survive until the synchronous call returns.
enum NativeString {
    Utf8(std::ffi::CString),
    Utf16(Vec<u16>),
}
impl NativeString {
    fn new(text: &str, encoding: NativeMarshalling) -> Result<Self, Fault> {
        if text.contains('\0') {
            return Err(Fault::new("native input string contains an embedded NUL"));
        }
        Ok(match encoding {
            NativeMarshalling::Utf8 => Self::Utf8(std::ffi::CString::new(text).unwrap()),
            NativeMarshalling::Utf16 => Self::Utf16(text.encode_utf16().chain([0]).collect()),
            _ => return Err(Fault::new("native String requires a string encoding")),
        })
    }

    fn address(&self) -> *mut c_void {
        match self {
            Self::Utf8(value) => value.as_ptr() as *mut c_void,
            Self::Utf16(value) => value.as_ptr() as *mut c_void,
        }
    }
}

#[derive(Debug, Default)]
pub struct NativeLibraries {
    libraries: HashMap<String, std::sync::Arc<libloading::Library>>,
    symbols: HashMap<(String, String), unsafe extern "C" fn()>,
}
impl NativeLibraries {
    pub(crate) fn bind_pointer(
        &self,
        module: &crate::Module,
        ty: &Type,
        abi: &NativeAbi,
        pointer: &crate::memory::Pointer,
    ) -> Result<NativeCall, Fault> {
        let function = indirect_contract(module, ty, abi)?;
        if pointer.address == 0 {
            return Err(Fault::new("cannot bind a null native function pointer"));
        }
        if pointer.allocation.is_some() {
            return Err(Fault::new(
                "guest data storage cannot be bound as native code",
            ));
        }
        Ok(NativeCall {
            _libraries: self.libraries.values().cloned().collect(),
            address: pointer.address,
            function,
            abi: abi.clone(),
        })
    }

    pub fn loaded_count(&self) -> usize {
        self.libraries.len()
    }

    pub(crate) unsafe fn prepare(&mut self, function: &Function) -> Result<NativeCall, Fault> {
        validate(function)?;
        let import = function
            .pinvoke
            .as_ref()
            .ok_or_else(|| Fault::new("missing native import"))?;
        if !self.libraries.contains_key(&import.library) {
            // Extensionless names use the platform prefix/suffix, preserving the directory.
            let path = Path::new(&import.library);
            let path = if path.extension().is_none() {
                path.with_file_name(libloading::library_filename(
                    path.file_name()
                        .ok_or_else(|| Fault::new("invalid native library path"))?,
                ))
            } else {
                path.to_path_buf()
            };
            // SAFETY: caller trusts library initializers, as required by run_with_native.
            let library = unsafe { libloading::Library::new(&path) }.map_err(|e| {
                Fault::new(format!(
                    "cannot load native library {}: {e}",
                    path.display()
                ))
            })?;
            self.libraries
                .insert(import.library.clone(), std::sync::Arc::new(library));
        }
        let library = self
            .libraries
            .get(&import.library)
            .ok_or_else(|| Fault::new("missing loaded library"))?;
        let key = (import.library.clone(), import.entry_point.clone());
        let function_pointer = if let Some(pointer) = self.symbols.get(&key) {
            *pointer
        } else {
            // SAFETY: caller trusts the symbol and ABI; the owning library is retained.
            let symbol =
                unsafe { library.get::<unsafe extern "C" fn()>(import.entry_point.as_bytes()) }
                    .map_err(|e| {
                        Fault::new(format!(
                            "cannot resolve native entry {}: {e}",
                            import.entry_point
                        ))
                    })?;
            let pointer = *symbol;
            self.symbols.insert(key, pointer);
            pointer
        };
        Ok(NativeCall {
            _libraries: vec![library.clone()],
            address: function_pointer as usize,
            abi: NativeAbi::from(import),
            function: function.clone(),
        })
    }
}

#[derive(Debug)]
pub(crate) struct NativeCall {
    _libraries: Vec<std::sync::Arc<libloading::Library>>,
    address: usize,
    pub(crate) function: Function,
    abi: NativeAbi,
}
impl NativeCall {
    /// The caller holds the tracked argument borrow for the entire call.
    pub(crate) unsafe fn invoke(
        &self,
        values: Vec<Value>,
        memory: &SharedMemory,
        runner: &mut CallbackRunner<'_>,
    ) -> Result<Value, Fault> {
        let function = &self.function;
        let import = &self.abi;
        let function_pointer = CodePtr(self.address as *mut c_void);
        if values.len() != function.parameters.len() {
            return Err(Fault::new("native argument count mismatch"));
        }
        let values: Vec<Value> = values
            .into_iter()
            .enumerate()
            .map(|(i, value)| match value {
                Value::Boolean(value) => {
                    match parameter_marshalling(import, i, &function.parameters[i]) {
                        Some(NativeMarshalling::Bool8) => Ok(Value::Byte(u8::from(value))),
                        Some(NativeMarshalling::Bool32) => Ok(Value::Int32(i32::from(value))),
                        _ => Err(Fault::new("native Boolean requires explicit width")),
                    }
                }
                value => Ok(value),
            })
            .collect::<Result<_, _>>()?;
        let callback_runner = std::cell::RefCell::new(runner);
        let callback_fault = std::sync::Mutex::new(None);
        let contexts =
            callbacks::contexts(function, import, &values, &callback_runner, &callback_fault)?;
        let closures = contexts
            .iter()
            .map(callbacks::Closure::new)
            .collect::<Result<Vec<_>, _>>()?;
        let strings: Vec<Option<NativeString>> = values
            .iter()
            .enumerate()
            .map(|(i, value)| match value {
                Value::String(text) => {
                    let encoding = parameter_marshalling(import, i, &function.parameters[i])
                        .ok_or_else(|| Fault::new("native String requires explicit marshalling"))?;
                    NativeString::new(text.as_str(), encoding).map(Some)
                }
                _ => Ok(None),
            })
            .collect::<Result<_, _>>()?;
        let cif = Cif::new_with_abi(
            function
                .parameters
                .iter()
                .enumerate()
                .map(|(i, ty)| argument_type(import, i, ty))
                .collect::<Result<Vec<_>, _>>()?,
            result_type(&function.returns, import)?,
            native_abi(import.calling_convention),
        );
        // Pointer arguments must reference an address-sized slot, not the Pointer struct.
        let addresses: Vec<*mut c_void> = values
            .iter()
            .enumerate()
            .map(|(i, value)| match value {
                Value::String(_) => strings[i]
                    .as_ref()
                    .expect("marshalled input string")
                    .address(),
                Value::Pointer(p) => p.address as *mut c_void,
                Value::Function(_) => closures
                    .iter()
                    .find(|closure| closure.index == i)
                    .map_or(std::ptr::null_mut(), |closure| closure.address()),
                _ => std::ptr::null_mut(),
            })
            .collect();
        let args: Vec<Arg> = values
            .iter()
            .enumerate()
            .map(|(i, value)| {
                Ok(match value {
                    Value::SByte(n) => arg(n),
                    Value::Byte(n) => arg(n),
                    Value::Int16(n) => arg(n),
                    Value::UInt16(n) => arg(n),
                    Value::Int32(n) => arg(n),
                    Value::UInt32(n) => arg(n),
                    Value::Int64(n) => arg(n),
                    Value::UInt64(n) => arg(n),
                    Value::IntPtr(n) => arg(n),
                    Value::UIntPtr(n) => arg(n),
                    Value::Single(n) => arg(n),
                    Value::Double(n) => arg(n),
                    Value::Pointer(_) | Value::String(_) | Value::Function(_) => arg(&addresses[i]),
                    _ => return Err(Fault::new("unsupported native argument value")),
                })
            })
            .collect::<Result<_, _>>()?;
        // SAFETY: slots match the validated CIF; all remain alive for the call, and
        // the library stays loaded. Caller guarantees the actual export matches
        // this signature and upholds all native pointer and lifetime requirements.
        let result = (|| unsafe {
            Ok(match &function.returns {
                Type::Void => {
                    cif.call::<()>(function_pointer, &args);
                    Value::Void
                }
                Type::Boolean => match import.return_marshalling {
                    Some(NativeMarshalling::Bool8) => {
                        Value::Boolean(cif.call::<u8>(function_pointer, &args) != 0)
                    }
                    Some(NativeMarshalling::Bool32) => {
                        Value::Boolean(cif.call::<i32>(function_pointer, &args) != 0)
                    }
                    _ => return Err(Fault::new("native Boolean result requires explicit width")),
                },
                Type::SByte => Value::SByte(cif.call(function_pointer, &args)),
                Type::Byte => Value::Byte(cif.call(function_pointer, &args)),
                Type::Int16 => Value::Int16(cif.call(function_pointer, &args)),
                Type::UInt16 => Value::UInt16(cif.call(function_pointer, &args)),
                Type::Int32 => Value::Int32(cif.call(function_pointer, &args)),
                Type::UInt32 => Value::UInt32(cif.call(function_pointer, &args)),
                Type::Int64 => Value::Int64(cif.call(function_pointer, &args)),
                Type::UInt64 => Value::UInt64(cif.call(function_pointer, &args)),
                Type::IntPtr => Value::IntPtr(cif.call(function_pointer, &args)),
                Type::UIntPtr => Value::UIntPtr(cif.call(function_pointer, &args)),
                Type::Single => Value::Single(cif.call(function_pointer, &args)),
                Type::Double => Value::Double(cif.call(function_pointer, &args)),
                Type::Function(_) => {
                    let address: *mut c_void = cif.call(function_pointer, &args);
                    if address.is_null() {
                        return Err(Fault::new("native function returned a null callable"));
                    }
                    if closures.iter().any(|closure| closure.address() == address) {
                        return Err(Fault::new(
                            "native function cannot return a call-scoped callback pointer",
                        ));
                    }
                    if memory
                        .pointer_at_address(address as usize, Type::Void)
                        .allocation
                        .is_some()
                    {
                        return Err(Fault::new(
                            "guest data storage cannot be returned as native code",
                        ));
                    }
                    let abi = import
                        .function_return
                        .as_ref()
                        .ok_or_else(|| Fault::new("native Function result requires an ABI"))?;
                    crate::function_objects::bind_native(
                        &function.returns,
                        NativeCall {
                            _libraries: self._libraries.clone(),
                            address: address as usize,
                            function: callable_contract(&function.returns)?,
                            abi: *abi.clone(),
                        },
                    )
                }
                Type::Ptr(target) => {
                    let address: *mut c_void = cif.call(function_pointer, &args);
                    Value::Pointer(memory.pointer_at_address(address as usize, *target.clone()))
                }
                _ => return Err(Fault::new("unsupported native return type")),
            })
        })();
        if let Some(fault) = callback_fault
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()
        {
            return Err(fault);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_binding_retains_library_after_session_drop() {
        let directory =
            std::env::temp_dir().join(format!("neoclr-binding-owner-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join(libloading::library_filename("binding_owner"));
        assert!(
            std::process::Command::new("rustc")
                .args([
                    "--edition=2024",
                    "--crate-type=cdylib",
                    "--crate-name=binding_owner"
                ])
                .arg(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("examples/native/library.rs")
                )
                .arg("-o")
                .arg(&path)
                .status()
                .unwrap()
                .success()
        );
        // SAFETY: this is the test fixture compiled above; its initializers and export
        // signature are controlled by this test. No arbitrary pointer is executed.
        let library = std::sync::Arc::new(unsafe { libloading::Library::new(&path).unwrap() });
        let address = unsafe {
            *library
                .get::<unsafe extern "C" fn()>(b"neoclr_add")
                .unwrap()
        } as usize;
        let owner = std::sync::Arc::downgrade(&library);
        let mut libraries = NativeLibraries::default();
        libraries.libraries.insert("binding_owner".into(), library);
        let ty = crate::assembler::parse_type("fn<Int32,Int32,Int32>").unwrap();
        let abi: NativeAbi = serde_json::from_str(r#"{"calling_convention":"Cdecl"}"#).unwrap();
        let module = crate::assemble(".module Test").unwrap();
        let call = libraries
            .bind_pointer(
                &module,
                &ty,
                &abi,
                &crate::memory::Pointer {
                    address,
                    allocation: None,
                    offset: 0,
                    target: Type::Void,
                },
            )
            .unwrap();
        drop(libraries);
        assert!(owner.upgrade().is_some());
        drop(call);
        assert!(owner.upgrade().is_none());
        std::fs::remove_dir_all(directory).unwrap();
    }
}
