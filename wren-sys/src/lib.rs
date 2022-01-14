#![allow(improper_ctypes)]

extern crate libc;
use libc::{c_char, c_double, c_int, c_void, size_t};

#[repr(C)]
pub struct WrenVM;

#[repr(C)]
pub struct WrenHandle;

pub type WrenReallocateFn = Option<
    unsafe extern "C" fn(
        memory: *mut c_void,
        new_size: size_t,
        user_data: *mut c_void,
    ) -> *mut c_void,
>;
pub type WrenForeignMethodFn = Option<unsafe extern "C" fn(vm: *mut WrenVM)>;
pub type WrenFinalizerFn = Option<unsafe extern "C" fn(data: *mut c_void)>;
pub type WrenResolveModuleFn = Option<
    unsafe extern "C" fn(
        vm: *mut WrenVM,
        importer: *const c_char,
        name: *const c_char,
    ) -> *const c_char,
>;

#[repr(C)]
pub struct WrenLoadModuleResult {
    pub source: *const c_char,
    pub on_complete: WrenLoadModuleCompleteFn,
    pub user_data: *mut c_void,
}

pub type WrenLoadModuleCompleteFn = Option<
    unsafe extern "C" fn(vm: *mut WrenVM, name: *const c_char, result: WrenLoadModuleResult),
>;

pub type WrenLoadModuleFn =
    Option<unsafe extern "C" fn(vm: *mut WrenVM, name: *const c_char) -> WrenLoadModuleResult>;
pub type WrenBindForeignMethodFn = Option<
    unsafe extern "C" fn(
        vm: *mut WrenVM,
        module: *const c_char,
        class_name: *const c_char,
        is_static: c_int,
        signature: *const c_char,
    ) -> WrenForeignMethodFn,
>;
pub type WrenWriteFn = Option<unsafe extern "C" fn(vm: *mut WrenVM, text: *const c_char)>;

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub enum WrenErrorType {
    Compile,
    Runtime,
    StackTrace,
}

pub type WrenErrorFn = Option<
    unsafe extern "C" fn(
        vm: *mut WrenVM,
        _type: WrenErrorType,
        module: *const c_char,
        line: c_int,
        message: *const c_char,
    ),
>;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct WrenForeignClassMethods {
    pub allocate: WrenForeignMethodFn,
    pub finalize: WrenFinalizerFn,
}

pub type WrenBindForeignClassFn = Option<
    unsafe extern "C" fn(
        vm: *mut WrenVM,
        module: *const c_char,
        class_name: *const c_char,
    ) -> WrenForeignClassMethods,
>;

