use crate::err::Result;
use diesel::prelude::*;
use diesel_migrations::{EmbeddedMigrations, embed_migrations};
use sea_query::{Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

pub fn establish_connection(database_url: &str) -> Result<SqliteConnection> {
    SqliteConnection::establish(database_url)
        .with_whatever_context(|err| format!("Failed to connect to {}: {}", database_url, err))
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::todos)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub completed: bool,
}

pub fn create_todo(conn: &mut SqliteConnection, item: &Todo) -> Result<usize> {
    use crate::schema::todos::dsl::*;

    diesel::insert_into(todos)
        .values(item)
        .execute(conn)
        .with_whatever_context(|err| format!("Failed to insert todo: {}", err))
}

pub fn create_todo_v2(conn: &mut rusqlite::Connection, item: &Todo) -> Result<usize> {
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
    let result = stmt.execute(&*values.as_params());

    Ok(result?)
}

pub fn read_todos(conn: &mut SqliteConnection) -> Result<Vec<Todo>> {
    use crate::schema::todos::dsl::*;

    let results = todos.select(Todo::as_select()).load(conn)?;
    Ok(results)
}

pub fn update_todo(conn: &mut SqliteConnection, item: &Todo) -> Result<usize> {
    use crate::schema::todos::dsl::*;

    diesel::update(todos.filter(id.eq(item.id.clone())))
        .set(item)
        .execute(conn)
        .with_whatever_context(|err| format!("Failed to update todo: {}", err))
}

pub fn delete_todo(conn: &mut SqliteConnection, todo_id: &String) -> Result<usize> {
    use crate::schema::todos::dsl::*;

    diesel::delete(todos.filter(id.eq(todo_id)))
        .execute(conn)
        .with_whatever_context(|err| format!("Failed to delete todo: {}", err))
}

pub fn toggle_todo(conn: &mut SqliteConnection, item_id: &String) -> Result<usize> {
    use crate::schema::todos::dsl::*;

    let todo_completed = todos
        .select(completed)
        .filter(id.eq(item_id))
        .first::<bool>(conn)?;
    Ok(diesel::update(todos.filter(id.eq(item_id)))
        .set(completed.eq(!todo_completed))
        .execute(conn)?)
}

pub fn read_todo(conn: &mut SqliteConnection, todo_id: &String) -> Result<Todo> {
    use crate::schema::todos::dsl::*;

    let todo = todos.filter(id.eq(todo_id)).first::<Todo>(conn)?;
    Ok(todo)
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel_migrations::MigrationHarness;

    #[test]
    fn test_create_todo() {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        let report = embedded::migrations::runner()
            .run(&mut conn)
            .expect("should be able to run migration");
        println!("Report: {:?}", report);
        let new_todo = Todo {
            id: "1".to_string(),
            title: "Test".to_string(),
            completed: false,
        };
        create_todo_v2(&mut conn, &new_todo).expect("should be able to create todo");
    }

    #[test]
    fn test_read_todos() {
        let mut conn = establish_connection(&":memory:".to_owned())
            .expect("Should be able to create in-memory database");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Should be able to run migrations");

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
        let mut conn = establish_connection(&":memory:".to_owned())
            .expect("Should be able to create in-memory database");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Should be able to run migrations");

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
        update_todo(&mut conn, &updated_todo).expect("Should be able to update todo");

        let todos = read_todos(&mut conn).expect("Should be able to read todos");
        assert_eq!(todos.len(), 1);

        let todo = todos.first().expect("Should be able to get first todo");
        assert_eq!(todo.id, updated_todo.id);
        assert_eq!(todo.title, updated_todo.title);
        assert_eq!(todo.completed, updated_todo.completed);
    }

    #[test]
    fn test_delete_todo() {
        let mut conn = establish_connection(&":memory:".to_owned())
            .expect("Should be able to create in-memory database");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Should be able to run migrations");

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
        let mut conn = establish_connection(&":memory:".to_owned())
            .expect("Should be able to create in-memory database");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Should be able to run migrations");

        let new_todo = Todo {
            id: "1".to_string(),
            title: "Test".to_string(),
            completed: false,
        };
        create_todo(&mut conn, &new_todo).expect("Should be able to create todo");

        let todos = read_todos(&mut conn).expect("Should be able to read todos");
        assert_eq!(todos.len(), 1);

        toggle_todo(&mut conn, &new_todo.id).expect("Should be able to toggle todo");

        let todos = read_todos(&mut conn).expect("Should be able to read todos");
        assert_eq!(todos.len(), 1);

        let todo = todos.first().expect("Should be able to get first todo");
        assert_eq!(todo.id, new_todo.id);
        assert_eq!(todo.title, new_todo.title);
        assert_eq!(todo.completed, !new_todo.completed);
    }
}
