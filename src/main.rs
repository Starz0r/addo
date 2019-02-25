extern crate winapi;

use std::ptr;
use std::process;
use std::env;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::mem::size_of_val;

use winapi::shared::minwindef::*;
use winapi::um::shellapi::{SHELLEXECUTEINFOW, SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, ShellExecuteExW};
use winapi::um::winbase::{INFINITE};
use winapi::um::synchapi::{WaitForSingleObject};
use winapi::um::wincon::{AttachConsole, FreeConsole};
use winapi::um::securitybaseapi::{AllocateAndInitializeSid, CheckTokenMembership, FreeSid};
use winapi::um::processthreadsapi::{GetCurrentProcessId};
use winapi::um::winnt::{PSID, SECURITY_NT_AUTHORITY, SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_ADMINS, SID_IDENTIFIER_AUTHORITY};
use winapi::um::winuser::{SW_HIDE};

fn main() {
    let argv: Vec<String> = env::args().collect();

    if (argv.len() == 0) {
        println!("elevate: no arguments passed");
        process::exit(1);
    }

    if argv[1] == "--internal-server-mode" {
        unsafe {
            process::exit(elevate(argv[2].parse().unwrap(), &argv[3], argv[4..].to_vec()));
        }
    }

    unsafe {
        if (!is_admin()) {
            println!("elevate: you must be an administrator to run elevate");
            process::exit(1);
        }

        //TODO: Handle fail cases (https://doc.rust-lang.org/1.16.0/std/env/fn.current_dir.html)
        let wd = env::current_dir().unwrap();
        let pid = GetCurrentProcessId().to_string();
        let fork_args = vec!["--internal-server-mode", &pid, wd.to_str().unwrap(), &argv[1..].join(" ")].join(" ");

        shell_execute_and_wait("runas".to_string(), argv[0].to_string(), fork_args, wd.to_str().unwrap().to_string(), SW_HIDE).unwrap();
        /*let mut cmd = process::Command::new("runas")
            .args(&["powershell"])
            .spawn()
            .expect("elevate: insufficient access rights");*/

        //FreeConsole(); // TODO: catch this
        //AttachConsole();

    }
}


unsafe fn is_admin() -> bool {
    // https://docs.microsoft.com/en-us/windows/desktop/api/securitybaseapi/nf-securitybaseapi-checktokenmembership
    let mut b: BOOL;
    let mut nt_authority = SID_IDENTIFIER_AUTHORITY{Value: SECURITY_NT_AUTHORITY};
    let mut admin: PSID = ptr::null_mut();
    b = AllocateAndInitializeSid(&mut nt_authority,
    2,
    SECURITY_BUILTIN_DOMAIN_RID,
    DOMAIN_ALIAS_RID_ADMINS,
    0, 0, 0, 0, 0, 0,
    &mut admin);

    // TODO: Fix this with yields
    if (b != 0) {
        if CheckTokenMembership(ptr::null_mut(), admin, &mut b) == 0 {
            b = FALSE;
        } else {
            b = TRUE;
        }
        FreeSid(admin);
    }

    b != 0
}

unsafe fn shell_execute_and_wait(lp_operation: String, lp_file: String, lp_parameters: String, lp_directory: String, n_show_cmd: i32,) -> Result<u32, &'static str> {
    use winapi::_core::iter::once;

    // Encode the arguments correctly as Wide or UTF-16-CS
    let operation_wide: Vec<u16> = OsStr::new(&lp_operation)
        .encode_wide()
        .chain(once(0))
        .collect();

    let parameters_wide: Vec<u16> = OsStr::new(&lp_parameters)
        .encode_wide()
        .chain(once(0))
        .collect();

    let directory_wide: Vec<u16> = OsStr::new(&lp_directory)
        .encode_wide()
        .chain(once(0))
        .collect();

    let file_wide: Vec<u16> = OsStr::new(&lp_file).encode_wide().chain(once(0)).collect();

    // Define ShellExecuteInfoW
    let mut info = SHELLEXECUTEINFOW {
        cbSize: 0,
        fMask: SEE_MASK_NOASYNC | SEE_MASK_NOCLOSEPROCESS,
        hwnd: ptr::null_mut(),
        lpVerb: operation_wide.as_ptr(),
        lpFile: file_wide.as_ptr(),
        lpParameters: parameters_wide.as_ptr(),
        lpDirectory: directory_wide.as_ptr(),
        nShow: n_show_cmd,
        hInstApp: ptr::null_mut(),
        lpIDList: ptr::null_mut(),
        lpClass: ptr::null_mut(),
        hkeyClass: ptr::null_mut(),
        dwHotKey: 0,
        hMonitor: ptr::null_mut(),
        hProcess: ptr::null_mut(),
    };
    info.cbSize = size_of_val(&info) as u32;

    // Carry out the task and wait
    let task_completed = ShellExecuteExW(&mut info);
    if (task_completed == 1) && (info.fMask & SEE_MASK_NOCLOSEPROCESS != 0) {
        //TODO: Match statement here so we can actually return a error if it fails
        WaitForSingleObject(info.hProcess, INFINITE);
    }

    // Return the result
    return Ok(0);
}

unsafe fn elevate(parent: u32, directory: &str, args: Vec<String>) -> i32 {
    FreeConsole(); // TODO: catch this
    AttachConsole(parent);

    let cmd = process::Command::new(&args[0])
        .current_dir(directory)
        .args(&args[1..])
        .stdin(process::Stdio::inherit())
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit())
        .output();

    cmd.unwrap().status.code().unwrap()
}
