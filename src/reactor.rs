use std::io::{self, Read, Write};
use std::os::unix::io::AsRawFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Token, Waker};
use serialport::TTYPort;

const SERIAL_TOKEN: Token = Token(0);
const WAKER_TOKEN: Token = Token(1);

pub struct SerialReactor {
    keep_running: Arc<AtomicBool>,
    waker: Arc<Waker>,
    worker_thread: Option<JoinHandle<()>>,
}

impl SerialReactor {
    pub fn start(mut port_reader: TTYPort) -> io::Result<Self> {
        let keep_running = Arc::new(AtomicBool::new(true));
        let keep_running_clone = keep_running.clone();
        let mut poll = Poll::new()?;
        
        let waker = Arc::new(Waker::new(poll.registry(), WAKER_TOKEN)?);
        let raw_fd = port_reader.as_raw_fd();

        poll.registry().register(
            &mut SourceFd(&raw_fd),
            SERIAL_TOKEN,
            Interest::READABLE,
        )?;

        let worker_thread = thread::spawn(move || {
            let mut serial_buf = [0u8; 1024];
            // let mut serial_buf = [0u8; 16];
            let mut stdout = io::stdout();
            let mut events = Events::with_capacity(128);

            while keep_running_clone.load(Ordering::Relaxed) {
                match poll.poll(&mut events, None) {
                    Ok(_) => {}
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => {
                        eprintln!("\r\n[Reactor] epoll failed: {:?}", e);
                        break;
                    }
                }

                for event in events.iter() {
                    match event.token() {
                        SERIAL_TOKEN => {
                            if event.is_readable() {
                                // EPOLLET drain loop
                                match port_reader.read(&mut serial_buf) {
                                    Ok(t) if t > 0 => {
                                        let _ = stdout.write_all(&serial_buf[..t]);
                                        let _ = stdout.flush();
                                    }
                                    Ok(_) => break, // EOF
                                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                                    Err(e) => {
                                        eprintln!("\r\n[Reactor] read failed: {:?}", e);
                                        break;
                                    }
                                }
                            }
                        }
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

    pub fn stop(&mut self) {
        self.keep_running.store(false, Ordering::Relaxed);
        let _ = self.waker.wake();
        if let Some(thread) = self.worker_thread.take() {
            let _ = thread.join();
        }
    }
}