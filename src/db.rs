use crate::err::Result;
use sea_query::{Expr, ExprTrait, Iden, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

pub fn establish_connection(database_url: &str) -> Result<rusqlite::Connection> {
    rusqlite::Connection::open(database_url)
        .with_whatever_context(|err| format!("Failed to connect to {}: {}", database_url, err))
}

pub fn run_migrations(conn: &mut rusqlite::Connection) -> Result<()> {
    embedded::migrations::runner()
        .run(conn)
        .map(|_| ())
        .map_err(|_| crate::err::Error::DatabaseMigration {})
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub completed: bool,
}

pub fn create_todo(conn: &mut rusqlite::Connection, item: &Todo) -> Result<usize> {
    let (sql, values) = Query::insert()
        .into_table("todos")
        .columns(["id", "title", "completed"])
        .values_panic([
            item.id.clone().into(),
            item.title.clone().into(),
            item.completed.into(),
        ])
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = conn.prepare_cached(sql.as_str())?;
    Ok(stmt.execute(&*values.as_params())?)
}

pub fn read_todos(conn: &mut rusqlite::Connection) -> Result<Vec<Todo>> {
    let (sql, values) = Query::select()
        .columns(["id", "title", "completed"])
        .from("todos")
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = conn.prepare_cached(sql.as_str())?;
    let rows = stmt.query_map(&*values.as_params(), |row| {
        Ok(Todo {
            id: row.get(0)?,
            title: row.get(1)?,
            completed: row.get(2)?,
        })
    })?;

    Ok(rows.collect::<rusqlite::Result<Vec<Todo>>>()?)
}

pub fn update_todo(conn: &mut rusqlite::Connection, item: &Todo) -> Result<usize> {
    let (sql, values) = Query::update()
        .table("todos")
        .value("title", item.title.clone())
        .value("completed", item.completed)
        .and_where(Expr::col("id").equals(item.id.clone()))
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = conn.prepare_cached(sql.as_str())?;
    Ok(stmt.execute(&*values.as_params())?)
}

pub fn delete_todo(conn: &mut rusqlite::Connection, todo_id: &str) -> Result<usize> {
    let (sql, values) = Query::delete()
        .from_table("todos")
        .and_where(Expr::col("id").equals(todo_id.to_string()))
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = conn.prepare_cached(sql.as_str())?;
    Ok(stmt.execute(&*values.as_params())?)
}

pub fn toggle_todo(conn: &mut rusqlite::Connection, item_id: &String) -> Result<usize> {
    let todo = read_todo(conn, item_id)?;

    let (sql, values) = Query::update()
        .table("todos")
        .value("completed", !todo.completed)
        .and_where(Expr::col("id").equals(item_id.to_string()))
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = conn.prepare_cached(sql.as_str())?;
    Ok(stmt.execute(&*values.as_params())?)
}

pub fn read_todo(conn: &mut rusqlite::Connection, todo_id: &str) -> Result<Todo> {
    let (sql, values) = Query::select()
        .columns(["id", "title", "completed"])
        .from("todos")
        .and_where(Expr::col("id").eq(todo_id.to_string()).into())
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = conn.prepare_cached(sql.as_str())?;
    let todo = stmt.query_row(&*values.as_params(), |row| {
        Ok(Todo {
            id: row.get(0)?,
            title: row.get(1)?,
            completed: row.get(2)?,
        })
    })?;

    Ok(todo)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> rusqlite::Connection {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        embedded::migrations::runner()
            .run(&mut conn)
            .expect("should be able to run migration");
        conn
    }

    #[test]
    fn test_create_todo() {
        let mut conn = setup();
        let new_todo = Todo {
            id: "1".to_string(),
            title: "Test".to_string(),
            completed: false,
        };
        create_todo(&mut conn, &new_todo).expect("should be able to create todo");
    }

    #[test]
    fn test_read_todos() {
        let mut conn = setup();

        let todos = read_todos(&mut conn).expect("Should be able to read todos");
        assert_eq!(todos.len(), 0);

        let new_todo = Todo {
            id: "1".to_string(),
            title: "Test".to_string(),
            completed: false,
        };
        create_todo(&mut conn, &new_todo).expect("Should be able to create todo");

        let todos = read_todos(&mut conn).expect("Should be able to read todos");
        assert_eq!(todos.len(), 1);

        let todo = todos.first().expect("Should be able to get first todo");
        assert_eq!(todo.id, new_todo.id);
    }

    #[test]
    fn test_update_todo() {
        let mut conn = setup();

        let new_todo = Todo {
            id: "1".to_string(),
            title: "Test".to_string(),
            completed: false,
        };
        create_todo(&mut conn, &new_todo).expect("Should be able to create todo");

        let updated_todo = Todo {
            id: "1".to_string(),
            title: "Updated".to_string(),
            completed: true,
        };
        let row_count =
            update_todo(&mut conn, &updated_todo).expect("Should be able to update todo");
        assert!(row_count > 0);

        let todos = read_todos(&mut conn).expect("Should be able to read todos");
        assert_eq!(todos.len(), 1);

        let todo = todos.first().expect("Should be able to get first todo");
        assert_eq!(todo.id, updated_todo.id);
        assert_eq!(todo.title, updated_todo.title);
        assert_eq!(todo.completed, updated_todo.completed);
    }

    #[test]
    fn test_delete_todo() {
        let mut conn = setup();

        let new_todo = Todo {
            id: "1".to_string(),
            title: "Test".to_string(),
            completed: false,
        };
        create_todo(&mut conn, &new_todo).expect("Should be able to create todo");

        let todos = read_todos(&mut conn).expect("Should be able to read todos");
        assert_eq!(todos.len(), 1);

        delete_todo(&mut conn, &new_todo.id).expect("Should be able to delete todo");

        let todos = read_todos(&mut conn).expect("Should be able to read todos");
        assert_eq!(todos.len(), 0);
    }

    #[test]
    fn test_toggle_todo() {
        let mut conn = setup();

        let new_todo = Todo {
            id: "1".to_string(),
            title: "Test".to_string(),
            completed: false,
        };
        create_todo(&mut conn, &new_todo).expect("Should be able to create todo");

        toggle_todo(&mut conn, &new_todo.id).expect("Should be able to toggle todo");

        let todos = read_todos(&mut conn).expect("Should be able to read todos");
        assert_eq!(todos.len(), 1);

        let todo = todos.first().expect("Should be able to get first todo");
        assert_eq!(todo.id, new_todo.id);
        assert_eq!(todo.title, new_todo.title);
        assert_eq!(todo.completed, !new_todo.completed);
    }
}
