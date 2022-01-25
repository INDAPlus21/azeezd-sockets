# Azeez Daoud - Sockets

This is chat server that uses both Client and Server side in the same application. (Now it's all on address `localhost:8080`, you can just change the address under `src/socket_chat/mod.rs`)

# Host a server
To start a server type (while in this directory)
```
cargo run -- -s
```
Yes, the `--` is essential to tell Cargo `-s` is a trailing argument.

# Join the server as a client
To join the newly created server as a client, open another terminal or command line and type
```
cargo run <username>
```
where `<username>` is the name you want to have in the chat.
When you join, you can directly type in the stdin and it would be sent to the server

# Special Commands
There are a number of commands that you can use in the server
- `/w <target>` to whisper someone in the chat
- `/exit` to disconnect from the server

# Notes
- I have tried to protect against all possible states that might lead to one. There *could* be some way as a client to poison some lock somewhere with some action or command (plz dont).
- If a lock poisoning occurs, no one would be able to log into the server until it is restarted. (Available clients will not be able to interact with the server)