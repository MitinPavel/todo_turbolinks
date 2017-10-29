use rocket::request::Form;
use rocket_contrib::Template;
use rocket::response::status;
use rocket::http;

use {todo_filter, CookieSessionId, QueryParams};
use super::visit_todos;
use db::redis::RedisConnection;
use db::todos::Repository;
use view_models::todos::context;

#[derive(Debug, FromForm)]
struct UpdateForm {
    command: Option<String>,
}

#[get("/?<query_params>")]
pub fn show(
    query_params: Option<QueryParams>,
    session_id: CookieSessionId,
    db: RedisConnection,
) -> Result<Template, status::Custom<String>> {
    let repo = Repository::new(session_id.into(), db);

    render_todos(query_params, repo)
}

#[patch("/?<query_params>", data = "<form>")]
fn update(
    form: Form<UpdateForm>,
    query_params: Option<QueryParams>,
    session_id: CookieSessionId,
    db: RedisConnection,
) -> Result<String, status::Custom<String>> {
    let form_data = form.get();
    let repo = Repository::new(session_id.into(), db);

    match form_data.command {
        Some(ref x) if x == "clear_completed" => repo.clear_completed()?,
        Some(ref x) if x == "activate_all" => repo.activate_all()?,
        Some(ref x) if x == "complete_all" => repo.complete_all()?,
        _ => Err(status::Custom(
            http::Status::BadRequest,
            "Unexpected command".into(),
        ))?,
    };

    visit_todos(query_params)
}

fn render_todos(
    query_params: Option<QueryParams>,
    repo: Repository,
) -> Result<Template, status::Custom<String>> {
    let filter = todo_filter(query_params)?;
    let context = context(repo, &filter)?;

    Ok(Template::render("todos/list", &context))
}
