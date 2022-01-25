use colored::{Colorize, ColoredString};
extern crate chrono;
use chrono::Local;

/// # `LogMessageType`
/// The type of message to log. This will prepend the logging function with an appropiate suffix depending on the value of this enum.
/// - Information: `INF`
/// - IncomingRequest: `REQ`
/// - EncounteredError: `ERR`
/// - RequestHandled: `OK`
pub enum LogMessagType {
    Information,
    IncomingRequest,
    EncounteredError,
    RequestHandled
}

impl LogMessagType {
    /// # `as_abbreviation`
    /// Converts the the `LogMessageType` into its appropiate suffix for the logging functions
    fn as_abbreviation(&self) -> ColoredString {
        match self {
            Self::EncounteredError => "ERR".bold().red(),
            Self::IncomingRequest => "REQ".bold().yellow(),
            Self::Information => "INFO".bold(),
            Self::RequestHandled => "OK".bold().green()
        }
    }
}

/// # `server_log`
/// Takes a message as `String` and type of log message as `LogMessageType` and prints the message with the appropiate suffix and prepended with current time
pub fn server_log(message: String, log_message_type: LogMessagType) {
    println!("{} | {}: {}", Local::now().format("%y%m%d %H:%M:%S"), log_message_type.as_abbreviation(), message);
}

/// # `client_log`
/// Takes a message as `String` and type of log message as `LogMessageType` and prints the message with the appropiate suffix
pub fn client_log(message: String, log_message_type: LogMessagType) {
    println!("{}: {}", log_message_type.as_abbreviation(), message);
}