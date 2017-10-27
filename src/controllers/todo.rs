use rocket::request::Form;
use rocket::response::{status, Redirect};
use rocket::State;

use UserSession;
use db::redis::RedisConnection;
use db::todos::Repository;

#[derive(FromForm)]
struct CreateForm {
    description: String,
}

#[post("/", data = "<form>")]
fn create(
    form: Form<CreateForm>,
    session: State<UserSession>,
    db: RedisConnection,
) -> Result<Redirect, status::Custom<String>> {
    let todo_fields = form.get();
    let repo = Repository::new(session.id, db);
    let description = todo_fields.description.trim();

    repo.create(&description)?;

    Ok(Redirect::to("/todos"))
}
