use std::io::Write;
use std::io::Read;
use std::{error::Error, path::Path, fs, os::unix::{prelude::FileTypeExt, net::UnixStream}};

use libc::kill;

use crate::VirtualMachine;

use super::process_utils::{get_process_info, get_tmp_path};

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

        if !result.trim().starts_with("0") {
            return Err(format!("Could not load agent").into());
        }

        Ok(())
    }
}

fn read_response(stream: &mut UnixStream) -> Result<String, Box<dyn Error>> {
    let mut response = String::new();
    let mut buf = [0; 8192];

    let bytes = stream.read(&mut buf)?;
    if bytes <= 0 {
        return Err("Could not read from socket".into());
    }

    let response_code = buf[0] as char;
    if response_code != '0' {
        return Err(format!("Response code was not 0, it was {}", response_code).into());
    }

    for i in 1..bytes {
        if buf[i] == 0 {
            break;
        }

        response.push(buf[i] as char);
    }

    Ok(response)
}

fn write_command(stream: &mut UnixStream, argv: Vec<&str>) -> Result<(), Box<dyn Error>> {
    if stream.write(b"1\0")? <= 0 {
        return Err("Could not write to socket".into());
    }
    
    for i in 0..4 {
        let arg = format!("{}\0", if argv.len() > i {
            argv[i]
        } else {
            ""
        });

        if stream.write(arg.as_bytes())? <= 0 {
            return Err("Could not write to socket".into());
        }
    } 

    stream.flush()?;
    stream.shutdown(std::net::Shutdown::Write)?;

    Ok(())
}

fn check_socket(pid: i32) -> bool {
    let _path_binding = &format!("{}/.java_pid{}", get_tmp_path(pid).to_str().expect("Could not convert path"), pid);
    let path = Path::new(_path_binding);

    // True if it is a socket
    fs::metadata(path).is_ok_and(|meta| meta.file_type().is_socket())
}

fn start_attach_mechanism(pid: i32) -> Result<(), Box<dyn Error>> {
    let _path_binding = &format!("/proc/{}/cwd/.attach_pid{}", pid, pid);
    let path = Path::new(_path_binding);

    match fs::write(&path, "") {
        Ok(_) => (),
        Err(_) => return Err(format!("Could not write to {}", path.to_str().expect("Could not convert path")).into()),
    };

    unsafe {
        kill(pid as i32, libc::SIGQUIT);
    }

    loop {
        if check_socket(pid) {
            break;
        }
    }

    fs::remove_file(path)?;

    Ok(())
}

fn connect_socket(pid: i32) -> Result<UnixStream, Box<dyn Error>> {
    let _path_binding = &format!("{}/.java_pid{}", get_tmp_path(pid).to_str().expect("Could not convert path"), pid);
    let path = Path::new(_path_binding);

    Ok(UnixStream::connect(path)?)
}