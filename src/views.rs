use askama::Template;
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
