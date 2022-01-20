use std::fs;
use wren_rs::{Configuration, Handle, InterpretResult, VM};

fn main() {
    let mut config = Configuration::new();
    let mut vm = VM::new(&mut config);

    let source = fs::read("./src/call.wren").unwrap();

    let result: InterpretResult = vm.interpret("call", source);
    if result != InterpretResult::Success {
        panic!("error");
    }

    vm.ensure_slots(1);
    vm.get_variable("call", "Call", 0);
    let call_class: Handle = vm.get_slot_handle(0);

    let no_params: Handle = vm.make_call_handle("noParams");
    let zero: Handle = vm.make_call_handle("zero()");
    let one: Handle = vm.make_call_handle("one(_)");
    let two: Handle = vm.make_call_handle("two(_,_)");
    let unary: Handle = vm.make_call_handle("-");
    let binary: Handle = vm.make_call_handle("-(_)");
    let subscript: Handle = vm.make_call_handle("[_,_]");
    let subscript_set: Handle = vm.make_call_handle("[_,_]=(_)");

    vm.ensure_slots(1);
    vm.set_slot_handle(0, &call_class);
    vm.call(&no_params);

    vm.ensure_slots(1);
    vm.set_slot_handle(0, &call_class);
    vm.call(&zero);

    vm.ensure_slots(2);
    vm.set_slot_handle(0, &call_class);
    vm.set_slot_double(1, 1.0);
    vm.call(&one);

    vm.ensure_slots(3);
    vm.set_slot_handle(0, &call_class);
    vm.set_slot_double(1, 1.0);
    vm.set_slot_double(2, 2.0);
    vm.call(&two);

    vm.ensure_slots(1);
    vm.set_slot_handle(0, &call_class);
    vm.call(&unary);

    vm.ensure_slots(2);
    vm.set_slot_handle(0, &call_class);
    vm.set_slot_double(1, 1.0);
    vm.call(&binary);

    vm.ensure_slots(3);
    vm.set_slot_handle(0, &call_class);
    vm.set_slot_double(1, 1.0);
    vm.set_slot_double(2, 2.0);
    vm.call(&subscript);

    vm.ensure_slots(4);
    vm.set_slot_handle(0, &call_class);
    vm.set_slot_double(1, 1.0);
    vm.set_slot_double(2, 2.0);
    vm.set_slot_double(3, 3.0);
    vm.call(&subscript_set);

    let get_value: Handle = vm.make_call_handle("getValue()");
    vm.ensure_slots(1);
    vm.set_slot_handle(0, &call_class);
    vm.call(&get_value);
    println!("slots after call: {}", vm.get_slot_count());
    let value: Handle = vm.get_slot_handle(0);

    vm.ensure_slots(3);
    vm.set_slot_handle(0, &call_class);
    vm.set_slot_bool(1, true);
    vm.set_slot_bool(2, false);
    vm.call(&two);

    vm.ensure_slots(3);
    vm.set_slot_handle(0, &call_class);
    vm.set_slot_double(1, 1.2);
    vm.set_slot_double(2, 3.4);
    vm.call(&two);

    vm.ensure_slots(3);
    vm.set_slot_handle(0, &call_class);
    vm.set_slot_string(1, "string");
    vm.set_slot_string(2, "another");
    vm.call(&two);

    vm.ensure_slots(3);
    vm.set_slot_handle(0, &call_class);
    vm.set_slot_null(1);
    vm.set_slot_handle(2, &value);
    vm.call(&two);

    vm.ensure_slots(3);
    vm.set_slot_handle(0, &call_class);
    vm.set_slot_bytes_by_length(1, b"string", 3);
    vm.set_slot_bytes_by_length(2, b"b\0y\0t\0e", 7);
    vm.call(&two);

    vm.ensure_slots(10);
    vm.set_slot_handle(0, &call_class);
    for i in 1..10 {
        vm.set_slot_double(i, (i as f64) * 0.1);
    }
    vm.call(&one);
}
