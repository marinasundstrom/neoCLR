//! Explicit trusted native imports. No string marshalling or ownership inference.
use crate::{
    Fault, Value,
    memory::PointerHeap,
    metadata::{Function, Type},
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
            "PInvoke must be static with no IL body, locals, or implementation flags",
        ));
    }
    for name in [&import.library, &import.entry_point] {
        if name.is_empty() || name.contains('\0') {
            return Err(Fault::new("invalid native library or entry point"));
        }
    }
    for ty in &function.parameters {
        if *ty == Type::Void {
            return Err(Fault::new("Void is not a native argument type"));
        }
        ffi_type(ty)?;
    }
    ffi_type(&function.returns)?;
    Ok(())
}

#[derive(Debug, Default)]
pub struct NativeLibraries {
    libraries: HashMap<String, libloading::Library>,
}
impl NativeLibraries {
    pub fn loaded_count(&self) -> usize {
        self.libraries.len()
    }

    pub(crate) unsafe fn invoke(
        &mut self,
        function: &Function,
        values: Vec<Value>,
        memory: &PointerHeap,
    ) -> Result<Value, Fault> {
        validate(function)?;
        let import = function
            .pinvoke
            .as_ref()
            .ok_or_else(|| Fault::new("missing native import"))?;
        for value in &values {
            if let Value::Pointer(pointer) = value {
                memory.validate_native_pointer(pointer)?;
            }
        }
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
            self.libraries.insert(import.library.clone(), library);
        }
        let library = self
            .libraries
            .get(&import.library)
            .ok_or_else(|| Fault::new("missing loaded library"))?;
        // SAFETY: the import's symbol and C signature are part of the caller's contract.
        let symbol =
            unsafe { library.get::<unsafe extern "C" fn()>(import.entry_point.as_bytes()) }
                .map_err(|e| {
                    Fault::new(format!(
                        "cannot resolve native entry {}: {e}",
                        import.entry_point
                    ))
                })?;
        let function_pointer = CodePtr(*symbol as *mut c_void);
        let cif = Cif::new(
            function
                .parameters
                .iter()
                .map(ffi_type)
                .collect::<Result<Vec<_>, _>>()?,
            ffi_type(&function.returns)?,
        );
        // Pointer arguments must reference an address-sized slot, not the Pointer struct.
        let addresses: Vec<*mut c_void> = values
            .iter()
            .map(|value| match value {
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
                    Value::Pointer(_) => arg(&addresses[i]),
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
