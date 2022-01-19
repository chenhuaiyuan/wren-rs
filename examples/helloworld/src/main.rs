fn main() {
    let mut config = wren_rs::Configuration::new();
    let mut vm = wren_rs::VM::new(&mut config);
    vm.interpret("my_module", "System.print(\"hello world!\")");
}
