use rocket::response::Redirect;

#[get("/")]
pub fn index() -> Redirect {
    Redirect::to("/todos?filter=all")
}
