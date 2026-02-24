// FFI from termios //
// mod serial;
// mod tasker;

// use rustyline::{
//     history::DefaultHistory,
//     Editor,
// };
// use std::ffi::CString;

// unsafe extern "C" {
//     fn start_serial_terminal(dev: *const libc::c_char, baud: i32);
// }

// fn print_help() {
//     println!("Available commands:");
//     println!("  exit         - Exit the program");
//     println!("  lngterm      - Start serial terminal");
//     println!("  tasker       - Run and manage tasks defined in YAML files");
//     println!("  ?            - Help");
// }

// fn main() {
//     let args: Vec<String> = std::env::args().collect();
//     if args.len() == 1 {
//         println!("lngterm CLI");
//         println!("Type '?' to see the list of commands, or 'exit' to quit");
        
//         let mut rl = Editor::<(), DefaultHistory>::new().unwrap();
//         loop {
//             let readline = rl.readline("> ");
//             match readline {
//                 Ok(line) => {
//                     let trimmed = line.trim();
//                     if trimmed.is_empty() {
//                         continue;
//                     }

//                     let _ = rl.add_history_entry(trimmed);
        
//                     let parts: Vec<&str> = trimmed.split_whitespace().collect();
//                     if parts.is_empty() {
//                         continue;
//                     }
        
//                     match parts[0] {
//                         "exit" => {
//                             println!("👋 Goodbye!");
//                             break;
//                         }
//                         "lngterm" => serial::handle(&parts[1..]),
//                         "tasker" => tasker::handle(&parts[1..]),
//                         "?" => print_help(),
//                         cmd => {
//                             println!("❗ Unknown command: '{}'", cmd);
//                             println!("Type '?' to see the list of commands.");
//                         },
//                     }
//                 },
//                 Err(_) => {
//                     println!("💥 Error reading input. Exiting.");
//                     break;
//                 }
//             }

//         }
//     }

//     if args.len() == 3 {
//         let dev = CString::new(args[1].clone()).expect("CString::new failed");
//         let baud: i32 = args[2].parse().unwrap_or_else(|_| {
//             eprintln!("Invalid baudrate.");
//             std::process::exit(1);
//         });
    
//         unsafe {
//             start_serial_terminal(dev.as_ptr(), baud);
//         }
    
//         println!("Serial terminal exited.");
//     }
// }
// FFI from termios //


// Pure Rust using userspace tools //
// use std::io::{self, Read, Write};
// use std::os::unix::io::AsRawFd;
// use std::time::Duration;
// use std::thread;
// use std::sync::atomic::{AtomicBool, Ordering};
// use std::sync::Arc;

// use anyhow::{Context, Result};
// use clap::Parser;
// use crossterm::{
//     execute,
//     terminal::{disable_raw_mode, enable_raw_mode},
//     event::{self, Event, KeyCode, KeyModifiers},
//     style::{Print, ResetColor, SetForegroundColor, Color},
// };
// use mio::{Events, Interest, Poll, Token, Waker};
// use mio::unix::SourceFd;

// #[derive(Parser, Debug)]
// #[command(author, version, about, long_about = None)]
// struct Args {
//     #[arg(short, long)]
//     device: String,

//     #[arg(short, long, default_value_t = 115200)]
//     baud: u32,
// }

// fn main() -> Result<()> {
//     let args = Args::parse();

//     println!("Connecting to {} at {} baud...", args.device, args.baud);

//     let mut port = serialport::new(&args.device, args.baud)
//         .timeout(Duration::from_millis(0)) 
//         .open_native()
//         .with_context(|| format!("Failed to open serial port natively: {}", args.device))?;

//     let mut port_reader = port.try_clone_native().context("Failed to duplicate native TTYPort")?;
    
//     enable_raw_mode()?;
//     let mut stdout = io::stdout();

//     execute!(
//         stdout,
//         SetForegroundColor(Color::Green),
//         Print(format!("Connected to {} at {} baud.\r\n", args.device, args.baud)),
//         Print("Press 'Ctrl + Q' to exit.\r\n"),
//         Print("--------------------------------------------------\r\n"),
//         ResetColor
//     )?;

//     let keep_running = Arc::new(AtomicBool::new(true));
//     let keep_running_clone = keep_running.clone();
//     let mut poll = match Poll::new() {
//         Ok(p) => p,
//         Err(_) => return Ok(()),
//     };
//     let mut events = Events::with_capacity(128);

