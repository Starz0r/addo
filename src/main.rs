extern crate named_pipe;
extern crate winapi;
extern crate winutil;

use std::env;
use std::ffi::OsStr;
use std::io;
use std::iter::once;
use std::mem::size_of_val;
use std::os::windows::ffi::OsStrExt;
use std::process;
use std::ptr;
use std::thread;

use winapi::um::shellapi;
use winapi::um::synchapi;
use winapi::um::winbase;
use winapi::um::winuser;

fn main() {
    //TODO: Check if the length of the Vector is 0
    let argv: Vec<String> = env::args().collect();

    // Check if we are the client or server
    if argv[1] != "client_mode" {
        // If we are the server, start up a Named Pipe
        let pipe_dir = format!("\\\\.\\\\pipe\\elevate\\{}", process::id());
        let clone_dir = pipe_dir.clone();
        thread::spawn(move || wait_for_connection_and_relay(clone_dir));

        unsafe {
            // Get Working Directory
            //TODO: Handle fail cases (https://doc.rust-lang.org/1.16.0/std/env/fn.current_dir.html)
            let wd = env::current_dir().unwrap();

            // Execute Process
            let fork_args = vec!["client_mode", &pipe_dir, &argv[1..].join(" ")].join(" ");
            let fork = &argv[0];
            shell_execute_and_wait(
                "runas".to_string(),
                fork.to_string(),
                fork_args,
                wd.to_str().unwrap().to_string(),
                winuser::SW_HIDE,
            )
            .unwrap();
        }
    } else {
        // If we are the client, connect to the server, and begin piping i/o
        //TODO: Pattern match the Result instead of unwrapping it.
        let mut conn = named_pipe::PipeClient::connect(&argv[2]).unwrap();

        // Execute the process command
        let mut cmd = process::Command::new(&argv[3])
            .args(&argv[4..])
            .stdout(process::Stdio::piped())
            .spawn()
            .expect("Could not find, or insufficient access rights to executable.");
        let mut cmd_out = cmd.stdout.take().unwrap();

        // Spawn a new thread to handle sending data through the pipe
        thread::spawn(move || relay_stdin_to_out(&mut conn, &mut cmd_out));

        // Wait for it to finish before exiting
        //TODO: Use this exit code
        cmd.wait().unwrap();
    }
}

unsafe fn shell_execute_and_wait(
    lp_operation: String,
    lp_file: String,
    lp_parameters: String,
    lp_directory: String,
    n_show_cmd: i32,
) -> Result<u32, &'static str> {
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
    let mut info = shellapi::SHELLEXECUTEINFOW {
        cbSize: 0,
        fMask: shellapi::SEE_MASK_NOASYNC | shellapi::SEE_MASK_NOCLOSEPROCESS,
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
    let task_completed = shellapi::ShellExecuteExW(&mut info);
    if (task_completed == 1) && (info.fMask & shellapi::SEE_MASK_NOCLOSEPROCESS != 0) {
        //TODO: Match statement here so we can actually return a error if it fails
        synchapi::WaitForSingleObject(info.hProcess, winbase::INFINITE);
    }

    // Return the result
    return Ok(0);
}

fn relay_stdin_to_out(
    mut conn: &mut named_pipe::PipeClient,
    mut cmd_out: &mut process::ChildStdout,
) {
    'relay: loop {
        io::copy(&mut cmd_out, &mut conn).unwrap();
    }
}

fn wait_for_connection_and_relay(pipe_dir: String) {
    //TODO: Handle all the monads leading up to PipeServer
    let mut pipe = named_pipe::PipeOptions::new(&pipe_dir)
        .single()
        .unwrap()
        .wait()
        .unwrap();

    let mut stdout = io::stdout();

    'relay: loop {
        io::copy(&mut pipe, &mut stdout).unwrap();
    }
}
