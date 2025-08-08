use std::ffi::CString;

unsafe extern "C" {
    fn start_serial_terminal(dev: *const libc::c_char, baud: i32);
}

pub fn handle(args: &[&str]) {
    match args.get(0) {
        Some(&serial) => {
            if let Some(baudrate) = args.get(1) {
                runserial(serial, baudrate);
            } else {
                println!("serial error!");
            }
        }
        _ => print_help()
    }
}

fn print_help() {
    println!("  lngterm <serial> <baudrate>");
}

fn runserial(serial: &str, baudrate: &str) {
    let dev = CString::new(serial).expect("CString::new failed");
    let baud: i32 = baudrate.parse().unwrap_or_else(|_| {
        eprintln!("Invalid baudrate.");
        std::process::exit(1);
    });
    
    unsafe {
        start_serial_terminal(dev.as_ptr(), baud);
    }
    
    println!("Serial terminal exited.");
}