use crate::ffi;
use crate::VM;
use libc::{c_char, c_int, c_void};
use std::ffi::CStr;
use std::mem;
use std::ptr;

#[macro_export]
macro_rules! wren_write_fn {
    ($f:path) => {
        $crate::macros::_write_fn($f)
    };
}

#[macro_export]
macro_rules! wren_error_fn {
    ($f:path) => {
        $crate::macros::_error_fn($f)
    };
}

#[macro_export]
macro_rules! wren_load_module_fn {
    ($f:path) => {
        $crate::macros::_load_module_fn($f)
    };
}

#[macro_export]
macro_rules! wren_resolve_module_fn {
    ($f:path) => {
        $crate::macros::_resolve_module_fn($f)
    };
}

#[macro_export]
macro_rules! wren_foreign_method_fn {
    ($f: path) => {
        $crate::macros::_foreign_method_fn($f)
    };
}

#[macro_export]
macro_rules! wren_finalizer_fn {
    ($f: path) => {
        $crate::macros::_finalizer_fn($f)
    };
}

#[macro_export]
macro_rules! wren_load_module_complete_fn {
    ($f: path) => {
        $crate::macros::_load_module_complete_fn($f)
    };
}

#[macro_export]
macro_rules! wren_bind_foreign_method_fn {
    ($f: path) => {
        $crate::macros::_bind_foreign_method_fn($f)
    };
}

#[macro_export]
macro_rules! wren_bind_foreign_class_fn {
    ($f: path) => {
        $crate::macros::_bind_foreign_class_fn($f)
    };
}

#[doc(hidden)]
#[inline]
fn _asset_size<F>() {
    let size = mem::size_of::<F>();
    assert!(size == 0, "Wrapped closures must be zero-sized");
}

#[doc(hidden)]
#[inline]
pub fn _write_fn<F: Fn(&mut VM, &str)>(_: F) -> ffi::WrenWriteFn {
    unsafe extern "C" fn f<F: Fn(&mut VM, &str)>(vm: *mut ffi::WrenVM, text: *const c_char) {
        mem::transmute::<&(), &F>(&())(
            &mut VM::from_ptr(vm),
            CStr::from_ptr(text).to_str().unwrap(),
        );
    }
    _asset_size::<F>();
    Some(f::<F>)
}

#[doc(hidden)]
#[inline]
pub fn _error_fn<F: Fn(&mut VM, ffi::WrenErrorType, &str, i32, &str)>(_: F) -> ffi::WrenErrorFn {
    unsafe extern "C" fn f<F: Fn(&mut VM, ffi::WrenErrorType, &str, i32, &str)>(
        vm: *mut ffi::WrenVM,
        _type: ffi::WrenErrorType,
        module: *const c_char,
        line: c_int,
        message: *const c_char,
    ) {
        let mut vm = VM::from_ptr(vm);
        let module = if module.is_null() {
            ""
        } else {
            CStr::from_ptr(module).to_str().unwrap()
        };
        let message = CStr::from_ptr(message).to_str().unwrap();
        mem::transmute::<&(), &F>(&())(&mut vm, _type, module, line, message);
    }
    _asset_size::<F>();
    Some(f::<F>)
}

unsafe extern "C" fn _load_module_complete(
    _: *mut ffi::WrenVM,
    _: *const c_char,
    result: ffi::WrenLoadModuleResult,
) {
    if result.source.is_null() {
        libc::free(result.source as *mut c_void);
    }
}
#[doc(hidden)]
#[inline]
pub fn _load_module_fn<F: Fn(&mut VM, &str) -> Vec<u8>>(_: F) -> ffi::WrenLoadModuleFn {
    unsafe extern "C" fn f<F: Fn(&mut VM, &str) -> Vec<u8>>(
        vm: *mut ffi::WrenVM,
        module: *const c_char,
    ) -> ffi::WrenLoadModuleResult {
        let mut source = mem::transmute::<&(), &F>(&())(
            &mut VM::from_ptr(vm),
            CStr::from_ptr(module).to_str().unwrap(),
        );
        source.push(b'\0');
        let source = CStr::from_bytes_with_nul(source.as_slice()).unwrap();

        ffi::WrenLoadModuleResult {
            source: source.as_ptr(),
            on_complete: Some(_load_module_complete),
            user_data: ptr::null_mut(),
        }
    }

    _asset_size::<F>();
    Some(f::<F>)
}

