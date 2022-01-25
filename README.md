# Azeez Daoud - Sockets

This is chat bot that uses both Client and Server side in the same application. (Now it's all on localhost. Just change the address under `src/socket_chat/mod.rs`)

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

# Special Commands
There are a number of commands that you can use in the server
- `/w <target>` to whisper someone in the chat
- `/exit` to disconnect from the server