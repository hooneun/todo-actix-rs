use crate::models::TodoList;
use deadpool_postgres::Client;
use std::io;
use tokio_pg_mapper::FromTokioPostgresRow;

pub async fn get_todos(client: &Client) -> Result<Vec<TodoList>, io::Error> {
    let statement = client.prepare("SELECT * FROM todo_list").await.unwrap();

    let todos = client
        .query(&statement, &[])
        .await
        .expect("Error getting todo lists")
        .iter()
        .map(|row| TodoList::from_row_ref(row).unwrap())
        .collect::<Vec<TodoList>>();

    Ok(todos)
}
