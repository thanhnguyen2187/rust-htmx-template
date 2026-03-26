use crate::AppState;
use crate::db::{read_todo, read_todos, toggle_todo};
use crate::err::Result;
use crate::templates::todo_row;
use askama::Template;
use askama_web::WebTemplate;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use maud::html;
use snafu::ResultExt;
use std::sync::{Arc, Mutex};

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

pub struct Todo {
    pub id: String,
    pub title: String,
    pub completed: bool,
}

#[derive(Template, WebTemplate)]
#[template(path = "home.html")]
pub struct HomeTemplate {
    todos: Vec<Todo>,
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
