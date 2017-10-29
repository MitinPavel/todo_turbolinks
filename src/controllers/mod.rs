use rocket::response::status;

use {todo_filter, QueryParams};
use view_models::todos::to_query_value;

pub mod root;
pub mod todo;
pub mod todos;

fn visit_todos(query_params: Option<QueryParams>) -> Result<String, status::Custom<String>> {
    let filter = todo_filter(query_params)?;
    let path = format!("/todos?filter={}", to_query_value(&filter));
    let command = format!("Turbolinks.clearCache(); Turbolinks.visit('{}')", path);

    Ok(command)
}
