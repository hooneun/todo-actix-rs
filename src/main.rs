mod config;
mod db;
mod handlers;
mod models;

use crate::handlers::*;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use std::io;
use tokio_postgres::NoTls;

#[actix_rt::main]
async fn main() -> io::Result<()> {
    dotenv().ok();

    let config = config::Config::from_env().unwrap();

    let pool = config.pg.create_pool(NoTls).unwrap();

    println!(
        "Server Start http://{}:{}",
        config.server.host, config.server.port
    );

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
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
