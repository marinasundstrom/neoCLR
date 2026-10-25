//! Publish generated instructions through the platform cache-maintenance contract.
use cranelift_jit::{JITBuilder, JITModule};

pub(crate) fn new_module(builder: JITBuilder) -> JITModule {
    #[allow(unused_mut)]
    let mut builder = builder;
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    builder.memory_provider(Box::new(apple::Memory::default()));
    JITModule::new(builder)
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
mod apple {
    use cranelift_jit::{BranchProtection, JITMemoryProvider, SystemMemoryProvider};
    use cranelift_module::ModuleResult;
    use std::{ffi::c_void, io};

    unsafe extern "C" {
        fn sys_icache_invalidate(start: *const c_void, len: usize);
    }

    pub(super) struct Memory {
        inner: SystemMemoryProvider,
        pending_code: Vec<(*mut u8, usize)>,
    }

    impl Default for Memory {
        fn default() -> Self {
            Self { inner: SystemMemoryProvider::new(), pending_code: Vec::new() }
        }
    }

    impl JITMemoryProvider for Memory {
        fn allocate_readexec(&mut self, size: usize, align: u64) -> io::Result<*mut u8> {
            let address = self.inner.allocate_readexec(size, align)?;
            if size != 0 { self.pending_code.push((address, size)); }
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
}
