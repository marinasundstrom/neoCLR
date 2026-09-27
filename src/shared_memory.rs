use crate::memory::{Layout, Pointer, PointerHeap};
use crate::{Fault, Value, metadata::Type};

/// One invocation's native allocation identities and quotas. Never hold this
/// mutex during host I/O; foreign calls use a short-lived allocation borrow.
#[derive(Clone)]
pub(crate) struct SharedMemory {
    heap: std::sync::Arc<std::sync::Mutex<PointerHeap>>,
    byte_limit: usize,
    allocation_limit: usize,
}
impl Default for SharedMemory {
    fn default() -> Self {
        Self::new(crate::Limits::default())
    }
}
impl SharedMemory {
    pub(crate) fn new(limits: crate::Limits) -> Self {
        Self {
            heap: Default::default(),
            byte_limit: limits.pointer_bytes,
            allocation_limit: limits.pointer_allocations,
        }
    }
    pub(crate) fn into_heap(self) -> Result<PointerHeap, Fault> {
        std::sync::Arc::try_unwrap(self.heap)
            .map_err(|_| Fault::new("native memory still has active contexts or foreign calls"))?
            .into_inner()
            .map_err(|_| Fault::new("native heap lock poisoned"))
    }
    fn lock(&self) -> std::sync::MutexGuard<'_, PointerHeap> {
        self.heap.lock().expect("native heap lock poisoned")
    }
    pub(crate) fn allocate(
        &self,
        ty: Type,
        layout: &Layout,
        count: usize,
        bytes: usize,
        allocations: usize,
    ) -> Result<Pointer, Fault> {
        self.lock().allocate(
            ty,
            layout,
            count,
            bytes.min(self.byte_limit),
            allocations.min(self.allocation_limit),
        )
    }
    pub(crate) fn allocate_local(
        &self,
        size: usize,
        bytes: usize,
        allocations: usize,
    ) -> Result<Pointer, Fault> {
        self.lock().allocate_local(
            size,
            bytes.min(self.byte_limit),
            allocations.min(self.allocation_limit),
        )
    }
    pub(crate) fn pointer_at_address(&self, address: usize, target: Type) -> Pointer {
        self.lock().pointer_at_address(address, target)
    }
    pub(crate) fn live_allocations(&self) -> usize {
        self.lock().live_allocations()
    }
    pub(crate) fn debug_allocations(&self) -> Vec<crate::debugger::NativeAllocation> {
        self.lock().debug_allocations()
    }
    pub(crate) fn read(&self, pointer: &Pointer, layout: &Layout) -> Result<Value, Fault> {
        self.lock().read(pointer, layout)
    }
    pub(crate) fn write(
        &self,
        pointer: &Pointer,
        layout: &Layout,
        value: &Value,
    ) -> Result<(), Fault> {
        self.lock().write(pointer, layout, value)
    }
    pub(crate) fn free(&self, pointer: &Pointer) -> Result<(), Fault> {
        self.lock().free(pointer)
    }
    pub(crate) fn release_local(&self, pointer: &Pointer) -> Result<(), Fault> {
        self.lock().release_local(pointer)
    }
    pub(crate) fn offset(&self, pointer: &Pointer, bytes: isize) -> Result<Pointer, Fault> {
        self.lock().offset(pointer, bytes)
    }
    pub(crate) fn field(
        &self,
        pointer: &Pointer,
        layout: &Layout,
        index: usize,
    ) -> Result<Pointer, Fault> {
        self.lock().field(pointer, layout, index)
    }
    pub(crate) fn fill(&self, pointer: &Pointer, layout: &Layout, byte: u8) -> Result<(), Fault> {
        self.lock().fill(pointer, layout, byte)
    }
    pub(crate) fn copy_block(
        &self,
        destination: &Pointer,
        source: &Pointer,
        layout: &Layout,
    ) -> Result<(), Fault> {
        self.lock().copy_block(destination, source, layout)
    }

    /// Validate every argument before borrowing any allocation. Untracked external
    /// pointers and native code's indirect/global accesses remain the trusted ABI
    /// caller's responsibility; no ownership can be inferred for those addresses.
    pub(crate) fn borrow_native(&self, values: &[Value]) -> Result<NativeBorrow, Fault> {
        let mut heap = self.lock();
        let mut ids = Vec::new();
        for value in values {
            if let Value::Pointer(pointer) = value {
                let recovered;
                let pointer = if pointer.allocation.is_none() {
                    recovered = heap.pointer_at_address(pointer.address, pointer.target.clone());
                    &recovered
                } else {
                    pointer
                };
                heap.validate_native_pointer(pointer)?;
                if let Some(id) = pointer.allocation {
                    if !ids.contains(&id) {
                        ids.push(id);
                    }
                }
            }
        }
        for id in &ids {
            heap.allocations[*id].as_mut().unwrap().native_borrowed = true;
        }
        Ok(NativeBorrow {
            memory: self.clone(),
            ids,
        })
    }
}
pub(crate) struct NativeBorrow {
    memory: SharedMemory,
    ids: Vec<usize>,
}
impl Drop for NativeBorrow {
    fn drop(&mut self) {
        let mut heap = self.memory.lock();
        for id in &self.ids {
            heap.allocations[*id]
                .as_mut()
                .expect("borrowed allocation remains live")
                .native_borrowed = false;
        }
    }
}

#[cfg(test)]
#[path = "shared_memory_tests.rs"]
mod tests;
