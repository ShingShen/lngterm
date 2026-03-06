use std::io::{self, Read, Write};
use std::os::unix::io::AsRawFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Token, Waker};
use serialport::TTYPort;

// Unique tokens to identify events in the epoll registry
const SERIAL_TOKEN: Token = Token(0);
const WAKER_TOKEN: Token = Token(1);

/// SerialReactor manages a background thread that polls the serial port for incoming data
pub struct SerialReactor {
    /// Atomic flag to signal the worker thread to stop
    keep_running: Arc<AtomicBool>,
    /// Waker used to break the poll loop immediately when stopping
    waker: Arc<Waker>,
    /// Handle to the worker thread for joining upon shutdown
    worker_thread: Option<JoinHandle<()>>,
}

impl SerialReactor {
    /// Starts the reactor thread and begins polling the provided TTYPort
    pub fn start(mut port_reader: TTYPort) -> io::Result<Self> {
        let keep_running = Arc::new(AtomicBool::new(true));
        let keep_running_clone = keep_running.clone();

        // Initialize mio Poll instance
        let mut poll = Poll::new()?;

        // Create a waker to allow external threads to wake up the poll loop
        let waker = Arc::new(Waker::new(poll.registry(), WAKER_TOKEN)?);

        // Register the serial port file descriptor with the poll registry
        let raw_fd = port_reader.as_raw_fd();
        poll.registry().register(
            &mut SourceFd(&raw_fd),
            SERIAL_TOKEN,
            Interest::READABLE,
        )?;

        // Spawn the worker thread
        let worker_thread = thread::spawn(move || {
            let mut serial_buf = [0u8; 1024];
            let mut stdout = io::stdout();
            let mut events = Events::with_capacity(128);

            // Continuous event loop
            while keep_running_clone.load(Ordering::Relaxed) {
                // Wait for events (no timeout, blocks until event occurs)
                match poll.poll(&mut events, None) {
                    Ok(_) => {}
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => {
                        eprintln!("\r\n[Reactor] epoll failed: {:?}", e);
                        break;
                    }
                }

                // Process triggered events
                for event in events.iter() {
                    match event.token() {
                        SERIAL_TOKEN => {
                            if event.is_readable() {
                                // Drain the serial port buffer
                                match port_reader.read(&mut serial_buf) {
                                    Ok(t) if t > 0 => {
                                        // Directly write received data to stdout
                                        let _ = stdout.write_all(&serial_buf[..t]);
                                        let _ = stdout.flush();
                                    }
                                    Ok(_) => break, // EOF reached
                                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                                    Err(e) => {
                                        eprintln!("\r\n[Reactor] read failed: {:?}", e);
                                        break;
                                    }
                                }
                            }
                        }
                        // WAKER_TOKEN events are used to break the poll wait
                        WAKER_TOKEN => continue,
                        _ => unreachable!(),
                    }
                }
            }
        });

        Ok(Self {
            keep_running,
            waker,
            worker_thread: Some(worker_thread),
        })
    }

    /// Stops the reactor thread and waits for it to exit
    pub fn stop(&mut self) {
        // Set the stop flag
        self.keep_running.store(false, Ordering::Relaxed);
        // Wake up the poll loop if it's currently blocking
        let _ = self.waker.wake();
        // Join the worker thread to ensure clean shutdown
        if let Some(thread) = self.worker_thread.take() {
            let _ = thread.join();
        }
    }
}