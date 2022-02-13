#[macro_use]
extern crate wren_rs;

use wren_rs::{Configuration, ForeignMethodFn, InterpretResult, VM};

fn api(vm: &mut VM) {
    vm.ensure_slots(10);
    vm.set_slot_new_list(0);
    for i in 1..10 {
        vm.set_slot_double(i, i.into());
        vm.insert_in_list(0, -1, i);
    }
}

fn call_calls_foreign_bind_method(
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
    if full_name == "static CallCallsForeign.api()" {
        wren_foreign_method_fn!(api)
    } else {
        None
    }
}

fn main() {
    let mut config = Configuration::new();
    config.set_bind_foreign_method_fn(wren_bind_foreign_method_fn!(call_calls_foreign_bind_method));
    let mut vm = VM::new(&mut config);
    let result = vm.read_file("./src/call_calls_foreign.wren");
    if result != InterpretResult::Success {
        panic!("error");
    }
    vm.ensure_slots(1);
    vm.get_variable("./src/call_calls_foreign", "CallCallsForeign", 0);
    let api_class = vm.get_slot_handle(0);
    let call = vm.make_call_handle("call(_)");

    vm.ensure_slots(2);
    vm.set_slot_handle(0, &api_class);
    vm.set_slot_string(1, "parameter");

    println!("slot before {}", vm.get_slot_count());
    vm.call(&call);

    println!("slots after {}", vm.get_slot_count());
}
