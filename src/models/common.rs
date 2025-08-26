use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Pagination {
    pub start: Option<i64>,
    pub length: Option<i64>,
    pub search: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub total: i64,
    pub data: Vec<T>,
}

#[derive(Debug, Serialize)]
pub struct LoadedMoreResponse<T> {
    pub loaded: i64,
    pub data: Vec<T>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum PaginatedLoadedMoreResponse<T> {
    Paginated(PaginatedResponse<T>),
    LoadedMore(LoadedMoreResponse<T>),
}

#[derive(Serialize)]
pub struct DataResponse<T> {
    pub data: T,
}

#[derive(Serialize)]
pub struct HeaderResponse<T> {
    pub id: i64,
    pub header: T,
}
