use std::{io::Write, net::TcpStream};

/// # `ClientList`
/// Structure that takes care of a list of Client sockets and their names.
pub struct ClientList {
    clients: Vec<(String, TcpStream)>,
}

impl ClientList {
    /// # `new`
    /// Creates a new empty `ClientList` and returns it
    pub fn new() -> ClientList {
        ClientList {
            clients: Vec::new(),
        }
    }

    /// # `add`
    /// Adds a socket and its user's name to the list. Returns a `Result<(), &str>` if name exists in client list already.
    pub fn add(&mut self, name: String, socket: TcpStream) -> Result<(), &str> {
        if let Some(_) = self.name_exists(&name) {
            Err("Name already exists!")
        } else {
            self.clients.push((name, socket));
            Ok(())
        }
    }

    /// # `name_exists`
    /// Takes a name as a `&String` and returns `Option<usize>` containing the index in the vector of the list.
    /// Else if the user does not exists then it return `None`
    pub fn name_exists(&self, name: &String) -> Option<usize> {
        // I thought about using a HashMap for direct access but it became more complex especially when sending. Linear search should be fine, I hope.
        for (i, client) in self.clients.iter().enumerate() {
            if client.0 == *name {
                return Some(i);
            }
        }

        None
    }

    /// # `send_to_all`
    /// Send a given message as `String` to all clients in the list
    pub fn send_to_all(&mut self, message: &String) {
        let message = message.as_bytes();
        for client in self.clients.iter_mut() {
            client
                .1
                .write(message)
                .expect(format!("Error sending to client {}", client.0).as_str());
        }
    }

    /// # `send_to`
    /// Takes a target client name as `String` and a message as `String` and sends a message to that client.
    /// This returns `Result<(), &str>` where Error is if the user does not exists
    pub fn send_to(&mut self, target: &String, message: &String) -> Result<(), &str> {
        if let Some(id) = self.name_exists(&target) {
            self.clients
                .get_mut(id)
                .expect("Error getting target client")
                .1
                .write(message.as_bytes())
                .expect("Error sending message to target");
            Ok(())
        } else {
            Err("Not such client")
        }
    }

    /// # `remove`
    /// Takes a name as `String` and removes that client from the list.
    /// This returns `Result<(), &str>` where Error is if the user does not exists&
    pub fn remove(&mut self, name: String) -> Result<(), &str> {
        match self.name_exists(&name) {
            Some(idx) => {self.clients.remove(idx); Ok(())},
            _ => Err("Client does not exists in list")
        }
    }
}