#[repr(C)]
pub struct WrenConfiguration {
    pub reallocate_fn: WrenReallocateFn,
    pub resolve_module_fn: WrenResolveModuleFn,
    pub load_module_fn: WrenLoadModuleFn,
    pub bind_foreign_method_fn: WrenBindForeignMethodFn,
    pub bind_foreign_class_fn: WrenBindForeignClassFn,
    pub write_fn: WrenWriteFn,
    pub error_fn: WrenErrorFn,
    pub initial_heap_size: size_t,
    pub min_heap_size: size_t,
    pub heap_growth_percent: c_int,
    pub user_data: *mut c_void,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub enum WrenInterpretResult {
    Success,
    CompileError,
    RuntimeError,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub enum WrenType {
    Bool,
    Num,
    Foreign,
    List,
    Null,
    String,
    Unknown,
}

extern "C" {
    pub fn wrenGetVersionNumber() -> c_int;
    pub fn wrenInitConfiguration(configuration: *mut WrenConfiguration);
    pub fn wrenNewVM(configuration: *mut WrenConfiguration) -> *mut WrenVM;
    pub fn wrenFreeVM(vm: *mut WrenVM);
    pub fn wrenCollectGarbage(vm: *mut WrenVM);
    pub fn wrenInterpret(
        vm: *mut WrenVM,
        module: *const c_char,
        source: *const c_char,
    ) -> WrenInterpretResult;
    pub fn wrenMakeCallHandle(vm: *mut WrenVM, signature: *const c_char) -> *mut WrenHandle;
    pub fn wrenCall(vm: *mut WrenVM, method: *mut WrenHandle) -> WrenInterpretResult;
    pub fn wrenReleaseHandle(vm: *mut WrenVM, handle: *mut WrenHandle);

    pub fn wrenGetSlotCount(vm: *mut WrenVM) -> c_int;
    pub fn wrenEnsureSlots(vm: *mut WrenVM, num_slots: c_int);
    pub fn wrenGetSlotType(vm: *mut WrenVM, slot: c_int) -> WrenType;
    pub fn wrenGetSlotBool(vm: *mut WrenVM, slot: c_int) -> c_int;
    pub fn wrenGetSlotBytes(vm: *mut WrenVM, slot: c_int, length: *mut c_int) -> *const c_char;
    pub fn wrenGetSlotDouble(vm: *mut WrenVM, slot: c_int) -> c_double;
    pub fn wrenGetSlotForeign(vm: *mut WrenVM, slot: c_int) -> *mut c_void;
    pub fn wrenGetSlotString(vm: *mut WrenVM, slot: c_int) -> *const c_char;
    pub fn wrenGetSlotHandle(vm: *mut WrenVM, slot: c_int) -> *mut WrenHandle;

    pub fn wrenSetSlotBool(vm: *mut WrenVM, slot: c_int, value: c_int);
    pub fn wrenSetSlotBytes(vm: *mut WrenVM, slot: c_int, bytes: *const c_char, length: size_t);
    pub fn wrenSetSlotDouble(vm: *mut WrenVM, slot: c_int, value: c_double);
    pub fn wrenSetSlotNewForeign(
        vm: *mut WrenVM,
        slot: c_int,
        class_slot: c_int,
        size: size_t,
    ) -> *mut c_void;
    pub fn wrenSetSlotNewList(vm: *mut WrenVM, slot: c_int);
    pub fn wrenSetSlotNewMap(vm: *mut WrenVM, slot: c_int);
    pub fn wrenSetSlotNull(vm: *mut WrenVM, slot: c_int);
    pub fn wrenSetSlotString(vm: *mut WrenVM, slot: c_int, text: *const c_char);
    pub fn wrenSetSlotHandle(vm: *mut WrenVM, slot: c_int, handle: *mut WrenHandle);

    pub fn wrenGetListCount(vm: *mut WrenVM, slot: c_int) -> c_int;
    pub fn wrenGetListElement(vm: *mut WrenVM, list_slot: c_int, index: c_int, element_slot: c_int);
    pub fn wrenSetListElement(vm: *mut WrenVM, list_slot: c_int, index: c_int, element_slot: c_int);
    pub fn wrenInsertInList(vm: *mut WrenVM, list_slot: c_int, index: c_int, element_slot: c_int);
    pub fn wrenGetMapCount(vm: *mut WrenVM, slot: c_int) -> c_int;
    pub fn wrenGetMapContainsKey(vm: *mut WrenVM, map_slot: c_int, key_slot: c_int) -> c_int;
    pub fn wrenGetMapValue(vm: *mut WrenVM, map_slot: c_int, key_slot: c_int, value_slot: c_int);
    pub fn wrenSetMapValue(vm: *mut WrenVM, map_slot: c_int, key_slot: c_int, value_slot: c_int);
    pub fn wrenRemoveMapValue(
        vm: *mut WrenVM,
        map_slot: c_int,
        key_slot: c_int,
        removed_value_slot: c_int,
    );
    pub fn wrenGetVariable(
        vm: *mut WrenVM,
        module: *const c_char,
        name: *const c_char,
        slot: c_int,
    );
    pub fn wrenHasVariable(vm: *mut WrenVM, module: *const char, name: *const char) -> c_int;
    pub fn wrenHasModule(vm: *mut WrenVM, module: *const char) -> c_int;
    pub fn wrenAbortFiber(vm: *mut WrenVM, slot: c_int);
    pub fn wrenGetUserData(vm: *mut WrenVM) -> *mut c_void;
    pub fn wrenSetUserData(vm: *mut WrenVM, user_data: *mut c_void);
}
