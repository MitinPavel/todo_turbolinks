pub type TodoId = usize;
pub type SessionId = usize;

#[derive(Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: TodoId,
    pub description: String,
}
