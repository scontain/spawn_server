use serde_derive::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use tracing::{error, info, warn};

const DEFAULT_SPAWN_SERVER_PORT: u16 = 8099;
const DEFAULT_SPAWN_SERVER_HOST: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

/// A command to run, either through the system shell or as a direct exec.
///
/// `Exec` avoids shell interpretation entirely which reduces injection risk from
/// arguments containing spaces, quotes, `$()`, and additional (potentially dangerous) commands etc.
/// `Exec` should therefore, for security reasons, be preferred over `Shell` whenever possible, especially if  
/// you're invoking a known binary with known args.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SpawnServerCommandRequest {
    /// Run via the system shell: `sh -c command`.
    Shell { command: String },
    /// Exec a binary directly.
    Exec {
        binary: String,
        args: Vec<String>,
        /// Extra env vars to set on top of the inherited environment.
        env: Vec<(String, String)>,
        /// Env vars to strip from the inherited environment before `env` is applied.
        env_remove: Vec<String>,
    },
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SpawnServerCommandResponse {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl std::error::Error for SpawnServerCommandResponse {}

impl std::fmt::Display for SpawnServerCommandResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "command exited with code {}", self.exit_code)?;
        if !self.stdout.is_empty() {
            write!(f, "\n  Standard Output:\n")?;
            for line in self.stdout.lines() {
                writeln!(f, "    {}", line)?;
            }
        }

        if !self.stderr.is_empty() {
            write!(f, "\n  Standard Error:\n")?;
            for line in self.stderr.lines() {
                writeln!(f, "    {}", line)?;
            }
        }
        Ok(())
    }
}

pub fn get_spawn_server_addr() -> SocketAddr {
    let ip: IpAddr = IpAddr::V4(DEFAULT_SPAWN_SERVER_HOST);

    let port = std::env::var("SPAWN_SERVER_PORT")
        .unwrap_or_else(|_| DEFAULT_SPAWN_SERVER_PORT.to_string())
        .parse::<u16>()
        .unwrap_or_else(|_| {
            eprintln!("Invalid SPAWN_SERVER_PORT value. Must be a number between 1 and 65535.");
            std::process::exit(1);
        });

    SocketAddr::new(ip, port)
}

fn get_spawner_command_url() -> String {
    let server_addr = get_spawn_server_addr();
    format!("http://{server_addr}/command")
}

fn sync_remote_execute_command(cmd: SpawnServerCommandRequest) -> SpawnServerCommandResponse {
    use std::time::Duration;
    let very_long_timeout = Duration::new(60 * 60 * 24, 0); // 24h

    let client = match reqwest::blocking::Client::builder()
        .timeout(very_long_timeout)
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "Failed to build reqwest client");
            return SpawnServerCommandResponse {
                exit_code: -3,
                stdout: "".to_string(),
                stderr: format!("Client Build Error: {e} (ERROR 24111-3657-3813)"),
            };
        }
    };
    let url = get_spawner_command_url();
    match client.post(url).json(&cmd).send() {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<SpawnServerCommandResponse>() {
                    Ok(result) => {
                        info!(
                            code = result.exit_code,
                            "sync command executed successfully"
                        );
                        result
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to parse success response JSON");
                        SpawnServerCommandResponse {
                            exit_code: -4,
                            stdout: "".to_string(),
                            stderr: format!("JSON parse error: {e} (ERROR 13797-13665-5237)"),
                        }
                    }
                }
            } else {
                warn!(status = %resp.status(), "sync command returned non-success status");
                match resp.json::<SpawnServerCommandResponse>() {
                    Ok(result) => SpawnServerCommandResponse {
                        exit_code: -2,
                        stdout: result.stdout,
                        stderr: format!(
                            "No Success Error: {} (ERROR 23250-17340-19357)",
                            result.stderr
                        ),
                    },
                    Err(e) => {
                        error!(error = %e, "Failed to parse error response JSON");
                        SpawnServerCommandResponse {
                            exit_code: -5,
                            stdout: "".to_string(),
                            stderr: format!("JSON parse error: {e} (ERROR 6702-11519-24912)"),
                        }
                    }
                }
            }
        }
        Err(e) => {
            error!(error = %e, "sync command request failed");
            SpawnServerCommandResponse {
                exit_code: -1,
                stdout: "".to_string(),
                stderr: format!("RPC Error: {e} (ERROR 7931-24668-7035)"),
            }
        }
    }
}

