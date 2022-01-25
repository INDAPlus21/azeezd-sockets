use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        mpsc::{self, TryRecvError},
        Arc, Mutex,
    },
    thread,
};

use colored::*;

use super::request_type as rt;
use super::{server_log, ClientList, LogMessagType};

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
            server_log(
                "Error starting server".to_string(),
                LogMessagType::EncounteredError,
            );
            None
        }
    }

    /// # `init`
    /// Initializes the server. Will block until the server is closed.
    pub fn init(&mut self) {
        // Server socket
        let _server_socket = match self.server_socket.try_clone() {
            Ok(val) => val,
            Err(_) => {
                server_log(
                    "Error acquiring server socket. Closing Server...".to_string(),
                    LogMessagType::EncounteredError,
                );
                std::process::exit(1);
            }
        };

        // Thread-communication channels
        let (sender, receiver) = mpsc::channel::<String>();

        // Get reference of clients
        let _clients = self.clients.clone();

        // == REQUEST HANDLING THREAD ==
        thread::spawn(move || loop {
            match receiver.try_recv() {
                Ok(msg) => {
                    server_log(format!("{}", msg), LogMessagType::IncomingRequest);
                    Self::handle_request(_clients.clone(), &msg);
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    server_log(
                        "Sender channel is closed.".to_string(),
                        LogMessagType::EncounteredError,
                    );
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
                    if let Err(_) = client_socket.read(&mut buffer) {
                        server_log(
                            format!(
                                "Error reading connection request message from client {}",
                                connected.1.to_string()
                            ),
                            LogMessagType::EncounteredError,
                        );
                    }

                    let request = String::from_utf8_lossy(&buffer)
                        .trim_end_matches('\u{0}')
                        .to_string();

                    // Confirm it is the CON request
                    match &request[..3] {
                        super::request_type::CONNECT => {
                            // Cloned to use in separate thread
                            let mut _socket = match client_socket.try_clone() {
                                Ok(val) => val,
                                Err(_) => {
                                    format!("{}", connected.1.to_string()).to_string();
                                    server_log(
                                        format!(
                                            "Error cloning socket for client {}",
                                            connected.1.to_string()
                                        ),
                                        LogMessagType::EncounteredError,
                                    );
                                    continue;
                                }
                            };

                            // Acquire client list
                            let mut _clients = match self.clients.lock() {
                                Ok(val) => val,
                                Err(_) => {
                                    server_log(
                                        "Error acquiring client list from server".to_string(),
                                        LogMessagType::EncounteredError,
                                    );
                                    continue;
                                }
                            };

                            // Error adding name to client list. Usually means name already exists
                            if let Err(_) = _clients.add(request[4..].to_string(), client_socket) {
                                server_log(
                                    format!(
                                        "Denied access for {} while adding them to client list",
                                        connected.1.to_string()
                                    ),
                                    LogMessagType::Information,
                                );
                                _socket.write(super::resposne_type::CONNECTION_DENIED.as_bytes()); // ACCESS DENIED!!!!!!!!!!!!!!!!!!!!!!!!!!
                            } else {
                                // Connection OK!
                                // Tell the client
                                if let Err(_) = _socket
                                    .write(super::resposne_type::CONNECTION_ACCEPTED.as_bytes())
                                {
                                    server_log(
                                        format!(
                                            "Error writing connection acceptance message to {}",
                                            connected.1.to_string()
                                        ),
                                        LogMessagType::Information,
                                    );
                                }
                                // Tell other clients
                                _clients.send_to_all(&format!(
                                    "{} {}",
                                    super::resposne_type::USER_JOINED,
                                    request[4..].to_string()
                                ));
                                server_log(
                                    format!(
                                        "Client {} [{}] joined the server",
                                        request[4..].to_string(),
                                        connected.1.to_string()
                                    ),
                                    LogMessagType::Information,
                                );

                                // Open thread for client
                                let _sender = sender.clone();
                                thread::spawn(move || loop {
                                    let mut buffer = [0; 1024];
                                    if let Err(e) = _socket.read(&mut buffer) {
                                        server_log(format!("Error \"{}\" reading from client {}. Closing thread", e, connected.1.to_string()), LogMessagType::EncounteredError);
                                        break;
                                    }
                                    let request = String::from_utf8_lossy(&mut buffer).to_string();
                                    if let Err(_) = _sender.send(request) {
                                        server_log(
                                            format!(
                                                "Error sending request from {} for handling",
                                                connected.1.to_string()
                                            ),
                                            LogMessagType::EncounteredError,
                                        );
                                    }
                                });
                            }
                        }
                        _ => {
                            server_log(
                                format!("Client {} sent invalid request", connected.1.to_string())
                                    .to_string(),
                                LogMessagType::Information,
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_request(clients: Arc<Mutex<ClientList>>, request: &String) {
        // Acquire client list
        let mut clients = match clients.try_lock() {
            Ok(val) => val,
            Err(_) => {
                server_log(
                    format!("Error acquiring client list for request \"{}\"", request),
                    LogMessagType::EncounteredError,
                );
                return;
            }
        };

        // First 3 characters of a request
        let identifier = &request[..3];

        let sender = request.split_ascii_whitespace().nth(1).unwrap();

        // Where the rest of the request starts
        let request_start = request.find(sender).unwrap() + sender.len() + 1;
        let message_content = request[request_start..].to_string();

        match identifier {
            rt::MESSAGE => {
                // Public message
                clients.send_to_all(request);
            }
            rt::COMMAND => {
                // Special commands
                let command = message_content.split_ascii_whitespace().nth(0).unwrap();

                match command {
                    super::commands::WHISPER => {
                        // Direct messages
                        // Acquire target
                        let target = message_content
                            .split_ascii_whitespace()
                            .nth(1)
                            .unwrap()
                            .to_string();
                        if let Ok(_) = clients.send_to(
                            &target.clone(),
                            &format!(
                                "{} {} {}",
                                super::resposne_type::PRIVATE_MESSAGE,
                                sender,
                                message_content
                                    [message_content.find(&target).unwrap() + target.len() + 1..]
                                    .to_string()
                            ),
                        ) {}
                    }
                    super::commands::LOGOUT => {
                        // Remove client from server
                        // Tell the client just in case
                        clients
                            .send_to(
                                &sender.to_string(),
                                &format!("{}", super::resposne_type::CONNECTION_DENIED),
                            )
                            .unwrap();

                        // Tell other clients
                        clients.send_to_all(&format!(
                            "{} {}",
                            super::resposne_type::USER_LEFT,
                            sender
                        ));

                        if let Err(e) = clients.remove(sender.to_string()) {
                            server_log(format!("{}", e), LogMessagType::EncounteredError);
                            return;
                        }
                    }
                    _ => {
                        server_log(
                            format!("Client sent invalid command \"{}\"", command),
                            LogMessagType::Information,
                        );
                        return;
                    }
                }
            }

            _ => {
                server_log(
                    format!("Client sent invalid identifier \"{}\"", identifier),
                    LogMessagType::Information,
                );
                return;
            }
        }

        server_log(
            format!("Request \"{}\" handled", request),
            LogMessagType::RequestHandled,
        );
    }
}
