use chrono::Local;
use std::{
    ffi::CString,
    fs::File,
    io::Write,
    os::raw::{c_char, c_int},
    time::Instant,
};

#[derive(Debug, serde::Deserialize)]
struct Config {
    commands: Vec<String>,
}

unsafe extern "C" {
    fn open_serial(device: *const libc::c_char, baudrate: libc::c_int) -> libc::c_int;
    fn close_serial(fd: libc::c_int);
    fn run_command_on_serial(fd: c_int, cmd: *const c_char, output: *mut c_char, outmax: c_int) -> c_int;
}

pub fn handle(args: &[&str]) {
    match args.get(0) {
        Some(&"run") => {
            if let Some(ymlfile) = args.get(1) && let Some(device) = args.get(2) && let Some(baud) = args.get(3) {
                let _ = run(&ymlfile, &device, baud.parse().unwrap());
            } else {
                println!("tasker error!");
            }
        },
        _ => print_help()
    }
}

fn print_help() {
    println!("  tasker run <file.yml> <serial> <baudrate>");
}

pub fn run(yml_path: &str, device: &str, baud: i32) -> anyhow::Result<()> {
    let config: Config = serde_yaml::from_reader(File::open(yml_path)?)?;
    // let log_file = File::create("output.log")?;
    // let mut writer = std::io::BufWriter::new(log_file);

    println!("Connecting with {} @ {}.", device, baud);
    
    let c_device = CString::new(device)?;
    let fd = unsafe { open_serial(c_device.as_ptr(), baud) };
    if fd < 0 {
        eprintln!("Failed to open serial device.");
        return Ok(());
    }

    // Generate timestamped filename
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let filename = format!("output_{}.log", timestamp);
    let filepath = std::path::PathBuf::from(&filename);

    // Open writer
    let log_file = File::create(&filepath)?;
    let mut writer = std::io::BufWriter::new(log_file);

    let start_time = Local::now();
    writeln!(writer, "=== Start: {} ===", start_time.format("%Y-%m-%d %H:%M:%S"))?;
    let timer = Instant::now();

    for cmd in config.commands {
        writeln!(writer, ">>> {}", cmd)?;
        let c_cmd = CString::new(cmd)?;
        let mut buffer = vec![0u8; 4096];
        let outlen = unsafe {
            run_command_on_serial(fd, c_cmd.as_ptr(), buffer.as_mut_ptr() as *mut c_char, buffer.len() as c_int)
        };
        let out_str = String::from_utf8_lossy(&buffer[..outlen as usize]);
        writeln!(writer, "{}", out_str)?;
        writeln!(writer, "----------------")?;
    }

    unsafe { close_serial(fd) };

    // --- end time ---
    let end_time = Local::now();
    writeln!(writer, "=== End: {} ===", end_time.format("%Y-%m-%d %H:%M:%S"))?;
    writeln!(writer, "=== Duration: {:.2?} ===", timer.elapsed())?;

    Ok(())
}