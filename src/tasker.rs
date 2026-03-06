use chrono::Local;
use std::{
    ffi::CString,
    fs::File,
    io::Write,
    os::raw::{c_char, c_int},
    time::Instant,
};

/// Structure representing the YAML configuration file
#[derive(Debug, serde::Deserialize)]
struct Config {
    /// A list of commands to be executed sequentially
    commands: Vec<String>,
}

// External C functions for low-level serial operations (likely implemented in C)
unsafe extern "C" {
    fn open_serial(device: *const libc::c_char, baudrate: libc::c_int) -> libc::c_int;
    fn close_serial(fd: libc::c_int);
    fn run_command_on_serial(fd: c_int, cmd: *const c_char, output: *mut c_char, outmax: c_int) -> c_int;
}

/// Dispatches commands based on arguments passed to the tasker module
pub fn handle(args: &[&str]) {
    match args.get(0) {
        Some(&"run") => {
            // Expects: run <file.yml> <serial> <baudrate>
            if let Some(ymlfile) = args.get(1) && let Some(device) = args.get(2) && let Some(baud) = args.get(3) {
                let _ = run(&ymlfile, &device, baud.parse().unwrap());
            } else {
                println!("tasker error!");
            }
        },
        _ => print_help()
    }
}

/// Prints help information for the tasker module
fn print_help() {
    println!("  tasker run <file.yml> <serial> <baudrate>");
}

/// Executes the commands defined in the YAML file on the specified serial device
pub fn run(yml_path: &str, device: &str, baud: i32) -> anyhow::Result<()> {
    // Load and parse the YAML configuration
    let config: Config = serde_yaml::from_reader(File::open(yml_path)?)?;

    println!("Connecting with {} @ {}.", device, baud);

    // Open the serial device using the C FFI
    let c_device = CString::new(device)?;
    let fd = unsafe { open_serial(c_device.as_ptr(), baud) };
    if fd < 0 {
        eprintln!("Failed to open serial device.");
        return Ok(());
    }

    // Generate a timestamped filename for logging (e.g., output_20231027_123456.log)
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let filename = format!("output_{}.log", timestamp);
    let filepath = std::path::PathBuf::from(&filename);

    // Create the log file and initialize a buffered writer
    let log_file = File::create(&filepath)?;
    let mut writer = std::io::BufWriter::new(log_file);

    // Log the start time
    let start_time = Local::now();
    writeln!(writer, "=== Start: {} ===", start_time.format("%Y-%m-%d %H:%M:%S"))?;
    let timer = Instant::now();

    // Iterate through and execute each command
    for cmd in config.commands {
        writeln!(writer, ">>> {}", cmd)?;
        let c_cmd = CString::new(cmd)?;
        let mut buffer = vec![0u8; 4096]; // Buffer to capture command output

        // Execute the command via the C FFI and get the output length
        let outlen = unsafe {
            run_command_on_serial(fd, c_cmd.as_ptr(), buffer.as_mut_ptr() as *mut c_char, buffer.len() as c_int)
        };

        // Convert the output buffer to a string (lossy conversion) and log it
        let out_str = String::from_utf8_lossy(&buffer[..outlen as usize]);
        writeln!(writer, "{}", out_str)?;
        writeln!(writer, "----------------")?;
    }

    // Close the serial device
    unsafe { close_serial(fd) };

    // Log the end time and total duration
    let end_time = Local::now();
    writeln!(writer, "=== End: {} ===", end_time.format("%Y-%m-%d %H:%M:%S"))?;
    writeln!(writer, "=== Duration: {:.2?} ===", timer.elapsed())?;

    Ok(())
}