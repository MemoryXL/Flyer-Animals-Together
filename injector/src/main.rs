use hudhook::inject::Process;
use std::env;
use std::path::PathBuf;
use std::io::{self, Read};
use std::thread;
use std::time::Duration;
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

fn is_elevated() -> bool {
    unsafe {
        let mut token_handle = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle).is_err() {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut return_length = 0;
        let result = GetTokenInformation(
            token_handle,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut return_length,
        );

        result.is_ok() && elevation.TokenIsElevated != 0
    }
}

fn request_admin_restart() {
    use std::process::Command;

    let current_exe = env::current_exe().unwrap();
    let _ = Command::new("powershell")
        .args([
            "-Command",
            &format!("Start-Process '{}' -Verb RunAs", current_exe.display()),
        ])
        .status();
}

fn main() {
    if !is_elevated() {
        println!("This injector requires administrator privileges.");
        println!("Requesting administrator privileges...");
        request_admin_restart();
        return;
    }

    let process_name = "Climber Animals Together.exe";
    let dll_name = "tool.dll";

    println!("Looking for process: {}", process_name);

    let mut dll_path = env::current_exe().unwrap();
    dll_path.pop();
    dll_path.push(dll_name);

    if !dll_path.exists() {
        let debug_path = PathBuf::from("target/debug/tool.dll");
        let release_path = PathBuf::from("target/release/tool.dll");
        
        if debug_path.exists() {
            dll_path = debug_path;
        } else if release_path.exists() {
            dll_path = release_path;
        }
    }

    if !dll_path.exists() {
        eprintln!("Error: Could not find '{}'. Make sure to build the overlay first.", dll_name);
        println!("Press ENTER to exit...");
        let _ = io::stdin().read(&mut [0u8]).unwrap();
        return;
    }

    let dll_path = match dll_path.canonicalize() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error resolving DLL path: {}", e);
            println!("Press ENTER to exit...");
            let _ = io::stdin().read(&mut [0u8]).unwrap();
            return;
        }
    };

    println!("Found DLL at: {:?}", dll_path);

    loop {
        match Process::by_name(process_name) {
            Ok(proc) => {
                println!("Found process. Injecting...");
                match proc.inject(dll_path.clone()) {
                    Ok(_) => {
                        println!("Injection successful!");
                        println!("Auto-closing in 15 seconds...");
                        thread::sleep(Duration::from_secs(15));
                        return;
                    },
                    Err(e) => {
                        eprintln!("Injection failed: {:?}", e);
                        println!("Press ENTER to exit...");
                        let _ = io::stdin().read(&mut [0u8]).unwrap();
                        return;
                    }
                }
            },
            Err(_) => {
                println!("Game not running. Waiting for game to start...");
                thread::sleep(Duration::from_secs(2));
            }
        }
    }
}
