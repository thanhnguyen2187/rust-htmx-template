use askama::Template;
use askama_web::WebTemplate;
use axum::response::IntoResponse;

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

pub async fn home_handler() -> impl IntoResponse {
    HomeTemplate { todos: vec![] }
}