#[doc(hidden)]
#[inline]
pub fn _resolve_module_fn<F: Fn(&mut VM, &str, &str) -> Vec<u8>>(_: F) -> ffi::WrenResolveModuleFn {
    unsafe extern "C" fn f<F: Fn(&mut VM, &str, &str) -> Vec<u8>>(
        vm: *mut ffi::WrenVM,
        module: *const c_char,
        importer: *const c_char,
    ) -> *const c_char {
        let mut path = mem::transmute::<&(), &F>(&())(
            &mut VM::from_ptr(vm),
            CStr::from_ptr(module).to_str().unwrap(),
            CStr::from_ptr(importer).to_str().unwrap(),
        );
        path.push(b'\0');
        let path = CStr::from_bytes_with_nul(path.as_slice()).unwrap();
        path.as_ptr()
    }

    _asset_size::<F>();
    Some(f::<F>)
}

#[doc(hidden)]
#[inline]
pub fn _foreign_method_fn<F: Fn(&mut VM)>(_: F) -> ffi::WrenForeignMethodFn {
    unsafe extern "C" fn f<F: Fn(&mut VM)>(vm: *mut ffi::WrenVM) {
        mem::transmute::<&(), &F>(&())(&mut VM::from_ptr(vm));
    }
    _asset_size::<F>();
    Some(f::<F>)
}

#[doc(hidden)]
#[inline]
pub fn _finalizer_fn<F: Fn(*mut c_void)>(_: F) -> ffi::WrenFinalizerFn {
    unsafe extern "C" fn f<F: Fn(*mut c_void)>(data: *mut c_void) {
        mem::transmute::<&(), &F>(&())(data)
    }
    _asset_size::<F>();
    Some(f::<F>)
}

#[doc(hidden)]
#[inline]
pub fn _load_module_complete_fn<F: Fn(&mut VM, &str, ffi::WrenLoadModuleResult)>(
    _: F,
) -> ffi::WrenLoadModuleCompleteFn {
    unsafe extern "C" fn f<F: Fn(&mut VM, &str, ffi::WrenLoadModuleResult)>(
        vm: *mut ffi::WrenVM,
        name: *const c_char,
        result: ffi::WrenLoadModuleResult,
    ) {
        mem::transmute::<&(), &F>(&())(
            &mut VM::from_ptr(vm),
            CStr::from_ptr(name).to_str().unwrap(),
            result,
        );
    }
    _asset_size::<F>();
    Some(f::<F>)
}

#[doc(hidden)]
#[inline]
pub fn _bind_foreign_method_fn<
    F: Fn(&mut VM, &str, &str, bool, &str) -> ffi::WrenForeignMethodFn,
>(
    _: F,
) -> ffi::WrenBindForeignMethodFn {
    unsafe extern "C" fn f<F: Fn(&mut VM, &str, &str, bool, &str) -> ffi::WrenForeignMethodFn>(
        vm: *mut ffi::WrenVM,
        module: *const c_char,
        class_name: *const c_char,
        is_static: c_int,
        signature: *const c_char,
    ) -> ffi::WrenForeignMethodFn {
        mem::transmute::<&(), &F>(&())(
            &mut VM::from_ptr(vm),
            CStr::from_ptr(module).to_str().unwrap(),
            CStr::from_ptr(class_name).to_str().unwrap(),
            is_static != 0,
            CStr::from_ptr(signature).to_str().unwrap(),
        )
    }
    _asset_size::<F>();
    Some(f::<F>)
}

#[doc(hidden)]
#[inline]
pub fn _bind_foreign_class_fn<F: Fn(&mut VM, &str, &str) -> ffi::WrenForeignClassMethods>(
    _: F,
) -> ffi::WrenBindForeignClassFn {
    unsafe extern "C" fn f<F: Fn(&mut VM, &str, &str) -> ffi::WrenForeignClassMethods>(
        vm: *mut ffi::WrenVM,
        module: *const c_char,
        class_name: *const c_char,
    ) -> ffi::WrenForeignClassMethods {
        mem::transmute::<&(), &F>(&())(
            &mut VM::from_ptr(vm),
            CStr::from_ptr(module).to_str().unwrap(),
            CStr::from_ptr(class_name).to_str().unwrap(),
        )
    }
    _asset_size::<F>();
    Some(f::<F>)
}
