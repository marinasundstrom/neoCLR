//! Experimental native selection bounds, shared by lowering and code generation.
// HTTP + JSON + managed TLS requires more than the original small POC budget.
// Match NEOCLR_NATIVE_TYPE_LIMIT in the native host header.
pub const TYPES: usize = 1024;
pub const FUNCTIONS: usize = 2048;
// Every generic clone also counts toward FUNCTIONS.
pub const FUNCTION_CLONES: usize = FUNCTIONS;
pub const INPUT_TYPES: usize = 1024;
pub const INPUT_FUNCTIONS: usize = 8192;
// Match NEOCLR_NATIVE_LAYOUT_LANES in the native host header.
pub const INLINE_LANES: usize = 128;
pub const OBJECT_LANES: usize = INLINE_LANES;

#[test]
fn type_budget_matches_native_collector_contract() {
    let header = include_str!("../../../docs/experiments/aot-console/text-arena.h");
    assert!(header.contains(&format!("#define NEOCLR_NATIVE_TYPE_LIMIT {TYPES}")));
}
