// Simple example of how to use the synchronous client API to
// talk to the spawn_server
use spawn_server::{SpawnServerCommandResponse, srpc_or_local_exec, srpc_or_local_sh};

fn main() {
    let SpawnServerCommandResponse {
        exit_code: code,
        stdout,
        stderr,
    } = srpc_or_local_sh!("ls -lrt");
    println!("Blocking:\n - code={code}\n - stdout={stdout}\n - stderr={stderr}");
    let SpawnServerCommandResponse {
        exit_code: code,
        stdout,
        stderr,
    } = srpc_or_local_sh!("ls -la");
    println!("Blocking:\n - code={code}\n - stdout={stdout}\n - stderr={stderr}");
    let SpawnServerCommandResponse {
        exit_code: code,
        stdout,
        stderr,
    } = srpc_or_local_sh!("ls -la > ls.log");
    println!("Blocking and redirected:\n - code={code}\n - stdout={stdout}\n - stderr={stderr}");

    let SpawnServerCommandResponse {
        exit_code: code,
        stdout,
        stderr,
    } = srpc_or_local_exec!("ls", &["-lrt"], [("FOO", "bar_value")]);
    println!("Blocking:\n - code={code}\n - stdout={stdout}\n - stderr={stderr}");

    let SpawnServerCommandResponse {
        exit_code: code,
        stdout,
        stderr,
    } = srpc_or_local_exec!("ls", &["-lrt"], Vec::<(&str, &str)>::new(), ["FOO"]);

    println!("Blocking:\n - code={code}\n - stdout={stdout}\n - stderr={stderr}");
    let SpawnServerCommandResponse {
        exit_code: code,
        stdout,
        stderr,
    } = srpc_or_local_exec!("ls", &["-la"]);
    println!("Blocking:\n - code={code}\n - stdout={stdout}\n - stderr={stderr}");
    let SpawnServerCommandResponse {
        exit_code: code,
        stdout,
        stderr,
    } = srpc_or_local_exec!("ls", &["-la", ">", "ls.log"]);
    println!("Blocking and redirected:\n - code={code}\n - stdout={stdout}\n - stderr={stderr}");
}
