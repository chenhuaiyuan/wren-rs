use wren_sys as ffi;

#[macro_use]
mod macros;
mod vm;
pub use ffi::WrenForeignClassMethods as ForeignClassMethods; // 待优化
pub use ffi::WrenForeignMethodFn as ForeignMethodFn; // 待优化
pub use ffi::WrenInterpretResult as InterpretResult;
pub use ffi::WrenLoadModuleResult as LoadModuleResult; // 待优化
pub use ffi::WrenType as Type;
pub use vm::Configuration;
pub use vm::Handle;
pub use vm::VM;
pub type Point = *mut libc::c_void;
