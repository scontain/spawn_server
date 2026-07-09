// Simple example of how to use the synchronous client API to
// talk to the spawn_server

use spawn_server::{SpawnServerCommandResponse, arpc_or_local_exec, arpc_or_local_sh};

#[tokio::main]
async fn main() {
    let SpawnServerCommandResponse {
        exit_code,
        stdout,
        stderr,
    } = arpc_or_local_sh!("ls -lrt").await;
    println!("Async:\n - code={exit_code}\n - stdout={stdout}\n - stderr={stderr}");

    let SpawnServerCommandResponse {
        exit_code,
        stdout,
        stderr,
    } = arpc_or_local_exec!("ls", &["-lrt"]).await;
    println!("Async:\n - code={exit_code}\n - stdout={stdout}\n - stderr={stderr}");

    // arpc_or_local_exec with env
    arpc_or_local_exec!(
        "ls",
        &["-la", ">", "ls.log"],
        [("TEST_VAR".to_string(), "value1".to_string())]
    )
    .await;

    // arpc_or_local_exec with env_remove and env. This will remove TEST_VAR from the environment, and add it again with the new value.
    arpc_or_local_exec!(
        "ls",
        &["-la", ">", "ls.log"],
        [("TEST_VAR".to_string(), "value2".to_string())],
        ["TEST_VAR".to_string()]
    )
    .await;
}
