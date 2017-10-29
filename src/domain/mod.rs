pub type SessionId = usize;
pub type TodoId = usize;

#[derive(Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: TodoId,
    pub description: String,
    pub completed: bool,
}

#[derive(Debug, Serialize)]
pub enum TodoFilter {
    All,
    Completed,
    Active,
}
