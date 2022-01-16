use wren_sys as ffi;

#[macro_use]
mod macros;
mod value_type;
mod vm;
pub use ffi::WrenInterpretResult as InterpretResult;
pub use ffi::WrenType as SlotType;
pub use vm::Configuration;
pub use vm::VM;
