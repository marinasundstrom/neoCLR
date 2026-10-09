//! Experimental native selection bounds, shared by lowering and code generation.
pub const TYPES: usize = 512;
pub const FUNCTIONS: usize = 1024;
// Closed collection adapters can use most of the existing function budget.
// Every clone still counts toward FUNCTIONS; this does not raise the image limit.
pub const FUNCTION_CLONES: usize = FUNCTIONS;