async fn async_remote_execute_command(
    cmd: SpawnServerCommandRequest,
) -> SpawnServerCommandResponse {
    use std::time::Duration;
    let very_long_timeout = Duration::new(60 * 60 * 24, 0);

    let client = match reqwest::Client::builder()
        .timeout(very_long_timeout)
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "Failed to build async reqwest client");
            return SpawnServerCommandResponse {
                exit_code: -3,
                stdout: "".to_string(),
                stderr: format!("Client Build Error: {e} (ERROR 9346-11546-25221)"),
            };
        }
    };
    let url = get_spawner_command_url();

    match client.post(url).json(&cmd).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<SpawnServerCommandResponse>().await {
                    Ok(result) => {
                        info!(
                            code = result.exit_code,
                            "async command executed successfully"
                        );
                        result
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to parse async success response JSON");
                        SpawnServerCommandResponse {
                            exit_code: -4,
                            stdout: "".to_string(),
                            stderr: format!("JSON parse error: {e} (ERROR 16134-5921-23390"),
                        }
                    }
                }
            } else {
                warn!(status = %resp.status(), "async command returned non-success status");
                SpawnServerCommandResponse {
                    exit_code: -2,
                    stdout: "".to_string(),
                    stderr: "No Success Error (ERROR 18218-21746-5563)".to_string(),
                }
            }
        }
        Err(e) => {
            error!(error = %e, "async command request failed");
            SpawnServerCommandResponse {
                exit_code: -1,
                stdout: "".to_string(),
                stderr: format!("RPC Error: {e} (ERROR 32694-17841-22165)"),
            }
        }
    }
}

pub fn sync_remote_execute_shell<T: AsRef<str>>(cmd: T) -> SpawnServerCommandResponse {
    sync_remote_execute_command(SpawnServerCommandRequest::Shell {
        command: cmd.as_ref().to_string(),
    })
}

pub async fn async_remote_execute_shell<T: AsRef<str>>(cmd: T) -> SpawnServerCommandResponse {
    async_remote_execute_command(SpawnServerCommandRequest::Shell {
        command: cmd.as_ref().to_owned(),
    })
    .await
}

pub fn sync_remote_execute_exec<B, A, E, S, K, V, G, H>(
    binary: B,
    args: A,
    env: E,
    env_remove: G,
) -> SpawnServerCommandResponse
where
    B: AsRef<str>,
    A: IntoIterator<Item = S>,
    S: AsRef<str>,
    E: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
    G: IntoIterator<Item = H>,
    H: AsRef<str>,
{
    sync_remote_execute_command(SpawnServerCommandRequest::Exec {
        binary: binary.as_ref().to_string(),
        args: args.into_iter().map(|a| a.as_ref().to_string()).collect(),
        env: env
            .into_iter()
            .map(|(k, v)| (k.as_ref().to_string(), v.as_ref().to_string()))
            .collect(),
        env_remove: env_remove
            .into_iter()
            .map(|g| g.as_ref().to_string())
            .collect(),
    })
}

pub async fn async_remote_execute_exec<B, A, E, S, K, V, G, H>(
    binary: B,
    args: A,
    env: E,
    env_remove: G,
) -> SpawnServerCommandResponse
where
    B: AsRef<str>,
    A: IntoIterator<Item = S>,
    S: AsRef<str>,
    E: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
    G: IntoIterator<Item = H>,
    H: AsRef<str>,
{
    async_remote_execute_command(SpawnServerCommandRequest::Exec {
        binary: binary.as_ref().to_string(),
        args: args.into_iter().map(|a| a.as_ref().to_string()).collect(),
        env: env
            .into_iter()
            .map(|(k, v)| (k.as_ref().to_string(), v.as_ref().to_string()))
            .collect(),
        env_remove: env_remove
            .into_iter()
            .map(|g| g.as_ref().to_string())
            .collect(),
    })
    .await
}

pub fn sync_remote_or_local_shell<T: AsRef<str>>(cmd: T) -> SpawnServerCommandResponse {
    let cmd_str = cmd.as_ref();
    let res = sync_remote_execute_shell(cmd_str);
    if res.exit_code == -1 {
        info!("Spawn server unreachable, falling back to local shell");
        run_local_shell(cmd_str)
    } else {
        res
    }
}

pub async fn async_remote_or_local_shell<T: AsRef<str>>(cmd: T) -> SpawnServerCommandResponse {
    let cmd_str = cmd.as_ref();
    let res = async_remote_execute_shell(cmd_str).await;
    if res.exit_code == -1 {
        info!("Spawn server unreachable, falling back to local shell");
        tokio::task::spawn_blocking({
            let c = cmd_str.to_string();
            move || run_local_shell(&c)
        })
        .await
        .unwrap_or(SpawnServerCommandResponse {
            exit_code: -6,
            stdout: "".into(),
            stderr: "Task join error (ERROR 21874-19963-15097)".to_string(),
        })
    } else {
        res
    }
}

