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

/// Command line arguments for lngterm
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the serial device (e.g., /dev/ttyUSB0)
    #[arg(short, long)]
    device: String,
    /// Baud rate for the serial connection
    #[arg(short, long, default_value_t = 115200)]
    baud: u32,
}

fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();
    println!("Connecting to {} at {} baud...", args.device, args.baud);

    // Initialize the serial port with a zero timeout for non-blocking behavior
    let mut port = serialport::new(&args.device, args.baud)
        .timeout(Duration::from_millis(0)) 
        .open_native()
        .with_context(|| format!("Failed to open serial port natively: {}", args.device))?;

    // Clone the port handle for the background reader thread (reactor)
    let port_reader = port.try_clone_native().context("Failed to duplicate native TTYPort")?;

    // Start the asynchronous reactor to handle incoming serial data
    let mut reactor = SerialReactor::start(port_reader).context("Failed to start epoll reactor")?;

    // Enable raw mode to capture individual key presses without waiting for Enter
    enable_raw_mode()?;
    let mut stdout = io::stdout();

    // Print connection status message with styling
    execute!(
        stdout,
        SetForegroundColor(Color::Green),
        Print(format!("Connected to {} at {} baud.\r\n", args.device, args.baud)),
        Print("Press 'Ctrl + Q' to exit.\r\n"),
        Print("--------------------------------------------------\r\n"),
        ResetColor
    )?;

    // Main loop for handling keyboard input
    loop {
        // Poll for terminal events every 100ms
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    // Exit the application on Ctrl+Q
                    KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => break,

                    // Forward Enter key as carriage return (\r)
                    KeyCode::Enter => { port.write_all(b"\r")?; }

                    // Handle standard character input
                    KeyCode::Char(c) => {
                        let mut buf = [0u8; 4];
                        let bytes = c.encode_utf8(&mut buf);
                        port.write_all(bytes.as_bytes())?;
                    }

                    // Forward Backspace (0x08)
                    KeyCode::Backspace => { port.write_all(&[0x08])?; }

                    // Forward Escape (0x1b)
                    KeyCode::Esc => { port.write_all(&[0x1b])?; }

                    _ => {}
                }
            }
        }
    }

    // Cleanup: stop the reactor and restore terminal mode
    reactor.stop();
    disable_raw_mode()?;
    println!("Disconnected.");
    Ok(())
}