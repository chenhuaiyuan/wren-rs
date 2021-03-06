#[macro_use]
extern crate wren_rs;

use libc::c_void;
use std::mem;
use wren_rs::{
    Configuration, ForeignClassMethods, ForeignData, ForeignMethodFn, InterpretResult, VM,
};

static mut FINALIZED: i32 = 0;

fn api_finalized(vm: &mut VM) {
    unsafe {
        vm.set_slot_double(0, FINALIZED as f64);
    }
}

fn counter_allocate(vm: &mut VM) {
    let value: *mut f64 = vm.set_slot_new_foreign::<f64>(0, 0);
    unsafe {
        *value = 0.0;
    }
}

fn counter_allocate2(vm: &mut VM) {
    let mut value: ForeignData<f64> = vm.set_slot_new_foreign2::<f64>(0, 0);
    let inner = value.inner();
    *inner = 0.0;
}

fn counter_increment(vm: &mut VM) {
    let value: *mut f64 = vm.get_slot_foreign(0);
    let increment: f64 = vm.get_slot_double(1).unwrap();

    unsafe {
        *value += increment;
    }
}

fn counter_increment2(vm: &mut VM) {
    let mut value: ForeignData<f64> = vm.get_slot_foreign2(0);
    let increment: f64 = vm.get_slot_double(1).unwrap();

    let inner = value.inner();
    *inner += increment;
}

fn counter_value(vm: &mut VM) {
    let value: *mut f64 = vm.get_slot_foreign(0);
    unsafe {
        vm.set_slot_double(0, *value);
    }
}

fn counter_value2(vm: &mut VM) {
    let mut value: ForeignData<f64> = vm.get_slot_foreign2(0);
    let inner = value.inner();
    vm.set_slot_double(0, *inner);
}

fn point_allocate(vm: &mut VM) {
    let coordinates: *mut [f64; 3] = vm.set_slot_new_foreign::<[f64; 3]>(0, 0);

    if vm.get_slot_count() == 1 {
        unsafe {
            (*coordinates)[0] = 0.0;
            (*coordinates)[1] = 0.0;
            (*coordinates)[2] = 0.0;
        }
    } else {
        unsafe {
            (*coordinates)[0] = vm.get_slot_double(1).unwrap();
            (*coordinates)[1] = vm.get_slot_double(2).unwrap();
            (*coordinates)[2] = vm.get_slot_double(3).unwrap();
        }
    }
}

fn point_allocate2(vm: &mut VM) {
    let mut coordinates: ForeignData<[f64; 3]> = vm.set_slot_new_foreign2::<[f64; 3]>(0, 0);
    let inner = coordinates.inner();

    if vm.get_slot_count() == 1 {
        inner[0] = 0.0;
        inner[1] = 0.0;
        inner[2] = 0.0;
    } else {
        inner[0] = vm.get_slot_double(1).unwrap();
        inner[1] = vm.get_slot_double(2).unwrap();
        inner[2] = vm.get_slot_double(3).unwrap();
    }
}

fn point_translate(vm: &mut VM) {
    let coordinates: *mut [f64; 3] = vm.get_slot_foreign(0);
    unsafe {
        (*coordinates)[0] += vm.get_slot_double(1).unwrap();
        (*coordinates)[1] += vm.get_slot_double(2).unwrap();
        (*coordinates)[2] += vm.get_slot_double(3).unwrap();
    }
}

fn point_translate2(vm: &mut VM) {
    let mut coordinates: ForeignData<[f64; 3]> = vm.get_slot_foreign2(0);
    let inner = coordinates.inner();
    inner[0] += vm.get_slot_double(1).unwrap();
    inner[1] += vm.get_slot_double(2).unwrap();
    inner[2] += vm.get_slot_double(3).unwrap();
}

