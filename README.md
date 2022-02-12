Rust bindings to the Wren scripting language API

将[wren](https://github.com/wren-lang/wren)脚本语言API绑定到rust

So far it's only been tested on MacOS

目前只在macos系统上进行了测试

目前还有很多工作未完成，还不能真正使用

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
}
```