pub fn sync_remote_or_local_exec<B, A, E, S, K, V, G, H>(
    binary: B,
    args: A,
    env: E,
    env_remove: G,
) -> SpawnServerCommandResponse
where
    B: AsRef<str>,
    A: IntoIterator<Item = S>,
    S: AsRef<str>,
    E: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
    G: IntoIterator<Item = H>,
    H: AsRef<str>,
{
    let binary = binary.as_ref().to_string();
    let args: Vec<String> = args.into_iter().map(|a| a.as_ref().to_string()).collect();
    let env: Vec<(String, String)> = env
        .into_iter()
        .map(|(k, v)| (k.as_ref().to_string(), v.as_ref().to_string()))
        .collect();
    let env_remove: Vec<String> = env_remove
        .into_iter()
        .map(|g| g.as_ref().to_string())
        .collect();
    let res = sync_remote_execute_exec(&binary, args.clone(), env.clone(), env_remove.clone());
    if res.exit_code == -1 {
        info!("Spawn server unreachable, falling back to local exec");
        run_local_exec(&binary, &args, &env, &env_remove)
    } else {
        res
    }
}

pub async fn async_remote_or_local_exec<B, A, E, S, K, V, G, H>(
    binary: B,
    args: A,
    env: E,
    env_remove: G,
) -> SpawnServerCommandResponse
where
    B: AsRef<str>,
    A: IntoIterator<Item = S>,
    S: AsRef<str>,
    E: IntoIterator<Item = (K, V)>,
    G: IntoIterator<Item = H>,
    H: AsRef<str>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    let binary = binary.as_ref().to_string();
    let args: Vec<String> = args.into_iter().map(|a| a.as_ref().to_string()).collect();

    let env: Vec<(String, String)> = env
        .into_iter()
        .map(|(k, v)| (k.as_ref().to_string(), v.as_ref().to_string()))
        .collect();
    let env_remove: Vec<String> = env_remove
        .into_iter()
        .map(|g| g.as_ref().to_string())
        .collect();

    let res =
        async_remote_execute_exec(&binary, args.clone(), env.clone(), env_remove.clone()).await;
    if res.exit_code == -1 {
        info!("Spawn server unreachable, falling back to local exec");
        tokio::task::spawn_blocking(move || run_local_exec(&binary, &args, &env, &env_remove))
            .await
            .unwrap_or(SpawnServerCommandResponse {
                exit_code: -6,
                stdout: "".into(),
                stderr: "Task join error (ERROR 139-11505-18643)".to_string(),
            })
    } else {
        res
    }
}

/// Executes a command via the system shell directly.
pub fn run_local_shell(cmd: &str) -> SpawnServerCommandResponse {
    let output = std::process::Command::new("sh").args(["-c", cmd]).output();
    match output {
        Ok(out) => SpawnServerCommandResponse {
            exit_code: out.status.code().unwrap_or(0),
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
        },
        Err(e) => SpawnServerCommandResponse {
            exit_code: -1,
            stdout: "".to_string(),
            stderr: format!("Local Execution Error: {e} (ERROR 13923-22388-7958)"),
        },
    }
}

/// Executes a binary with arguments directly.
pub fn run_local_exec(
    binary: &str,
    args: &[String],
    env: &[(String, String)],
    env_remove: &[String],
) -> SpawnServerCommandResponse {
    let mut command = std::process::Command::new(binary);
    command.args(args);
    for k in env_remove {
        command.env_remove(k);
    }
    for (k, v) in env {
        command.env(k, v);
    }
    match command.output() {
        Ok(out) => SpawnServerCommandResponse {
            exit_code: out.status.code().unwrap_or(0),
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
        },
        Err(e) => SpawnServerCommandResponse {
            exit_code: -1,
            stdout: "".to_string(),
            stderr: format!("Local Execution Error: {e} (ERROR 15400-27832-29418)"),
        },
    }
}

/// This macro executes a command via the system shell synchronously, either remotely via the spawn_server or locally if the spawn_server is unreachable.
/// Prefer using `srpc_or_local_exec!` for known binaries and arguments to avoid shell interpretation and reduce injection risk.
#[macro_export]
macro_rules! srpc_or_local_sh {
    ( $( $cmd:tt )* ) => {{
        $crate::sync_remote_or_local_shell(format!($( $cmd )*))
    }};
}

/// This macro executes a command via the system shell synchronously remotely via the spawn_server. This fails if the spawn_server is unreachable.
/// Prefer using `srpc_exec!` or `srpc_or_local_exec!` for known binaries and arguments to avoid shell interpretation and reduce injection risk.
#[macro_export]
macro_rules! srpc_sh {
    ( $( $cmd:tt )* ) => {{
        $crate::sync_remote_execute_shell(format!($( $cmd )*))
    }};
}

