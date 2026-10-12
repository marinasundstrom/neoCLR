//! Experimental native selection bounds, shared by lowering and code generation.
// HTTP + JSON + managed TLS requires more than the original small POC budget.
pub const TYPES: usize = 512;
pub const FUNCTIONS: usize = 2048;
// Every generic clone also counts toward FUNCTIONS.
pub const FUNCTION_CLONES: usize = FUNCTIONS;
pub const INPUT_TYPES: usize = 1024;
pub const INPUT_FUNCTIONS: usize = 8192;
