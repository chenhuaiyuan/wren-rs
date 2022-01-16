use std::collections::HashMap;
use std::ffi::CString;

use ffi::WrenHandle;

use crate::ffi;
use crate::{InterpretResult, SlotType};

fn default_write(_: &mut VM, text: &str) {
    print!("{}", text);
}

fn default_error(_: &mut VM, _type: ffi::WrenErrorType, module: &str, line: i32, message: &str) {
    match _type {
        ffi::WrenErrorType::Compile => println!("[{} line {}] {}", module, line, message),
        ffi::WrenErrorType::Runtime => println!("{}", message),
        ffi::WrenErrorType::StackTrace => println!("[{} line {}] in {}", module, line, message),
    }
}

fn default_load_module(_: &mut VM, module: &str) -> Vec<u8> {
    use std::fs;
    use std::path::PathBuf;

    let mut path = PathBuf::from(module);
    path.set_extension("wren");
    fs::read(path).unwrap()
}

enum PathType {
    Absolute,
    Relative,
    Simple,
}

impl PartialEq for PathType {
    #[inline]
    fn eq(&self, other: &PathType) -> bool {
        matches!(
            (self, other),
            (PathType::Absolute, PathType::Absolute)
                | (PathType::Relative, PathType::Relative)
                | (PathType::Simple, PathType::Simple)
        )
    }
}

#[inline]
fn is_separator(c: char) -> bool {
    if c == '/' {
        return true;
    }

    if cfg!(target_os = "windows") && c == '\\' {
        return true;
    }
    false
}

fn path_type(path: &[u8]) -> PathType {
    let first = *path.first().unwrap();
    let first = char::from(first);
    let second = *path.get(1).unwrap();
    let second = char::from(second);
    let third = *path.get(2).unwrap();
    let third = char::from(third);
    if cfg!(target_os = "windows") && first.is_ascii_alphabetic() && second == ':' {
        return PathType::Absolute;
    }

    if is_separator(first) {
        return PathType::Absolute;
    }

    if first == '.' && is_separator(second) || first == '.' && second == '.' && is_separator(third)
    {
        return PathType::Relative;
    }

    PathType::Simple
}

fn default_resolve_module(_: &mut VM, module: &str, importer: &str) -> Vec<u8> {
    use std::path::PathBuf;
    if path_type(importer.as_bytes()) == PathType::Simple {
        return Vec::from(importer);
    }
    let mut path = PathBuf::from(module);
    path.pop();

    path.push(importer);
    let p = path.to_str().unwrap_or("");
    Vec::from(p)
}

// fn resolve_module(_: &mut VM, module: &str, importer: &str) -> String {}
pub struct VM(*mut ffi::WrenVM);
impl VM {
    pub fn new(config: &mut Configuration) -> VM {
        let raw = unsafe { ffi::wrenNewVM(&mut config.0) };
        VM(raw)
    }
    pub fn from_ptr(ptr: *mut ffi::WrenVM) -> VM {
        VM(ptr)
    }
    pub fn interpret<S: Into<Vec<u8>>>(&mut self, module: &str, source: S) -> InterpretResult {
        let module = CString::new(module).unwrap();
        let source = CString::new(source.into()).unwrap();
        unsafe { ffi::wrenInterpret(self.0, module.as_ptr(), source.as_ptr()) }
    }
    pub fn close(&mut self) {
        unsafe { ffi::wrenFreeVM(self.0) }
    }
    pub fn version() {
        let v = unsafe { ffi::wrenGetVersionNumber() };
        println!("Wren Version: {}", v);
    }
    pub fn collect_garbage(&mut self) {
        unsafe { ffi::wrenCollectGarbage(self.0) }
    }
    pub fn make_call_handle(&mut self, signature: &str) -> Handle {
        let signature = CString::new(signature).unwrap();
        let handle = unsafe { ffi::wrenMakeCallHandle(self.0, signature.as_ptr()) };
        Handle { handle, vm: self.0 }
    }
    pub fn Get_slot_count(&mut self) -> i32 {
        unsafe { ffi::wrenGetSlotCount(self.0) }
    }
    pub fn slot(&mut self, num_slots: i32) {
        unsafe {
            ffi::wrenEnsureSlots(self.0, num_slots);
        }
    }
    pub fn get_slot_type(&mut self, slot: i32) -> SlotType {
        unsafe { ffi::wrenGetSlotType(self.0, slot) }
    }
}

pub struct Handle {
    handle: *mut ffi::WrenHandle,
    vm: *mut ffi::WrenVM,
}
impl Handle {
    pub fn call(&mut self) {
        unsafe {
            ffi::wrenCall(self.vm, self.handle);
        }
    }
    // pub fn call<T>(&mut self, argv: T) {}

    pub fn close(&mut self) {
        unsafe {
            ffi::wrenReleaseHandle(self.vm, self.handle);
        }
    }
}

pub struct Object(*mut WrenHandle);

pub struct Configuration(ffi::WrenConfiguration);

impl Configuration {
    pub fn new() -> Configuration {
        let config = std::mem::MaybeUninit::<ffi::WrenConfiguration>::uninit();
        let mut config = unsafe { config.assume_init() };
        unsafe { ffi::wrenInitConfiguration(&mut config) }
        let mut cfg = Configuration(config);
        cfg.set_write_fn(wren_write_fn!(default_write));
        cfg.set_error_fn(wren_error_fn!(default_error));
        cfg.set_load_module_fn(wren_load_module_fn!(default_load_module));
        cfg.set_resolve_module_fn(wren_resolve_module_fn!(default_resolve_module));
        cfg
    }
    pub fn set_reallocate_fn(&mut self, f: ffi::WrenReallocateFn) {
        self.0.reallocate_fn = f;
    }
    pub fn set_write_fn(&mut self, f: ffi::WrenWriteFn) {
        self.0.write_fn = f;
    }
    pub fn set_error_fn(&mut self, f: ffi::WrenErrorFn) {
        self.0.error_fn = f;
    }
    pub fn set_resolve_module_fn(&mut self, f: ffi::WrenResolveModuleFn) {
        self.0.resolve_module_fn = f;
    }
    pub fn set_load_module_fn(&mut self, f: ffi::WrenLoadModuleFn) {
        self.0.load_module_fn = f;
    }
    pub fn set_bind_foreign_method_fn(&mut self, f: ffi::WrenBindForeignMethodFn) {
        self.0.bind_foreign_method_fn = f;
    }
    pub fn set_bind_foreign_class_fn(&mut self, f: ffi::WrenBindForeignClassFn) {
        self.0.bind_foreign_class_fn = f;
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self::new()
    }
}
