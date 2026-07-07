use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web::Json};
use spawn_server::{
    SpawnServerCommandRequest, SpawnServerCommandResponse, get_spawn_server_addr, run_local_exec,
    run_local_shell,
};
use tracing::{error, info, level_filters::LevelFilter, warn};
use tracing_actix_web::TracingLogger;
use tracing_subscriber::EnvFilter;

fn execute_command_locally(cmd: SpawnServerCommandRequest) -> SpawnServerCommandResponse {
    match cmd {
        SpawnServerCommandRequest::Shell { command } => run_local_shell(&command),
        SpawnServerCommandRequest::Exec {
            binary,
            args,
            env,
            env_remove,
        } => run_local_exec(&binary, &args, &env, &env_remove),
    }
}

#[post("/command")]
async fn request(command: Json<SpawnServerCommandRequest>) -> impl Responder {
    let cmd = command.into_inner();

    let is_empty = match &cmd {
        SpawnServerCommandRequest::Shell { command } => command.trim().is_empty(),
        SpawnServerCommandRequest::Exec { binary, .. } => binary.trim().is_empty(),
    };
    if is_empty {
        return HttpResponse::BadRequest().body("command must not be empty");
    }

    let cmd_desc = match &cmd {
        SpawnServerCommandRequest::Shell { command } => command.clone(),
        SpawnServerCommandRequest::Exec { binary, args, .. } => {
            format!("{binary} {}", args.join(" "))
        }
    };

    let response = if let Ok(SpawnServerCommandResponse {
        exit_code,
        stdout,
        stderr,
    }) = tokio::task::spawn_blocking(move || execute_command_locally(cmd)).await
    {
        if exit_code != 0 {
            warn!(cmd = %cmd_desc, %stdout, %stderr, "command failed");
        } else {
            info!(cmd = %cmd_desc, "command executed successfully");
        }
        SpawnServerCommandResponse {
            exit_code,
            stdout,
            stderr,
        }
    } else {
        error!(cmd = %cmd_desc, "failed to spawn command");
        SpawnServerCommandResponse {
            exit_code: 100,
            stdout: format!("spawn_server: command '{cmd_desc}' failed"),
            stderr: "spawn error".to_string(),
        }
    };

    HttpResponse::Ok().json(response)
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().json(r#"{"server": "spawn_server", "version": "0.1.0"}"#)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(LevelFilter::INFO.into())) // RUST_LOG controls level, info as default
        .with_target(false)
        .init();

    let addr = get_spawn_server_addr();

    tracing::info!("Starting spawn_server on {addr}");

    HttpServer::new(|| {
        App::new()
            .wrap(TracingLogger::default()) // <-- per-request spans
            .service(index)
            .service(request)
    })
    .bind(addr)?
    .run()
    .await
}
