use std::error::Error;

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[cfg_attr(any(target_os = "linux", target_os = "macos"), path = "posix/mod.rs")]
mod posix;

#[cfg(target_os = "windows")]
#[cfg_attr(target_os = "windows", path = "windows/mod.rs")]
mod windows;

pub trait VirtualMachine {
    fn execute_command(&mut self, command: &str, args: [&str; 3]) -> Result<String, Box<dyn Error>>;

    fn load_agent(&mut self, agent_path: &str, agent_arguments: &str) -> Result<(), Box<dyn Error>>;
    fn get_properties(&mut self) -> Result<Vec<(String, String)>, Box<dyn Error>>;
}

pub struct JAttach;

impl JAttach {

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub fn attach(pid: i32) -> Result<Box<dyn VirtualMachine>, Box<dyn Error>> {
        Ok(Box::new(posix::HotspotVirtualMachine::attach(pid)?))
    }

    #[cfg(target_os = "windows")]
    pub fn attach(pid: i32) -> Result<Box<dyn VirtualMachine>, Box<dyn Error>> {
        Ok(Box::new(windows::HotspotVirtualMachine::attach(pid)?))
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    pub fn attach(pid: i32) -> Result<Box<dyn VirtualMachine>, Box<dyn Error>> {
        Err("Unsupported platform".into())
    }
}