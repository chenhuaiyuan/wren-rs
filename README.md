Rust bindings to the Wren scripting language API

Rust绑定到Wren脚本语言API

目前只在macos系统上进行了测试，还有很多功能未对接

#### example

Cargo.toml
```rust
[dependencies]
wren-rs = {git ="https://github.com/chenhuaiyuan/wren-rs" }
```

main.rs
```rust
fn main() {
    let mut config = wren_rs::Configuration::new();
    let mut vm = wren_rs::VM::new(&mut config);
    vm.interpret("my_module", "System.print(\"hello world!\")");
    vm.close();
}
```