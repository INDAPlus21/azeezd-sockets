use std::{
    io::{Read, Write},
    net::TcpStream,
    thread,
};
extern crate colored;
use colored::*;

/// # `Client`
/// Structure that handles a client and its communication with the server
pub struct Client {
    socket: TcpStream,
    name: String,
}

impl Client {
    /// # `new`
    /// Creates a new client by taking the name of the client as `String`
    /// This returns `Option<Client>` where None is returned if the connection was not successful for some reason
    pub fn new(name: String) -> Option<Client> {
        if let Ok(mut socket) = TcpStream::connect(super::SERVER_ADDRESS) {
            let mut response_buffer = [0; 32];
            // Send join request to server
            socket
                .write(format!("CON {}", name).as_bytes())
                .expect("Unable to initiate communication with server");

            // Read resposne
            socket
                .read(&mut response_buffer)
                .expect("Error while reading from server");

            let response = String::from_utf8_lossy(&response_buffer)
                .trim_end_matches('\u{0}') // Remove trailing 0's
                .to_string();

            match response.as_str() {
                super::resposne_type::CONNECTION_ACCEPTED => { // Yay
                    println!("Connection Accepted! Welcome!");
                    Some(Client {
                        socket: socket,
                        name: name,
                    })
                }
                super::resposne_type::CONNECTION_DENIED => { // Sadge
                    println!("Connection Denied!");
                    None
                }
                _ => {
                    println!("An error occured!");
                    None
                }
            }
        } else {
            None
        }
    }

    /// # `init`
    /// Initializes the client. This will block until the client is shut down
    pub fn init(&mut self) {
        let mut socket = self.socket.try_clone().expect("Error acquiring socket.");

        // Spawn response listening and handling thread
        thread::spawn(move || loop {
            let mut buffer = [0; 1024];
            socket.read(&mut buffer).expect("Error reading from socket");
            Self::parse_resposne(&buffer);
        });

        // Reads from stdin and send to server
        loop {
            let mut buffer = String::new();
            std::io::stdin()
                .read_line(&mut buffer)
                .expect("Error reading message");

            match self.parse_message(&buffer) {
                Ok(buffer) => {
                    self.socket
                        .write(buffer.as_bytes())
                        .expect("Error sending message.");
                }
                Err(e) => println!("{}: {}", "ERR".bold().red(), e),
            }
        }
    }

    /// # `parse_response`
    /// takes a response buffer `&[u8]` and handles it
    fn parse_resposne(response: &[u8]) {
        let response = String::from_utf8_lossy(response).to_string();

        // Get resposne id (first three characters)
        match &response[..3] {
            super::resposne_type::CONNECTION_DENIED => { // Sadge
                std::process::exit(0);
            }
            super::resposne_type::PUBLIC_MESSAGE => { // Public message from some other client
                // Get sender's name
                let sender = response.split_ascii_whitespace().nth(1).unwrap();
                // Get exactly where the actual message content starts
                let message_start = sender.len() + response.find(sender).unwrap() + 1;
                let message = response[message_start..].to_string();
                print!("{}> {}\r", sender.bold().bright_blue(), message);
            }
            super::resposne_type::PRIVATE_MESSAGE => { // Private message from some other client
                // Get sender's name
                let sender = response.split_ascii_whitespace().nth(1).unwrap();
                // Get exactly where the actual message is
                let message_start = sender.len() + response.find(sender).unwrap() + 1;
                let message = response[message_start..].to_string();
                print!(
                    "{} {}: {}\r",
                    sender.italic().bright_blue(),
                    "whispered".italic(),
                    message.italic()
                );
            },
            super::resposne_type::USER_JOINED => {
                let joined = response[4..].to_string();
                println!("{} {}", joined.bold().blue(), "joined the server!".italic());
            },
            super::resposne_type::USER_LEFT => {
                let left = response[4..].to_string();
                println!("{} {}", left.bold().blue(), "left the server!".italic());
            }
            _ => {}
        }
    }

    /// # `parse_message`
    /// Takes a message as `&String` and returns it in request format (with identifier and other neccessary content for the server to interpert).
    /// This will however return it as `Result<String, &str>` where Error is returned if a command was given which is not known to this application.
    fn parse_message(&self, message: &String) -> Result<String, &str> {
        // Gets what ID to return (first 3 characters of a request)
        let identifier = if message.starts_with('/') { // Commands starts with a / such as /whisper
            let command = message.split_ascii_whitespace().nth(0).unwrap();
            if super::commands::LIST.contains(&command) { 
                "CMD"
            } else { // If the given command does not exist tell client
                return Err("No such command");
            }
        } else {
            "MSG"
        };

        // This is how the request will look like
        Ok(format!("{} {} {}", identifier, self.name, message))
    }
}