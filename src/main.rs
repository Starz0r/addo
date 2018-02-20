use std::env;
use std::process::Command;
use std::path::Path;
use std::fs::{metadata};

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
}

fn elevate()
{
    //TODO: Add a pattern match here
    let exec = std::env::current_exe().unwrap();
}

fn find_in_path(targ: String) -> Result<String, &'static str>
{
    if (Path::new(&targ).is_absolute() == true)
    {
        let result: Result<String, &'static str> = Ok(targ);
        return result
    }

    match env::var_os("PATH")
    {
        Some(paths) =>
        {
            for path in env::split_paths(&paths)
            {
                // Check if the file just exists in path
                let mut abspath = Path::new(&path).join(&targ);
                let mut meta = metadata(&abspath);
                println!("Absolute Path: {:?}", abspath);

                match meta
                {
                    Ok(m) =>
                        if (m.is_file() == true)
                        {
                            let result: Result<String, &'static str> = Ok(abspath.to_string_lossy().into_owned());
                            return result
                        },
                    _ => {}
                }
            }

            return Err("Command not found in path")
        }

        None =>
        {
            let result: Result<String, &'static str> = Err("No Path Set?");
            return result
        }
    }
}
