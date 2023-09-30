# jattach-rs
Rust port of [jattach](https://github.com/jattach/jattach/)

> [!IMPORTANT]
> This is a work in progress and is not yet ready for use.

## Supports
| VM      | Windows  | Linux  | MacOS  |
|---------|----------|--------|--------|
| Hotspot | ❌       | ✅     | ❌     |
| OpenJ9  | ❌       | ❌     | ❌     |

## Usage
```rs
use std::error::Error;

use jattach_rs::{posix::HotspotVirtualMachine, VirtualMachine};

fn main() -> Result<(), Box<dyn Error>> {
    let pid = 244680;
    let mut vm = match HotspotVirtualMachine::attach(pid) {
        Ok(vm) => vm,
        Err(e) => panic!("Couldn't attach to process: {}", e),
    };

    println!("{:#?}", vm.get_properties().unwrap());
    match vm.load_agent("/absolute/path/to/javaagent.jar", "arguments_for_agent") {
        Ok(_) => println!("JavaAgent loaded"),
        Err(e) => println!("Couldn't load JavaAgent"),
    }
}
```
