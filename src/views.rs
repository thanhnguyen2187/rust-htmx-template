use crate::AppState;
use crate::db::{Todo, create_todo, read_todo, read_todos, toggle_todo, update_todo};
use crate::err::Result;
use askama::Template;
use askama_web::WebTemplate;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use std::sync::{Arc, Mutex};
use axum::Form;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Template)] // this will generate the code...
#[template(path = "hello.html")] // using the template in this path, relative
// to the `templates` dir in the crate root
struct HelloTemplate<'a> {
    // the name of the struct can be anything
    name: &'a str, // the field name should match the variable name
                   // in your template
}

pub async fn hello_handler() -> impl IntoResponse {
    HelloTemplate { name: "world" }.to_string()
}

#[derive(Template, WebTemplate)]
#[template(path = "home.html")]
pub struct HomeTemplate {
    todos: Vec<Todo>,
}

#[derive(Template, WebTemplate)]
#[template(path = "todo_row.html")]
pub struct TodoRow {
    todo: Todo,
}

#[derive(Template, WebTemplate)]
#[template(path = "todo_row_editing.html")]
pub struct TodoRowEditing {
    todo: Todo,
}

pub async fn home_handler(State(state_arc): State<Arc<Mutex<AppState>>>) -> Result<HomeTemplate> {
    if let Ok(mut state) = state_arc.lock() {
        let todos = read_todos(&mut state.conn)?;
        let todos_dto = todos
            .iter()
            .map(|todo| Todo {
                id: todo.id.clone(),
                title: todo.title.clone(),
                completed: todo.completed,
            })
            .collect::<Vec<_>>();
        let template: HomeTemplate = HomeTemplate { todos: todos_dto };
        return Ok(template);
    }

    snafu::whatever!("unable to lock mutex")
}

pub async fn create_todo_handler(State(state_arc): State<Arc<Mutex<AppState>>>) -> Result<TodoRow> {
    if let Ok(mut state) = state_arc.lock() {
        let id = Uuid::new_v4().to_string();
        let todo_new = Todo {
            id,
            title: "New item".to_string(),
            completed: false,
        };
        create_todo(&mut state.conn, &todo_new)?;

        return Ok(TodoRow { todo: todo_new });
    }

    snafu::whatever!("unable to lock mutex")
}

pub async fn toggle_todo_handler(
    State(state_arc): State<Arc<Mutex<AppState>>>,
    Path(todo_id): Path<String>,
) -> Result<TodoRow> {
    if let Ok(mut state) = state_arc.lock() {
        toggle_todo(&mut state.conn, &todo_id)?;
        let todo = read_todo(&mut state.conn, &todo_id)?;

        return Ok(TodoRow { todo });
    }

    snafu::whatever!("unable to lock mutex")
}

pub async fn edit_todo_handler(
    State(state_arc): State<Arc<Mutex<AppState>>>,
    Path(todo_id): Path<String>,
) -> Result<TodoRowEditing> {
    if let Ok(mut state) = state_arc.lock() {
        let todo = read_todo(&mut state.conn, &todo_id)?;
        return Ok(TodoRowEditing { todo });
    }

    snafu::whatever!("unable to lock mutex")
}

#[derive(Debug, Deserialize)]
pub struct TodoForm {
    pub title: String,
}

pub async fn save_todo_handler(
    State(state_arc): State<Arc<Mutex<AppState>>>,
    Path(todo_id): Path<String>,
    Form(todo_form): Form<TodoForm>,
) -> Result<TodoRow> {
    if let Ok(mut state) = state_arc.lock() {
        let mut todo = read_todo(&mut state.conn, &todo_id)?;
        todo.title = todo_form.title.clone();
        update_todo(&mut state.conn, &todo)?;
        return Ok(TodoRow { todo });
    }

    snafu::whatever!("unable to lock mutex")
}
