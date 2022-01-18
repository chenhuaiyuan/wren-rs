use libc::{c_char, c_int};
use std::ffi::{CStr, CString};
use std::{mem, slice};

use crate::ffi;
use crate::{InterpretResult, Point, Type};

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
        Handle {
            raw: handle,
            vm: self.0,
        }
    }
    pub fn get_slot_count(&mut self) -> i32 {
        unsafe { ffi::wrenGetSlotCount(self.0) }
    }
    pub fn ensure_slots(&mut self, num_slots: i32) {
        unsafe {
            ffi::wrenEnsureSlots(self.0, num_slots);
        }
    }
    pub fn call(&mut self, handle: Handle) -> InterpretResult {
        unsafe { ffi::wrenCall(self.0, handle.raw) }
    }
    // pub fn handle_close(&mut self, handle: Handle) {
    //     unsafe { ffi::wrenReleaseHandle(self.0, handle.raw) }
    // }
    pub fn get_slot_type(&mut self, slot: i32) -> Type {
        assert!(
            self.get_slot_count() > slot,
            "Slot {} is out of bounds",
            slot
        );
        unsafe { ffi::wrenGetSlotType(self.0, slot) }
    }
    pub fn get_slot_bool(&mut self, slot: i32) -> Option<bool> {
        if self.get_slot_type(slot) == Type::Bool {
            Some(unsafe { ffi::wrenGetSlotBool(self.0, slot) != 0 })
        } else {
            None
        }
    }
    pub fn get_slot_bytes(&mut self, slot: i32) -> Option<&[u8]> {
        if self.get_slot_type(slot) == Type::String {
            let len: mem::MaybeUninit<i32> = mem::MaybeUninit::uninit();
            let mut len = unsafe { len.assume_init() };
            let ptr = unsafe { ffi::wrenGetSlotBytes(self.0, slot, &mut len) };
            Some(unsafe { slice::from_raw_parts(ptr as *const u8, len as usize) })
        } else {
            None
        }
    }
    pub fn get_slot_double(&mut self, slot: i32) -> Option<f64> {
        if self.get_slot_type(slot) == Type::Num {
            Some(unsafe { ffi::wrenGetSlotDouble(self.0, slot) })
        } else {
            None
        }
    }
    pub fn get_slot_foreign<T>(&mut self, slot: i32) -> Option<Point> {
        if self.get_slot_type(slot) == Type::Foreign {
            Some(unsafe { ffi::wrenGetSlotForeign(self.0, slot) })
        } else {
            None
        }
    }
    pub fn get_slot_foreign_typed<T>(&mut self, slot: i32) -> &mut T {
        assert!(
            self.get_slot_type(slot) == Type::Foreign,
            "Slot {} must contain a foreign object",
            slot
        );
        unsafe { &mut *(ffi::wrenGetSlotForeign(self.0, slot) as *mut T) }
    }
    pub fn get_slot_str(&mut self, slot: i32) -> Option<&str> {
        if self.get_slot_type(slot) == Type::String {
            let ptr = unsafe { ffi::wrenGetSlotString(self.0, slot) };
            Some(unsafe { CStr::from_ptr(ptr).to_str().unwrap() })
        } else {
            None
        }
    }
    pub fn get_slot_string(&mut self, slot: i32) -> Option<String> {
        if self.get_slot_type(slot) == Type::String {
            let ptr = unsafe { ffi::wrenGetSlotString(self.0, slot) };
            Some(unsafe { CString::from_raw(ptr as *mut i8).into_string().unwrap() })
        } else {
            None
        }
    }
    pub fn get_slot_handle(&mut self, slot: i32) -> Handle {
        assert!(
            self.get_slot_count() > slot,
            "Slot {} is out of bounds",
            slot
        );
        Handle {
            raw: unsafe { ffi::wrenGetSlotHandle(self.0, slot) },
            vm: self.0,
        }
    }
    pub fn set_slot_bool(&mut self, slot: i32, value: bool) {
        self.ensure_slots(slot + 1);
        unsafe { ffi::wrenSetSlotBool(self.0, slot, value as c_int) }
    }
    pub fn set_slot_bytes(&mut self, slot: i32, bytes: &[u8]) {
        self.ensure_slots(slot + 1);
        let ptr = bytes.as_ptr() as *const c_char;
        let len = bytes.len();
        unsafe { ffi::wrenSetSlotBytes(self.0, slot, ptr, len) }
    }
    pub fn set_slot_double(&mut self, slot: i32, value: f64) {
        self.ensure_slots(slot + 1);
        unsafe { ffi::wrenSetSlotDouble(self.0, slot, value) }
    }
    pub fn set_slot_new_foreign(&mut self, slot: i32, class_slot: i32, size: usize) -> Point {
        self.ensure_slots(slot + 1);
        unsafe { ffi::wrenSetSlotNewForeign(self.0, slot, class_slot, size) }
    }
    pub fn set_slot_new_foreign_typed<T>(&mut self, slot: i32, class_slot: i32) -> *mut T {
        self.set_slot_new_foreign(slot, class_slot, mem::size_of::<T>()) as *mut T
    }
    pub fn set_slot_new_list(&mut self, slot: i32) {
        self.ensure_slots(slot + 1);
        unsafe { ffi::wrenSetSlotNewList(self.0, slot) }
    }
    pub fn set_slot_new_map(&mut self, slot: i32) {
        self.ensure_slots(slot + 1);
        unsafe { ffi::wrenSetSlotNewMap(self.0, slot) }
    }
    pub fn set_slot_null(&mut self, slot: i32) {
        self.ensure_slots(slot + 1);
        unsafe { ffi::wrenSetSlotNull(self.0, slot) }
    }
    pub fn set_slot_string(&mut self, slot: i32, s: &str) {
        self.ensure_slots(slot + 1);
        let cstr = CString::new(s).unwrap();
        unsafe { ffi::wrenSetSlotString(self.0, slot, cstr.as_ptr()) }
    }
    pub fn set_slot_handle(&mut self, slot: i32, handle: &Handle) {
        self.ensure_slots(slot + 1);
        unsafe { ffi::wrenSetSlotHandle(self.0, slot, handle.raw) }
    }
    pub fn get_list_count(&mut self, slot: i32) -> i32 {
        if self.get_slot_type(slot) == Type::List {
            unsafe { ffi::wrenGetListCount(self.0, slot) }
        } else {
            0
        }
    }
}

pub struct Handle {
    raw: *mut ffi::WrenHandle,
    vm: *mut ffi::WrenVM,
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe { ffi::wrenReleaseHandle(self.vm, self.raw) }
    }
}

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
