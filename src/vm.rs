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
        if mem::discriminant(self) == mem::discriminant(other) {
            matches!(
                (self, other),
                (PathType::Absolute, PathType::Absolute)
                    | (PathType::Relative, PathType::Relative)
                    | (PathType::Simple, PathType::Simple)
            )
        } else {
            false
        }
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
pub struct VM {
    raw: *mut ffi::WrenVM,
    owned: bool,
}
impl VM {
    pub fn new(config: &mut Configuration) -> VM {
        let raw = unsafe { ffi::wrenNewVM(&mut config.0) };
        VM { raw, owned: true }
    }
    pub fn from_ptr(ptr: *mut ffi::WrenVM) -> VM {
        VM {
            raw: ptr,
            owned: false,
        }
    }
    pub fn interpret<S: Into<Vec<u8>>>(&mut self, module: &str, source: S) -> InterpretResult {
        let module = CString::new(module).unwrap();
        let source = CString::new(source.into()).unwrap();
        unsafe { ffi::wrenInterpret(self.raw, module.as_ptr(), source.as_ptr()) }
    }
    // pub fn close(&mut self) {
    //     unsafe { ffi::wrenFreeVM(self.raw) }
    // }
    pub fn version() {
        let v = unsafe { ffi::wrenGetVersionNumber() };
        println!("Wren Version: {}", v);
    }
    pub fn collect_garbage(&mut self) {
        unsafe { ffi::wrenCollectGarbage(self.raw) }
    }
    pub fn make_call_handle(&mut self, signature: &str) -> Handle {
        let signature = CString::new(signature).unwrap();
        let handle = unsafe { ffi::wrenMakeCallHandle(self.raw, signature.as_ptr()) };
        Handle {
            raw: handle,
            vm: self.raw,
        }
    }
    pub fn get_slot_count(&mut self) -> i32 {
        unsafe { ffi::wrenGetSlotCount(self.raw) }
    }
    pub fn ensure_slots(&mut self, num_slots: i32) {
        unsafe {
            ffi::wrenEnsureSlots(self.raw, num_slots);
        }
    }
    pub fn call(&mut self, handle: Handle) -> InterpretResult {
        unsafe { ffi::wrenCall(self.raw, handle.raw) }
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
        unsafe { ffi::wrenGetSlotType(self.raw, slot) }
    }
    pub fn get_slot_bool(&mut self, slot: i32) -> Option<bool> {
        if self.get_slot_type(slot) == Type::Bool {
            Some(unsafe { ffi::wrenGetSlotBool(self.raw, slot) != 0 })
        } else {
            None
        }
    }
    pub fn get_slot_bytes(&mut self, slot: i32) -> Option<&[u8]> {
        if self.get_slot_type(slot) == Type::String {
            let len: mem::MaybeUninit<i32> = mem::MaybeUninit::uninit();
            let mut len = unsafe { len.assume_init() };
            let ptr = unsafe { ffi::wrenGetSlotBytes(self.raw, slot, &mut len) };
            Some(unsafe { slice::from_raw_parts(ptr as *const u8, len as usize) })
        } else {
            None
        }
    }
    pub fn get_slot_double(&mut self, slot: i32) -> Option<f64> {
        if self.get_slot_type(slot) == Type::Num {
            Some(unsafe { ffi::wrenGetSlotDouble(self.raw, slot) })
        } else {
            None
        }
    }
    // pub fn get_slot_foreign(&mut self, slot: i32) -> Option<Point> {
    //     if self.get_slot_type(slot) == Type::Foreign {
    //         Some(unsafe { ffi::wrenGetSlotForeign(self.raw, slot) })
    //     } else {
    //         None
    //     }
    // }
    pub fn get_slot_foreign<T>(&mut self, slot: i32) -> *mut T {
        assert!(
            self.get_slot_type(slot) == Type::Foreign,
            "Slot {} must contain a foreign object",
            slot
        );
        unsafe { ffi::wrenGetSlotForeign(self.raw, slot) as *mut T }
    }
    pub fn get_slot_str(&mut self, slot: i32) -> Option<&str> {
        if self.get_slot_type(slot) == Type::String {
            let ptr = unsafe { ffi::wrenGetSlotString(self.raw, slot) };
            Some(unsafe { CStr::from_ptr(ptr).to_str().unwrap() })
        } else {
            None
        }
    }
    pub fn get_slot_string(&mut self, slot: i32) -> Option<String> {
        if self.get_slot_type(slot) == Type::String {
            let ptr = unsafe { ffi::wrenGetSlotString(self.raw, slot) };
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
            raw: unsafe { ffi::wrenGetSlotHandle(self.raw, slot) },
            vm: self.raw,
        }
    }
    pub fn set_slot_bool(&mut self, slot: i32, value: bool) {
        // self.ensure_slots(slot + 1);
        unsafe { ffi::wrenSetSlotBool(self.raw, slot, value as c_int) }
    }
    pub fn set_slot_bytes(&mut self, slot: i32, bytes: &[u8]) {
        // self.ensure_slots(slot + 1);
        let ptr = bytes.as_ptr() as *const c_char;
        let len = bytes.len();
        unsafe { ffi::wrenSetSlotBytes(self.raw, slot, ptr, len) }
    }
    pub fn set_slot_double(&mut self, slot: i32, value: f64) {
        // self.ensure_slots(slot + 1);
        unsafe { ffi::wrenSetSlotDouble(self.raw, slot, value) }
    }
    pub fn set_slot_new_foreign<T>(&mut self, slot: i32, class_slot: i32) -> *mut T {
        // self.ensure_slots(slot + 1);
        unsafe {
            ffi::wrenSetSlotNewForeign(self.raw, slot, class_slot, mem::size_of::<T>()) as *mut T
        }
    }
    // pub fn set_slot_new_foreign_typed<T>(&mut self, slot: i32, class_slot: i32) -> *mut T {
    //     self.set_slot_new_foreign(slot, class_slot, mem::size_of::<T>()) as *mut T
    // }
    pub fn set_slot_new_list(&mut self, slot: i32) {
        // self.ensure_slots(slot + 1);
        unsafe { ffi::wrenSetSlotNewList(self.raw, slot) }
    }
    pub fn set_slot_new_map(&mut self, slot: i32) {
        // self.ensure_slots(slot + 1);
        unsafe { ffi::wrenSetSlotNewMap(self.raw, slot) }
    }
    pub fn set_slot_null(&mut self, slot: i32) {
        // self.ensure_slots(slot + 1);
        unsafe { ffi::wrenSetSlotNull(self.raw, slot) }
    }
    pub fn set_slot_string(&mut self, slot: i32, s: &str) {
        // self.ensure_slots(slot + 1);
        let cstr = CString::new(s).unwrap();
        unsafe { ffi::wrenSetSlotString(self.raw, slot, cstr.as_ptr()) }
    }
    pub fn set_slot_handle(&mut self, slot: i32, handle: &Handle) {
        // self.ensure_slots(slot + 1);
        unsafe { ffi::wrenSetSlotHandle(self.raw, slot, handle.raw) }
    }
    pub fn get_list_count(&mut self, slot: i32) -> i32 {
        if self.get_slot_type(slot) == Type::List {
            unsafe { ffi::wrenGetListCount(self.raw, slot) }
        } else {
            0
        }
    }
    fn check_index(&mut self, list_slot: i32, index: i32) -> i32 {
        assert!(
            self.get_slot_type(list_slot) == Type::List,
            "Slot {} must contain a list",
            list_slot
        );
        let list_count = self.get_list_count(list_slot);
        let index = if index < 0 {
            list_count + 1 + index
        } else {
            index
        };
        assert!(index <= list_count, "List index out of bounds");
        index
    }
    pub fn get_list_element(&mut self, list_slot: i32, index: i32, element_slot: i32) {
        // self.ensure_slots(element_slot + 1);
        let index = self.check_index(list_slot, index);
        unsafe { ffi::wrenGetListElement(self.raw, list_slot, index, element_slot) }
    }
    pub fn set_list_element(&mut self, list_slot: i32, index: i32, element_slot: i32) {
        // self.ensure_slots(element_slot + 1);
        let index = self.check_index(list_slot, index);
        unsafe { ffi::wrenSetListElement(self.raw, list_slot, index, element_slot) }
    }
    pub fn insert_in_list(&mut self, list_slot: i32, index: i32, element_slot: i32) {
        assert!(
            element_slot < self.get_slot_count(),
            "No element in slot {}",
            element_slot
        );
        let index = self.check_index(list_slot, index);
        unsafe { ffi::wrenInsertInList(self.raw, list_slot, index, element_slot) }
    }
    pub fn get_map_count(&mut self, slot: i32) -> i32 {
        if self.get_slot_type(slot) == Type::Map {
            unsafe { ffi::wrenGetMapCount(self.raw, slot) }
        } else {
            0
        }
    }
    pub fn get_map_contains_key(&mut self, map_slot: i32, key_slot: i32) -> bool {
        if self.get_slot_type(map_slot) == Type::Map {
            unsafe { ffi::wrenGetMapContainsKey(self.raw, map_slot, key_slot) != 0 }
        } else {
            false
        }
    }
    pub fn get_map_value(&mut self, map_slot: i32, key_slot: i32, value_slot: i32) {
        unsafe { ffi::wrenGetMapValue(self.raw, map_slot, key_slot, value_slot) }
    }
    pub fn set_map_value(&mut self, map_slot: i32, key_slot: i32, value_slot: i32) {
        assert!(
            self.get_slot_type(map_slot) == Type::Map,
            "Slot {} must contain a map",
            map_slot
        );
        unsafe { ffi::wrenSetMapValue(self.raw, map_slot, key_slot, value_slot) }
    }
    pub fn remove_map_value(&mut self, map_slot: i32, key_slot: i32, removed_value_slot: i32) {
        assert!(
            self.get_slot_type(map_slot) == Type::Map,
            "Slot {} must contain a map",
            map_slot
        );
        unsafe { ffi::wrenRemoveMapValue(self.raw, map_slot, key_slot, removed_value_slot) }
    }
    pub fn get_variable(&mut self, module: &str, name: &str, slot: i32) {
        let module_cstr = CString::new(module).unwrap();
        let name_cstr = CString::new(name).unwrap();
        unsafe { ffi::wrenGetVariable(self.raw, module_cstr.as_ptr(), name_cstr.as_ptr(), slot) }
    }
    pub fn has_variablle(&mut self, module: &str, name: &str) -> bool {
        let module_cstr = CString::new(module).unwrap();
        let name_cstr = CString::new(name).unwrap();
        unsafe { ffi::wrenHasVariable(self.raw, module_cstr.as_ptr(), name_cstr.as_ptr()) != 0 }
    }
    pub fn has_module(&mut self, module: &str) -> bool {
        let module_cstr = CString::new(module).unwrap();
        unsafe { ffi::wrenHasModule(self.raw, module_cstr.as_ptr()) != 0 }
    }
    pub fn abort_fiber(&mut self, slot: i32) {
        unsafe { ffi::wrenAbortFiber(self.raw, slot) }
    }
    pub fn get_user_data<T>(&mut self) -> &mut T {
        unsafe { &mut *(ffi::wrenGetUserData(self.raw) as *mut T) }
    }
    unsafe fn _set_user_data<T>(&mut self, user_data: *mut T) {
        let user_data = mem::transmute::<*mut T, Point>(user_data);
        ffi::wrenSetUserData(self.raw, user_data)
    }
    pub fn set_user_data<T>(&mut self, user_data: &mut T) {
        unsafe {
            self._set_user_data::<T>(user_data as *mut T);
        }
    }
}

impl Drop for VM {
    fn drop(&mut self) {
        if self.owned {
            unsafe { ffi::wrenFreeVM(self.raw) }
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
