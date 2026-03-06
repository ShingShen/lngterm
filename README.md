# 📟 lngterm

A lightweight serial terminal written in Rust. Uses mio/epoll for efficient I/O, crossterm for raw terminal handling, and supports keyboard forwarding (Ctrl+Q to exit).

---

## ✨ Features

- Serial terminal connection via `-d/--device` and `-b/--baud`
- epoll-based reactor pattern for non-blocking serial read
- Raw terminal mode with crossterm (Enter, Backspace, Esc forwarding)
- Press **Ctrl + Q** to exit

---

## 📦 Installation

### Prerequisites

- Rust toolchain (stable)
- Linux (uses native TTY / epoll)

```bash
git clone https://github.com/ShingShen/lngterm.git
cd lngterm
cargo build --release
```

Binary: `target/release/lngterm`

---

## 🚀 Usage

```bash
# Connect to serial port
./lngterm -d /dev/ttyUSB0 -b 115200
```

Options:

| Option | Description |
|--------|-------------|
| `-d, --device` | Serial device path (required) |
| `-b, --baud`   | Baud rate (default: 115200) |

---

## 🏗️ Architecture

- **Main thread**: crossterm event loop for keyboard input → serial write
- **Reactor thread**: mio epoll on serial fd → read → stdout write

See `src/reactor.rs` for the epoll reactor implementation.

---

## 📁 pyterm

`pyterm/` contains a Python serial terminal (`pyterm.py`) used as a reference implementation for **performance comparison** with lngterm. It is functionally similar (serial read/write, raw mode, Ctrl+Q exit) but implemented with pyserial + termios + threading.

---

## 📊 Performance Comparison

Benchmarked on Linux with virtual serial (`socat` pty pair). Startup = time to first "Connected" output. Memory = RSS while idle.

| Metric | lngterm (Rust) | pyterm (Python) |
|--------|----------------|-----------------|
| Binary / deploy size | 1.36 MB (single binary) | 1.6 KB script + 682 KB pyserial + Python 3 runtime |
| Startup time | ~0.01 s | ~0.19 s |
| Memory (RSS, idle) | ~2.7 MB | ~13 MB |

Run `./bench.sh` (requires `socat`, `script`) to reproduce.

---

## ⚠️ Platform Support

- ✅ Linux (epoll, termios, native serial)
- ❌ Windows: Not supported
- ❌ macOS: Not supported

---

## 📄 License

MIT License. See [LICENSE](./LICENSE) for details.
