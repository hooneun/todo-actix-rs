mod config;
mod db;
mod errors;
mod handlers;
mod models;

use crate::config::Config;
use crate::{handlers::*, models::AppState};
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use slog::info;
use std::io;

#[actix_rt::main]
async fn main() -> io::Result<()> {
    dotenv().ok();

    let config = Config::from_env().unwrap();

    let pool = config.configure_pool();

    let log = Config::configure_log();
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

#[cfg(test)]
mod integration_tests {

    use crate::handlers::*;
    use crate::models::AppState;
    use crate::{config::Config, models::TodoList};
    use actix_web::{test, web, App};
    use dotenv::dotenv;
    use lazy_static::lazy_static;
    use serde_json::json;

    lazy_static! {
        static ref APP_STATE: AppState = {
            dotenv().ok();

            let config = Config::from_env().unwrap();

            let pool = config.configure_pool();

            let log = Config::configure_log();

            AppState {
                pool: pool.clone(),
                log: log.clone(),
            }
        };
    }

    #[actix_rt::test]
    async fn test_get_todos() {
        let app = App::new()
            .data(APP_STATE.clone())
            .route("/todos{_:/?}", web::get().to(get_todos));

        let mut app = test::init_service(app).await;
        let req = test::TestRequest::get().uri("/todos").to_request();

        let res = test::call_service(&mut app, req).await;

        assert_eq!(res.status(), 200, "GET /todos should return status 200")
    }

    #[actix_rt::test]
    async fn test_create_todos() {
        let app = App::new()
            .data(APP_STATE.clone())
            .route("/todos{_:/?}", web::get().to(get_todos))
            .route("/todos{_:/?}", web::post().to(create_todo));

        let mut app = test::init_service(app).await;

        // Test create todo
        let todo_title = "Create todo list";

        let create_todo_list = json!({ "title": todo_title });

        let req = test::TestRequest::post()
            .uri("/todos")
            .header("Content-Type", "application/json")
            .set_payload(create_todo_list.to_string())
            .to_request();

        let res = test::call_service(&mut app, req).await;

        assert_eq!(res.status(), 200, "POST /todos should return status 200");

        let body = test::read_body(res).await;

        let try_created: Result<TodoList, serde_json::error::Error> = serde_json::from_slice(&body);

        assert!(try_created.is_ok(), "Response couldn't be parsed");

        let created_list = try_created.unwrap();

        // Test get created todo
        let req = test::TestRequest::get().uri("/todos").to_request();

        let todos: Vec<TodoList> = test::read_response_json(&mut app, req).await;

        let maybe_todo = todos.iter().find(|todo| todo.id == created_list.id);

        assert!(maybe_todo.is_some(), "Item not found!");
    }
}
