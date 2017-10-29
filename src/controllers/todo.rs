use rocket::request::Form;
use rocket::response::status;

use {CookieSessionId, QueryParams};
use super::visit_todos;
use db::redis::RedisConnection;
use db::todos::Repository;
use domain::TodoId;

#[derive(FromForm)]
struct CreateForm {
    description: String,
}

#[derive(Debug, FromForm)]
struct UpdateForm {
    description: Option<String>,
    completed: Option<String>,
}

#[post("/?<query_params>", data = "<form>")]
fn create(
    form: Form<CreateForm>,
    query_params: Option<QueryParams>,
    session_id: CookieSessionId,
    db: RedisConnection,
) -> Result<String, status::Custom<String>> {
    let todo_fields = form.get();
    let repo = Repository::new(session_id.into(), db);

    repo.create(&todo_fields.description.trim())?; //TODO Handle empty description.

    visit_todos(query_params)
}

#[patch("/<id>?<query_params>", data = "<form>")]
fn update(
    id: TodoId,
    form: Form<UpdateForm>,
    query_params: Option<QueryParams>,
    session_id: CookieSessionId,
    db: RedisConnection,
) -> Result<String, status::Custom<String>> {
    let repo = Repository::new(session_id.into(), db);
    let todo_fields = form.get();

    match todo_fields.completed {
        Some(ref x) if x == "on" => repo.complete(id)?,
        Some(_) => repo.activate(id)?,
        None => (),
    };

    match todo_fields.description {
        Some(ref description) if description.trim().is_empty() => (),
        Some(ref description) => repo.update_description(id, description)?,
        None => (),
    };

    visit_todos(query_params)
}

#[delete("/<id>?<query_params>")]
fn destroy(
    id: TodoId,
    query_params: Option<QueryParams>,
    session_id: CookieSessionId,
    db: RedisConnection,
) -> Result<String, status::Custom<String>> {
    let repo = Repository::new(session_id.into(), db);

    repo.clear(id)?;

    visit_todos(query_params)
}
