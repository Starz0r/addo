#![feature(getpid)]

extern crate winapi;
extern crate winutil;
extern crate named_pipe;

use std::env;
use std::path::{Path, PathBuf};
use std::io;
use std::ptr;
use std::ffi::OsStr;
use std::os::windows::io::AsRawHandle;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::mem::size_of_val;
use std::process;
use std::thread;

use winapi::um::winnt::{WCHAR, LPWSTR};
use winapi::shared::minwindef::{DWORD, LPDWORD};
use winapi::um::processthreadsapi;
use winapi::um::winbase;
use winapi::um::processenv;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::shellapi;
use winapi::um::winuser;
use winapi::um::synchapi;

fn main()
{
    //TODO: Check if the length of the Vector is 0
    let argv: Vec<String> = env::args().collect();

    // Check if we are the client or server
    if argv[1] != "client_mode"
    {
        println!("Mode: {}", "server");
        // If we are the server, start up a Named Pipe
        let pipe_dir = format!("\\\\.\\\\pipe\\elevate\\{}", process::id());
        let clone_dir = pipe_dir.clone();
        //thread::spawn(move || pipe_wait_for_connection(pipe_dir.clone()));
        thread::spawn(move || wait_for_connection_and_relay(clone_dir));

        println!("Process ID: {:?}", process::id());

        println!("Verb: {:?}", "runas");
        println!("Program: {:?}", &argv[1]);
        println!("Arguments: {:?}", &argv[2..]);

        //TODO: Handle if there is nothing found in path by panicing
        let appname = find_in_path(argv[1].clone()).unwrap();
        let appnameUTF16: Vec<u16> = OsStr::new(&appname)
            .encode_wide()
            .chain(once(0))
            .collect();

        println!("Found in path: {:?}", appname);

        unsafe
        {
            // Explicitly synchronize relay threads
            //thread::spawn(|| relay_stdout_server());

            // Handle Standard I/O
            let mut stdin = io::stdin().as_raw_handle();
            let mut stdout = io::stdout().as_raw_handle();
            let mut stderr = io::stderr().as_raw_handle();

            //

            // Get Working Directory
            //TODO: Handle fail cases (https://doc.rust-lang.org/1.16.0/std/env/fn.current_dir.html)
            let wd = env::current_dir().unwrap();

            // Execute Process
            let fork_args = vec!["client_mode", &pipe_dir, &argv[1..].join(" ")].join(" ");
            let fork = &argv[0];
            shell_execute_and_wait("runas".to_string(), fork.to_string(), fork_args, wd.to_str().unwrap().to_string(), winuser::SW_HIDE);

            println!("Last Error: {:?}", GetLastError());
        }
    }
    else
    {
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
        cmd.wait();
    }
}

unsafe fn shell_execute_and_wait(lpOperation: String, lpFile: String, lpParameters: String, lpDirectory: String, nShowCmd: i32) -> Result<u32, &'static str>
{

    // Encode the arguments correctly as Wide or UTF-16-CS
    let operationWide: Vec<u16> = OsStr::new(&lpOperation)
        .encode_wide()
        .chain(once(0))
        .collect();

    let parametersWide: Vec<u16> = OsStr::new(&lpParameters)
        .encode_wide()
        .chain(once(0))
        .collect();

    let directoryWide: Vec<u16> = OsStr::new(&lpDirectory)
        .encode_wide()
        .chain(once(0))
        .collect();

    let fileWide: Vec<u16> = OsStr::new(&lpFile)
        .encode_wide()
        .chain(once(0))
        .collect();

    // Define ShellExecuteInfoW
    let mut info = shellapi::SHELLEXECUTEINFOW
    {
        cbSize: 0,
        fMask: shellapi::SEE_MASK_NOASYNC | shellapi::SEE_MASK_NOCLOSEPROCESS,
        hwnd: ptr::null_mut(),
        lpVerb: operationWide.as_ptr(),
        lpFile: fileWide.as_ptr(),
        lpParameters: parametersWide.as_ptr(),
        lpDirectory: directoryWide.as_ptr(),
        nShow: nShowCmd,
        hInstApp: ptr::null_mut(),
        lpIDList: ptr::null_mut(),
        lpClass: ptr::null_mut(),
        hkeyClass: ptr::null_mut(),
        dwHotKey: 0,
        hMonitor: ptr::null_mut(),
        hProcess: ptr::null_mut(),
    };
    info.cbSize = size_of_val(&info) as u32;

    println!("Size: {:?}\nlpVerb: {:?}\nlpFile: {:?}\nlpParameters: {:?}\nlpDirectory: {:?}",
             size_of_val(&info) as u32,
             String::from_utf16_lossy(&operationWide),
             String::from_utf16_lossy(&fileWide),
             String::from_utf16_lossy(&parametersWide),
             String::from_utf16_lossy(&directoryWide));

    // Carry out the task and wait
    let task_completed = shellapi::ShellExecuteExW(&mut info);
    if (task_completed == 1) && (info.fMask&shellapi::SEE_MASK_NOCLOSEPROCESS != 0)
    {
        //TODO: Match statement here so we can actually return a error if it fails
        let result = synchapi::WaitForSingleObject(info.hProcess, winbase::INFINITE);
        println!("Ok, we are done waiting.");
    }

    // Return the result
    return Ok(0);
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

fn relay_stdin_to_out(mut conn: &mut named_pipe::PipeClient, mut cmd_out: &mut process::ChildStdout)
{
    use std::io::Read;
    use std::io::Write;

    let mut stdout = io::stdout();
    //let mut mutex_out = stdout.lock();

    let mut stdin = io::stdin();
    //let mut mutex_in = stdin.lock();

    let mut buf: [u8; 1024] = [0; 1024];

    conn.write(b"hai").unwrap();
    conn.flush().unwrap();

    'relay: loop
    {
        //TODO: Use async here and match the monad instead of wasting cycles
        /*let solution = match mutex_in.read(&mut buf)
        {
            Ok(0) =>
            {
                // Do nothing, there are no bytes to be read and relayed
            },
            Ok(size) =>
            {
                // Write to stdout and flush to display new information
                //TODO: Handle any result monads from attempting to write to a pipe
                conn.write(&buf).unwrap();
                conn.flush().unwrap();
            },
            Err(_) =>
            {
                //TODO: Likely EOF, there are no more operations to be returned
                break 'relay;
            }
        };*/
        let size = io::copy(&mut cmd_out, &mut conn).unwrap();
    }
}

fn wait_for_connection_and_relay(pipe_dir: String)
{
    use std::io::Read;
    use std::io::Write;

    //TODO: Handle all the monads leading up to PipeServer
    let mut pipe = named_pipe::PipeOptions::new(&pipe_dir).single().unwrap().wait().unwrap();
    println!("Connection established!");

    let mut stdout = io::stdout();

    let mut stdin = io::stdin();

    let mut buf: [u8; 1024] = [0; 1024];

    'relay: loop
    {
        let size = io::copy(&mut pipe, &mut stdout).unwrap();
        //println!("Reply received, byte size: {:?}", size);
    }
}