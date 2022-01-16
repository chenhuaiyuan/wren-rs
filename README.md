Rust bindings to the Wren scripting language API

将[wren](https://github.com/wren-lang/wren)脚本语言API绑定到rust

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