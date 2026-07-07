# Spawn Server

This Rust library permits to execute programs without replying on `fork`. This crate exports macros `srpc!` (synchronous) and `arpc!` (asynchronous) that is similar to macro `sh!`. Have a look in the examples directory on how to use this macro.

## Usage

The spawn server supports commands in two modes:

1. Shell Mode (`srpc_sh!`, `arpc_sh!`): The command is ran through a system shell.

    Use macro `srpc_sh!` to send requests to the spawn server. This is a synchronous call. 

    For asynchronous calls, use macro `arpc_sh!`. Import these macros in your Rust program as follows:

    ```rust
    use spawn_server::{arpc_sh, srpc_sh};
    fn main() {
        srpc_sh!("ls -lrt");
    }
    ```

2. Exec Mode (`srpc_exec`, `arpc_exec`): The command to run is directly executed.

    Use macro `srpc_exec!` to send requests to the spawn server. This is a synchronous call. 

    For asynchronous calls, use macro `arpc_exec!`. Import these macros in your Rust program as follows:

    ```rust
    use spawn_server::{arpc_exec, srpc_exec};
    fn main() {
        srpc_exec!("ls", &["-la", ">", "ls.log"]);
    }
    ```
Also, add the following to your `Cargo.toml` file:

```toml
[dependencies]
spawn_server = { version="*", git = "https://github.com/scontain/spawn_server.git" }
```

## Deployment

You should run the spawn server in the same container as the program that uses the spawn server. The spawn server will - for now - use port 8099.

## Build

Just execute `cargo build --release` to build the spawn server.

## Examples

To run on specific host and port:

```bash
SPAWN_SERVER_PORT=9000 cargo run
# -> listens on 127.0.0.1:9000
```

Blocking execution of programs:

```bash
cargo run --example blocking
```

Asychronous execution of programs:

```bash
cargo run --example async
```
