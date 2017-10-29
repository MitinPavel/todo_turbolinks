use std::ops::Deref;
use r2d2;
use r2d2_redis::RedisConnectionManager;
use redis::Commands;
use redis::RedisError;
use serde_json;
use rocket::response::status;
use rocket::http;

use db::redis::RedisConnection;
use domain::{SessionId, Todo, TodoFilter, TodoId};

impl Deref for RedisConnection {
    type Target = r2d2::PooledConnection<RedisConnectionManager>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub enum RepositoryError {
    RedisErr(String),
    JsonErr(String),
}

pub struct Repository {
    session_id: SessionId,
    connection: RedisConnection,
}

impl From<RepositoryError> for status::Custom<String> {
    fn from(err: RepositoryError) -> Self {
        let message = match err {
            RepositoryError::RedisErr(msg) => msg,
            RepositoryError::JsonErr(msg) => msg,
        };

        status::Custom(http::Status::ServiceUnavailable, message)
    }
}

impl From<RedisError> for RepositoryError {
    fn from(err: RedisError) -> Self {
        RepositoryError::RedisErr(err.category().into())
    }
}

impl From<serde_json::Error> for RepositoryError {
    fn from(err: serde_json::Error) -> Self {
        RepositoryError::JsonErr(err.to_string())
    }
}

impl Repository {
    pub fn new(session_id: SessionId, connection: RedisConnection) -> Self {
        Repository {
            session_id: session_id,
            connection: connection,
        }
    }

    //
    // Commands
    //

    pub fn create(&self, description: &str) -> Result<(), RepositoryError> {
        let sequence_key = format!("todo_id_sequence:{}", self.session_id);
        let todo_id: TodoId = self.connection.incr(sequence_key, 1)?;
        let todo = Todo {
            id: todo_id,
            completed: false,
            description: description.to_string(),
        };
        let payload = serde_json::to_string(&todo)?;

        let _: () = self.connection.hset(&self.session_key(), todo_id, payload)?;

        Ok(())
    }

    pub fn clear(&self, todo_id: TodoId) -> Result<(), RepositoryError> {
        let _: () = self.connection
            .hdel(&self.session_key(), hash_field(todo_id))?;

        Ok(())
    }

    pub fn update_description(
        &self,
        todo_id: TodoId,
        description: &String,
    ) -> Result<(), RepositoryError> {
        self.update_todo(todo_id, &|t: &mut Todo| t.description = description.clone())
    }

    pub fn complete(&self, todo_id: TodoId) -> Result<(), RepositoryError> {
        self.update_todo(todo_id, &|t: &mut Todo| t.completed = true)
    }

    pub fn activate(&self, todo_id: TodoId) -> Result<(), RepositoryError> {
        self.update_todo(todo_id, &|t: &mut Todo| t.completed = false)
    }

    pub fn complete_all(&self) -> Result<(), RepositoryError> {
        self.active()?
            .into_iter()
            .map(|t| self.complete(t.id))
            .collect::<Result<Vec<()>, _>>()?;

        Ok(())
    }

    pub fn activate_all(&self) -> Result<(), RepositoryError> {
        self.completed()?
            .into_iter()
            .map(|t| self.activate(t.id))
            .collect::<Result<Vec<()>, _>>()?;

        Ok(())
    }

    pub fn clear_completed(&self) -> Result<(), RepositoryError> {
        self.completed()?
            .into_iter()
            .map(|t| self.clear(t.id))
            .collect::<Result<Vec<()>, _>>()?;

        Ok(())
    }

    //
    // Queries
    //

    pub fn list(&self, filter: &TodoFilter) -> Result<Vec<Todo>, RepositoryError> {
        match *filter {
            TodoFilter::All => self.all(),
            TodoFilter::Active => self.active(),
            TodoFilter::Completed => self.completed(),
        }
    }

    pub fn active_count(&self) -> Result<usize, RepositoryError> {
        Ok(self.active()?.len())
    }

    pub fn is_all_completed(&self) -> Result<bool, RepositoryError> {
        Ok(self.all()?.iter().all(|t| t.completed))
    }

    pub fn is_any_completed(&self) -> Result<bool, RepositoryError> {
        Ok(self.all()?.iter().any(|t| t.completed))
    }

    pub fn todo_count(&self) -> Result<usize, RepositoryError> {
        Ok(self.connection.hlen(self.session_key())?)
    }

    //
    // Private functions
    //

    fn all(&self) -> Result<Vec<Todo>, RepositoryError> {
        let ids_and_payloads: Vec<(usize, String)> = self.connection.hgetall(self.session_key())?;

        let todos = ids_and_payloads
            .into_iter()
            .map(|(_, payload)| serde_json::from_str(&payload))
            .flat_map(|r| r)
            .collect();

        Ok(todos)
    }

    fn active(&self) -> Result<Vec<Todo>, RepositoryError> {
        self.all()
            .map(|todos| todos.into_iter().filter(|t| !t.completed).collect())
    }

    fn completed(&self) -> Result<Vec<Todo>, RepositoryError> {
        self.all()
            .map(|todos| todos.into_iter().filter(|t| t.completed).collect())
    }

    fn update_todo(
        &self,
        todo_id: TodoId,
        updater: &Fn(&mut Todo) -> (),
    ) -> Result<(), RepositoryError> {
        let json: String = self.connection.hget(&self.session_key(), todo_id)?;
        let mut todo: Todo = serde_json::from_str(&json)?;

        updater(&mut todo);

        let updated_json = serde_json::to_string(&todo)?;
        let _: () = self.connection
            .hset(&self.session_key(), hash_field(todo_id), updated_json)?;

        Ok(())
    }

    fn session_key(&self) -> String {
        format!("todos:{}", self.session_id)
    }
}

fn hash_field(todo_id: TodoId) -> String {
    format!("{}", todo_id)
}
