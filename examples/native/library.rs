// A native library exposing a C ABI. It has no dependency on neoCLR.
#[unsafe(no_mangle)]
pub extern "C" fn neoclr_add(a: i32, b: i32) -> i32 {
    a.wrapping_add(b)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_set_i32(destination: *mut i32, value: i32) {
    // SAFETY: caller supplies a valid writable i32 pointer or null.
    if !destination.is_null() {
        unsafe { destination.write(value) };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn neoclr_identity_ptr(value: *mut i32) -> *mut i32 {
    value
}

#[unsafe(no_mangle)]
pub extern "C" fn neoclr_mixed(a: i8, b: u16, c: f32, d: f64) -> f64 {
    a as f64 + b as f64 + c as f64 + d
}

macro_rules! identity {
    ($name:ident, $ty:ty) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name(value: $ty) -> $ty {
            value
        }
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
pub extern "C" fn neoclr_static_ptr() -> *mut i32 {
    &raw mut NUMBER
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_read_i32(value: *const i32) -> i32 {
    // SAFETY: caller supplies a live initialized i32 pointer or null.
    if value.is_null() {
        0
    } else {
        unsafe { value.read() }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_utf8_len(value: *const std::ffi::c_char) -> usize {
    // SAFETY: caller supplies a terminated, readable C string for this call.
    unsafe { std::ffi::CStr::from_ptr(value) }.to_bytes().len()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_utf16_len(value: *const u16) -> usize {
    // SAFETY: caller supplies a terminated, readable UTF-16 buffer for this call.
    let mut length = 0;
    while unsafe { *value.add(length) } != 0 {
        length += 1;
    }
    length
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_utf8_pair(
    a: *const std::ffi::c_char,
    bias: i32,
    b: *const std::ffi::c_char,
) -> i32 {
    // SAFETY: caller keeps both terminated strings readable throughout the call.
    let a = unsafe { std::ffi::CStr::from_ptr(a) }.to_bytes();
    let b = unsafe { std::ffi::CStr::from_ptr(b) }.to_bytes();
    if a == "hé🦀".as_bytes() && b == "世界".as_bytes() {
        bias + 1
    } else {
        -1
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_utf16_check(value: *const u16) -> i32 {
    // SAFETY: caller supplies the UTF-16 encoding of hé🦀 plus its terminator.
    let expected = [0x68, 0xe9, 0xd83e, 0xdd80, 0];
    i32::from(unsafe { std::slice::from_raw_parts(value, expected.len()) } == expected)
}

#[unsafe(no_mangle)]
pub extern "system" fn neoclr_system_add(a: i32, b: i32) -> i32 {
    a + b
}

#[unsafe(no_mangle)]
pub extern "C" fn neoclr_bool8(value: u8) -> u8 {
    if value == 1 { 255 } else { 0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn neoclr_bool32(value: i32) -> i32 {
    if value == 1 { -7 } else { 0 }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_mixed_encoding(a: *const std::ffi::c_char, b: *const u16) -> i32 {
    // SAFETY: both explicit input buffers remain readable through the call.
    let narrow = unsafe { std::ffi::CStr::from_ptr(a) }.to_bytes();
    let wide = unsafe { std::slice::from_raw_parts(b, 5) };
    if narrow == "hé🦀".as_bytes() && wide == [0x68, 0xe9, 0xd83e, 0xdd80, 0] {
        42
    } else {
        -1
    }
}

// Address-returning exports exercise structural Function bindings independently of names.
#[unsafe(no_mangle)]
pub extern "C" fn neoclr_get_add() -> *const std::ffi::c_void {
    neoclr_add as *const std::ffi::c_void
}
#[unsafe(no_mangle)]
pub extern "C" fn neoclr_get_utf8_pair() -> *const std::ffi::c_void {
    neoclr_utf8_pair as *const std::ffi::c_void
}
#[unsafe(no_mangle)]
pub extern "C" fn neoclr_get_bool8() -> *const std::ffi::c_void {
    neoclr_bool8 as *const std::ffi::c_void
}
