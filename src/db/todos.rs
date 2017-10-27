use std::ops::Deref;
use r2d2;
use r2d2_redis::RedisConnectionManager;
use redis::Commands;
use redis::RedisError;
use serde_json;
use rocket::response::status;
use rocket::http;

use db::redis::RedisConnection;
use domain::{SessionId, Todo, TodoId};

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
            description: description.to_string(),
        };
        let payload = serde_json::to_string(&todo)?;

        let _: () = self.connection.hset(&self.session_key(), todo_id, payload)?;

        Ok(())
    }

    //
    // Queries
    //

    pub fn list(&self) -> Result<Vec<Todo>, RepositoryError> {
        let ids_and_payloads: Vec<(usize, String)> = self.connection.hgetall(self.session_key())?;

        let todos = ids_and_payloads
            .into_iter()
            .map(|(_, payload)| serde_json::from_str(&payload))
            .flat_map(|r| r)
            .collect();

        Ok(todos)
    }

    //
    // Private functions
    //


    fn session_key(&self) -> String {
        format!("todos:{}", self.session_id)
    }
}
