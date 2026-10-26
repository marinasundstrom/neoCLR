//! Publish generated instructions through the platform cache-maintenance contract.
use cranelift_jit::{JITBuilder, JITModule};

#[cfg(any(test, all(target_os = "macos", target_arch = "aarch64")))]
#[path = "jit_memory_fallback.rs"]
mod fallback;

pub(crate) fn new_module(builder: JITBuilder) -> JITModule {
    new_module_with_policy(builder, false)
}

pub(crate) fn new_lazy_module(builder: JITBuilder) -> JITModule {
    new_module_with_policy(builder, true)
}

#[cfg(any(test, all(target_os = "macos", target_arch = "aarch64")))]
fn arena_with_fallback() -> Box<dyn cranelift_jit::JITMemoryProvider> {
    Box::new(fallback::Fallback::new(
        cranelift_jit::ArenaMemoryProvider::new_with_size(64 * 1024 * 1024)
            .ok()
            .map(|p| Box::new(p) as Box<dyn cranelift_jit::JITMemoryProvider>),
        Box::new(cranelift_jit::SystemMemoryProvider::new()),
    ))
}

fn new_module_with_policy(builder: JITBuilder, lazy: bool) -> JITModule {
    #[cfg(test)]
    if let Some(value) = std::env::var_os("NEOCLR_TEST_JIT_ARENA") {
        let provider: Box<dyn cranelift_jit::JITMemoryProvider> = if value == "1" {
            Box::new(
                cranelift_jit::ArenaMemoryProvider::new_with_size(64 * 1024 * 1024)
                    .expect("reserve experimental JIT arena"),
            )
        } else if value == "fallback" {
            arena_with_fallback()
        } else if value == "system" {
            Box::new(cranelift_jit::SystemMemoryProvider::new())
        } else {
            panic!("NEOCLR_TEST_JIT_ARENA must be 1, fallback or system");
        };
        return new_with_provider(builder, provider);
    }
    // First-call compilation creates many separately finalized allocations.
    // The qualified Apple ARM64 policy bounds the reservation, then spills
    // without invalidating published code. Other hosts retain system placement.
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    if lazy {
        return new_with_provider(builder, arena_with_fallback());
    }
    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    let _ = lazy;
    new_with_provider(
        builder,
        Box::new(cranelift_jit::SystemMemoryProvider::new()),
    )
}

