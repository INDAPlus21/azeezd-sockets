mod client;
mod server;
mod tools;

pub mod request_type {
    pub const CONNECT: &str = "CON";
    pub const MESSAGE: &str = "MSG";
    pub const COMMAND: &str = "CMD";
}

pub mod resposne_type {

    pub const PUBLIC_MESSAGE: &str = super::request_type::MESSAGE;
    pub const PRIVATE_MESSAGE: &str = "PRM";
    pub const CONNECTION_ACCEPTED: &str = "CAC";
    pub const CONNECTION_DENIED: &str = "CDE";
    pub const USER_JOINED: &str = "UJS";
    pub const USER_LEFT: &str = "ULS";
}

pub mod commands {
    pub const WHISPER: &str = "/w";
    pub const LOGOUT: &str = "/exit";
    pub const LIST: [&str; 2] = [WHISPER, LOGOUT];
}

/// CHANGE THIS IF YOU WANT ANOTHER HOST ADDRESS!!
pub const SERVER_ADDRESS: &str = "localhost:8080";

pub use self::{client::Client, server::Server, tools::{ClientList, server_log, LogMessagType}};
