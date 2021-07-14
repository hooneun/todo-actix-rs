use crate::{
    errors::{AppError, AppErrorType},
    models::{TodoItem, TodoList},
};
use deadpool_postgres::Client;
use std::io;
use tokio_pg_mapper::FromTokioPostgresRow;

pub async fn get_todos(client: &Client) -> Result<Vec<TodoList>, AppError> {
    let statement = client
        .prepare("SELECT * FROM todo_list ORDER BY id DESC LIMIT 10")
        .await
        .map_err(AppError::db_error)?;

    let todos = client
        .query(&statement, &[])
        .await
        .map_err(AppError::db_error)?
        .iter()
        .map(|row| TodoList::from_row_ref(row).unwrap())
        .collect::<Vec<TodoList>>();

    Ok(todos)
}

pub async fn get_items(client: &Client, list_id: i32) -> Result<Vec<TodoItem>, AppError> {
    let statement = client
        .prepare("SELECT * FROM todo_item WHERE list_id = $1 ORDER BY ID")
        .await
        .map_err(AppError::db_error)?;

    let items = client
        .query(&statement, &[&list_id])
        .await
        .map_err(AppError::db_error)?
        .iter()
        .map(|row| TodoItem::from_row_ref(row).unwrap())
        .collect::<Vec<TodoItem>>();

    Ok(items)
}

pub async fn create_todo(client: &Client, title: String) -> Result<TodoList, AppError> {
    let statement = client
        .prepare("INSERT INTO todo_list (title) VALUES ($1) RETURNING id, title")
        .await
        .map_err(AppError::db_error)?;

    client
        .query(&statement, &[&title])
        .await
        .map_err(AppError::db_error)?
        .iter()
        .map(|row| TodoList::from_row_ref(row).unwrap())
        .collect::<Vec<TodoList>>()
        .pop()
        .ok_or(AppError {
            message: Some("Error creating Todo list".to_string()),
            cause: Some("Unknown error".to_string()),
            error_type: AppErrorType::DbError,
        })
}

pub async fn check_item(client: &Client, list_id: i32, item_id: i32) -> Result<bool, AppError> {
    let statement = client
        .prepare("UPDATE todo_item SET checked = true where list_id = $1 and id = $2 and checked = false")
        .await
        .map_err(AppError::db_error)?;

    let result = client
        .execute(&statement, &[&list_id, &item_id])
        .await
        .map_err(AppError::db_error)?;

    match result {
        ref updated if *updated == 1 => Ok(true),
        _ => Ok(false),
    }
}
