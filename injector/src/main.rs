use hudhook::inject::Process;
use std::env;
use std::path::PathBuf;
use std::io::{self, Read};

fn main() {
    let process_name = "Climber Animals Together.exe";
    let dll_name = "tool.dll";

    println!("Looking for process: {}", process_name);

    // Try to find the DLL
    let mut dll_path = env::current_exe().unwrap();
    dll_path.pop(); // remove injector.exe
    dll_path.push(dll_name);

    if !dll_path.exists() {
        // Try common cargo build locations relative to current dir
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
        return;
    }

    let dll_path = match dll_path.canonicalize() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error resolving DLL path: {}", e);
            return;
        }
    };

    println!("Found DLL at: {:?}", dll_path);

    // Inject
    match Process::by_name(process_name) {
        Ok(proc) => {
            println!("Found process. Injecting...");
            match proc.inject(dll_path) {
                Ok(_) => {
                    println!("Injection successful!");
                    println!("Press ENTER to exit...");
                    let _ = io::stdin().read(&mut [0u8]).unwrap();
                },
                Err(e) => eprintln!("Injection failed: {:?}", e),
            }
        },
        Err(e) => {
            eprintln!("Error finding process '{}': {:?}", process_name, e);
            eprintln!("Please ensure the game is running before running the injector.");
        }
    }
}
