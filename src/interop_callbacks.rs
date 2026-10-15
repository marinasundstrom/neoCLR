//! Call-scoped libffi entry stubs. No managed fault or Rust panic crosses C.
use super::*;
use std::{cell::RefCell, marker::PhantomData, sync::Mutex, thread::ThreadId};

pub(super) struct Context<'a, 'r, 'e> {
    index: usize,
    value: &'a Value,
    function: Function,
    abi: &'a NativeAbi,
    runner: &'a RefCell<&'r mut CallbackRunner<'e>>,
    fault: &'a Mutex<Option<Fault>>,
    thread: ThreadId,
}

pub(super) fn contexts<'a, 'r, 'e>(
    function: &Function,
    abi: &'a NativeAbi,
    values: &'a [Value],
    runner: &'a RefCell<&'r mut CallbackRunner<'e>>,
    fault: &'a Mutex<Option<Fault>>,
) -> Result<Vec<Context<'a, 'r, 'e>>, Fault> {
    abi.callback_parameters
        .iter()
        .map(|(key, abi)| {
            let index: usize = key
                .parse()
                .map_err(|_| Fault::new("invalid callback index"))?;
            let value = &values[index];
            let Value::Function(binding) = value else {
                return Err(Fault::new("native callback requires a non-null Function"));
            };
            if binding.view.is_some() || binding.ty != function.parameters[index] {
                return Err(Fault::new("native callback Function signature mismatch"));
            }
            Ok(Context {
                index,
                value,
                function: callable_contract(&binding.ty)?,
                abi,
                runner,
                fault,
                thread: std::thread::current().id(),
            })
        })
        .collect()
}

pub(super) struct Closure<'a> {
    pub(super) index: usize,
    _cif: Box<Cif>,
    allocation: *mut libffi::low::ffi_closure,
    code: CodePtr,
    _context: PhantomData<&'a ()>,
}
impl<'a> Closure<'a> {
    pub(super) fn new(context: &'a Context<'_, '_, '_>) -> Result<Self, Fault> {
        let cif = Box::new(Cif::new_with_abi(
            context
                .function
                .parameters
                .iter()
                .enumerate()
                .map(|(i, ty)| marshalled_type(ty, parameter_marshalling(context.abi, i, ty)))
                .collect::<Result<Vec<_>, _>>()?,
            result_type(&context.function.returns, context.abi)?,
            native_abi(context.abi.calling_convention),
        ));
        let (allocation, code) = libffi::low::try_closure_alloc()
            .ok_or_else(|| Fault::new("cannot allocate native callback stub"))?;
        let closure = Self {
            index: context.index,
            _cif: cif,
            allocation,
            code,
            _context: PhantomData,
        };
        // SAFETY: the boxed CIF and borrowed context outlive the closure. The raw
        // entry handles void without constructing a reference to a null result slot.
        let status = unsafe {
            libffi::raw::ffi_prep_closure_loc(
                allocation,
                closure._cif.as_raw_ptr(),
                Some(entry),
                (context as *const Context<'_, '_, '_>).cast_mut().cast(),
                code.as_mut_ptr(),
            )
        };
        if status != libffi::raw::ffi_status_FFI_OK {
            return Err(Fault::new("cannot prepare native callback ABI"));
        }
        Ok(closure)
    }
    pub(super) fn address(&self) -> *mut c_void {
        self.code.as_mut_ptr()
    }
}
impl Drop for Closure<'_> {
    fn drop(&mut self) {
        // SAFETY: this allocation is owned exactly once and native use has ended.
        unsafe { libffi::low::closure_free(self.allocation) }
    }
}

