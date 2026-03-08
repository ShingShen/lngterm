mod reactor;

use std::io::{self, Write};
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
    event::{self, Event, KeyCode, KeyModifiers},
    style::{Print, ResetColor, SetForegroundColor, Color},
};
use reactor::SerialReactor;
use serialport::{DataBits, FlowControl, Parity, StopBits};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum CliDataBits {
    #[value(name = "5")] Five,
    #[value(name = "6")] Six,
    #[value(name = "7")] Seven,
    #[value(name = "8")] Eight,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum CliParity {
    None,
    Odd,
    Even,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum CliStopBits {
    #[value(name = "1")] One,
    #[value(name = "2")] Two,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum CliFlowControl {
    None,
    Software,
    Hardware,
}

impl From<CliDataBits> for DataBits {
    fn from(v: CliDataBits) -> Self {
        match v {
            CliDataBits::Five => DataBits::Five,
            CliDataBits::Six => DataBits::Six,
            CliDataBits::Seven => DataBits::Seven,
            CliDataBits::Eight => DataBits::Eight,
        }
    }
}

impl From<CliParity> for Parity {
    fn from(v: CliParity) -> Self {
        match v {
            CliParity::None => Parity::None,
            CliParity::Odd => Parity::Odd,
            CliParity::Even => Parity::Even,
        }
    }
}

impl From<CliStopBits> for StopBits {
    fn from(v: CliStopBits) -> Self {
        match v {
            CliStopBits::One => StopBits::One,
            CliStopBits::Two => StopBits::Two,
        }
    }
}

impl From<CliFlowControl> for FlowControl {
    fn from(v: CliFlowControl) -> Self {
        match v {
            CliFlowControl::None => FlowControl::None,
            CliFlowControl::Software => FlowControl::Software,
            CliFlowControl::Hardware => FlowControl::Hardware,
        }
    }
}

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

    /// Data bits (5, 6, 7, 8)
    #[arg(long, value_enum, default_value_t = CliDataBits::Eight)]
    data_bits: CliDataBits,

    /// Parity checking
    #[arg(long, value_enum, default_value_t = CliParity::None)]
    parity: CliParity,

    /// Stop bits (1, 2)
    #[arg(long, value_enum, default_value_t = CliStopBits::One)]
    stop_bits: CliStopBits,

    /// Flow control
    #[arg(long, value_enum, default_value_t = CliFlowControl::None)]
    flow_control: CliFlowControl,
}

fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();
    println!("Connecting to {} at {} baud...", args.device, args.baud);

    // Initialize the serial port with a zero timeout for non-blocking behavior
    let mut port = serialport::new(&args.device, args.baud)
    .data_bits(args.data_bits.into()) 
    .parity(args.parity.into())
    .stop_bits(args.stop_bits.into())
    .flow_control(args.flow_control.into())
    .timeout(Duration::from_millis(0))
    .open_native()
    .with_context(|| format!("Failed to open serial port: {}", args.device))?;

    // Clone the port handle for the background reader thread (reactor)
    let port_reader = port.try_clone_native().context("Failed to duplicate native TTYPort")?;
    
    // Enable raw mode to capture individual key presses without waiting for Enter
    enable_raw_mode().context("Failed to enable raw mode")?;
    
    // Start the asynchronous reactor to handle incoming serial data
    let mut reactor = SerialReactor::start(port_reader).context("Failed to start epoll reactor")?;
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
                if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    break;
                }

                match key.code {
                    KeyCode::Up => port.write_all(b"\x1b[A")?,
                    KeyCode::Down => port.write_all(b"\x1b[B")?,
                    KeyCode::Right => port.write_all(b"\x1b[C")?,
                    KeyCode::Left => port.write_all(b"\x1b[D")?,
                    KeyCode::Home => port.write_all(b"\x1b[H")?, 
                    KeyCode::End => port.write_all(b"\x1b[F")?,
                    KeyCode::PageUp => port.write_all(b"\x1b[5~")?,
                    KeyCode::PageDown => port.write_all(b"\x1b[6~")?,
                    KeyCode::Insert => port.write_all(b"\x1b[2~")?,
                    KeyCode::Delete => port.write_all(b"\x1b[3~")?,
                    
                    KeyCode::Backspace => port.write_all(&[0x08])?,
                    KeyCode::Enter => port.write_all(b"\r")?,
                    KeyCode::Tab => port.write_all(b"\t")?,
                    KeyCode::Esc => port.write_all(b"\x1b")?,

                    KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        let ctrl_byte = c as u8 & 0x1f;
                        port.write_all(&[ctrl_byte])?;
                    }

                    KeyCode::Char(c) => {
                        let mut buf = [0u8; 4];
                        let bytes = c.encode_utf8(&mut buf);
                        port.write_all(bytes.as_bytes())?;
                    }

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