// Both placement policies retain the platform instruction-publication wrapper.
fn new_with_provider(
    mut builder: JITBuilder,
    provider: Box<dyn cranelift_jit::JITMemoryProvider>,
) -> JITModule {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    let provider = Box::new(apple::Memory::new(provider));
    builder.memory_provider(provider);
    JITModule::new(builder)
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
mod apple {
    use cranelift_jit::{BranchProtection, JITMemoryProvider};
    use cranelift_module::ModuleResult;
    use std::{ffi::c_void, io};

    unsafe extern "C" {
        fn sys_icache_invalidate(start: *const c_void, len: usize);
    }

    pub(super) struct Memory {
        inner: Box<dyn JITMemoryProvider>,
        pending_code: Vec<(*mut u8, usize)>,
    }

    impl Memory {
        pub(super) fn new(inner: Box<dyn JITMemoryProvider>) -> Self {
            Self {
                inner,
                pending_code: Vec::new(),
            }
        }
    }

    impl JITMemoryProvider for Memory {
        fn allocate_readexec(&mut self, size: usize, align: u64) -> io::Result<*mut u8> {
            let address = self.inner.allocate_readexec(size, align)?;
            if size != 0 {
                self.pending_code.push((address, size));
            }
            Ok(address)
        }

        fn allocate_readwrite(&mut self, size: usize, align: u64) -> io::Result<*mut u8> {
            self.inner.allocate_readwrite(size, align)
        }

        fn allocate_readonly(&mut self, size: usize, align: u64) -> io::Result<*mut u8> {
            self.inner.allocate_readonly(size, align)
        }

        unsafe fn free_memory(&mut self) {
            self.pending_code.clear();
            unsafe { self.inner.free_memory() };
        }

        fn finalize(&mut self, protection: BranchProtection) -> ModuleResult<()> {
            // Cranelift has completed relocation, but has not yet exposed the
            // finalized entry points. Its pinned cache helper is a no-op on
            // macOS ARM64; changing page permissions alone is insufficient.
            #[cfg(test)]
            let flush = std::env::var_os("NEOCLR_TEST_SKIP_JIT_ICACHE").is_none();
            #[cfg(not(test))]
            let flush = true;
            if flush {
                for &(address, size) in &self.pending_code {
                    // The provider owns this complete, live code allocation.
                    unsafe { sys_icache_invalidate(address.cast(), size) };
                }
            }
            self.inner.finalize(protection)?;
            self.pending_code.clear();
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use cranelift_jit::ArenaMemoryProvider;

        unsafe fn write_return(pointer: *mut u8, value: u32) {
            // mov w0, #value; ret. This test runs only on Apple ARM64.
            unsafe {
                pointer.cast::<u32>().write(0x52800000 | (value << 5));
                pointer.add(4).cast::<u32>().write(0xd65f03c0);
            }
        }

        #[test]
        fn arena_publication_exhaustion_and_reuse() {
            for value in 0..128 {
                let mut memory = Memory::new(Box::new(
                    ArenaMemoryProvider::new_with_size(64 * 1024).unwrap(),
                ));
                let first = memory.allocate_readexec(8, 4).unwrap();
                let writable = memory.allocate_readwrite(8, 8).unwrap();
                let readonly = memory.allocate_readonly(8, 8).unwrap();
                unsafe {
                    write_return(first, value);
                    writable.cast::<u64>().write(42);
                    readonly.cast::<u64>().write(73);
                }
                assert!(memory.allocate_readexec(128 * 1024, 4).is_err());
                assert_eq!(memory.pending_code.len(), 1);
                memory.finalize(BranchProtection::None).unwrap();
                assert!(memory.pending_code.is_empty());
                let call: unsafe extern "C" fn() -> u32 = unsafe { std::mem::transmute(first) };
                assert_eq!(unsafe { call() }, value);
                assert_eq!(unsafe { readonly.cast::<u64>().read() }, 73);
                unsafe { writable.cast::<u64>().write(99) };

                let second = memory.allocate_readexec(8, 4).unwrap();
                unsafe { write_return(second, value + 1) };
                memory.finalize(BranchProtection::None).unwrap();
                let next: unsafe extern "C" fn() -> u32 = unsafe { std::mem::transmute(second) };
                assert_eq!(unsafe { call() }, value);
                assert_eq!(unsafe { next() }, value + 1);
                assert_eq!(unsafe { writable.cast::<u64>().read() }, 99);
                // No callable pointer is used after owner teardown.
                unsafe {
                    memory.free_memory();
                    memory.free_memory();
                }
                assert!(memory.pending_code.is_empty());
            }
        }

        #[test]
        fn arena_spill_keeps_published_code_and_static_data_alive() {
            for value in 0..64 {
                let provider = super::super::fallback::Fallback::new(
                    Some(Box::new(
                        ArenaMemoryProvider::new_with_size(64 * 1024).unwrap(),
                    )),
                    Box::new(cranelift_jit::SystemMemoryProvider::new()),
                );
                let mut memory = Memory::new(Box::new(provider));
                let first = memory.allocate_readexec(8, 4).unwrap();
                let data = memory.allocate_readonly(8, 8).unwrap();
                unsafe {
                    write_return(first, value);
                    data.cast::<u64>().write(73);
                }
                memory.finalize(BranchProtection::None).unwrap();
                let large = memory.allocate_readwrite(128 * 1024, 8).unwrap();
                let second = memory.allocate_readexec(8, 4).unwrap();
                unsafe {
                    large.cast::<u64>().write(99);
                    write_return(second, value + 1);
                }
                memory.finalize(BranchProtection::None).unwrap();
                let call: unsafe extern "C" fn() -> u32 = unsafe { std::mem::transmute(first) };
                let next: unsafe extern "C" fn() -> u32 = unsafe { std::mem::transmute(second) };
                assert_eq!(unsafe { call() }, value);
                assert_eq!(unsafe { next() }, value + 1);
                assert_eq!(unsafe { data.cast::<u64>().read() }, 73);
                assert_eq!(unsafe { large.cast::<u64>().read() }, 99);
                unsafe {
                    memory.free_memory();
                    memory.free_memory();
                }
                assert!(memory.pending_code.is_empty());
            }
        }

        #[test]
        fn arena_partial_allocation_cleanup() {
            for _ in 0..128 {
                let mut memory = Memory::new(Box::new(
                    ArenaMemoryProvider::new_with_size(64 * 1024).unwrap(),
                ));
                memory.allocate_readexec(8, 4).unwrap();
                assert!(memory.allocate_readwrite(128 * 1024, 8).is_err());
                unsafe { memory.free_memory() };
                assert!(memory.pending_code.is_empty());
            }
        }
    }
}
