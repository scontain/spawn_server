use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web::Json};
use serde_derive::Serialize;
use shells::sh;
use spawn_server::{Command, CommandResponse, get_spawn_server_addr};
use tracing::{error, info, level_filters::LevelFilter, warn};
use tracing_actix_web::TracingLogger;
use tracing_subscriber::EnvFilter;

#[post("/command")]
async fn info(command: Json<Command>) -> impl Responder {
    let cmd = command.command.clone();

    if command.command.trim().is_empty() {
        return HttpResponse::BadRequest().body("command must not be empty");
    }

    let response = if let Ok((code, stdout, stderr)) =
        tokio::task::spawn_blocking(move || sh!("{}", command.command)).await
    {
        if code != 0 {
            warn!(%cmd, %stdout, %stderr, "command failed");
        } else {
            info!(%cmd, "command executed successfully");
        }
        CommandResponse {
            code,
            stdout,
            stderr,
        }
    } else {
        error!(%cmd, "failed to spawn command \n (ERROR 128912-12128-18492)");
        CommandResponse {
            code: 100,
            stdout: format!("spawn_server: command '{cmd}' failed \n (ERROR 128912-12128-18492)"),
            stderr: "spawn error".to_string(),
        }
    };

    HttpResponse::Ok().json(response)
}

#[derive(Serialize)]
struct ServerInfo {
    server: &'static str,
    version: &'static str,
}

#[get("/")]
async fn index() -> impl Responder {
    let version = env!("CARGO_PKG_VERSION");
    let server_info = ServerInfo {
        server: "spawn_server",
        version,
    };
    HttpResponse::Ok().json(server_info)
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
            .service(info)
    })
    .bind(addr)?
    .run()
    .await
}
