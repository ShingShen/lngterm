mod serial;
mod tasker;

use rustyline::{
    history::DefaultHistory,
    Editor,
};
use std::ffi::CString;

unsafe extern "C" {
    fn start_serial_terminal(dev: *const libc::c_char, baud: i32);
}

fn print_help() {
    println!("Available commands:");
    println!("  exit         - Exit the program");
    println!("  lngterm      - Start serial terminal");
    println!("  tasker       - Run and manage tasks defined in YAML files");
    println!("  ?            - Help");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        println!("lngterm CLI");
        println!("Type '?' to see the list of commands, or 'exit' to quit");
        
        let mut rl = Editor::<(), DefaultHistory>::new().unwrap();
        loop {
            let readline = rl.readline("> ");
            match readline {
                Ok(line) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    let _ = rl.add_history_entry(trimmed);
        
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.is_empty() {
                        continue;
                    }
        
                    match parts[0] {
                        "exit" => {
                            println!("👋 Goodbye!");
                            break;
                        }
                        "lngterm" => serial::handle(&parts[1..]),
                        "tasker" => tasker::handle(&parts[1..]),
                        "?" => print_help(),
                        cmd => {
                            println!("❗ Unknown command: '{}'", cmd);
                            println!("Type '?' to see the list of commands.");
                        },
                    }
                },
                Err(_) => {
                    println!("💥 Error reading input. Exiting.");
                    break;
                }
            }

        }
    }

    if args.len() == 3 {
        let dev = CString::new(args[1].clone()).expect("CString::new failed");
        let baud: i32 = args[2].parse().unwrap_or_else(|_| {
            eprintln!("Invalid baudrate.");
            std::process::exit(1);
        });
    
        unsafe {
            start_serial_terminal(dev.as_ptr(), baud);
        }
    
        println!("Serial terminal exited.");
    }
}