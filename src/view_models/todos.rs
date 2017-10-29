use std::collections::BTreeMap;
use rocket::response::status;

use domain::{Todo, TodoFilter};
use db::todos::{Repository, RepositoryError};

type Flags = BTreeMap<String, BTreeMap<String, bool>>;

#[derive(Serialize)]
pub struct Context {
    todos: Vec<Todo>,
    active_count: usize,
    current_filter: String,
    flags: Flags,
}

pub fn context(todos: Repository, filter: &TodoFilter) -> Result<Context, status::Custom<String>> {
    let items = match todos.list(&filter) {
        Ok(is) => is,
        Err(e) => return http_status_error(e),
    };

    let active_count = match todos.active_count() {
        Ok(c) => c,
        Err(e) => return http_status_error(e),
    };

    let flags = match flags(&todos, filter) {
        Ok(fs) => fs,
        Err(e) => return http_status_error(e),
    };

    let current_filter = to_query_value(filter);

    Ok(Context {
        todos: items,
        active_count: active_count,
        current_filter: current_filter,
        flags: flags,
    })
}

pub fn to_query_value(filter: &TodoFilter) -> String {
    format!("{:?}", filter).to_lowercase()
}

fn http_status_error(err: RepositoryError) -> Result<Context, status::Custom<String>> {
    Err(RepositoryError::into(err))
}

fn flags(todos: &Repository, filter: &TodoFilter) -> Result<Flags, RepositoryError> {
    let mut result = BTreeMap::new();

    let mut filters = BTreeMap::new();
    filters.insert(to_query_value(&filter), true);
    result.insert("filters".into(), filters);

    let mut special_cases = BTreeMap::new();
    special_cases.insert("all_completed".into(), todos.is_all_completed()?);
    special_cases.insert("any_completed".into(), todos.is_any_completed()?);
    special_cases.insert("at_least_one".into(), todos.todo_count()? > 0);
    result.insert("special_cases".into(), special_cases);


    Ok(result)
}
