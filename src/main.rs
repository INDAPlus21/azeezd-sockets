mod socket_chat;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&"-s".to_string()) { // Become server
        socket_chat::Server::new()
            .expect("Error creating a new server")
            .init();
    } else { // Become client
        socket_chat::Client::new(args.get(1).expect("No Name Given").to_string())
            .expect("Error connecting to server")
            .init();
    }
}
