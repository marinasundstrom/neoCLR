//! Explicit trusted native imports with call-scoped input string marshalling.
use crate::{
    Fault, Value,
    memory::SharedMemory,
    metadata::{
        CallingConvention, Function, NativeImport, NativeMarshalling, NativeStringMarshalling, Type,
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
        marshalled_type(ty, parameter_marshalling(import, index, ty))?;
    }
    // String returns need an explicit allocator/length/lifetime contract first.
    if function.returns == Type::String {
        return Err(Fault::new(
            "native String returns require an ownership contract",
        ));
    }
    marshalled_type(&function.returns, import.return_marshalling)?;
    Ok(())
}

fn parameter_marshalling(
    import: &NativeImport,
    index: usize,
    ty: &Type,
) -> Option<NativeMarshalling> {
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
            _library: library.clone(),
            function_pointer,
            function: function.clone(),
        })
    }
}

pub(crate) struct NativeCall {
    _library: std::sync::Arc<libloading::Library>,
    function_pointer: unsafe extern "C" fn(),
    function: Function,
}
impl NativeCall {
    /// The caller holds the tracked argument borrow for the entire call.
    pub(crate) unsafe fn invoke(
        &self,
        values: Vec<Value>,
        memory: &SharedMemory,
    ) -> Result<Value, Fault> {
        let function = &self.function;
        let import = function.pinvoke.as_ref().expect("prepared native import");
        let function_pointer = CodePtr(self.function_pointer as *mut c_void);
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
                .map(|(i, ty)| marshalled_type(ty, parameter_marshalling(import, i, ty)))
                .collect::<Result<Vec<_>, _>>()?,
            marshalled_type(&function.returns, import.return_marshalling)?,
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
                    Value::Pointer(_) | Value::String(_) => arg(&addresses[i]),
                    _ => return Err(Fault::new("unsupported native argument value")),
                })
            })
            .collect::<Result<_, _>>()?;
        // SAFETY: slots match the validated CIF; all remain alive for the call, and
        // the library stays loaded. Caller guarantees the actual export matches
        // this signature and upholds all native pointer and lifetime requirements.
        unsafe {
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
                Type::Ptr(target) => {
                    let address: *mut c_void = cif.call(function_pointer, &args);
                    Value::Pointer(memory.pointer_at_address(address as usize, *target.clone()))
                }
                _ => return Err(Fault::new("unsupported native return type")),
            })
        }
    }
}
