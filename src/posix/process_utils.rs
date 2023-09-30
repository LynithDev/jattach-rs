use std::{path::{Path, PathBuf}, fs, error::Error};

pub fn get_tmp_path(pid: i32) -> PathBuf {
    let attach_path = std::env::var("ATTACH_PATH");

    match attach_path {
        Ok(path) => Path::new(&path).to_path_buf(),
        Err(_) => {
            let path = Path::new(&format!("/proc/{}/root/tmp", pid)).to_path_buf();
            
            if path.exists() {
                path
            } else {
                Path::new("/tmp").to_path_buf()
            }
        }
    }
}

pub fn get_process_info(pid: i32) -> Result<(u32, u32), Box<dyn Error>> {
    let path = Path::new(&format!("/proc/{}/status", pid)).to_path_buf();
    let content = fs::read_to_string(path)?;

    let mut uid: Option<u32> = None;
    let mut gid: Option<u32> = None;
    
    for line in content.lines() {
        if uid != None && gid != None {
            break;
        }

        match line {
            line if line.starts_with("Uid:") => {
                let mut split = line.split_whitespace();
                split.next();
                uid = Some(split.next().unwrap().parse()?);
            },

            line if line.starts_with("Gid:") => {
                let mut split = line.split_whitespace();
                split.next();
                gid = Some(split.next().unwrap().parse()?);
            },

            _ => continue,
        }
    }

    match (uid, gid) {
        (Some(uid), Some(gid)) => Ok((uid, gid)),
        _ => Err("Could not get uid and gid".into()),
    }
}