/// This macro executes a command via the system shell asynchronously, either remotely via the spawn_server or locally if the spawn_server is unreachable.
/// Prefer using `arpc_or_local_exec!` for known binaries and arguments to avoid shell interpretation and reduce injection risk.
#[macro_export]
macro_rules! arpc_or_local_sh {
    ( $( $cmd:tt )* ) => {{
        $crate::async_remote_or_local_shell(format!($( $cmd )*))
    }};
}

/// This macro executes a command via the system shell asynchronously remotely via the spawn_server. This fails if the spawn_server is unreachable.
/// The `arpc_or_local_sh!` macro is preferred for cases where the spawn_server may be unreachable.
/// Prefer using `arpc_exec!` or `arpc_or_local_exec!` for known binaries and arguments to avoid shell interpretation and reduce injection risk.
#[macro_export]
macro_rules! arpc_sh {
    ( $( $cmd:tt )* ) => {{
        $crate::async_remote_execute_shell(format!($( $cmd )*))
    }};
}

/// This macro executes a command via the system shell locally .
#[macro_export]
macro_rules! local_sh {
    ( $( $cmd:tt )* ) => {{
        $crate::run_local_shell($( $cmd )*)
    }};
}

/// This macro executes a binary with arguments synchronously, either remotely via the spawn_server or locally if the spawn_server is unreachable.
#[macro_export]
macro_rules! srpc_or_local_exec {
    ($binary:expr, $args:expr) => {{
        $crate::sync_remote_or_local_exec(
            $binary,
            $args,
            Vec::<(&str, &str)>::new(),
            Vec::<(&str)>::new(),
        )
    }};
    ($binary:expr, $args:expr, $env:expr) => {{ $crate::sync_remote_or_local_exec($binary, $args, $env, Vec::<&str>::new()) }};
    ($binary:expr, $args:expr, $env:expr, $env_remove:expr) => {{ $crate::sync_remote_or_local_exec($binary, $args, $env, $env_remove) }};
}

/// This macro executes a binary with arguments synchronously remotely via the spawn_server. This fails if the spawn_server is unreachable.
/// Prefer using `srpc_or_local_exec!` which also allows for local execution if the spawn_server is unreachable.
#[macro_export]
macro_rules! srpc_exec {
    ($binary:expr, $args:expr) => {{
        $crate::sync_remote_execute_exec(
            $binary,
            $args,
            Vec::<(&str, &str)>::new(),
            Vec::<(&str)>::new(),
        )
    }};
    ($binary:expr, $args:expr, $env:expr) => {{ $crate::sync_remote_execute_exec($binary, $args, $env, Vec::<&str>::new()) }};
    ($binary:expr, $args:expr, $env:expr, $env_remove:expr) => {{ $crate::sync_remote_execute_exec($binary, $args, $env, $env_remove) }};
}

/// This macro executes a binary with arguments asynchronously, either remotely via the spawn_server or locally if the spawn_server is unreachable.
#[macro_export]
macro_rules! arpc_or_local_exec {
    ($binary:expr, $args:expr) => {{
        $crate::async_remote_or_local_exec(
            $binary,
            $args,
            Vec::<(&str, &str)>::new(),
            Vec::<(&str)>::new(),
        )
    }};
    ($binary:expr, $args:expr, $env:expr) => {{ $crate::async_remote_or_local_exec($binary, $args, $env, Vec::<&str>::new()) }};
    ($binary:expr, $args:expr, $env:expr, $env_remove:expr) => {{ $crate::async_remote_or_local_exec($binary, $args, $env, $env_remove) }};
}

/// This macro executes a binary with arguments asynchronously remotely via the spawn_server. This fails if the spawn_server is unreachable.
/// Prefer using `arpc_or_local_exec!` which also allows for local execution if the spawn_server is unreachable.
#[macro_export]
macro_rules! arpc_exec {
    ($binary:expr, $args:expr) => {{
        $crate::async_remote_execute_exec(
            $binary,
            $args,
            Vec::<(&str, &str)>::new(),
            Vec::<(&str)>::new(),
        )
    }};
    ($binary:expr, $args:expr, $env:expr) => {{ $crate::async_remote_execute_exec($binary, $args, $env, Vec::<&str>::new()) }};
    ($binary:expr, $args:expr, $env:expr, $env_remove:expr) => {{ $crate::async_remote_execute_exec($binary, $args, $env, $env_remove) }};
}

/// This macro executes a binary with arguments locally.
#[macro_export]
macro_rules! local_exec {
    ($binary:expr, $args:expr) => {{ $crate::run_local_exec($binary, $args, &[], &[]) }};
    ($binary:expr, $args:expr, $env:expr) => {{ $crate::run_local_exec($binary, $args, $env, &[]) }};
    ($binary:expr, $args:expr, $env:expr, $env_remove:expr) => {{ $crate::run_local_exec($binary, $args, $env, $env_remove) }};
}
