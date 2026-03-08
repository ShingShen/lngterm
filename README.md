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
./lngterm -d /dev/ttyUSB0 -b 115200 --parity even --stop-bits 2

# Or just input
./lngterm -d /dev/ttyUSB0 -b 115200
```

Options:

| Option | Description |
|--------|-------------|
| `-d, --device` | Serial device path (required) |
| `-b, --baud`   | Baud rate (default: 115200) |
| `--data-bits` | Data bits (5, 6, 7, **8**) |
| `--parity` | Parity checking (**none**, odd, even) |
| `--stop-bits` | Stop bits (**1**, 2) |
| `--flow-control` | Flow control (**none**, software, hardware) |

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

Benchmarked on Linux with a virtual serial link (`socat` pty pair). Startup time is measured from process start until the first \"Connected\" line appears on stdout.

| Metric | lngterm (Rust) | pyterm (Python) |
|--------|----------------|-----------------|
| Binary / deploy size | 1.36 MB (single binary) | 1.6 KB script + 682 KB pyserial + Python 3 runtime |
| Startup time (approx.) | ~0.01 s | ~0.19 s |
| Memory (RSS, idle, manual test) | ~2.7 MB | ~13 MB |

The memory numbers were measured separately using `ps` on real runs (not via `bench.sh`), and are meant as rough guidance only.

### Running Benchmarks

`bench.sh` compares lngterm and pyterm using a virtual serial pair, and prints 5 startup-time runs for each implementation.

#### 1. Install dependencies

| Tool | Purpose | Install |
|------|---------|---------|
| `socat` | Create virtual serial pair (pty) | `sudo apt install socat` (Debian/Ubuntu) |
| `script` | Provide pseudo-TTY for memory test | `sudo apt install bsdutils` (Debian/Ubuntu) |
| Python 3 | Run pyterm | `sudo apt install python3` |
| pyserial | pyterm serial I/O | `pip install pyserial` or `pip3 install pyserial` |

On Fedora/RHEL: `sudo dnf install socat util-linux python3 && pip3 install pyserial`

#### 2. Build lngterm and pyterm

```bash
cargo build --release          # lngterm binary → target/release/lngterm
pip install pyserial           # pyterm dependency
```

#### 3. Run

```bash
./bench.sh
```

Script runs in project root and expects `target/release/lngterm` and `pyterm/pyterm.py`.

---

## ⚠️ Platform Support

- ✅ Linux (epoll, termios, native serial)
- ❌ Windows: Not supported
- ❌ macOS: Not supported

---

## 📄 License

MIT License. See [LICENSE](./LICENSE) for details.
