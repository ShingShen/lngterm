#include "serial.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <termios.h>
#include <errno.h>

static speed_t baud_to_speed(int baud) {
    switch (baud) {
        case 9600: return B9600;
        case 19200: return B19200;
        case 38400: return B38400;
        case 57600: return B57600;
        case 115200: return B115200;
        default:
            fprintf(stderr, "Unsupported baud rate: %d\n", baud);
            exit(1);
    }
}

void set_serial_raw(int fd, speed_t speed) {
    struct termios tty;
    tcgetattr(fd, &tty);
    cfmakeraw(&tty);
    cfsetspeed(&tty, speed);
    tty.c_cflag |= (CLOCAL | CREAD);
    tty.c_cflag &= ~CRTSCTS;
    tcsetattr(fd, TCSANOW, &tty);
}

int open_serial(const char *device, int baudrate) {
    int fd = open(device, O_RDWR | O_NOCTTY);
    if (fd < 0) return -1;
    set_serial_raw(fd, baud_to_speed(baudrate));
    return fd;
}

void close_serial(int fd) {
    close(fd);
}

int write_serial(int fd, const char *cmd) {
    size_t len = strlen(cmd);
    return write(fd, cmd, len);
}

int read_serial(int fd, char *buffer, int maxlen) {
    fd_set rfds;
    struct timeval timeout;
    int total = 0;

    while (1) {
        FD_ZERO(&rfds);
        FD_SET(fd, &rfds);
        timeout.tv_sec = 0;
        timeout.tv_usec = 200000;  // 200ms idle

        int ret = select(fd + 1, &rfds, NULL, NULL, &timeout);
        if (ret > 0) {
            int len = read(fd, buffer + total, maxlen - total);
            if (len > 0) {
                total += len;
            } else {
                break;
            }
        } else {
            break; // timeout
        }
    }
    buffer[total] = '\0';
    return total;
}

void set_stdin_raw() {
    struct termios tty;
    tcgetattr(STDIN_FILENO, &tty);
    tty.c_lflag &= ~(ICANON | ECHO);
    tcsetattr(STDIN_FILENO, TCSANOW, &tty);
}

int run_command_on_serial(int fd, const char *cmd, char *output, int outmax) {
    char fullcmd[512];
    snprintf(fullcmd, sizeof(fullcmd), "%s\r", cmd); // append \r
    write_serial(fd, fullcmd);
    usleep(2500000);  
    return read_serial(fd, output, outmax);
}

void start_serial_terminal(const char *device, int baudrate) {
    int serial_fd = open_serial(device, baudrate);
    set_stdin_raw();

    printf("Connected to %s at %d baud. Press Ctrl+C to exit.\n", device, baudrate);
    fflush(stdout);

    while (1) {
        fd_set readfds;
        FD_ZERO(&readfds);
        FD_SET(serial_fd, &readfds);
        FD_SET(STDIN_FILENO, &readfds);

        int max_fd;
        if (serial_fd > STDIN_FILENO) {
            max_fd = serial_fd;
        } else {
            max_fd = STDIN_FILENO;
        }

        int ret = select(max_fd + 1, &readfds, NULL, NULL, NULL);
        if (ret < 0) {
            perror("select");
            break;
        }

        if (FD_ISSET(serial_fd, &readfds)) {
            char buf[256];
            int len = read(serial_fd, buf, sizeof(buf));
            if (len > 0) {
                (void)write(STDOUT_FILENO, buf, len); // ignore result
            }
        }

        if (FD_ISSET(STDIN_FILENO, &readfds)) {
            char ch;
            int len = read(STDIN_FILENO, &ch, 1);
            if (len > 0) {
                if (ch == '\n') ch = '\r';
                (void)write(serial_fd, &ch, 1); // ignore result
            }
        }
    }

    close(serial_fd);
}
