use wren_sys as ffi;

#[macro_use]
mod macros;
mod vm;
pub use ffi::WrenInterpretResult as InterpretResult;
pub use vm::Configuration;
pub use vm::VM;
