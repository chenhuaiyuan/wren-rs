use crate::ffi;
pub struct VM {
    raw: *mut ffi::WrenVM,
}
pub struct Configuration(ffi::WrenConfiguration);

impl Configuration {
    // pub fn new() -> Configuration {}
}
