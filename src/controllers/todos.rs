use std::collections::BTreeMap;

use rocket::State;
use rocket_contrib::Template;
use rocket::response::status;

use UserSession;
use domain::Todo;
use db::redis::RedisConnection;
use db::todos::Repository;

#[derive(Debug, FromForm)]
struct UpdateForm {
    command: Option<String>,
}

#[get("/")]
pub fn show(
    session: State<UserSession>,
    db: RedisConnection,
) -> Result<Template, status::Custom<String>> {
    let repo = Repository::new(session.id, db);
    let todos = repo.list()?;

    render_list(todos)
}

fn render_list(todos: Vec<Todo>) -> Result<Template, status::Custom<String>> {
    let mut context: BTreeMap<String, Vec<Todo>> = BTreeMap::new();
    context.insert("todos".into(), todos);

    Ok(Template::render("todos/list", &context))
}
