use std::{os::unix::net::UnixStream, error::Error};

use crate::VirtualMachine;

use self::{attach_hotspot::{check_socket, start_attach_mechanism, connect_socket, write_command, read_response}, process_utils::get_process_info};

pub mod attach_hotspot;
pub mod process_utils;

pub struct HotspotVirtualMachine {
    stream: UnixStream
}

impl HotspotVirtualMachine {
    pub fn attach(pid: i32) -> Result<HotspotVirtualMachine, Box<dyn Error>> {
        let (my_uid, my_gid) = unsafe { 
            (libc::getuid(), libc::getgid())
        };
    
        let (target_uid, target_gid) = match get_process_info(pid) {
            Ok(info) => info,
            Err(_) => return Err(format!("Could not get process info for {}", pid).into()),
        };
    
        if (my_uid != target_uid) || (my_gid != target_gid) {
            return Err("You must be the same user as the target process to attach to it".into());
        }
    
        if !check_socket(pid) && start_attach_mechanism(pid).is_err() {
            return Err("Could not start attach mechanism".into());
        }
    
        let stream = connect_socket(pid)?;

        Ok(Self { stream })
    }
}

impl VirtualMachine for HotspotVirtualMachine {
    fn execute_command(&mut self, command: &str, args: [&str; 3]) -> Result<String, Box<dyn Error>> {
        let argv: Vec<&str> = vec![&command, &args[0], &args[1], &args[2]];
        write_command(&mut self.stream, argv)?;
        read_response(&mut self.stream)
    }

    fn get_properties(&mut self) -> Result<Vec<(String, String)>, Box<dyn Error>> {
        let mut properties: Vec<(String, String)> = Vec::new();

        let result = self.execute_command("properties", ["", "", ""])?;
        for mut line in result.split("\n") {
            line = line.trim();
            if line.is_empty() || line.starts_with("#") {
                continue;
            }

            let mut split = line.split("=");
            
            let key = split.next();
            if key.is_none() {
                continue;
            }

            let value = split.next().unwrap_or("").to_owned();

            properties.push((key.unwrap().to_owned(), value));
        }

        Ok(properties)
    }

    fn load_agent(&mut self, agent_path: &str, agent_arguments: &str) -> Result<(), Box<dyn Error>> {
        let result = self.execute_command("load", ["instrument", "false", &format!("{}={}", agent_path, agent_arguments)])?;

        let mut result = result.trim();

        if result.starts_with("return code: ") {
            result = result.split("return code: ").collect::<Vec<&str>>()[1];
        }

        if !result.starts_with("0") {
            return Err(format!("Could not load agent").into());
        }

        Ok(())
    }
}
