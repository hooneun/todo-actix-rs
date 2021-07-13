use crate::models::{TodoItem, TodoList};
use deadpool_postgres::Client;
use std::io;
use tokio_pg_mapper::FromTokioPostgresRow;

pub async fn get_todos(client: &Client) -> Result<Vec<TodoList>, io::Error> {
    let statement = client
        .prepare("SELECT * FROM todo_list ORDER BY id DESC LIMIT 10")
        .await
        .unwrap();

    let todos = client
        .query(&statement, &[])
        .await
        .expect("Error getting todo lists")
        .iter()
        .map(|row| TodoList::from_row_ref(row).unwrap())
        .collect::<Vec<TodoList>>();

    Ok(todos)
}

pub async fn get_items(client: &Client, list_id: i32) -> Result<Vec<TodoItem>, io::Error> {
    let statement = client
        .prepare("SELECT * FROM todo_item WHERE list_id = $1 ORDER BY ID")
        .await
        .unwrap();

    let items = client
        .query(&statement, &[&list_id])
        .await
        .expect("Error getting todo lists")
        .iter()
        .map(|row| TodoItem::from_row_ref(row).unwrap())
        .collect::<Vec<TodoItem>>();

    Ok(items)
}

pub async fn create_todo(client: &Client, title: String) -> Result<TodoList, io::Error> {
    let statement = client
        .prepare("INSERT INTO todo_list (title) VALUES ($1) RETURNING id, title")
        .await
        .unwrap();

    client
        .query(&statement, &[&title])
        .await
        .expect("Error creating todo list")
        .iter()
        .map(|row| TodoList::from_row_ref(row).unwrap())
        .collect::<Vec<TodoList>>()
        .pop()
        .ok_or(io::Error::new(
            io::ErrorKind::Other,
            "Error creating todo list",
        ))
}

pub async fn check_item(client: &Client, list_id: i32, item_id: i32) -> Result<(), io::Error> {
    let statement = client
        .prepare("UPDATE todo_item SET checked = true where list_id = $1 and id = $2 and checked = false")
        .await
        .unwrap();

    let result = client
        .execute(&statement, &[&list_id, &item_id])
        .await
        .expect("Error checking todo item");

    match result {
        ref updated if *updated == 1 => Ok(()),
        _ => Err(io::Error::new(io::ErrorKind::Other, "Failed to check list")),
    }
}
