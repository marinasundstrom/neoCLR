//! Experimental placement: retain arena allocations while spilling future requests.
use cranelift_jit::{BranchProtection, JITMemoryProvider};
use cranelift_module::ModuleResult;
use std::io;

pub(super) struct Fallback {
    primary: Option<Box<dyn JITMemoryProvider>>,
    secondary: Box<dyn JITMemoryProvider>,
    spilled: bool,
    freed: bool,
}

impl Fallback {
    pub(super) fn new(primary: Option<Box<dyn JITMemoryProvider>>, secondary: Box<dyn JITMemoryProvider>) -> Self {
        Self { primary, secondary, spilled: false, freed: false }
    }

    fn allocate(&mut self, size: usize, align: u64, kind: u8) -> io::Result<*mut u8> {
        if self.freed {
            return Err(io::Error::other("JIT placement owner already freed"));
        }
        fn request(provider: &mut dyn JITMemoryProvider, size: usize, align: u64, kind: u8) -> io::Result<*mut u8> {
            match kind {
                0 => provider.allocate_readexec(size, align),
                1 => provider.allocate_readwrite(size, align),
                2 => provider.allocate_readonly(size, align),
                _ => unreachable!(),
            }
        }
        if !self.spilled {
            if let Some(primary) = &mut self.primary {
                if let Ok(pointer) = request(primary.as_mut(), size, align, kind) {
                    return Ok(pointer);
                }
            }
            // Allocation failure does not invalidate previous arena addresses.
            // Stop retrying the exhausted/unavailable reservation on every request.
            self.spilled = true;
        }
        request(self.secondary.as_mut(), size, align, kind)
    }
}

impl JITMemoryProvider for Fallback {
    fn allocate_readexec(&mut self, size: usize, align: u64) -> io::Result<*mut u8> {
        self.allocate(size, align, 0)
    }
    fn allocate_readwrite(&mut self, size: usize, align: u64) -> io::Result<*mut u8> {
        self.allocate(size, align, 1)
    }
    fn allocate_readonly(&mut self, size: usize, align: u64) -> io::Result<*mut u8> {
        self.allocate(size, align, 2)
    }
    fn finalize(&mut self, protection: BranchProtection) -> ModuleResult<()> {
        // Never discard the primary after spill: it still owns live code/data.
        if let Some(primary) = &mut self.primary {
            primary.finalize(protection)?;
        }
        self.secondary.finalize(protection)
    }
    unsafe fn free_memory(&mut self) {
        if !self.freed {
            if let Some(mut primary) = self.primary.take() {
                unsafe { primary.free_memory() };
            }
            unsafe { self.secondary.free_memory() };
            self.freed = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};

    #[derive(Default)]
    struct Events { requests: Vec<u8>, finalizes: usize, frees: usize }
    struct Probe { events: Rc<RefCell<Events>>, successes: usize }
    impl Probe {
        fn request(&mut self, kind: u8) -> io::Result<*mut u8> {
            self.events.borrow_mut().requests.push(kind);
            if self.successes == 0 { return Err(io::Error::other("injected allocation failure")); }
            self.successes -= 1;
            Ok(std::ptr::NonNull::<u8>::dangling().as_ptr())
        }
    }
    impl JITMemoryProvider for Probe {
        fn allocate_readexec(&mut self, _: usize, _: u64) -> io::Result<*mut u8> { self.request(0) }
        fn allocate_readwrite(&mut self, _: usize, _: u64) -> io::Result<*mut u8> { self.request(1) }
        fn allocate_readonly(&mut self, _: usize, _: u64) -> io::Result<*mut u8> { self.request(2) }
        fn finalize(&mut self, _: BranchProtection) -> ModuleResult<()> { self.events.borrow_mut().finalizes += 1; Ok(()) }
        unsafe fn free_memory(&mut self) { self.events.borrow_mut().frees += 1; }
    }

    #[test]
    fn spill_retains_both_owners_and_stops_primary_retries() {
        let first = Rc::new(RefCell::new(Events::default()));
        let second = Rc::new(RefCell::new(Events::default()));
        let mut memory = Fallback::new(
            Some(Box::new(Probe { events: first.clone(), successes: 1 })),
            Box::new(Probe { events: second.clone(), successes: 3 }),
        );
        memory.allocate_readexec(8, 8).unwrap();
        memory.finalize(BranchProtection::None).unwrap();
        memory.allocate_readwrite(8, 8).unwrap();
        memory.allocate_readonly(8, 8).unwrap();
        memory.allocate_readexec(8, 8).unwrap();
        memory.finalize(BranchProtection::None).unwrap();
        assert_eq!(first.borrow().requests, [0, 1]);
        assert_eq!(second.borrow().requests, [1, 2, 0]);
        assert_eq!(first.borrow().finalizes, 2);
        assert_eq!(second.borrow().finalizes, 2);
        unsafe { memory.free_memory(); memory.free_memory(); }
        assert_eq!(first.borrow().frees, 1);
        assert_eq!(second.borrow().frees, 1);
        assert!(memory.allocate_readexec(8, 8).is_err());
    }

    #[test]
    fn unavailable_reservation_and_secondary_failure() {
        let events = Rc::new(RefCell::new(Events::default()));
        let mut memory = Fallback::new(None, Box::new(Probe { events: events.clone(), successes: 1 }));
        memory.allocate_readonly(8, 8).unwrap();
        assert!(memory.allocate_readexec(8, 8).unwrap_err().to_string().contains("injected"));
        unsafe { memory.free_memory() };
        assert_eq!(events.borrow().requests, [2, 0]);
        assert_eq!(events.borrow().frees, 1);
    }
}
