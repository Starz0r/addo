extern crate winapi;

use std::env;
use std::mem;
use std::process::Command;
use std::path::{Path, PathBuf};
use std::fs::{metadata};
use std::ptr::null_mut;
use std::str;
use std::ffi::OsString;

use winapi::um::winnt::{WCHAR, LPWSTR};
use winapi::shared::minwindef::{DWORD, LPDWORD};
use winapi::um::processthreadsapi;
use winapi::um::winbase;

fn main()
{
    //TODO: Check if the length of the Vector is 0
    let argv: Vec<String> = env::args().collect();

    println!("Verb: {:?}", "runas");
    println!("Program: {:?}", &argv[1]);
    println!("Arguments: {:?}", &argv[2..]);

    /*Command::new(&argv[0])
        .args(&argv[1..])
        .output()
        .expect("failed to execute proccess");*/

    println!("Found in path: {:?}", find_in_path(argv[1].clone()));

    //processthreadsapi::CreateProcessWithLogonW(null_mut(),)
    unsafe
    {
        // Get Currently Logged User
        //let mut buf: [WCHAR; 128] = mem::zeroed();
        let mut buf = vec![0u16; 128];
        let mut size = buf.len() as DWORD;
        winbase::GetUserNameW(buf.as_mut_ptr(), &mut size);

        // Resize array to get rid of empty entries
        buf.set_len(size as usize);

        let user = String::from_utf16_lossy(&buf);
        println!("Username: {:?}", &user);
        println!("Size: {:?}", &size);
    }
}


fn find_in_path<P: AsRef<Path>>(name: P) -> Option<PathBuf> {
    let paths = env::var_os("PATH")?;
    for mut file in env::split_paths(&paths) {
        file.push(&name);
        file.set_extension("exe");
        if file.is_file() {
            return Some(file);
        }
    }
    None
}