//     const SERIAL_TOKEN: Token = Token(0);
//     const WAKER_TOKEN: Token = Token(1);

//     let waker = match Waker::new(poll.registry(), WAKER_TOKEN) {
//         Ok(w) => Arc::new(w),
//         Err(_) => return Ok(()),
//     };

//     let raw_fd = port_reader.as_raw_fd();
    
//     if poll.registry().register(&mut SourceFd(&raw_fd), SERIAL_TOKEN, Interest::READABLE).is_err() {
//         return Ok(());
//     }

//     let reader_thread = thread::spawn(move || {
//         let mut serial_buf = [0u8; 1024];
//         // let mut serial_buf = [0u8; 16];
//         let mut stdout = io::stdout();

//         while keep_running_clone.load(Ordering::Relaxed) {
//             match poll.poll(&mut events, None) {
//                 Ok(_) => {}
//                 Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {
//                     continue;
//                 }
//                 Err(e) => {
//                     eprintln!("\r\n[Reader Thread] epoll failed: {:?}", e);
//                     break;
//                 }
//             }

//             for event in events.iter() {
//                 match event.token() {
//                     SERIAL_TOKEN => {
//                         if event.is_readable() {
//                             match port_reader.read(&mut serial_buf) {
//                                 Ok(t) if t > 0 => {
//                                     let _ = stdout.write_all(&serial_buf[..t]);
//                                     let _ = stdout.flush();
//                                 }
//                                 Ok(_) => {},
//                                 Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
//                                 Err(e) => {
//                                     eprintln!("\r\n[Reader Thread] read failed: {:?}", e);
//                                     break;
//                                 }
//                             }
//                         }
//                     },
//                     WAKER_TOKEN => {
//                         continue; 
//                     },
//                     _ => unreachable!(),
//                 }
//             }
//         }
//     });

//     loop {
//         if event::poll(Duration::from_millis(100))? {
//             if let Event::Key(key) = event::read()? {
//                 match key.code {
//                     KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
//                         break;
//                     }
//                     KeyCode::Enter => {
//                         port.write_all(b"\r")?;
//                     }
//                     KeyCode::Char(c) => {
//                         let mut buf = [0u8; 4];
//                         let bytes = c.encode_utf8(&mut buf);
//                         port.write_all(bytes.as_bytes())?;
//                     }
//                     KeyCode::Backspace => {
//                         port.write_all(&[0x08])?;
//                     }
//                     KeyCode::Esc => {
//                          port.write_all(&[0x1b])?;
//                     }
//                     _ => {}
//                 }
//             }
//         }
//     }

//     keep_running.store(false, Ordering::Relaxed);
//     let _ = waker.wake();
//     let _ = reader_thread.join();

//     disable_raw_mode()?;
//     println!("Disconnected.");

//     Ok(())
// }
// Pure Rust using userspace tools //

// //
mod reactor;

use std::io::{self, Write};
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
    event::{self, Event, KeyCode, KeyModifiers},
    style::{Print, ResetColor, SetForegroundColor, Color},
};
use reactor::SerialReactor;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    device: String,
    #[arg(short, long, default_value_t = 115200)]
    baud: u32,
}

fn main() -> Result<()> {
    let args = Args::parse();
    println!("Connecting to {} at {} baud...", args.device, args.baud);

    let mut port = serialport::new(&args.device, args.baud)
        .timeout(Duration::from_millis(0)) 
        .open_native()
        .with_context(|| format!("Failed to open serial port natively: {}", args.device))?;

    let port_reader = port.try_clone_native().context("Failed to duplicate native TTYPort")?;
    
    let mut reactor = SerialReactor::start(port_reader).context("Failed to start epoll reactor")?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        SetForegroundColor(Color::Green),
        Print(format!("Connected to {} at {} baud.\r\n", args.device, args.baud)),
        Print("Press 'Ctrl + Q' to exit.\r\n"),
        Print("--------------------------------------------------\r\n"),
        ResetColor
    )?;

    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Enter => { port.write_all(b"\r")?; }
                    KeyCode::Char(c) => {
                        let mut buf = [0u8; 4];
                        let bytes = c.encode_utf8(&mut buf);
                        port.write_all(bytes.as_bytes())?;
                    }
                    KeyCode::Backspace => { port.write_all(&[0x08])?; }
                    KeyCode::Esc => { port.write_all(&[0x1b])?; }
                    _ => {}
                }
            }
        }
    }

    reactor.stop();
    disable_raw_mode()?;
    println!("Disconnected.");
    Ok(())
}
// //