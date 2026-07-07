// Simple example of run locally execute command with the spawn server
use spawn_server::{SpawnServerCommandResponse, local_exec, local_sh};

fn main() {
    let SpawnServerCommandResponse {
        exit_code: code,
        stdout,
        stderr,
    } = local_sh!("ls -lrt");
    println!("Blocking:\n - code={code}\n - stdout={stdout}\n - stderr={stderr}");
    let SpawnServerCommandResponse {
        exit_code: code,
        stdout,
        stderr,
    } = local_sh!("ls -la");
    println!("Blocking:\n - code={code}\n - stdout={stdout}\n - stderr={stderr}");
    let SpawnServerCommandResponse {
        exit_code: code,
        stdout,
        stderr,
    } = local_sh!("ls -la > ls.log");
    println!("Blocking and redirected:\n - code={code}\n - stdout={stdout}\n - stderr={stderr}");

    let SpawnServerCommandResponse {
        exit_code: code,
        stdout,
        stderr,
    } = local_exec!(
        "ls",
        &["-lrt".to_string()],
        &[("FOO".to_string(), "bar_value".to_string())]
    );
    println!("Blocking:\n - code={code}\n - stdout={stdout}\n - stderr={stderr}");

    let SpawnServerCommandResponse {
        exit_code: code,
        stdout,
        stderr,
    } = local_exec!("ls", &["-lrt".to_string()], &[], &["FOO".to_string()]);

    println!("Blocking:\n - code={code}\n - stdout={stdout}\n - stderr={stderr}");
    let SpawnServerCommandResponse {
        exit_code: code,
        stdout,
        stderr,
    } = local_exec!("ls", &["-la".to_string()]);
    println!("Blocking:\n - code={code}\n - stdout={stdout}\n - stderr={stderr}");
    let SpawnServerCommandResponse {
        exit_code: code,
        stdout,
        stderr,
    } = local_exec!(
        "ls",
        &["-la".to_string(), ">".to_string(), "ls.log".to_string()]
    );
    println!("Blocking and redirected:\n - code={code}\n - stdout={stdout}\n - stderr={stderr}");
}