fn point_to_string(vm: &mut VM) {
    let coordinates: *mut [f64; 3] = vm.get_slot_foreign(0);
    let result = unsafe {
        format!(
            "({}, {}, {})",
            (*coordinates)[0],
            (*coordinates)[1],
            (*coordinates)[2]
        )
    };
    vm.set_slot_string(0, &result);
}

fn point_to_string2(vm: &mut VM) {
    let mut coordinates: ForeignData<[f64; 3]> = vm.get_slot_foreign2(0);
    let inner = coordinates.inner();
    let result = format!("({}, {}, {})", inner[0], inner[1], inner[2]);
    vm.set_slot_string(0, &result);
}

fn resource_allocate(vm: &mut VM) {
    let value: *mut i32 = vm.set_slot_new_foreign::<i32>(0, 0);
    unsafe {
        *value = 123;
    }
}

fn resource_allocate2(vm: &mut VM) {
    let mut value: ForeignData<i32> = vm.set_slot_new_foreign2::<i32>(0, 0);
    let inner = value.inner();
    *inner = 123;
}

fn resource_finalize(data: *mut c_void) {
    let value: *mut i32 = data as *mut i32;
    unsafe {
        if *value != 123 {
            panic!("value is not 123")
        }
    }

    unsafe {
        FINALIZED = FINALIZED + 1;
    }
}

fn resource_finalize2(data: *mut c_void) {
    let mut value: ForeignData<i32> =
        unsafe { mem::transmute::<*mut c_void, ForeignData<i32>>(data) };
    let inner = value.inner();
    if *inner != 123 {
        panic!("value is not 123")
    }

    unsafe {
        FINALIZED = FINALIZED + 1;
    }
}

fn bad_class_allocate(vm: &mut VM) {
    vm.ensure_slots(1);
    vm.set_slot_string(0, "Something went wrong");
    vm.abort_fiber(0);
}

fn foreign_class_bind_method(
    _: &mut VM,
    _: &str,
    class_name: &str,
    is_static: bool,
    signature: &str,
) -> ForeignMethodFn {
    let full_name = if is_static {
        format!("static {}.{}", class_name, signature)
    } else {
        format!("{}.{}", class_name, signature)
    };
    if full_name == "static ForeignClass.finalized" {
        wren_foreign_method_fn!(api_finalized)
    } else if full_name == "Counter.increment(_)" {
        wren_foreign_method_fn!(counter_increment2)
    } else if full_name == "Counter.value" {
        wren_foreign_method_fn!(counter_value2)
    } else if full_name == "Point.translate(_,_,_)" {
        wren_foreign_method_fn!(point_translate2)
    } else if full_name == "Point.toString" {
        wren_foreign_method_fn!(point_to_string2)
    } else {
        None
    }
}

fn foreign_class_bind_class(_: &mut VM, _: &str, class_name: &str) -> ForeignClassMethods {
    let mut method = ForeignClassMethods {
        allocate: None,
        finalize: None,
    };
    if class_name == "Counter" {
        method.allocate = wren_foreign_method_fn!(counter_allocate2);
        method
    } else if class_name == "Point" {
        method.allocate = wren_foreign_method_fn!(point_allocate2);
        method
    } else if class_name == "Resource" {
        method.allocate = wren_foreign_method_fn!(resource_allocate2);
        method.finalize = wren_finalizer_fn!(resource_finalize2);
        method
    } else if class_name == "BadClass" {
        method.allocate = wren_foreign_method_fn!(bad_class_allocate);
        method
    } else {
        method
    }
}

fn main() {
    let mut config = Configuration::new();
    config.set_bind_foreign_method_fn(wren_bind_foreign_method_fn!(foreign_class_bind_method));
    config.set_bind_foreign_class_fn(wren_bind_foreign_class_fn!(foreign_class_bind_class));
    let mut vm = VM::new(&mut config);
    let result = vm.read_file("./src/foreign_class.wren");
    if result != InterpretResult::Success {
        panic!("error");
    }
}
