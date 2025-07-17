use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Pagination {
    pub start: Option<i64>,
    pub length: Option<i64>,
    pub search: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
}

#[derive(Serialize)]
pub struct PaginatedResponse<T> {
    pub total: i64,
    pub data: Vec<T>,
}

#[derive(Serialize)]
pub struct DataResponse<T> {
    pub data: T,
}
