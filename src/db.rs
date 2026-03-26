use crate::err::Result;
use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub fn establish_connection(database_url: &String) -> Result<SqliteConnection> {
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

pub fn read_todos(conn: &mut SqliteConnection) -> Result<Vec<Todo>> {
    use crate::schema::todos::dsl::*;

    let results = todos
        .select(Todo::as_select())
        .load(conn)
        .with_whatever_context(|err| format!("Failed to load persons: {}", err))?;
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
        .first::<bool>(conn)
        .with_whatever_context(|err| format!("Failed to read todo: {}", err))?;
    diesel::update(todos.filter(id.eq(item_id)))
        .set(completed.eq(!todo_completed))
        .execute(conn)
        .with_whatever_context(|err| format!("Failed to toggle todo: {}", err))
}

pub fn read_todo(conn: &mut SqliteConnection, todo_id: &String) -> Result<Todo> {
    use crate::schema::todos::dsl::*;

    let todo = todos
        .filter(id.eq(todo_id))
        .first::<Todo>(conn)
        .with_whatever_context(|err| format!("Failed to read todo: {}", err))?;
    Ok(todo)
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel_migrations::MigrationHarness;

    #[test]
    fn test_create_todo() {
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
