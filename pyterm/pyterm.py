import argparse
import serial
import sys
import termios
import threading
import tty

def reader_thread(ser, stop_event):
    while not stop_event.is_set():
        try:
            data = ser.read(1024)
            if data:
                sys.stdout.buffer.write(data)
                sys.stdout.flush()
        except serial.SerialTimeoutException:
            continue
        except Exception:
            break

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('-d', '--device', required=True)
    parser.add_argument('-b', '--baud', type=int, default=115200)
    args = parser.parse_args()

    try:
        ser = serial.Serial(args.device, args.baud, timeout=0.01)
    except Exception as e:
        print(f"Failed to open {args.device}: {e}")
        sys.exit(1)

    fd = sys.stdin.fileno()
    old_settings = termios.tcgetattr(fd)
    tty.setraw(fd)

    stop_event = threading.Event()
    thread = threading.Thread(target=reader_thread, args=(ser, stop_event))
    thread.start()

    sys.stdout.write(f"Connected to {args.device} at {args.baud} baud.\r\n")
    sys.stdout.write("Press 'Ctrl + Q' to exit.\r\n")
    sys.stdout.flush()

    try:
        while True:
            ch = sys.stdin.read(1)
            if ch == '\x11': # Ctrl+Q
                break
            elif ch == '\r' or ch == '\n':
                ser.write(b'\r')
            else:
                ser.write(ch.encode('utf-8'))
    finally:
        termios.tcsetattr(fd, termios.TCSADRAIN, old_settings)
        stop_event.set()
        thread.join()
        ser.close()
        print("\nDisconnected.")

if __name__ == '__main__':
    main()