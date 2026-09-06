// A native library exposing a C ABI. It has no dependency on neoCLR.
#[unsafe(no_mangle)]
pub extern "C" fn neoclr_add(a: i32, b: i32) -> i32 { a.wrapping_add(b) }

#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_set_i32(destination: *mut i32, value: i32) {
    // SAFETY: caller supplies a valid writable i32 pointer or null.
    if !destination.is_null() { unsafe { destination.write(value) }; }
}

#[unsafe(no_mangle)]
pub extern "C" fn neoclr_identity_ptr(value: *mut i32) -> *mut i32 { value }

#[unsafe(no_mangle)]
pub extern "C" fn neoclr_mixed(a: i8, b: u16, c: f32, d: f64) -> f64 {
    a as f64 + b as f64 + c as f64 + d
}

macro_rules! identity {
    ($name:ident, $ty:ty) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name(value: $ty) -> $ty { value }
    };
}
identity!(neoclr_i8, i8);
identity!(neoclr_u8, u8);
identity!(neoclr_i16, i16);
identity!(neoclr_u16, u16);
identity!(neoclr_i32, i32);
identity!(neoclr_u32, u32);
identity!(neoclr_i64, i64);
identity!(neoclr_u64, u64);
identity!(neoclr_isize, isize);
identity!(neoclr_usize, usize);
identity!(neoclr_f32, f32);
identity!(neoclr_f64, f64);

static mut NUMBER: i32 = 73;
#[unsafe(no_mangle)]
pub extern "C" fn neoclr_static_ptr() -> *mut i32 { &raw mut NUMBER }

#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_read_i32(value: *const i32) -> i32 {
    // SAFETY: caller supplies a live initialized i32 pointer or null.
    if value.is_null() { 0 } else { unsafe { value.read() } }
}
