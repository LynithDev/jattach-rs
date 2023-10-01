use std::{error::Error, env};

use jattach_rs::JAttach;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} <pid> <cmd> (args)", args[0]);
        return Ok(());
    }

    let pid = args[1].parse::<i32>()?;
    let cmd = args[2].clone();
    let mut cmd_args: [&str; 3] = ["", "", ""];

    for i in 3..args.len() {
        cmd_args[i - 3] = &args[i];
    }

    let mut vm = JAttach::attach(pid)?;
    vm.execute_command(&cmd, cmd_args)?;

    Ok(())
}
