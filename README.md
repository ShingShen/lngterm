# 📟 lngterm

A lightweight serial terminal and task runner tool. Supports serial communication, simple command dispatch, history navigation (with arrow keys), and logging.

---

## ✨ Features

- Serial terminal connection via specified device and baudrate  
- CLI interface with command history (via arrow keys)
- Supports custom command handlers (e.g. `lngterm`, `tasker`)
- Logging with timestamped filenames
- Optional command-line and interactive modes

---

## 📦 Installation

### Prerequisites

- Clone the repository and build manually:

```bash
git clone https://github.com/yourusername/lngterm.git
cd lngterm
cargo build --release
```

---

## 🚀 Usage

### Run in CLI Mode

```bash
./lngterm
```

- Type `?` to see available commands.
- Type `exit` to quit the CLI.

### Run Serial Terminal

```bash
sudo ./lngterm /dev/ttyUSB0 115200
```
---

## 🧠 Command Reference

| Command     | Description                                  |
|-------------|----------------------------------------------|
| `?`         | Show help menu                               |
| `exit`      | Exit CLI mode                                |
| `lngterm`   | Custom serial command handler                |
| `tasker`    | Custom task runner (expand as needed)        |

You can extend this by editing the dispatch logic in `main.rs`.

---

## Tasker YAML Format

The `tasker` feature allows you to automate command sequences via a YAML file. The YAML file should contain the device path, baudrate, and a list of commands to execute sequentially.

### Example `task.yml`

```yaml
commands:
  - "admin"
  - "password"
  - "ifconfig"
  - "ls"
```

The output will be logged to a file with a timestamp in the filename to avoid overwriting existing logs.


## 🕓 Logging

- All logs are saved to files with the current timestamp in the filename.
- Log files are not overwritten on each run.

Example log filename:

```
log_2025-08-08_00-15-17.txt
```

---

## 📄 License

MIT License. See [LICENSE](./LICENSE) for details.