unsafe extern "C" fn entry(
    _cif: *mut libffi::low::ffi_cif,
    result: *mut c_void,
    arguments: *mut *mut c_void,
    data: *mut c_void,
) {
    // SAFETY: libffi receives this pointer only while Context is alive. On other
    // threads we inspect immutable metadata and the synchronized fault slot only;
    // the non-Send interpreter runner is never accessed there.
    let context = unsafe { &*data.cast::<Context<'_, '_, '_>>() };
    let outcome =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<(), Fault> {
            unsafe { zero_result(result, &context.function.returns) };
            if std::thread::current().id() != context.thread {
                return Err(Fault::new(
                    "call-scoped native callback requires the calling thread",
                ));
            }
            if context
                .fault
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .is_some()
            {
                return Ok(());
            }
            let mut runner = context
                .runner
                .try_borrow_mut()
                .map_err(|_| Fault::new("native callback runner is already active"))?;
            let values = context
                .function
                .parameters
                .iter()
                .enumerate()
                .map(|(index, ty)| {
                    // SAFETY: the trusted native caller supplies slots matching the CIF.
                    unsafe {
                        read_argument(
                            *arguments.add(index),
                            ty,
                            parameter_marshalling(context.abi, index, ty),
                        )
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            let value = runner(context.value, values)?;
            unsafe {
                write_result(
                    result,
                    value,
                    &context.function.returns,
                    context.abi.return_marshalling,
                )
            }
        }));
    let fault = match outcome {
        Ok(Ok(())) => return,
        Ok(Err(fault)) => fault,
        Err(_) => Fault::new("panic while executing native callback"),
    };
    let mut recorded = context.fault.lock().unwrap_or_else(|e| e.into_inner());
    if recorded.is_none() {
        *recorded = Some(fault);
    }
}

unsafe fn read_argument(
    address: *mut c_void,
    ty: &Type,
    mapping: Option<NativeMarshalling>,
) -> Result<Value, Fault> {
    macro_rules! read {
        ($rust:ty, $variant:ident) => {
            Value::$variant(unsafe { address.cast::<$rust>().read() })
        };
    }
    Ok(match ty {
        Type::SByte => read!(i8, SByte),
        Type::Byte => read!(u8, Byte),
        Type::Int16 => read!(i16, Int16),
        Type::UInt16 => read!(u16, UInt16),
        Type::Int32 => read!(i32, Int32),
        Type::UInt32 => read!(u32, UInt32),
        Type::Int64 => read!(i64, Int64),
        Type::UInt64 => read!(u64, UInt64),
        Type::IntPtr => read!(isize, IntPtr),
        Type::UIntPtr => read!(usize, UIntPtr),
        Type::Single => read!(f32, Single),
        Type::Double => read!(f64, Double),
        Type::Boolean => Value::Boolean(match mapping {
            Some(NativeMarshalling::Bool8) => unsafe { address.cast::<u8>().read() != 0 },
            Some(NativeMarshalling::Bool32) => unsafe { address.cast::<i32>().read() != 0 },
            _ => return Err(Fault::new("callback Boolean requires explicit width")),
        }),
        _ => return Err(Fault::new("unsupported callback argument")),
    })
}

unsafe fn zero_result(result: *mut c_void, ty: &Type) {
    // libffi requires integer closure returns narrower than ffi_arg to be widened.
    // Float slots keep their declared width; a void result need not have storage.
    unsafe {
        match ty {
            Type::Void => (),
            Type::Single => result.cast::<f32>().write(0.0),
            Type::Double => result.cast::<f64>().write(0.0),
            Type::Int64 | Type::UInt64 => result.cast::<u64>().write(0),
            _ => result.cast::<libffi::low::ffi_arg>().write(0),
        }
    }
}
unsafe fn write_result(
    result: *mut c_void,
    value: Value,
    ty: &Type,
    mapping: Option<NativeMarshalling>,
) -> Result<(), Fault> {
    if value.ty() != *ty {
        return Err(Fault::new("native callback result type mismatch"));
    }
    unsafe {
        match value {
            Value::Void => (),
            Value::SByte(n) => result.cast::<libffi::low::ffi_sarg>().write(n as _),
            Value::Int16(n) => result.cast::<libffi::low::ffi_sarg>().write(n as _),
            Value::Int32(n) => result.cast::<libffi::low::ffi_sarg>().write(n as _),
            Value::Byte(n) => result.cast::<libffi::low::ffi_arg>().write(n as _),
            Value::UInt16(n) => result.cast::<libffi::low::ffi_arg>().write(n as _),
            Value::UInt32(n) => result.cast::<libffi::low::ffi_arg>().write(n as _),
            Value::Int64(n) => result.cast::<i64>().write(n),
            Value::UInt64(n) => result.cast::<u64>().write(n),
            Value::IntPtr(n) => result.cast::<isize>().write(n),
            Value::UIntPtr(n) => result.cast::<usize>().write(n),
            Value::Single(n) => result.cast::<f32>().write(n),
            Value::Double(n) => result.cast::<f64>().write(n),
            Value::Boolean(n) => match mapping {
                Some(NativeMarshalling::Bool8 | NativeMarshalling::Bool32) => {
                    result.cast::<libffi::low::ffi_arg>().write(n as _)
                }
                _ => return Err(Fault::new("callback Boolean requires explicit width")),
            },
            _ => return Err(Fault::new("unsupported callback result")),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callback_panic_is_contained_and_subsequent_entry_is_suppressed() {
        let ty = crate::assembler::parse_type("fn<Int32,Int32>").unwrap();
        let value = Value::Function(crate::function_objects::FunctionObject {
            ty: ty.clone(),
            view: None,
            identity: std::sync::Arc::new(()),
            target: None,
            native: None,
            receiver: None,
        });
        let abi = serde_json::from_str(r#"{"calling_convention":"Cdecl"}"#).unwrap();
        let fault = Mutex::new(None);
        let calls = std::cell::Cell::new(0);
        // This boundary test substitutes a runner: the Function's managed target
        // is deliberately unused. The executable libffi closure itself is real.
        let mut runner = |_: &Value, _: Vec<Value>| -> Result<Value, Fault> {
            calls.set(calls.get() + 1);
            panic!("callback boundary test");
        };
        let runner: &mut CallbackRunner<'_> = &mut runner;
        let runner = RefCell::new(runner);
        let context = Context {
            index: 0,
            value: &value,
            function: callable_contract(&ty).unwrap(),
            abi: &abi,
            runner: &runner,
            fault: &fault,
            thread: std::thread::current().id(),
        };
        let closure = Closure::new(&context).unwrap();
        // SAFETY: the stub was prepared with precisely this scalar C signature,
        // and all its context and CIF storage remains live through both calls.
        let invoke: unsafe extern "C" fn(i32) -> i32 =
            unsafe { std::mem::transmute(closure.address()) };
        assert_eq!(unsafe { invoke(42) }, 0);
        assert_eq!(unsafe { invoke(42) }, 0);
        assert_eq!(calls.get(), 1);
        assert!(
            fault
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .message
                .contains("panic")
        );
    }
}
