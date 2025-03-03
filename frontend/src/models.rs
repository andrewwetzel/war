use serde::Deserialize;

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub struct TableData {
    pub id: usize,
    pub name: String,
    pub email: String,
    pub role: String,
    pub created_at: String
}

#[derive(Copy, Clone, PartialEq, Eq, std::hash::Hash, Debug)]
pub enum SortColumn {
    Id,
    Name,
    Email,
    Role,
    DateTime
}

#[derive(Clone, PartialEq, Debug)]
pub enum SortOrder {
    Ascending,
    Descending,
}
