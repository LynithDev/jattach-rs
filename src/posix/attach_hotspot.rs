use std::io::Write;
use std::io::Read;
use std::{error::Error, path::Path, fs, os::unix::{prelude::FileTypeExt, net::UnixStream}};

use libc::kill;

use super::process_utils::get_tmp_path;

pub (super) fn read_response(stream: &mut UnixStream) -> Result<String, Box<dyn Error>> {
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

pub (super) fn write_command(stream: &mut UnixStream, argv: Vec<&str>) -> Result<(), Box<dyn Error>> {
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

pub (super) fn check_socket(pid: i32) -> bool {
    let _path_binding = &format!("{}/.java_pid{}", get_tmp_path(pid).to_str().expect("Could not convert path"), pid);
    let path = Path::new(_path_binding);

    // True if it is a socket
    fs::metadata(path).is_ok_and(|meta| meta.file_type().is_socket())
}

pub (super) fn start_attach_mechanism(pid: i32) -> Result<(), Box<dyn Error>> {
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

pub (super) fn connect_socket(pid: i32) -> Result<UnixStream, Box<dyn Error>> {
    let _path_binding = &format!("{}/.java_pid{}", get_tmp_path(pid).to_str().expect("Could not convert path"), pid);
    let path = Path::new(_path_binding);

    Ok(UnixStream::connect(path)?)
}