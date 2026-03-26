use crate::db::{create_todo, delete_todo, read_todo, read_todos, toggle_todo, update_todo, Todo};
use crate::err::Result;
use crate::AppState;
use axum::extract::{Path, State};
use axum::Form;
use axum_login::{AuthSession, AuthUser, AuthnBackend};
use log::warn;
use maud::{html, Markup, DOCTYPE};
use serde::{Deserialize, Serialize};
use snafu::{whatever, ResultExt, Snafu};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub fn header(page_title: &str) -> Markup {
    html! {
        (DOCTYPE)
        head {
            meta charset="utf-8";
            title { (page_title) };
            script src="https://unpkg.com/htmx.org@2.0.4" {""};
            script defer src="https://unpkg.com/alpinejs@3.14.8" {""};
            link rel="stylesheet" type="text/css" href="./styles.css";
            link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png";
            link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png";
            link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png";
            link rel="manifest" href="/site.webmanifest";
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum TodoState {
    Default,
    Editing,
    Done,
}

pub fn todo_row(todo: &Todo) -> Markup {
    if !todo.completed {
        html! {
            tr {
                td {
                    (todo.title.clone())
                }
                td .flex .gap-2 {
                    button
                        .btn
                        .btn-success
                        hx-target="closest tr"
                        hx-post=(format!("/toggle/{}", todo.id.as_str()))
                        hx-swap="outerHTML"
                        { "Finish" }
                    ;
                    button
                        .btn
                        .btn-primary
                        hx-target="closest tr"
                        hx-post=(format!("/edit/{}", todo.id.as_str()))
                        hx-swap="outerHTML"
                        { "Edit" }
                    ;
                    button
                        .btn
                        hx-confirm="Are you sure?"
                        hx-delete=(format!("/delete/{}", todo.id.as_str()))
                        hx-swap="delete"
                        hx-target="closest tr"
                        { "Delete" }
                    ;
                }
            }
        }
    } else {
        html! {
            tr {
                td .line-through {
                    (todo.title.clone())
                }
                td .flex .gap-2 {
                    button
                        .btn
                        .btn-warning
                        hx-target="closest tr"
                        hx-post=(format!("/toggle/{}", todo.id.as_str()))
                        hx-swap="outerHTML"
                        { "Reopen" }
                    ;
                    button .btn { "Delete" }
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TodoWithTemp {
    pub data: Todo,
    pub data_temp: Todo,
}

pub async fn page_home(State(state_arc): State<Arc<Mutex<AppState>>>) -> Result<Markup> {
    let markup = if let Ok(mut state) = state_arc.lock() {
        let todos = read_todos(&mut state.conn)
            .with_whatever_context(|err| format!("Failed to read todos: {}", err))?;
        html! {
            (header("TODO Home"))
            body {
                div .container .mx-auto .p-4 .flex .flex-col .gap-2 {
                    h1 .text-3xl .font-bold { "TODO Home" }
                    table .table .w-full {
                        thead {
                            tr {
                                th { "Title" }
                                th { "Actions" }
                            }
                        }
                        tbody {
                            @for todo in todos {
                                (todo_row(&todo))
                            }
                            tr {
                                td {
                                    button
                                        .btn
                                        .btn-success
                                        hx-post="/create"
                                        hx-target="closest tr"
                                        hx-swap="beforebegin"
                                    { "Create" };
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        html! {
            "Unable to get global state"
        }
    };

    Ok(markup)
}

pub async fn page_toggle_todo(
    State(state_arc): State<Arc<Mutex<AppState>>>,
    Path(todo_id): Path<String>,
) -> Result<Markup> {
    let markup = if let Ok(mut state) = state_arc.lock() {
        toggle_todo(&mut state.conn, &todo_id)?;
        let todo = read_todo(&mut state.conn, &todo_id)?;
        todo_row(&todo)
    } else {
        html! {
            "Unable to get global state"
        }
    };

    Ok(markup)
}

pub async fn page_edit_todo(
    State(state_arc): State<Arc<Mutex<AppState>>>,
    Path(todo_id): Path<String>,
) -> Result<Markup> {
    let markup = if let Ok(mut state) = state_arc.lock() {
        let todo = read_todo(&mut state.conn, &todo_id)?;
        html! {
            tr {
                td {
                    input
                        .input
                        .input-bordered
                        type="text"
                        name="title"
                        value=(todo.title.clone())
                    ;
                }
                td .flex .gap-2 {
                    button
                        .btn
                        .btn-primary
                        hx-post=(format!("/save/{}", todo.id.as_str()))
                        hx-include="input[name='title']"
                        hx-target="closest tr"
                        hx-swap="outerHTML"
                        { "Save" }
                    ;
                    button
                        .btn
                        hx-target="closest tr"
                        hx-post=(format!("/default/{}", todo.id.as_str()))
                        hx-swap="outerHTML"
                        { "Reset" }
                    ;
                }
            }
        }
    } else {
        html! {
            "Unable to get global state"
        }
    };

    Ok(markup)
}

pub async fn page_default_todo(
    State(state_arc): State<Arc<Mutex<AppState>>>,
    Path(todo_id): Path<String>,
) -> Result<Markup> {
    let markup = if let Ok(mut state) = state_arc.lock() {
        let todo = read_todo(&mut state.conn, &todo_id)?;
        todo_row(&todo)
    } else {
        html! {
            "Unable to get global state"
        }
    };

    Ok(markup)
}

#[derive(Debug, Deserialize)]
pub struct TodoForm {
    pub title: String,
}

pub async fn page_save_todo(
    State(state_arc): State<Arc<Mutex<AppState>>>,
    Path(todo_id): Path<String>,
    Form(todo_form): Form<TodoForm>,
) -> Result<Markup> {
    let markup = if let Ok(mut state) = state_arc.lock() {
        let mut todo = read_todo(&mut state.conn, &todo_id)?;
        todo.title = todo_form.title.clone();
        update_todo(&mut state.conn, &todo)?;
        todo_row(&todo)
    } else {
        html! {
            "Unable to get global state"
        }
    };

    Ok(markup)
}

pub async fn page_create_todo(State(state_arc): State<Arc<Mutex<AppState>>>) -> Result<Markup> {
    let markup = if let Ok(mut state) = state_arc.lock() {
        let id = Uuid::new_v4().to_string();
        let todo_new = Todo {
            id,
            title: "New item".to_string(),
            completed: false,
        };
        create_todo(&mut state.conn, &todo_new)?;
        todo_row(&todo_new)
    } else {
        html! {
            "Unable to get global state"
        }
    };

    Ok(markup)
}

pub async fn page_delete_todo(
    State(state_arc): State<Arc<Mutex<AppState>>>,
    Path(todo_id): Path<String>,
) -> Result<()> {
    if let Ok(mut state) = state_arc.lock() {
        delete_todo(&mut state.conn, &todo_id)?;
    } else {
        warn!("Unable to get global state");
    };

    Ok(())
}

pub async fn page_login() -> Markup {
    html! {
        (header("Login"))
        body {
            div .container .mx-auto .p-4 {
                h1 .text-3xl .font-bold .mb-2 { "Login" }
                form method="post" {
                    div .flex .flex-col .w-80 {
                        label .label { "Username" }
                        input
                            .input
                            .input-bordered
                            type="text"
                            name="username"
                            placeholder="Username"
                        ;
                        label .label { "Password" }
                        input
                            .input
                            .input-bordered
                            type="password"
                            name="password"
                            placeholder="Password"
                        ;
                        div .mt-4 .flex .flex-row-reverse {
                            button
                                .btn
                                .btn-primary
                                type="submit"
                                { "Login" }
                            ;
                        }
                    }
                }
            }
        }
    }
}

pub async fn page_login_success() -> Markup {
    html! {
        (header("Login Success"))
        body {
            div .container .mx-auto .p-4 {
                h1 .text-3xl .font-bold .mb-2 { "Login Success" }
                p { "You are now logged in" }
            }
        }
    }
}

pub async fn page_unimplemented() -> Markup {
    html! {
        (header("Unimplemented!!1"))
        body {
            div .container .mx-auto .p-4 {
                h1 .text-3xl .font-bold .mb-2 { "Unimplemented" }
                p { "Please check again later" }
            }
        }
    }
}
