mod config;
mod db;
mod errors;
mod handlers;
mod models;

use crate::{handlers::*, models::AppState};
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use slog::{info, o, Drain, Logger};
use slog_async;
use slog_term;
use std::io;
use tokio_postgres::NoTls;

fn configure_log() -> Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let console_drain = slog_term::FullFormat::new(decorator).build().fuse();
    let console_drain = slog_async::Async::new(console_drain).build().fuse();

    slog::Logger::root(console_drain, o!("v" => env!("CARGO_PKG_VERSION")))
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    dotenv().ok();

    let config = config::Config::from_env().unwrap();

    let pool = config.pg.create_pool(NoTls).unwrap();

    let log = configure_log();
    info!(
        log,
        "Server Start http://{}:{}", config.server.host, config.server.port
    );

    HttpServer::new(move || {
        App::new()
            .data(AppState {
                pool: pool.clone(),
                log: log.clone(),
            })
            .route("/", web::get().to(status))
            .route("/todos{_:/?}", web::get().to(get_todos))
            .route("/todos{_:/?}", web::post().to(create_todo))
            .route("/todos/{list_id}/items", web::get().to(get_items))
            .route(
                "/todos/{list_id}/items/{item_id}{_:/?}",
                web::put().to(check_item),
            )
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
    .await
}
