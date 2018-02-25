extern crate winapi;
extern crate winutil;

use std::env;
use std::path::{Path, PathBuf};
use std::io;
use std::ptr;
use std::ffi::OsStr;
use std::os::windows::io::AsRawHandle;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::mem::size_of_val;

use winapi::um::winnt::{WCHAR, LPWSTR};
use winapi::shared::minwindef::{DWORD, LPDWORD};
use winapi::um::processthreadsapi;
use winapi::um::winbase;
use winapi::um::processenv;
use winapi::um::errhandlingapi::GetLastError;

fn main()
{
    //TODO: Check if the length of the Vector is 0
    let argv: Vec<String> = env::args().collect();

    println!("Verb: {:?}", "runas");
    println!("Program: {:?}", &argv[1]);
    println!("Arguments: {:?}", &argv[2..]);

    //TODO: Handle the Option monad instead of unwrapping it
    let appname = find_in_path(argv[1].clone()).unwrap();
    let appnameUTF16: Vec<u16> = OsStr::new(&appname)
        .encode_wide()
        .chain(once(0))
        .collect();

    println!("Found in path: {:?}", appname);

    unsafe
    {
        // Get Currently Logged User
        let mut user = vec![0u16; 128];
        let mut size = user.len() as DWORD;
        winbase::GetUserNameW(user.as_mut_ptr(), &mut size);

        // Resize array to get rid of empty entries
        user.set_len(size as usize);

        //Handle Standard I/O
        let mut stdin = io::stdin().as_raw_handle();
        let mut stdout = io::stdout().as_raw_handle();
        let mut stderr = io::stderr().as_raw_handle();

        //Define StartupInfo
        let mut deskwide: Vec<u16> = OsStr::new(&winutil::get_computer_name().unwrap())
            .encode_wide()
            .chain(once(0))
            .collect();
        let mut stup = processthreadsapi::STARTUPINFOW
        {
            cb: 0,
            lpReserved: ptr::null_mut(),
            lpDesktop: deskwide.as_mut_ptr(),
            lpTitle: ptr::null_mut(),
            dwX: 0,
            dwY: 0,
            dwXSize: 0,
            dwYSize: 0,
            dwXCountChars: 0,
            dwYCountChars: 0,
            dwFillAttribute: 0,
            dwFlags: winbase::STARTF_USESTDHANDLES,
            wShowWindow: 0,
            cbReserved2: 0,
            lpReserved2: ptr::null_mut(),
            hStdInput: stdin,
            hStdOutput: stdout,
            hStdError: stderr,
        };
        stup.cb = size_of_val(&stup).count_zeros();

        //Execute Process
        let cmdline = processenv::GetCommandLineW();
        processthreadsapi::CreateProcessWithLogonW
        (
            user.as_ptr(),
            ptr::null_mut(),
            ptr::null(),
            1,
            appnameUTF16.as_ptr(),
            cmdline,
            winbase::CREATE_DEFAULT_ERROR_MODE | winbase::CREATE_NEW_PROCESS_GROUP,
            ptr::null_mut(),
            ptr::null(),
            &mut stup,
            ptr::null_mut()
        );

        println!("Last Error: {:?}", GetLastError());
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