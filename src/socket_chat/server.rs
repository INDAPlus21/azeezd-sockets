use std::{
    io::{Read, Write},
    net::{TcpListener},
    sync::{
        mpsc::{self, TryRecvError},
        Arc, Mutex,
    },
    thread,
};

use colored::*;

use super::request_type as rt;
use super::ClientList;

/// # `Server`
/// Chat server struct that handles the hosting and requests of clients
pub struct Server {
    clients: Arc<Mutex<ClientList>>,
    server_socket: TcpListener,
}

impl Server {
    /// # `new`
    /// Create a new server. Returns `Option<Server>` where None is if an error was encountered while binding the `TcpListener` to the address.
    pub fn new() -> Option<Server> {
        if let Ok(server) = TcpListener::bind(super::SERVER_ADDRESS) {
            println!("{}", "Server Started!".bold().bright_green());

            Some(Server {
                clients: Arc::new(Mutex::new(ClientList::new())),
                server_socket: server,
            })
        } else {
            println!("Error while starting server!");
            None
        }
    }

    /// # `init`
    /// Initializes the server. Will block until the server is closed.
    pub fn init(&mut self) {
        // Server socket
        let _server_socket = self
            .server_socket
            .try_clone()
            .expect("Error acquiring server socket");

        // Thread-communication channels
        let (sender, receiver) = mpsc::channel::<String>();

        // Get reference of clients
        let _clients = self.clients.clone();

        // == REQUEST HANDLING THREAD ==
        thread::spawn(move || loop {
            match receiver.try_recv() {
                Ok(msg) => {
                    print!("{}: {}", "REQ".bold().yellow(), msg);
                    Self::handle_request(_clients.clone(), msg);
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    println!("Sender is disconnected!")
                }
            }
        });

        // == NEW CONNECTION LISTENING ==
        loop {
            match _server_socket.accept() {
                Ok(connected) => {
                    let mut buffer = [0; 1024];
                    let mut client_socket = connected.0;

                    // Get the connection requst from client (CON <name>) and handle it
                    client_socket
                        .read(&mut buffer)
                        .expect("Error reading first message from client");

                    let request = String::from_utf8_lossy(&buffer)
                        .trim_end_matches('\u{0}')
                        .to_string();

                    // Confirm it is the CON request
                    match &request[..3] {
                        super::request_type::CONNECT => {

                            // Cloned to use in separate thread
                            let mut _socket = client_socket
                                .try_clone()
                                .expect("Error while cloning socket for client thread");

                            // Acquire client list
                            let mut _clients =
                                self.clients.lock().expect("Error acquiring client list");

                            // Error adding name to client list. Usually means name already exists
                            if let Err(e) =_clients.add(request[4..].to_string(), client_socket) {
                                print!("{}: {}", "INF".bold(), e);
                                _socket.write(super::resposne_type::CONNECTION_DENIED.as_bytes()); // ACCESS DENIED!!!!!!!!!!!!!!!!!!!!!!!!!!
                            } else { // Connection OK!
                                // Tell the client
                                _socket
                                    .write(super::resposne_type::CONNECTION_ACCEPTED.as_bytes())
                                    .expect("Error writing message to client");

                                // Tell other clients
                                _clients.send_to_all(format!("{} {}", super::resposne_type::USER_JOINED, request[4..].to_string()).to_string());
                                println!("{} joined the server", request[4..].to_string());

                                // Open thread for client
                                let _sender = sender.clone();
                                thread::spawn(move || loop {
                                    let mut buffer = [0; 1024];
                                    if let Err(e) = _socket.read(&mut buffer) {
                                        println!("{}: {}", "INF".bold(), e);
                                        break;
                                    }
                                    let request = String::from_utf8_lossy(&mut buffer).to_string();
                                    _sender
                                        .send(request)
                                        .expect("Error sending request for handling");
                                });
                            }
                        }
                        _ => {
                            println!("Client sent invalid command");
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_request(clients: Arc<Mutex<ClientList>>, request: String) {
        // Acquire client list
        let mut clients = clients.lock().expect("Error acquiring client list");

        // First 3 characters of a request
        let identifier = &request[..3];
        
        let sender = request.split_ascii_whitespace().nth(1).unwrap();

        // Where the rest of the request starts
        let request_start = request.find(sender).unwrap() + sender.len() + 1;
        let message_content = request[request_start..].to_string();

        match identifier {
            rt::MESSAGE => { // Public message
                clients.send_to_all(request);
            }
            rt::COMMAND => { // Special commands
                let command = message_content.split_ascii_whitespace().nth(0).unwrap();

                match command {
                    super::commands::WHISPER => { // Direct messages
                        // Acquire target
                        let target = message_content.split_ascii_whitespace().nth(1).unwrap().to_string();
                        if let Ok(_) = clients.send_to(
                            target.clone(),
                            format!(
                                "{} {} {}",
                                super::resposne_type::PRIVATE_MESSAGE,
                                target,
                                message_content[message_content.find(&target).unwrap() + target.len() + 1..]
                                    .to_string()
                            ),
                        ) {
                            print!("{}: {}", "OK".bold().green(), request);
                        }
                    },
                    super::commands::LOGOUT => { // Remove client from server
                        // Tell the client just in case
                        clients.send_to(sender.to_string(), format!("{}", super::resposne_type::CONNECTION_DENIED)).unwrap();

                        // Tell other clients
                        clients.send_to_all(format!("{} {}", super::resposne_type::USER_LEFT, sender));

                        if let Err(e) = clients.remove(sender.to_string()) {
                            println!("{}: {}", "ERR".bold().red(), e);
                        }
                    }
                    _ => {}
                }
            }

            _ => {}
        }
    }
}
