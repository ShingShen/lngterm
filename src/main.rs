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