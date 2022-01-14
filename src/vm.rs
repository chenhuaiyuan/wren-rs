use crate::ffi;

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
    let source = fs::read(path).unwrap();
    return source;
}

enum PathType {
    PATH_TYPE_ABSOLUTE,
    PATH_TYPE_RELATIVE,
    PATH_TYPE_SIMPLE,
}

impl PartialEq for PathType {
    #[inline]
    fn eq(&self, other: &PathType) -> bool {
        match (self, other) {
            (PathType::PATH_TYPE_ABSOLUTE, PathType::PATH_TYPE_ABSOLUTE) => true,
            (PathType::PATH_TYPE_RELATIVE, PathType::PATH_TYPE_RELATIVE) => true,
            (PathType::PATH_TYPE_SIMPLE, PathType::PATH_TYPE_SIMPLE) => true,
            _ => false,
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
    return false;
}

fn path_type(path: &[u8]) -> PathType {
    let first = *path.first().unwrap();
    let first = char::from(first);
    let second = *path.get(1).unwrap();
    let second = char::from(second);
    let third = *path.get(2).unwrap();
    let third = char::from(third);
    if cfg!(target_os = "windows") && first.is_ascii_alphabetic() && second == ':' {
        return PathType::PATH_TYPE_ABSOLUTE;
    }

    if is_separator(first) {
        return PathType::PATH_TYPE_ABSOLUTE;
    }

    if first == '.' && is_separator(second) || first == '.' && second == '.' && is_separator(third)
    {
        return PathType::PATH_TYPE_RELATIVE;
    }

    PathType::PATH_TYPE_SIMPLE
}

fn default_resolve_module(_: &mut VM, module: &str, importer: &str) -> Vec<u8> {
    use std::path::PathBuf;
    if path_type(importer.as_bytes()) == PathType::PATH_TYPE_SIMPLE {
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
}
impl VM {
    pub fn new(config: &mut Configuration) -> VM {
        let raw = unsafe { ffi::wrenNewVM(&mut config.0) };
        VM { raw }
    }
    pub fn from_ptr(ptr: *mut ffi::WrenVM) -> VM {
        VM { raw: ptr }
    }
    // pub fn interpret(&mut self, module: &str, source: String) -> ffi::WrenInterpretResult {}
}
pub struct Configuration(ffi::WrenConfiguration);

impl Configuration {
    pub fn new() -> Configuration {
        let mut config = std::mem::MaybeUninit::<ffi::WrenConfiguration>::uninit();
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
