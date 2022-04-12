use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use log::info;
use signal_hook::consts::{SIGINT, SIGKILL};
use signal_hook::iterator::Signals;
use std::fs::remove_file;
use std::io::{self, prelude::*, BufReader, BufWriter};
use std::thread;

pub struct SocketListener {}

impl SocketListener {
    pub fn exit_signal_handler() -> Result<(), ()> {
        info!("Setting exit handler");
        let mut signals = Signals::new(&[SIGINT]).expect("failed to create signal listener");
        thread::spawn(move || {
            for sig in signals.forever() {
                match sig {
                    SIGINT => {
                        println!("received SIGINT, exiting...");
                        remove_file("/tmp/polydrive.sock").unwrap();
                        std::process::exit(0);
                    }
                    SIGKILL => {
                        println!("received SIGKILL, forcing exiting");
                        std::process::exit(0);
                    }
                    _ => {
                        println!("Received signal {:?}", sig);
                    }
                }
            }
        });
        Ok(())
    }

    pub fn handle_error(connection: io::Result<LocalSocketStream>) -> Option<LocalSocketStream> {
        connection
            .map_err(|error| eprintln!("Incoming connection failed: {}", error))
            .ok()
    }

    pub fn start() -> Result<(), ()> {
        SocketListener::exit_signal_handler().unwrap();

        std::thread::spawn(move || {
            info!("starting listen on socket");
            let listener = LocalSocketListener::bind("/tmp/polydrive.sock").unwrap();
            for stream in listener.incoming().filter_map(SocketListener::handle_error) {
                println!("Incoming connection!, {:?}", stream);
                let mut reader = BufReader::new(stream);
                let mut buffer = String::new();
                reader.read_line(&mut buffer).expect("failed to read");
                info!("received: {}", buffer);
                let command = buffer.trim();
                info!("Command received: {}", command);
                let mut response = String::new();
                if command == "list" {
                    info!("Listing...");
                    response = "Listing not yet implemented".to_string();
                }

                let mut writer = BufWriter::new(reader.get_mut());
                writer.write_all(response.as_bytes()).unwrap();
                info!("response sent");
            }
        });

        Ok(())
    }
}
