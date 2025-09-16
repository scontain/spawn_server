use serde_derive::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use tracing::{error, info, warn};

const DEFAULT_SPAWN_SERVER_PORT: u16 = 8099;
const DEFAULT_SPAWN_SERVER_HOST: &str = "127.0.0.1";
const ENV_SPAWN_SERVER_HOST: &str = "SPAWN_SERVER_HOST";
const ENV_SPAWN_SERVER_PORT: &str = "SPAWN_SERVER_PORT";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Command {
    pub command: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CommandResponse {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn get_spawn_server_addr() -> SocketAddr {
    // Get host string
    let host = std::env::var(ENV_SPAWN_SERVER_HOST)
        .unwrap_or_else(|_| DEFAULT_SPAWN_SERVER_HOST.to_string());

    // Parse host as IP
    let ip: IpAddr = host
        .parse()
        .unwrap_or_else(|_| {
            eprintln!(
                "Invalid {ENV_SPAWN_SERVER_HOST} value: {host}. Must be a valid IP address (e.g., 127.0.0.1 or 0.0.0.0)"
            );
            std::process::exit(1);
        });

    // Parse port
    let port = std::env::var(ENV_SPAWN_SERVER_PORT)
        .unwrap_or_else(|_| DEFAULT_SPAWN_SERVER_PORT.to_string())
        .parse::<u16>()
        .unwrap_or_else(|_| {
            eprintln!(
                "Invalid {ENV_SPAWN_SERVER_PORT} value. Must be a number between 1 and 65535."
            );
            std::process::exit(1);
        });

    SocketAddr::new(ip, port)
}

fn get_spawner_command_url() -> String {
    let server_addr = get_spawn_server_addr();
    format!("{server_addr}/command")
}
pub fn sync_remote_execute<T: AsRef<str>>(cmd: T) -> (i32, String, String) {
    let cmd = Command {
        command: cmd.as_ref().to_string(),
    };

    use std::time::Duration;
    let very_long_timeout = Duration::new(60 * 60 * 24, 0); // 24h

    let client_result = reqwest::blocking::Client::builder()
        .timeout(very_long_timeout)
        .build();

    let client = match client_result {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "Failed to build reqwest client \n (ERROR 910-21087-27552)");
            return (
                -3,
                "".to_string(),
                format!("Client Build Error: {e} \n (ERROR 910-21087-27552)"),
            );
        }
    };
    let spawn_server_command_url = get_spawner_command_url();
    match client.post(spawn_server_command_url).json(&cmd).send() {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<CommandResponse>() {
                    Ok(result) => {
                        info!(code = result.code, "sync command executed successfully");
                        (result.code, result.stdout, result.stderr)
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to parse success response JSON \n (ERROR 20242-29979-4861)");
                        (
                            -4,
                            "".to_string(),
                            format!("JSON parse error: {e} \n (ERROR 20242-29979-4861)"),
                        )
                    }
                }
            } else {
                warn!(status = %resp.status(), "sync command returned non-success status");
                match resp.json::<CommandResponse>() {
                    Ok(result) => (
                        -2,
                        result.stdout,
                        format!("No Success Error: {}", result.stderr),
                    ),
                    Err(e) => {
                        error!(error = %e, "Failed to parse error response JSON \n (ERROR 20242-29979-4861)");
                        (
                            -5,
                            "".to_string(),
                            format!("JSON parse error: {e} \n (ERROR 20242-29979-4861)"),
                        )
                    }
                }
            }
        }
        Err(e) => {
            error!(error = %e, "sync command request failed \n (ERROR 9620-6359-13560)");
            (
                -1,
                "".to_string(),
                format!("RPC Error: {e} \n (ERROR 9620-6359-13560)"),
            )
        }
    }
}

pub async fn async_remote_execute<T: AsRef<str>>(cmd: T) -> (i32, String, String) {
    let cmd = Command {
        command: cmd.as_ref().to_owned(),
    };

    use std::time::Duration;
    let very_long_timeout = Duration::new(60 * 60 * 24, 0);

    let client = match reqwest::Client::builder()
        .timeout(very_long_timeout)
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "Failed to build async reqwest client \n (ERROR 12233-4984-31642)");
            return (
                -3,
                "".to_string(),
                format!("Client Build Error: {e} \n (ERROR 12233-4984-31642)"),
            );
        }
    };
    let spawn_server_command_url = get_spawner_command_url();

    match client
        .post(spawn_server_command_url)
        .json(&cmd)
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<CommandResponse>().await {
                    Ok(result) => {
                        info!(code = result.code, "async command executed successfully");
                        (result.code, result.stdout, result.stderr)
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to parse async success response JSON \n Error (ERROR 20242-29979-4861)");
                        (
                            -4,
                            "".to_string(),
                            format!("JSON parse error: {e} \n Error (ERROR 20242-29979-4861)"),
                        )
                    }
                }
            } else {
                warn!(status = %resp.status(), "async command returned non-success status");
                (-2, "".to_string(), "No Success Error".to_string())
            }
        }
        Err(e) => {
            error!(error = %e, "async command request failed \n (ERROR 14588-16483-23342)");
            (
                -1,
                "".to_string(),
                format!("RPC Error: {e} \n (ERROR 14588-16483-23342)"),
            )
        }
    }
}

/// Macro to execute the given command on the spawn server using synchronous communication
#[macro_export]
macro_rules! srpc {
    ( $( $cmd:tt )* ) => {{
        $crate::sync_remote_execute(format!($( $cmd )*))
    }};
}

/// Macro to execute the given command on the spawn server using asynchronous communication
#[macro_export]
macro_rules! arpc {
    ( $( $cmd:tt )* ) => {{
        $crate::async_remote_execute(format!($( $cmd )*))
    }};
}
