use std::error::Error;

use jattach_rs::{posix::HotspotVirtualMachine, VirtualMachine};

fn main() -> Result<(), Box<dyn Error>> {
    let mut vm = HotspotVirtualMachine::attach(244680)?;

    // println!("{:#?}", vm.get_properties()?);
    vm.load_agent("/home/lynith/Documents/Projects/Java/TestAgent/build/libs/TestAgent2.jar", "")?;

    Ok(())
}
