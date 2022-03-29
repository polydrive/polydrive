use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use log::info;
use std::fs::remove_file;
use std::io::{self, prelude::*, BufReader, BufWriter};

pub struct SocketListener {}

impl SocketListener {
    pub fn ctrlc_handler() -> Result<(), ()> {
        info!("Setting ctrl-c handler");
        ctrlc::set_handler(move || {
            println!("received Ctrl+C!");
            remove_file("/tmp/polydrive.sock").unwrap();
            std::process::exit(0);
        })
        .unwrap();
        Ok(())
    }

    pub fn handle_error(connection: io::Result<LocalSocketStream>) -> Option<LocalSocketStream> {
        connection
            .map_err(|error| eprintln!("Incoming connection failed: {}", error))
            .ok()
    }

    pub fn start() -> Result<(), ()> {
        SocketListener::ctrlc_handler().unwrap();

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
